use crate::device_config::{DEVICE_CONFIG_LOCK, read_device_api_response};
use crate::sunshine;
use log::{debug, info};
use serde::{Deserialize, Serialize};

/// 控制器中心相关的 sunshine.conf 配置（仿真模式 / DS4 行为 / DSU 体感服务器）
#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct ControllerHubConfig {
    pub gamepad: String,
    pub gamepad_driver: String,
    pub motion_as_ds4: bool,
    pub touchpad_as_ds4: bool,
    pub ds4_back_as_touchpad_click: bool,
    pub ds5_inputtino_randomize_mac: bool,
    pub enable_dsu_server: bool,
    pub dsu_server_port: u16,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct MicrophoneStatus {
    pub success: bool,
    pub configured_backend: String,
    pub active_backend: String,
    pub fallback_reason: String,
    pub component_available: bool,
    pub online: bool,
    pub device_created: bool,
    pub host_streaming: bool,
    pub generation: u32,
    pub state: String,
    pub buffered_bytes: u32,
    pub underruns: u32,
    pub dropped_frames: u32,
    pub submit_errors: u32,
    pub last_error: i32,
    pub error_code: String,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct MicrophoneTestResult {
    pub success: bool,
    pub error_code: String,
    pub backend: String,
}

async fn device_api_url(path: &str) -> Result<String, String> {
    let base_url = sunshine::get_sunshine_url().await?;
    Ok(format!("{}{}", base_url.trim_end_matches('/'), path))
}

#[tauri::command]
pub async fn get_virtual_microphone_status() -> Result<MicrophoneStatus, String> {
    let endpoint = device_api_url("/api/microphone/status").await?;
    let response = sunshine::create_https_client()?
        .get(endpoint)
        .send()
        .await
        .map_err(|error| format!("Unable to query virtual microphone status: {error}"))?;
    read_device_api_response(response, "virtual microphone status").await
}

#[tauri::command]
pub async fn test_virtual_microphone() -> Result<MicrophoneTestResult, String> {
    let endpoint = device_api_url("/api/microphone/test").await?;
    let response = sunshine::create_https_client()?
        .post(endpoint)
        .send()
        .await
        .map_err(|error| format!("Unable to test the virtual microphone: {error}"))?;
    read_device_api_response(response, "virtual microphone test").await
}

const GAMEPAD_MODES: [&str; 8] = [
    "auto", "generic", "x360", "xone", "xseries", "ds4", "ds5", "switch",
];
const GAMEPAD_DRIVERS: [&str; 3] = ["vigembus", "virtualhid", "all"];

/// 对齐 Sunshine 核心 to_bool 的接受列表（大小写不敏感）
fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_lowercase().as_str() {
        "true" | "yes" | "enable" | "enabled" | "on" | "1" => Some(true),
        "false" | "no" | "disable" | "disabled" | "off" | "0" => Some(false),
        _ => None,
    }
}

fn parse_controller_hub_config(
    response: &serde_json::Value,
) -> Result<ControllerHubConfig, String> {
    if response.get("status").and_then(serde_json::Value::as_str) != Some("true") {
        return Err("Sunshine could not read the controller configuration".to_string());
    }

    let get_str = |key: &str| -> Option<String> {
        response.get(key).and_then(|v| v.as_str()).map(String::from)
    };

    let gamepad = get_str("gamepad")
        .filter(|v| GAMEPAD_MODES.contains(&v.as_str()))
        .unwrap_or_else(|| "auto".to_string());
    let gamepad_driver = get_str("gamepad_driver")
        .filter(|v| GAMEPAD_DRIVERS.contains(&v.as_str()))
        .unwrap_or_else(|| "vigembus".to_string());

    let get_bool = |key: &str, default: bool| -> bool {
        get_str(key).and_then(|v| parse_bool(&v)).unwrap_or(default)
    };

    let port = get_str("dsu_server_port")
        .and_then(|v| v.trim().parse::<u16>().ok())
        .filter(|p| (1024..=65535).contains(p))
        .unwrap_or(26760);

    Ok(ControllerHubConfig {
        gamepad,
        gamepad_driver,
        motion_as_ds4: get_bool("motion_as_ds4", true),
        touchpad_as_ds4: get_bool("touchpad_as_ds4", true),
        ds4_back_as_touchpad_click: get_bool("ds4_back_as_touchpad_click", true),
        ds5_inputtino_randomize_mac: get_bool("ds5_inputtino_randomize_mac", true),
        enable_dsu_server: get_bool("enable_dsu_server", false),
        dsu_server_port: port,
    })
}

