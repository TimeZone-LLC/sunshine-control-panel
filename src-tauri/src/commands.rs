use crate::{toolbar, windows};
use base64::Engine as _;
use log::{info, warn};
use serde::Serialize;
use std::sync::OnceLock;
use tauri::{AppHandle, Emitter, Manager};
use url::Url;

/// Shared CDN HTTP client with connection pooling.
pub fn cdn_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        // Redirect targets intentionally bypass the initial asset allowlist.
        // assets.alkaidlab.com and its redirect configuration are controlled by
        // the domain owner, so redirects are part of the trusted delivery chain.
        reqwest::Client::builder()
            .pool_max_idle_per_host(5)
            .timeout(std::time::Duration::from_secs(15))
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to create CDN HTTP client")
    })
}

/// CDN response state.
enum CdnResult {
    /// 304 Not Modified.
    NotModified,
    /// 200 OK response and optional ETag.
    Fresh(reqwest::Response, Option<String>),
}

fn is_allowed_asset_url(value: &str) -> bool {
    Url::parse(value).is_ok_and(|url| {
        url.scheme() == "https"
            && url.host_str() == Some("assets.alkaidlab.com")
            && url.port_or_known_default() == Some(443)
            && url.username().is_empty()
            && url.password().is_none()
    })
}

/// CDN GET with one retry and optional ETag conditional request.
async fn cdn_get(url: &str, if_none_match: Option<&str>) -> Result<CdnResult, String> {
    let client = cdn_client();

    let build_request = || {
        let mut req = client.get(url);
        if let Some(etag) = if_none_match {
            req = req.header("If-None-Match", etag);
        }
        req
    };

    let result = build_request().send().await;
    let response = match result {
        Ok(resp) => resp,
        Err(_first_err) => {
            warn!("CDN request failed; retrying in 1s");
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            build_request()
                .send()
                .await
                .map_err(|e| format!("Request failed after retry: {}", e))?
        }
    };

    // 304 Not Modified
    if response.status() == reqwest::StatusCode::NOT_MODIFIED {
        return Ok(CdnResult::NotModified);
    }

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let etag = response
        .headers()
        .get(reqwest::header::ETAG)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    Ok(CdnResult::Fresh(response, etag))
}

#[tauri::command]
pub async fn toggle_dark_mode(_window: tauri::Window) -> Result<bool, String> {
    // Theme state is controlled by the frontend.
    Ok(true)
}

#[tauri::command]
pub async fn open_tool_window(app: AppHandle, tool_name: String) -> Result<(), String> {
    info!("Opening tool window: {}", tool_name);

    match tool_name.as_str() {
        "main" => {
            if let Some(window) = app.get_webview_window("main") {
                windows::show_and_activate_window(&window);
            }
        }
        "vdd" => {
            if let Some(window) = app.get_webview_window("main") {
                windows::show_and_activate_window(&window);
                let _ = window.emit("open-vdd-settings", ());
            }
        }
        "about" => {
            windows::open_about_window(&app)?;
        }
        "logs" | "log_console" => {
            windows::open_log_console(&app);
        }
        "nr" => toolbar::create_tool_window_internal(&app, "nr")?,
        "performance" | "host_performance" => {
            toolbar::create_tool_window_internal(&app, "performance")?;
        }
        _ => return Err(format!("Unknown tool name: {}", tool_name)),
    }
    Ok(())
}

/// Remote asset response with optional ETag.
#[derive(Serialize)]
pub struct RemoteBytesResponse {
    pub data_url: String,
    pub etag: Option<String>,
}

/// Fetch a remote asset through Rust to avoid WebView CORS limits.
/// Returns None when the remote responds 304 Not Modified.
#[tauri::command]
pub async fn fetch_remote_bytes(
    url: String,
    if_none_match: Option<String>,
) -> Result<Option<RemoteBytesResponse>, String> {
    // Allowlist the asset CDN only.
    if !is_allowed_asset_url(&url) {
        return Err(format!("Proxying this URL is not allowed: {}", url));
    }

    let result = cdn_get(&url, if_none_match.as_deref()).await?;
    let (response, etag) = match result {
        CdnResult::NotModified => return Ok(None),
        CdnResult::Fresh(resp, etag) => (resp, etag),
    };

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    let data_url = format!("data:{};base64,{}", content_type, b64);

    info!("Proxied remote asset successfully ({} bytes)", bytes.len());
    Ok(Some(RemoteBytesResponse { data_url, etag }))
}

// ===== Desktop screenshot =====

#[cfg(test)]
mod tests {
    use super::is_allowed_asset_url;

    #[test]
    fn remote_asset_url_requires_the_exact_https_cdn_host() {
        assert!(is_allowed_asset_url(
            "https://assets.alkaidlab.com/toolbar-spritesheet.webp?t=1"
        ));
        assert!(is_allowed_asset_url(
            "https://assets.alkaidlab.com:443/speech-phrases.json"
        ));
        assert!(!is_allowed_asset_url(
            "https://assets.alkaidlab.com.example.com/toolbar-spritesheet.webp"
        ));
        assert!(!is_allowed_asset_url(
            "http://assets.alkaidlab.com/toolbar-spritesheet.webp"
        ));
        assert!(!is_allowed_asset_url(
            "https://assets.alkaidlab.com:444/toolbar-spritesheet.webp"
        ));
        assert!(!is_allowed_asset_url(
            "https://user:secret@assets.alkaidlab.com/toolbar-spritesheet.webp"
        ));
    }
}
