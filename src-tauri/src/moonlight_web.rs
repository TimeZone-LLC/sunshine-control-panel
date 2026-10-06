use log::{debug, info, warn};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

// ========== 常量 ==========
const DEFAULT_BIND_PORT: u16 = 47790;
const SERVER_BINARY_NAME: &str = "web-server.exe";
const PROCESS_CHECK_NAME: &str = "web-server.exe";

// ========== 状态管理 ==========

/// 子进程句柄（由控制面板启动时持有）
static CHILD_PROCESS: Lazy<Mutex<Option<std::process::Child>>> = Lazy::new(|| Mutex::new(None));

/// 当前配置的绑定端口
static BIND_PORT: Lazy<Mutex<u16>> = Lazy::new(|| Mutex::new(DEFAULT_BIND_PORT));

// ========== 数据结构 ==========

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoonlightWebStatus {
    /// 是否已安装（binary 存在）
    pub installed: bool,
    /// 是否正在运行
    pub running: bool,
    /// 安装路径
    pub install_path: String,
    /// 版本号
    pub version: String,
    /// Web UI 访问地址
    pub access_url: String,
    /// 绑定端口
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoonlightWebConfig {
    #[serde(default)]
    pub web_server: WebServerConfig,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webrtc: Option<WebRtcConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_settings: Option<serde_json::Value>,
    // 保留上游新增的未知字段
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebServerConfig {
    #[serde(default = "default_bind_address")]
    pub bind_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate: Option<CertificateConfig>,
    #[serde(default)]
    pub url_path_prefix: String,
    #[serde(default = "default_true")]
    pub session_cookie_secure: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_cookie_expiration: Option<SessionExpiration>,
    #[serde(default = "default_true")]
    pub first_login_create_admin: bool,
    #[serde(default = "default_true")]
    pub first_login_assign_global_hosts: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_user_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forwarded_header: Option<serde_json::Value>,
    // 保留未知字段
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

impl Default for WebServerConfig {
    fn default() -> Self {
        Self {
            bind_address: default_bind_address(),
            certificate: None,
            url_path_prefix: String::new(),
            session_cookie_secure: false,
            session_cookie_expiration: None,
            first_login_create_admin: true,
            first_login_assign_global_hosts: true,
            default_user_id: None,
            forwarded_header: None,
            extra: std::collections::HashMap::new(),
        }
    }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionExpiration {
    pub secs: u64,
    pub nanos: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CertificateConfig {
    pub private_key_pem: String,
    pub certificate_pem: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebRtcConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ice_servers: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port_range: Option<PortRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nat_1to1: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_types: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortRange {
    pub min: u16,
    pub max: u16,
}

fn default_bind_address() -> String {
    format!("0.0.0.0:{}", DEFAULT_BIND_PORT)
}

// ========== 路径工具 ==========

/// 获取 moonlight-web 安装目录（%LOCALAPPDATA%\Sunshine\moonlight-web）
fn get_install_dir() -> PathBuf {
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        return PathBuf::from(local_app_data)
            .join("Sunshine")
            .join("moonlight-web");
    }
    // fallback: 与控制面板同级
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            return parent.join("moonlight-web");
        }
    }
    PathBuf::from("./moonlight-web")
}

/// 获取 web-server 可执行文件路径
fn get_server_binary_path() -> PathBuf {
    get_install_dir().join(SERVER_BINARY_NAME)
}

/// 获取配置文件路径
fn get_config_path() -> PathBuf {
    get_install_dir().join("server").join("config.json")
}

// ========== Tauri 命令 ==========

/// 获取 moonlight-web 状态
#[tauri::command]
pub async fn moonlight_web_get_status() -> Result<MoonlightWebStatus, String> {
    let install_dir = get_install_dir();
    let binary_path = get_server_binary_path();
    let installed = binary_path.exists();
    let running = is_process_running();
    let port = *BIND_PORT.lock().unwrap();

    let version = if installed {
        get_installed_version().unwrap_or_else(|| "unknown".to_string())
    } else {
        String::new()
    };

    // 根据配置判断协议并生成访问 URL
    let access_url = if running {
        let scheme = if has_https_certificate() {
            "https"
        } else {
            "http"
        };
        format!("{}://localhost:{}", scheme, port)
    } else {
        String::new()
    };

    Ok(MoonlightWebStatus {
        installed,
        running,
        install_path: install_dir.to_string_lossy().to_string(),
        version,
        access_url,
        port,
    })
}

/// 启动 moonlight-web 服务
#[tauri::command]
pub async fn moonlight_web_start() -> Result<String, String> {
    let binary_path = get_server_binary_path();
    if !binary_path.exists() {
        return Err("moonlight-web is not installed. Install it first".to_string());
    }

    // 检查是否已在运行
    if is_process_running() {
        return Ok("moonlight-web is already running".to_string());
    }

    // 确保配置存在
    ensure_config_exists().await?;

    info!("Starting the moonlight-web service: {:?}", binary_path);

    let install_dir = get_install_dir();

    #[cfg(target_os = "windows")]
    let child = {
        use std::os::windows::process::CommandExt;
        std::process::Command::new(&binary_path)
            .current_dir(&install_dir)
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .spawn()
            .map_err(|e| format!("Failed to start moonlight-web: {}", e))?
    };

    #[cfg(not(target_os = "windows"))]
    let child = {
        std::process::Command::new(&binary_path)
            .current_dir(&install_dir)
            .spawn()
            .map_err(|e| format!("Failed to start moonlight-web: {}", e))?
    };

    let pid = child.id();
    *CHILD_PROCESS.lock().unwrap() = Some(child);

    info!("moonlight-web started, PID: {}", pid);
    Ok(format!("moonlight-web started (PID: {})", pid))
}

/// 停止 moonlight-web 服务
#[tauri::command]
pub async fn moonlight_web_stop() -> Result<String, String> {
    // 先尝试停止我们自己启动的子进程
    {
        let mut guard = CHILD_PROCESS.lock().unwrap();
        if let Some(ref mut child) = *guard {
            info!(
                "Stopping the moonlight-web child process (PID: {})",
                child.id()
            );
            let _ = child.kill();
            let _ = child.wait();
            *guard = None;
            info!("moonlight-web child process stopped");
            return Ok("moonlight-web stopped".to_string());
        }
    }

    // 如果不是我们启动的，通过 taskkill 停止
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let output = std::process::Command::new("taskkill")
            .args(["/IM", PROCESS_CHECK_NAME, "/F"])
            .creation_flags(0x08000000)
            .output()
            .map_err(|e| format!("Failed to run taskkill: {}", e))?;

        if output.status.success() {
            info!("moonlight-web process terminated");
            Ok("moonlight-web stopped".to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("taskkill output: {}", stderr);
            Ok("moonlight-web may already be stopped".to_string())
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("This operation is currently available only on Windows".to_string())
    }
}

/// 读取 moonlight-web 配置
#[tauri::command]
pub async fn moonlight_web_get_config() -> Result<MoonlightWebConfig, String> {
    let config_path = get_config_path();
    if !config_path.exists() {
        debug!("The configuration file does not exist; returning defaults");
        return Ok(MoonlightWebConfig {
            web_server: WebServerConfig::default(),
            webrtc: None,
            default_settings: None,
            extra: std::collections::HashMap::new(),
        });
    }

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read the configuration: {}", e))?;

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse the configuration: {}", e))
}

/// 保存 moonlight-web 配置
#[tauri::command]
pub async fn moonlight_web_save_config(mut config: MoonlightWebConfig) -> Result<String, String> {
    // 校验 bind_address 不为空
    if config.web_server.bind_address.trim().is_empty() {
        config.web_server.bind_address = default_bind_address();
    }

    let config_path = get_config_path();

    // 确保目录存在
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create the configuration directory: {}", e))?;
    }

    // 更新本地端口缓存
    if let Some(port) = parse_port_from_bind_address(&config.web_server.bind_address) {
        *BIND_PORT.lock().unwrap() = port;
    }

    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize the configuration: {}", e))?;

    std::fs::write(&config_path, json)
        .map_err(|e| format!("Failed to write the configuration: {}", e))?;

    info!("moonlight-web configuration saved: {:?}", config_path);
    Ok("Configuration saved".to_string())
}

/// 获取安装目录路径
#[tauri::command]
pub fn moonlight_web_get_install_path() -> String {
    get_install_dir().to_string_lossy().to_string()
}

/// 自动生成自签名 HTTPS 证书
#[tauri::command]
pub async fn moonlight_web_generate_cert() -> Result<CertificateConfig, String> {
    let install_dir = get_install_dir();
    let server_dir = install_dir.join("server");
    std::fs::create_dir_all(&server_dir)
        .map_err(|e| format!("Failed to create the directory: {}", e))?;

    let key_path = server_dir.join("key.pem");
    let cert_path = server_dir.join("cert.pem");

    info!("Generating a self-signed HTTPS certificate...");

    // 使用 rcgen 生成自签名证书
    let mut params = rcgen::CertificateParams::new(vec!["localhost".to_string()])
        .map_err(|e| format!("Failed to create certificate parameters: {}", e))?;

    // 添加 SAN（Subject Alternative Names）
    params.subject_alt_names = vec![
        rcgen::SanType::DnsName(
            "localhost"
                .try_into()
                .map_err(|e| format!("Invalid DNS name: {}", e))?,
        ),
        rcgen::SanType::IpAddress(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))),
        rcgen::SanType::IpAddress(std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0))),
    ];

    // 设置有效期 10 年
    params.not_before = rcgen::date_time_ymd(2025, 1, 1);
    params.not_after = rcgen::date_time_ymd(2035, 1, 1);

    // 设置 DN
    params.distinguished_name = rcgen::DistinguishedName::new();
    params
        .distinguished_name
        .push(rcgen::DnType::CommonName, "Moonlight Web Stream");
    params
        .distinguished_name
        .push(rcgen::DnType::OrganizationName, "Sunshine Control Panel");

    let key_pair =
        rcgen::KeyPair::generate().map_err(|e| format!("Failed to generate a key: {}", e))?;
    let cert = params
        .self_signed(&key_pair)
        .map_err(|e| format!("Failed to sign the certificate: {}", e))?;

    // 写入文件
    std::fs::write(&key_path, key_pair.serialize_pem())
        .map_err(|e| format!("Failed to write the private key: {}", e))?;
    std::fs::write(&cert_path, cert.pem())
        .map_err(|e| format!("Failed to write the certificate: {}", e))?;

    info!(
        "Self-signed certificate generated: {:?}, {:?}",
        cert_path, key_path
    );

    // 返回相对于 moonlight-web 安装目录的路径（web-server 工作目录）
    Ok(CertificateConfig {
        private_key_pem: "./server/key.pem".to_string(),
        certificate_pem: "./server/cert.pem".to_string(),
    })
}