/// 读取控制器中心配置，缺失键回退到核心默认值
#[tauri::command]
pub async fn get_controller_hub_config() -> Result<ControllerHubConfig, String> {
    let endpoint = device_api_url("/api/gamepad/config").await?;
    let response = sunshine::create_https_client()?
        .get(endpoint)
        .send()
        .await
        .map_err(|error| {
            format!(
                "Unable to query controller configuration: {}",
                error.without_url()
            )
        })?;
    let response: serde_json::Value =
        read_device_api_response(response, "controller configuration").await?;
    parse_controller_hub_config(&response)
}

#[derive(Default)]
struct ControllerHubPatch {
    gamepad: Option<String>,
    gamepad_driver: Option<String>,
    motion_as_ds4: Option<bool>,
    touchpad_as_ds4: Option<bool>,
    ds4_back_as_touchpad_click: Option<bool>,
    ds5_inputtino_randomize_mac: Option<bool>,
    enable_dsu_server: Option<bool>,
    dsu_server_port: Option<u16>,
}

fn build_controller_hub_patch(
    values: ControllerHubPatch,
) -> Result<
    (
        serde_json::Map<String, serde_json::Value>,
        Vec<&'static str>,
    ),
    String,
> {
    if let Some(mode) = values.gamepad.as_deref()
        && !GAMEPAD_MODES.contains(&mode)
    {
        return Err(format!(
            "Invalid emulation mode: {mode} (expected auto, generic, x360, xone, xseries, ds4, ds5, or switch)"
        ));
    }
    if let Some(driver) = values.gamepad_driver.as_deref()
        && !GAMEPAD_DRIVERS.contains(&driver)
    {
        return Err(format!(
            "Invalid gamepad backend: {driver} (expected vigembus, virtualhid, or all)"
        ));
    }
    if let Some(port) = values.dsu_server_port
        && !(1024..=65535).contains(&port)
    {
        return Err(format!("DSU port must be between 1024 and 65535: {port}"));
    }

    let mut patch = serde_json::Map::new();
    let mut changed = Vec::new();
    for (key, value) in [
        ("gamepad", values.gamepad),
        ("gamepad_driver", values.gamepad_driver),
    ] {
        if let Some(value) = value {
            patch.insert(key.to_string(), serde_json::json!(value));
            changed.push(key);
        }
    }
    for (key, value) in [
        ("motion_as_ds4", values.motion_as_ds4),
        ("touchpad_as_ds4", values.touchpad_as_ds4),
        (
            "ds4_back_as_touchpad_click",
            values.ds4_back_as_touchpad_click,
        ),
        (
            "ds5_inputtino_randomize_mac",
            values.ds5_inputtino_randomize_mac,
        ),
        ("enable_dsu_server", values.enable_dsu_server),
    ] {
        if let Some(value) = value {
            patch.insert(
                key.to_string(),
                serde_json::json!(if value { "true" } else { "false" }),
            );
            changed.push(key);
        }
    }
    if let Some(port) = values.dsu_server_port {
        patch.insert(
            "dsu_server_port".to_string(),
            serde_json::json!(port.to_string()),
        );
        changed.push("dsu_server_port");
    }

    Ok((patch, changed))
}

