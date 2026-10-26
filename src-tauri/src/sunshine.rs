use log::{debug, error, info, warn};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, RwLock};
use url::Url;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SunshineConfig {
    pub port: Option<String>,
    pub adapter_name: Option<String>,
    pub resolutions: Option<String>,
    pub fps: Option<String>,
    pub client_fingerprint_remote_rules: Option<String>,
    pub client_fingerprint_rules_url: Option<String>,
    pub client_fingerprint_rules_refresh_hours: Option<String>,
}

// 缓存 Sunshine 路径，避免重复查找和记录日志
static SUNSHINE_PATH_CACHE: Lazy<Mutex<Option<PathBuf>>> = Lazy::new(|| Mutex::new(None));
static RUNTIME_SUNSHINE_URL: Lazy<RwLock<Option<String>>> = Lazy::new(|| RwLock::new(None));
static HTTPS_CLIENT: Lazy<Result<reqwest::Client, String>> = Lazy::new(|| {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create the HTTP client: {}", e))
});
const MAX_CONFIG_SAVE_RESPONSE_BYTES: usize = 64 * 1024;
static SSE_HTTPS_CLIENT: Lazy<Result<reqwest::Client, String>> = Lazy::new(|| {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Create Sunshine event client failed: {}", e))
});

pub const TRAY_PROTOCOL_VERSION: u32 = 1;
const CORE_COMPATIBILITY_CHECK_ARG: &str = "--check-core-compatibility";

pub fn try_run_core_compatibility_check_from_args() -> bool {
    if !std::env::args().any(|arg| arg == CORE_COMPATIBILITY_CHECK_ARG) {
        return false;
    }

    let compatible = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())
        .and_then(|runtime| runtime.block_on(get_tray_state()).map(|_| ()));

    std::process::exit(if compatible.is_ok() { 0 } else { 2 });
}

fn get_sunshine_path() -> PathBuf {
    // 先检查缓存
    {
        let cache = SUNSHINE_PATH_CACHE.lock().unwrap();
        if let Some(ref cached_path) = *cache {
            return cached_path.clone();
        }
    }

    // 缓存未命中，查找路径
    let path = get_sunshine_path_internal();

    // 更新缓存
    {
        let mut cache = SUNSHINE_PATH_CACHE.lock().unwrap();
        *cache = Some(path.clone());
    }

    path
}

#[cfg(target_os = "windows")]
fn bundled_sunshine_root(gui_executable: &Path) -> Option<PathBuf> {
    let gui_dir = gui_executable.parent()?;
    if !gui_dir
        .file_name()?
        .to_string_lossy()
        .eq_ignore_ascii_case("gui")
    {
        return None;
    }

    let assets_dir = gui_dir.parent()?;
    if !assets_dir
        .file_name()?
        .to_string_lossy()
        .eq_ignore_ascii_case("assets")
    {
        return None;
    }

    Some(assets_dir.parent()?.to_path_buf())
}

fn get_sunshine_path_internal() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        use winreg::RegKey;
        use winreg::enums::*;

        if let Ok(gui_executable) = std::env::current_exe()
            && let Some(root) = bundled_sunshine_root(&gui_executable)
            && root.join("sunshine.exe").is_file()
        {
            info!("Using bundled Sunshine path: {:?}", root);
            return root;
        }

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

        // 尝试多个可能的注册表位置
        let registry_paths = [
            r"SOFTWARE\AlkaidLab\Sunshine",
            r"SOFTWARE\LizardByte\Sunshine",
            r"SOFTWARE\WOW6432Node\LizardByte\Sunshine",
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Sunshine",
        ];

        for reg_path in &registry_paths {
            if let Ok(sunshine_key) = hklm.open_subkey(reg_path) {
                // 尝试读取多个可能的键名
                for key_name in &["InstallDir", "InstallLocation", "InstallPath", "Path", ""] {
                    if let Ok(path) = sunshine_key.get_value::<String, _>(key_name) {
                        let install_path = PathBuf::from(path);
                        if install_path.exists() {
                            info!(
                                "Read the Sunshine path from the registry: {:?}",
                                install_path
                            );
                            return install_path;
                        }
                    }
                }
            }
        }

        // 尝试默认安装路径
        let default_paths = [
            PathBuf::from(r"C:\Program Files\Sunshine"),
            PathBuf::from(r"C:\Program Files (x86)\Sunshine"),
        ];

        for path in &default_paths {
            if path.exists() {
                info!("Using the default Sunshine path: {:?}", path);
                return path.clone();
            }
        }

        warn!("Sunshine installation path was not found; using the default path");
        PathBuf::from(r"C:\Program Files\Sunshine")
    }

    #[cfg(not(target_os = "windows"))]
    {
        PathBuf::from("/usr/local/sunshine")
    }
}