// ========== 内部工具函数 ==========

/// 检测 moonlight-web 进程是否在运行
fn is_process_running() -> bool {
    // 先检查我们自己启动的子进程
    {
        let mut guard = CHILD_PROCESS.lock().unwrap();
        if let Some(ref mut child) = *guard {
            match child.try_wait() {
                Ok(None) => return true, // 还在运行
                Ok(Some(_)) => {
                    *guard = None;
                } // 已退出
                Err(_) => {
                    *guard = None;
                }
            }
        }
    }

    // 再通过 tasklist 检查
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        if let Ok(output) = std::process::Command::new("tasklist")
            .args([
                "/FI",
                &format!("IMAGENAME eq {}", PROCESS_CHECK_NAME),
                "/NH",
            ])
            .creation_flags(0x08000000)
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout.to_lowercase().contains("web-server");
        }
    }

    false
}

/// 获取已安装版本
fn get_installed_version() -> Option<String> {
    // 尝试从 version.txt 读取
    let version_file = get_install_dir().join("version.txt");
    if let Ok(version) = std::fs::read_to_string(&version_file) {
        let version = version.trim().to_string();
        if !version.is_empty() {
            return Some(version);
        }
    }

    // 尝试从二进制获取
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let binary = get_server_binary_path();
        if let Ok(output) = std::process::Command::new(&binary)
            .arg("--version")
            .creation_flags(0x08000000)
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // --version 输出格式: "web-server 2.4.0"，只取最后的版本号
            let version = stdout
                .trim()
                .rsplit_once(' ')
                .map(|(_, v)| v.to_string())
                .unwrap_or_else(|| stdout.trim().to_string());
            if !version.is_empty() {
                return Some(version);
            }
        }
    }

    None
}

