use crate::{toolbar, windows};
use log::info;
use std::sync::OnceLock;
use tauri::{AppHandle, Emitter, Manager};

/// Shared HTTP client with connection pooling for functional component downloads.
pub fn cdn_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .pool_max_idle_per_host(5)
            .timeout(std::time::Duration::from_secs(15))
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to create CDN HTTP client")
    })
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
