// ControllerMeta 集成模块
//
// ControllerMeta (https://github.com/HK560/ControllerMeta) 是一个基于 WebView2 的
// 高精度手柄分析工具。官方发布形式为单个 Windows EXE，本模块负责：
//   1. 检测本地是否已安装（%LOCALAPPDATA%\Sunshine\controllermeta\ControllerMeta.exe）
//   2. 启动 EXE（GUI 进程，不捕获窗口）

use log::info;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

// ========== 常量 ==========

const BINARY_NAME: &str = "ControllerMeta.exe";
const INSTALL_SUBDIR: &str = "controllermeta";

/// 已启动的子进程句柄（仅用于判断是否由我们拉起）
static CHILD_PROCESS: Lazy<Mutex<Option<std::process::Child>>> = Lazy::new(|| Mutex::new(None));

// ========== 数据结构 ==========

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ControllerMetaStatus {
    /// 是否已安装
    pub installed: bool,
    /// 是否正在运行
    pub running: bool,
    /// 安装目录
    pub install_path: String,
    /// 可执行文件完整路径
    pub binary_path: String,
    /// 已安装版本（来自 version.txt）
    pub version: String,
}

// ========== 路径工具 ==========

fn get_install_dir() -> PathBuf {
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        return PathBuf::from(local_app_data)
            .join("Sunshine")
            .join(INSTALL_SUBDIR);
    }
    // fallback：与控制面板同级
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            return parent.join(INSTALL_SUBDIR);
        }
    }
    PathBuf::from(format!("./{}", INSTALL_SUBDIR))
}

fn get_binary_path() -> PathBuf {
    get_install_dir().join(BINARY_NAME)
}

fn get_version_file() -> PathBuf {
    get_install_dir().join("version.txt")
}

fn read_installed_version() -> String {
    std::fs::read_to_string(get_version_file())
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

// ========== 进程检查 ==========

fn is_process_running() -> bool {
    // 先检查我们自己 spawn 的子进程
    {
        let mut guard = CHILD_PROCESS.lock().unwrap();
        if let Some(ref mut child) = *guard {
            match child.try_wait() {
                Ok(None) => return true,
                Ok(Some(_)) | Err(_) => *guard = None,
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        if let Ok(output) = std::process::Command::new("tasklist")
            .args(["/FI", &format!("IMAGENAME eq {}", BINARY_NAME), "/NH"])
            .creation_flags(0x08000000)
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout.to_lowercase().contains(&BINARY_NAME.to_lowercase());
        }
    }

    false
}

// ========== Tauri 命令 ==========

#[tauri::command]
pub async fn controllermeta_get_status() -> Result<ControllerMetaStatus, String> {
    let install_dir = get_install_dir();
    let binary_path = get_binary_path();
    let installed = binary_path.exists();
    let running = is_process_running();
    let version = if installed {
        read_installed_version()
    } else {
        String::new()
    };

    Ok(ControllerMetaStatus {
        installed,
        running,
        install_path: install_dir.to_string_lossy().to_string(),
        binary_path: binary_path.to_string_lossy().to_string(),
        version,
    })
}

/// 启动 ControllerMeta（GUI 进程，fire-and-forget）
#[tauri::command]
pub async fn controllermeta_launch() -> Result<String, String> {
    let binary_path = get_binary_path();
    if !binary_path.exists() {
        return Err("ControllerMeta is not installed".to_string());
    }

    // 如果已在运行，不再重复启动
    if is_process_running() {
        info!("ControllerMeta is already running");
        return Ok("already-running".to_string());
    }

    let install_dir = get_install_dir();
    info!("Starting ControllerMeta: {:?}", binary_path);

    // 下载的 exe 会被 Windows 打上 Zone.Identifier（MOTW）标记，
    // 某些场景下会导致运行时 UAC 拦截或被 SmartScreen 阻断，先清理掉
    #[cfg(target_os = "windows")]
    {
        let zone_ads = format!("{}:Zone.Identifier", binary_path.to_string_lossy());
        let _ = std::fs::remove_file(&zone_ads);
    }

    #[cfg(target_os = "windows")]
    let child = {
        use std::os::windows::process::CommandExt;
        // DETACHED_PROCESS(0x00000008)：与父控制台脱钩
        // 不使用 CREATE_BREAKAWAY_FROM_JOB：若父进程所在的 Job 对象禁止 breakaway
        // 会触发 ERROR_ACCESS_DENIED (os error 5)
        std::process::Command::new(&binary_path)
            .current_dir(&install_dir)
            .creation_flags(0x00000008)
            .spawn()
            .map_err(|e| format!("Failed to start ControllerMeta: {}", e))?
    };

    #[cfg(not(target_os = "windows"))]
    let child = std::process::Command::new(&binary_path)
        .current_dir(&install_dir)
        .spawn()
        .map_err(|e| format!("Failed to start ControllerMeta: {}", e))?;

    let pid = child.id();
    *CHILD_PROCESS.lock().unwrap() = Some(child);
    Ok(format!("started:{}", pid))
}

#[tauri::command]
pub fn controllermeta_get_install_path() -> String {
    get_install_dir().to_string_lossy().to_string()
}

/// 卸载：删除整个安装目录
#[tauri::command]
pub async fn controllermeta_uninstall() -> Result<String, String> {
    if is_process_running() {
        return Err("ControllerMeta is running. Close it before uninstalling".to_string());
    }
    let install_dir = get_install_dir();
    if !install_dir.exists() {
        return Ok("not-installed".to_string());
    }
    std::fs::remove_dir_all(&install_dir)
        .map_err(|e| format!("Failed to remove the installation directory: {}", e))?;
    info!("ControllerMeta uninstalled: {:?}", install_dir);
    Ok("uninstalled".to_string())
}