#[cfg(all(test, target_os = "windows"))]
mod bundled_path_tests {
    use super::*;

    #[test]
    fn derives_root_from_packaged_gui_layout() {
        let executable = Path::new(r"C:\Sunshine\assets\gui\sunshine-gui.exe");

        assert_eq!(
            bundled_sunshine_root(executable),
            Some(PathBuf::from(r"C:\Sunshine"))
        );
    }

    #[test]
    fn rejects_unrelated_gui_locations() {
        let executable = Path::new(r"C:\Tools\sunshine-gui.exe");

        assert_eq!(bundled_sunshine_root(executable), None);
    }
}

/// 获取 Sunshine 安装路径（暴露给前端）
#[tauri::command]
pub fn get_sunshine_install_path() -> String {
    get_sunshine_path().to_string_lossy().to_string()
}

/// 获取 Sunshine 安装路径（内部使用，返回 PathBuf）
pub fn install_dir() -> PathBuf {
    get_sunshine_path()
}

/// 获取 config 目录（install_dir/config）
pub fn config_dir() -> PathBuf {
    get_sunshine_path().join("config")
}

/// 获取 assets 目录（install_dir/assets）
pub fn assets_dir() -> PathBuf {
    get_sunshine_path().join("assets")
}

fn parse_sunshine_config_content(content: &str) -> SunshineConfig {
    let mut config = SunshineConfig::default();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "port" => config.port = Some(value.to_string()),
                "adapter_name" => config.adapter_name = Some(value.to_string()),
                "resolutions" => config.resolutions = Some(value.to_string()),
                "fps" => config.fps = Some(value.to_string()),
                "client_fingerprint_remote_rules" => {
                    config.client_fingerprint_remote_rules = Some(value.to_string())
                }
                "client_fingerprint_rules_url" => {
                    config.client_fingerprint_rules_url = Some(value.to_string())
                }
                "client_fingerprint_rules_refresh_hours" => {
                    config.client_fingerprint_rules_refresh_hours = Some(value.to_string())
                }
                _ => {}
            }
        }
    }

    config
}

pub(crate) fn parse_sunshine_config_sync() -> Result<SunshineConfig, String> {
    let config_path = get_sunshine_path().join("config").join("sunshine.conf");

    if !config_path.exists() {
        return Ok(SunshineConfig {
            port: Some("47989".to_string()),
            ..Default::default()
        });
    }

    let content = std::fs::read_to_string(config_path).map_err(|e| e.to_string())?;
    Ok(parse_sunshine_config_content(&content))
}

#[tauri::command]
pub async fn parse_sunshine_config() -> Result<SunshineConfig, String> {
    parse_sunshine_config_sync()
}

const DEFAULT_SUNSHINE_PORT: u16 = 47989;
const DEFAULT_WEB_UI_PORT: u16 = 47990;

#[tauri::command]
pub async fn get_sunshine_url() -> Result<String, String> {
    // 开发模式下优先使用环境变量，和本地代理保持一致
    if let Ok(url) = std::env::var("WEBUI_DEV_TARGET") {
        if let Some(base) = parse_url_to_base(&url) {
            return Ok(base);
        }
        return Err(format!("Invalid WEBUI_DEV_TARGET: {}", url));
    }

    // 优先检查命令行参数
    if let Some(url) = get_runtime_sunshine_url() {
        return Ok(url);
    }

    if let Some(url) = get_command_line_url() {
        return parse_url_to_base(&url).ok_or(url);
    }

    // 从配置文件读取端口
    let config = parse_sunshine_config().await?;

    Ok(local_sunshine_url_from_config(&config))
}

/// Resolve the Core instance installed on this machine. Tray traffic must not
/// follow the remote target selected for the main window.
pub async fn get_local_sunshine_url() -> Result<String, String> {
    let config = parse_sunshine_config().await?;
    Ok(local_sunshine_url_from_config(&config))
}

fn local_sunshine_url_from_config(config: &SunshineConfig) -> String {
    let port = config
        .port
        .as_deref()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(DEFAULT_SUNSHINE_PORT);
    format!("https://127.0.0.1:{}", port + 1)
}