/// 保存控制器中心配置；仅向 Core 提交发生变化的字段。
#[tauri::command]
pub async fn save_controller_hub_config(
    gamepad: Option<String>,
    gamepad_driver: Option<String>,
    motion_as_ds4: Option<bool>,
    touchpad_as_ds4: Option<bool>,
    ds4_back_as_touchpad_click: Option<bool>,
    ds5_inputtino_randomize_mac: Option<bool>,
    enable_dsu_server: Option<bool>,
    dsu_server_port: Option<u16>,
) -> Result<String, String> {
    let (patch, changed) = build_controller_hub_patch(ControllerHubPatch {
        gamepad,
        gamepad_driver,
        motion_as_ds4,
        touchpad_as_ds4,
        ds4_back_as_touchpad_click,
        ds5_inputtino_randomize_mac,
        enable_dsu_server,
        dsu_server_port,
    })?;

    // Serialize this Panel process's writes. Core merges the patch under its
    // own config-file lock, so unrelated settings are never written back.
    let _config_guard = DEVICE_CONFIG_LOCK.lock().await;
    if changed.is_empty() {
        return Ok("No changes to save".to_string());
    }

    debug!("Updating controller hub settings: {:?}", changed);
    let endpoint = device_api_url("/api/gamepad/config").await?;
    let response = sunshine::create_https_client()?
        .post(endpoint)
        .json(&patch)
        .send()
        .await
        .map_err(|error| {
            format!(
                "Unable to save controller configuration: {}",
                error.without_url()
            )
        })?;
    let response: serde_json::Value =
        read_device_api_response(response, "controller configuration save").await?;
    if response.get("status").and_then(serde_json::Value::as_str) != Some("true") {
        return Err("Sunshine could not save the controller configuration".to_string());
    }
    info!("Controller hub settings saved: {:?}", changed);
    Ok("saved".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expanded_gamepad_config_round_trips_backend_and_identity() {
        let response = serde_json::json!({
            "status": "true",
            "gamepad": "xseries",
            "gamepad_driver": "virtualhid",
            "motion_as_ds4": "false",
            "touchpad_as_ds4": "true",
            "ds4_back_as_touchpad_click": "false",
            "ds5_inputtino_randomize_mac": "false",
            "enable_dsu_server": "true",
            "dsu_server_port": "36760"
        });

        assert_eq!(
            parse_controller_hub_config(&response).unwrap(),
            ControllerHubConfig {
                gamepad: "xseries".to_string(),
                gamepad_driver: "virtualhid".to_string(),
                motion_as_ds4: false,
                touchpad_as_ds4: true,
                ds4_back_as_touchpad_click: false,
                ds5_inputtino_randomize_mac: false,
                enable_dsu_server: true,
                dsu_server_port: 36760,
            }
        );
    }

    #[test]
    fn controller_patch_accepts_expanded_profiles_and_serializes_new_fields() {
        let (patch, changed) = build_controller_hub_patch(ControllerHubPatch {
            gamepad: Some("switch".to_string()),
            gamepad_driver: Some("all".to_string()),
            ds5_inputtino_randomize_mac: Some(false),
            ..Default::default()
        })
        .unwrap();

        assert_eq!(patch.get("gamepad"), Some(&serde_json::json!("switch")));
        assert_eq!(patch.get("gamepad_driver"), Some(&serde_json::json!("all")));
        assert_eq!(
            patch.get("ds5_inputtino_randomize_mac"),
            Some(&serde_json::json!("false"))
        );
        assert_eq!(
            changed,
            vec!["gamepad", "gamepad_driver", "ds5_inputtino_randomize_mac"]
        );
    }

    #[test]
    fn controller_patch_rejects_unknown_backend_and_profile() {
        assert!(
            build_controller_hub_patch(ControllerHubPatch {
                gamepad: Some("unknown".to_string()),
                ..Default::default()
            })
            .is_err()
        );
        assert!(
            build_controller_hub_patch(ControllerHubPatch {
                gamepad_driver: Some("unknown".to_string()),
                ..Default::default()
            })
            .is_err()
        );
    }
}