/// 确保配置文件存在（首次启动时自动生成安全默认配置）
async fn ensure_config_exists() -> Result<(), String> {
    let config_path = get_config_path();
    if config_path.exists() {
        // 尝试加载并验证配置
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            // 用 serde_json::Value 加载，检查并修复无效字段
            if let Ok(mut raw) = serde_json::from_str::<serde_json::Value>(&content) {
                let mut fixed = false;
                if let Some(ws) = raw.get_mut("web_server").and_then(|v| v.as_object_mut()) {
                    // 修复 session_cookie_expiration: null（上游需要 Duration 结构）
                    if ws
                        .get("session_cookie_expiration")
                        .map_or(false, |v| v.is_null())
                    {
                        ws.insert(
                            "session_cookie_expiration".to_string(),
                            serde_json::json!({"secs": 86400, "nanos": 0}),
                        );
                        fixed = true;
                    }
                }
                // 移除顶级 null 字段（上游不支持 null）
                if let Some(obj) = raw.as_object_mut() {
                    let null_keys: Vec<String> = obj
                        .iter()
                        .filter(|(_, v)| v.is_null())
                        .map(|(k, _)| k.clone())
                        .collect();
                    for key in null_keys {
                        obj.remove(&key);
                        fixed = true;
                    }
                }
                if fixed {
                    if let Ok(json) = serde_json::to_string_pretty(&raw) {
                        let _ = std::fs::write(&config_path, json);
                        info!("Repaired invalid fields in config.json");
                    }
                }
            }

            // 加载端口到缓存
            if let Ok(reread) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str::<MoonlightWebConfig>(&reread) {
                    if let Some(port) =
                        parse_port_from_bind_address(&config.web_server.bind_address)
                    {
                        *BIND_PORT.lock().unwrap() = port;
                    }
                }
            }
        }
        return Ok(());
    }

    info!("Generating the default moonlight-web configuration...");

    let config = MoonlightWebConfig {
        web_server: WebServerConfig {
            bind_address: format!("0.0.0.0:{}", DEFAULT_BIND_PORT),
            certificate: None,
            url_path_prefix: String::new(),
            session_cookie_secure: false,
            session_cookie_expiration: Some(SessionExpiration {
                secs: 86400,
                nanos: 0,
            }),
            first_login_create_admin: true,
            first_login_assign_global_hosts: true,
            default_user_id: None,
            forwarded_header: None,
            extra: std::collections::HashMap::new(),
        },
        webrtc: None,
        default_settings: Some(serde_json::json!({
            "videoCodec": "h264",
            "fps": 60,
            "bitrate": 20000,
            "dataTransport": "auto",
        })),
        extra: std::collections::HashMap::new(),
    };

    // 确保目录存在
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create the configuration directory: {}", e))?;
    }

    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize the configuration: {}", e))?;

    std::fs::write(&config_path, json)
        .map_err(|e| format!("Failed to write the configuration: {}", e))?;

    info!("Default configuration generated: {:?}", config_path);
    Ok(())
}

/// 从 bind_address 中解析端口
fn parse_port_from_bind_address(addr: &str) -> Option<u16> {
    addr.rsplit(':').next()?.parse().ok()
}

/// 检查配置中是否启用了 HTTPS 证书
fn has_https_certificate() -> bool {
    let config_path = get_config_path();
    if let Ok(content) = std::fs::read_to_string(&config_path) {
        if let Ok(config) = serde_json::from_str::<MoonlightWebConfig>(&content) {
            return config.web_server.certificate.is_some();
        }
    }
    false
}

/// 应用退出时清理子进程
pub fn cleanup() {
    let mut guard = CHILD_PROCESS.lock().unwrap();
    if let Some(ref mut child) = *guard {
        info!(
            "Cleaning up the moonlight-web child process (PID: {})",
            child.id()
        );
        let _ = child.kill();
        let _ = child.wait();
        *guard = None;
    }
}