async fn local_tray_endpoint(resource: &str) -> Result<String, String> {
    let sunshine_url = get_local_sunshine_url().await?;
    Ok(format!(
        "{}/api/tray/{}",
        sunshine_url.trim_end_matches('/'),
        resource
    ))
}

pub async fn get_tray_events_url() -> Result<String, String> {
    local_tray_endpoint("events").await
}

fn parse_url_to_base(url: &str) -> Option<String> {
    url::Url::parse(url).ok().map(|parsed| {
        let host = parsed.host_str().unwrap_or("127.0.0.1");
        let port = parsed.port().unwrap_or(DEFAULT_WEB_UI_PORT);
        format!("{}://{}:{}", parsed.scheme(), host, port)
    })
}

pub fn set_runtime_sunshine_url(url: &str) -> Result<String, String> {
    let base = parse_url_to_base(url).ok_or_else(|| format!("Invalid Sunshine URL: {}", url))?;
    if let Ok(mut runtime_url) = RUNTIME_SUNSHINE_URL.write() {
        *runtime_url = Some(base.clone());
    }
    Ok(base)
}

fn get_runtime_sunshine_url() -> Option<String> {
    RUNTIME_SUNSHINE_URL
        .read()
        .ok()
        .and_then(|runtime_url| runtime_url.clone())
}

#[tauri::command]
pub fn get_command_line_url() -> Option<String> {
    std::env::args().find_map(|arg| arg.strip_prefix("--url=").map(String::from))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionInfo {
    pub client_name: String,
    pub client_address: String,
    pub state: String,
    pub session_id: i32,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub bitrate: u32, // Current bitrate in Kbps
    pub host_audio: bool,
    pub enable_hdr: bool,
    pub enable_mic: bool,
    pub app_name: String,
    pub app_id: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct TrayClientSession {
    #[serde(default)]
    pub id: u32,
    #[serde(default)]
    pub client_name: String,
    #[serde(default)]
    pub highly_suspected_unknown_client: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct TrayVddState {
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub keep_enabled: bool,
    #[serde(default)]
    pub headless_create_enabled: bool,
    #[serde(default)]
    pub cooldown: bool,
    #[serde(default)]
    pub awaiting_confirmation: bool,
    #[serde(default)]
    pub confirmation_operation_id: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct TrayNotificationState {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub action: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct TrayOperationState {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct TrayState {
    #[serde(default)]
    pub protocol_version: u32,
    #[serde(default)]
    pub instance_id: String,
    #[serde(default)]
    pub owner: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub tooltip: String,
    #[serde(default)]
    pub app_name: String,
    #[serde(default)]
    pub pairing_client_name: String,
    #[serde(default)]
    pub sessions: Vec<TrayClientSession>,
    #[serde(default)]
    pub revision: u64,
    #[serde(default)]
    pub updated_at_ms: i64,
    #[serde(default)]
    pub vdd: TrayVddState,
    #[serde(default)]
    pub notification: TrayNotificationState,
    #[serde(default)]
    pub operation: TrayOperationState,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TrayActionResponse {
    #[serde(default)]
    pub status: bool,
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub operation_id: u64,
    #[serde(default)]
    pub tray_state: Option<TrayState>,
}

fn validate_tray_state(state: &TrayState) -> Result<(), String> {
    if state.protocol_version != TRAY_PROTOCOL_VERSION {
        return Err(format!(
            "Unsupported tray protocol version {} (expected {})",
            state.protocol_version, TRAY_PROTOCOL_VERSION
        ));
    }
    if state.instance_id.is_empty() {
        return Err("Tray state is missing core instance_id".to_string());
    }
    if !matches!(state.owner.as_str(), "gui" | "core" | "disabled") {
        return Err(format!("Unsupported tray owner '{}'", state.owner));
    }
    if !state.capabilities.iter().any(|value| value == "state-v1") {
        return Err("Tray state is missing required state-v1 capability".to_string());
    }
    Ok(())
}

pub(crate) fn parse_tray_state_json(response_text: &str) -> Result<TrayState, String> {
    let state: TrayState = serde_json::from_str(response_text)
        .map_err(|e| format!("Parse tray state failed: {}; body: {}", e, response_text))?;
    validate_tray_state(&state)?;
    Ok(state)
}

#[cfg(test)]
mod tray_protocol_tests {
    use super::*;

    fn valid_state() -> TrayState {
        TrayState {
            protocol_version: TRAY_PROTOCOL_VERSION,
            instance_id: "core-instance".to_string(),
            owner: "gui".to_string(),
            capabilities: vec!["state-v1".to_string(), "actions-v1".to_string()],
            ..Default::default()
        }
    }

    #[test]
    fn accepts_supported_tray_protocol() {
        assert!(validate_tray_state(&valid_state()).is_ok());
    }

    #[test]
    fn rejects_incomplete_tray_protocol() {
        let mut state = valid_state();
        state.instance_id.clear();
        assert!(validate_tray_state(&state).is_err());

        let mut state = valid_state();
        state.owner = "unknown".to_string();
        assert!(validate_tray_state(&state).is_err());

        let mut state = valid_state();
        state.capabilities.clear();
        assert!(validate_tray_state(&state).is_err());
    }

    #[test]
    fn local_tray_url_uses_the_configured_core_port() {
        let config = SunshineConfig {
            port: Some("48000".to_string()),
            ..Default::default()
        };
        assert_eq!(
            local_sunshine_url_from_config(&config),
            "https://127.0.0.1:48001"
        );
    }

    #[test]
    fn restart_surfaces_definite_request_failures() {
        let error = handle_restart_action_error(TrayActionRequestError::Definite(
            "connection refused".to_string(),
        ))
        .unwrap_err();

        assert_eq!(error, "connection refused");
    }

    #[test]
    fn restart_tolerates_an_interrupted_response() {
        let response = handle_restart_action_error(TrayActionRequestError::DeliveryUnknown(
            "connection closed".to_string(),
        ))
        .unwrap();

        assert!(response.is_none());
    }

    #[test]
    fn config_save_response_requires_business_success() {
        assert!(validate_config_save_response(r#"{"status":"true"}"#).is_ok());
        assert!(validate_config_save_response(r#"{"status":true}"#).is_ok());

        let error = validate_config_save_response(
            r#"{"status":"false","error":"failed to persist configuration"}"#,
        )
        .unwrap_err();
        assert!(error.contains("failed to persist configuration"));
    }

    #[test]
    fn config_save_response_rejects_missing_or_invalid_status() {
        assert!(validate_config_save_response(r#"{}"#).is_err());
        assert!(validate_config_save_response("not json").is_err());
    }
}

#[derive(Debug)]
enum TrayActionRequestError {
    Definite(String),
    DeliveryUnknown(String),
}

impl TrayActionRequestError {
    fn into_message(self) -> String {
        match self {
            Self::Definite(message) | Self::DeliveryUnknown(message) => message,
        }
    }
}

fn handle_restart_action_error(
    error: TrayActionRequestError,
) -> Result<Option<TrayActionResponse>, String> {
    match error {
        TrayActionRequestError::Definite(message) => Err(message),
        TrayActionRequestError::DeliveryUnknown(message) => {
            debug!("Sunshine restart response was interrupted: {}", message);
            Ok(None)
        }
    }
}

async fn post_tray_action_request(
    action: &str,
    enabled: Option<bool>,
    notification_id: Option<u64>,
    operation_id: Option<u64>,
) -> Result<TrayActionResponse, TrayActionRequestError> {
    let action_url = local_tray_endpoint("action")
        .await
        .map_err(TrayActionRequestError::Definite)?;

    let client = create_https_client().map_err(TrayActionRequestError::Definite)?;
    let mut body = serde_json::json!({ "action": action });
    if let Some(enabled) = enabled {
        body["enabled"] = serde_json::json!(enabled);
    }
    if let Some(notification_id) = notification_id {
        body["notification_id"] = serde_json::json!(notification_id);
    }
    if let Some(operation_id) = operation_id {
        body["operation_id"] = serde_json::json!(operation_id);
    }

    let response = client
        .post(&action_url)
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            let message = format!("Post tray action failed: {}", e);
            if e.is_connect() {
                TrayActionRequestError::Definite(message)
            } else {
                TrayActionRequestError::DeliveryUnknown(message)
            }
        })?;

    let status = response.status();
    let response_text = response.text().await.map_err(|e| {
        TrayActionRequestError::DeliveryUnknown(format!("Read tray action response failed: {}", e))
    })?;

    if !status.is_success() {
        return Err(TrayActionRequestError::Definite(format!(
            "Tray action failed (status {}): {}",
            status, response_text
        )));
    }

    let result: TrayActionResponse = serde_json::from_str(&response_text).map_err(|e| {
        TrayActionRequestError::Definite(format!(
            "Parse tray action failed: {}; body: {}",
            e, response_text
        ))
    })?;
    let state = result.tray_state.as_ref().ok_or_else(|| {
        TrayActionRequestError::Definite("Tray action response is missing tray_state".to_string())
    })?;
    validate_tray_state(state).map_err(TrayActionRequestError::Definite)?;
    Ok(result)
}

pub async fn post_tray_action(
    action: &str,
    enabled: Option<bool>,
) -> Result<TrayActionResponse, String> {
    post_tray_action_request(action, enabled, None, None)
        .await
        .map_err(TrayActionRequestError::into_message)
}

pub async fn post_tray_restart_action() -> Result<Option<TrayActionResponse>, String> {
    match post_tray_action_request("restart", None, None, None).await {
        Ok(response) => Ok(Some(response)),
        Err(error) => handle_restart_action_error(error),
    }
}

pub async fn acknowledge_tray_notification(
    notification_id: u64,
) -> Result<TrayActionResponse, String> {
    post_tray_action_request("notification_ack", None, Some(notification_id), None)
        .await
        .map_err(TrayActionRequestError::into_message)
}

pub async fn confirm_vdd_keep(operation_id: u64, keep: bool) -> Result<TrayActionResponse, String> {
    post_tray_action_request("vdd_confirm_keep", Some(keep), None, Some(operation_id))
        .await
        .map_err(TrayActionRequestError::into_message)
}

impl SessionInfo {
    fn from_json(session_obj: &serde_json::Value) -> Self {
        Self {
            client_name: session_obj
                .get("client_name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            client_address: session_obj
                .get("client_address")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            state: session_obj
                .get("state")
                .and_then(|v| v.as_str())
                .unwrap_or("UNKNOWN")
                .to_string(),
            session_id: session_obj
                .get("session_id")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            width: session_obj
                .get("width")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            height: session_obj
                .get("height")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            fps: session_obj.get("fps").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            bitrate: session_obj
                .get("bitrate")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            host_audio: session_obj
                .get("host_audio")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            enable_hdr: session_obj
                .get("enable_hdr")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            enable_mic: session_obj
                .get("enable_mic")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            app_name: session_obj
                .get("app_name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            app_id: session_obj
                .get("app_id")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
        }
    }
}

pub async fn get_tray_state() -> Result<TrayState, String> {
    let tray_state_url = local_tray_endpoint("state").await?;

    let client = create_https_client()?;
    let response = client
        .get(&tray_state_url)
        .send()
        .await
        .map_err(|e| format!("Request tray state failed: {}", e))?;

    let status = response.status();
    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Read tray state response failed: {}", e))?;

    if !status.is_success() {
        return Err(format!(
            "Tray state request failed (status {}): {}",
            status, response_text
        ));
    }

    parse_tray_state_json(&response_text)
}

#[derive(Debug, Deserialize)]
pub struct ClientFingerprintRuleInstallResponse {
    pub status: bool,
    pub installed: bool,
    pub unchanged: bool,
    pub revision: u64,
}

pub async fn post_client_fingerprint_rules(
    envelope: &str,
) -> Result<ClientFingerprintRuleInstallResponse, String> {
    let endpoint = local_tray_endpoint("client-fingerprint-rules").await?;
    let client = create_https_client()?;
    let response = client
        .post(&endpoint)
        .json(&serde_json::json!({ "envelope": envelope }))
        .send()
        .await
        .map_err(|e| format!("Submit client fingerprint rules failed: {}", e))?;

    let status = response.status();
    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Read client fingerprint rule response failed: {}", e))?;
    if !status.is_success() {
        return Err(format!(
            "Client fingerprint rules were rejected (status {}): {}",
            status, response_text
        ));
    }

    let result: ClientFingerprintRuleInstallResponse = serde_json::from_str(&response_text)
        .map_err(|e| {
            format!(
                "Parse client fingerprint rule response failed: {}; body: {}",
                e, response_text
            )
        })?;
    if !result.status {
        return Err("Client fingerprint rule response reported failure".to_string());
    }
    Ok(result)
}

pub fn create_https_client() -> Result<reqwest::Client, String> {
    HTTPS_CLIENT.as_ref().cloned().map_err(Clone::clone)
}

pub fn create_sse_https_client() -> Result<reqwest::Client, String> {
    SSE_HTTPS_CLIENT.as_ref().cloned().map_err(Clone::clone)
}

fn validate_config_save_response(body: &str) -> Result<(), String> {
    let response: serde_json::Value = serde_json::from_str(body)
        .map_err(|error| format!("Sunshine Config API returned invalid JSON: {error}"))?;
    let succeeded = match response.get("status") {
        Some(serde_json::Value::Bool(status)) => *status,
        Some(serde_json::Value::String(status)) => status.eq_ignore_ascii_case("true"),
        _ => {
            return Err("Sunshine Config API response is missing a valid status".to_string());
        }
    };
    if succeeded {
        return Ok(());
    }

    let detail = response
        .get("error")
        .and_then(serde_json::Value::as_str)
        .filter(|error| !error.trim().is_empty())
        .unwrap_or("Sunshine rejected the configuration");
    Err(format!(
        "Sunshine Config API rejected configuration: {detail}"
    ))
}

/// POST 配置数据到 Sunshine Config API
/// 封装了获取 URL、创建客户端、POST 请求和错误处理的完整流程
pub async fn post_sunshine_config(
    config_data: &serde_json::Map<String, serde_json::Value>,
) -> Result<(), String> {
    let sunshine_url = get_sunshine_url()
        .await
        .map_err(|e| format!("Cannot get Sunshine URL: {}", e))?;
    post_sunshine_config_to(&sunshine_url, config_data).await
}

async fn post_sunshine_config_to(
    sunshine_url: &str,
    config_data: &serde_json::Map<String, serde_json::Value>,
) -> Result<(), String> {
    let config_url = format!("{}/api/config", sunshine_url.trim_end_matches('/'));

    let client = create_https_client()?;
    let mut response = client
        .post(&config_url)
        .json(config_data)
        .send()
        .await
        .map_err(|e| format!("Sunshine Config API request failed: {}", e))?;

    let status = response.status();
    if response
        .content_length()
        .is_some_and(|size| size > MAX_CONFIG_SAVE_RESPONSE_BYTES as u64)
    {
        return Err("Sunshine Config API response exceeds 64 KiB".to_string());
    }
    let mut response_bytes = Vec::with_capacity(
        response
            .content_length()
            .unwrap_or_default()
            .min(MAX_CONFIG_SAVE_RESPONSE_BYTES as u64) as usize,
    );
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| format!("Failed to read the Sunshine Config API response: {error}"))?
    {
        if response_bytes.len().saturating_add(chunk.len()) > MAX_CONFIG_SAVE_RESPONSE_BYTES {
            return Err("Sunshine Config API response exceeds 64 KiB".to_string());
        }
        response_bytes.extend_from_slice(&chunk);
    }
    let response_body = String::from_utf8(response_bytes)
        .map_err(|error| format!("Sunshine Config API response is not valid UTF-8: {error}"))?;
    if !status.is_success() {
        return Err(format!(
            "Sunshine Config API returned an error (status: {}): {}",
            status, response_body
        ));
    }
    validate_config_save_response(&response_body)
}

#[tauri::command]
pub async fn get_active_sessions() -> Result<Vec<SessionInfo>, String> {
    let sunshine_url = get_sunshine_url().await?;
    let sessions_url = format!(
        "{}/api/runtime/sessions",
        sunshine_url.trim_end_matches('/')
    );

    debug!("Requesting active sessions: {}", sessions_url);

    let client = create_https_client()?;

    let response = client
        .get(&sessions_url)
        .send()
        .await
        .map_err(|e| format!("Failed to request session information: {}", e))?;

    let status = response.status();

    debug!("Sessions response status: {}", status);

    // 检查 Content-Type
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("")
        .to_lowercase();

    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read the response: {}", e))?;

    debug!("Sessions response body: {}", response_text);

    // 如果是 404 或 XML 响应，返回空数组（没有会话是正常情况）
    if status == 404
        || content_type.contains("xml")
        || response_text.trim_start().starts_with("<?xml")
    {
        debug!("No active sessions (404 or XML response)");
        return Ok(Vec::new());
    }

    // 如果状态码不是成功，但也不是 404，返回错误
    if !status.is_success() {
        error!("Error response: {}", response_text);
        return Err(format!(
            "Failed to get session information (status: {}): {}",
            status, response_text
        ));
    }

    // 尝试解析 JSON
    let json: serde_json::Value = serde_json::from_str(&response_text).map_err(|e| {
        format!(
            "Failed to parse JSON: {}; response body: {}",
            e, response_text
        )
    })?;

    debug!("Parsed JSON: {:#}", json);

    // 检查 API 响应状态
    if json.get("success").and_then(|v| v.as_bool()) == Some(false) {
        let error_msg = json
            .get("status_message")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown error");
        return Err(format!("API returned an error: {}", error_msg));
    }

    // 解析会话列表
    let sessions = json
        .get("sessions")
        .and_then(|v| v.as_array())
        .map(|sessions_array| {
            debug!("Found {} sessions", sessions_array.len());
            sessions_array
                .iter()
                .map(SessionInfo::from_json)
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| {
            warn!("The response has no 'sessions' field or it is not an array");
            debug!("JSON structure: {:#}", json);
            Vec::new()
        });

    info!("Retrieved {} active sessions", sessions.len());
    Ok(sessions)
}

#[tauri::command]
pub async fn stop_all_sessions() -> Result<String, String> {
    let sunshine_url = get_sunshine_url().await?;
    let stop_url = format!(
        "{}/api/runtime/sessions/stop",
        sunshine_url.trim_end_matches('/')
    );

    info!(
        "Requesting disconnection of all streaming sessions: {}",
        stop_url
    );

    let client = create_https_client()?;
    let response = client
        .post(&stop_url)
        .json(&serde_json::json!({}))
        .send()
        .await
        .map_err(|e| format!("Failed to request session disconnection: {}", e))?;

    let status = response.status();
    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read the session-disconnection response: {}", e))?;

    if !status.is_success() {
        error!(
            "Session disconnection failed (status {}): {}",
            status, response_text
        );
        return Err(format!(
            "Session disconnection failed (status {}): {}",
            status, response_text
        ));
    }

    info!("Requested disconnection of all streaming sessions");
    Ok(response_text)
}

#[tauri::command]
pub async fn change_bitrate(client_name: String, bitrate: u32) -> Result<String, String> {
    // 验证码率范围
    if !(1..=800000).contains(&bitrate) {
        return Err("Bitrate must be between 1 and 800000 Kbps".to_string());
    }

    // 构建请求 URL
    let sunshine_url = get_sunshine_url().await?;
    let base_url = Url::parse(&sunshine_url)
        .map_err(|e| format!("Failed to parse the Sunshine URL: {}", e))?;

    let mut change_bitrate_url = base_url
        .join("api/runtime/bitrate")
        .map_err(|e| format!("Failed to build the URL: {}", e))?;

    change_bitrate_url
        .query_pairs_mut()
        .append_pair("bitrate", &bitrate.to_string())
        .append_pair("clientname", &client_name);

    info!("Adjusting bitrate for {} to {} Kbps", client_name, bitrate);
    debug!("Request URL: {}", change_bitrate_url);

    // 发送请求
    let client = create_https_client()?;
    let response = client
        .get(change_bitrate_url.as_str())
        .send()
        .await
        .map_err(|e| format!("Bitrate-adjustment request failed: {}", e))?;

    let status = response.status();
    debug!("HTTP status: {}", status);

    // 读取响应内容
    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read the response: {}", e))?;

    // 检查 HTTP 状态码
    if !status.is_success() {
        return Err(match status.as_u16() {
            401 => "Authentication failed. Check the Sunshine configuration".to_string(),
            403 => "Access denied. Only localhost access is allowed".to_string(),
            _ => format!("HTTP error (status: {}): {}", status, response_text),
        });
    }

    // 解析 JSON 响应
    let json: serde_json::Value = serde_json::from_str(&response_text).map_err(|e| {
        format!(
            "Failed to parse JSON: {}; response body: {}",
            e, response_text
        )
    })?;

    debug!("Parsed JSON: {:#}", json);

    // 检查响应状态
    match json.get("success").and_then(|v| v.as_bool()) {
        Some(true) => {
            info!("Bitrate adjusted successfully");
            Ok(format!("Bitrate adjusted to {} Kbps", bitrate))
        }
        Some(false) => {
            let error_msg = json
                .get("status_message")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            let status_code = json
                .get("status_code")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            error!(
                "Failed to adjust bitrate: {} (status: {})",
                error_msg, status_code
            );

            // 根据状态码提供详细提示
            let error_message = if status_code == 404 {
                format!(
                    "Failed to adjust bitrate: {}\n\nVerify that the client name is correct and the session is in the RUNNING state",
                    error_msg
                )
            } else {
                format!("Failed to adjust bitrate: {}", error_msg)
            };

            Err(error_message)
        }
        None => {
            warn!("Invalid response format; the success field could not be parsed");
            Err("Invalid response format".to_string())
        }
    }
}

/// 检查 Sunshine 是否以用户模式运行（非服务模式）
/// 内部实现：执行 sc / tasklist 等阻塞系统调用，必须在 spawn_blocking 中调用
pub(crate) fn is_sunshine_running_in_user_mode_impl() -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        use std::process::Command;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        // 检查服务是否正在运行（服务名不区分大小写，只需检查一次）
        if let Ok(result) = Command::new("sc")
            .args(["query", "SunshineService"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
        {
            let output_str = String::from_utf8_lossy(&result.stdout).to_uppercase();
            if output_str.contains("RUNNING") {
                return Ok(false); // 服务模式
            }
        }

        // 服务未运行，检查 sunshine.exe 进程是否存在
        if let Ok(result) = Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq sunshine.exe", "/FO", "CSV", "/NH"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
        {
            let output_str = String::from_utf8_lossy(&result.stdout).to_lowercase();
            if output_str.contains("sunshine.exe") {
                return Ok(true); // 用户模式
            }
        }

        Ok(false)
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(false)
    }
}

/// 检查 Sunshine 是否以用户模式运行（Tauri 命令，在后台线程执行阻塞逻辑，避免卡住 WebView）
#[tauri::command]
pub async fn is_sunshine_running_in_user_mode() -> Result<bool, String> {
    match tokio::task::spawn_blocking(is_sunshine_running_in_user_mode_impl).await {
        Ok(inner) => inner,
        Err(e) => Err(e.to_string()),
    }
}

/// 构建停止 Sunshine 的命令片段
#[cfg(target_os = "windows")]
fn build_stop_sunshine_command() -> String {
    "net stop SunshineService 2>$null; \
     net stop sunshineservice 2>$null; \
     taskkill /IM sunshine.exe /F 2>$null; \
     Start-Sleep -Seconds 1"
        .to_string()
}

/// 构建启动服务模式的命令片段
#[cfg(target_os = "windows")]
fn build_start_service_command(sunshine_path: &std::path::Path) -> String {
    format!(
        "$serviceExists = Get-Service -Name 'SunshineService' -ErrorAction SilentlyContinue; \
         if ($serviceExists) {{ \
             net start SunshineService \
         }} else {{ \
             Set-Location '{}'; \
             Start-Process -FilePath '.\\sunshine.exe' -WindowStyle Hidden \
         }}",
        sunshine_path.display()
    )
}

/// 切换 Sunshine 运行模式（用户模式 ↔ 服务模式）
#[tauri::command]
pub async fn toggle_sunshine_mode() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let is_user_mode = tokio::task::spawn_blocking(is_sunshine_running_in_user_mode_impl)
            .await
            .ok()
            .and_then(|r| r.ok())
            .unwrap_or(false);
        let sunshine_path = get_sunshine_path();
        let sunshine_exe = sunshine_path.join("sunshine.exe");

        if !sunshine_exe.exists() {
            return Err(format!(
                "sunshine.exe was not found: {}",
                sunshine_exe.display()
            ));
        }

        let stop_cmd = build_stop_sunshine_command();

        let (mode_name, command) = if is_user_mode {
            info!("Switching Sunshine from user mode to service mode");
            let start_cmd = build_start_service_command(&sunshine_path);
            ("service mode", format!("{}; {}", stop_cmd, start_cmd))
        } else {
            info!("Switching Sunshine from service mode to user mode");
            let start_cmd = format!(
                "Set-Location '{}'; Start-Process -FilePath '.\\sunshine.exe' -Verb RunAs -WindowStyle Hidden",
                sunshine_path.display()
            );
            ("user mode", format!("{}; {}", stop_cmd, start_cmd))
        };

        crate::utils::execute_powershell_command(
            &command,
            &format!("Failed to switch to {}", mode_name),
        )?;

        info!(
            "The command to switch to {} started in the background",
            mode_name
        );
        Ok(format!("Switching to {}", mode_name))
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("This feature is available only on Windows".to_string())
    }
}

#[tauri::command]
pub async fn restart_sunshine_service() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        info!("Restarting the Sunshine service...");

        let sunshine_path = get_sunshine_path();
        let stop_cmd = build_stop_sunshine_command();
        let start_cmd = build_start_service_command(&sunshine_path);
        let command = format!("{}; {}", stop_cmd, start_cmd);

        crate::utils::execute_powershell_command(&command, "Failed to start the restart command")?;

        info!("The restart command started in the background");
        Ok("success".to_string())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("This feature is available only on Windows".to_string())
    }
}

/// 以用户模式重启 Sunshine（非服务模式，但需要管理员权限）
/// @deprecated 使用 sunshine::toggle_sunshine_mode 代替
#[tauri::command]
pub async fn restart_sunshine_in_user_mode() -> Result<String, String> {
    toggle_sunshine_mode().await
}
