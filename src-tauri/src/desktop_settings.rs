use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use tauri::AppHandle;
#[cfg(debug_assertions)]
use tauri::Manager;

const SETTINGS_FILE: &str = "desktop-settings.json";
const RUN_VALUE_NAME: &str = "Sunshine GUI Desktop";
const REMOVE_AUTO_START_ARG: &str = "--remove-autostart";
static DESKTOP_SETTINGS_LOCK: Mutex<()> = Mutex::new(());

fn lock_desktop_settings() -> MutexGuard<'static, ()> {
    DESKTOP_SETTINGS_LOCK.lock().unwrap()
}

pub fn try_remove_auto_start_from_args() -> bool {
    if !std::env::args().any(|arg| arg == REMOVE_AUTO_START_ARG) {
        return false;
    }

    #[cfg(target_os = "windows")]
    {
        use winreg::RegKey;
        use winreg::enums::*;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(run_key) = hkcu.open_subkey_with_flags(
            r"Software\Microsoft\Windows\CurrentVersion\Run",
            KEY_SET_VALUE,
        ) {
            let _ = run_key.delete_value(RUN_VALUE_NAME);
        }
    }
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct DesktopSettings {
    pub auto_start: bool,
    pub start_minimized: bool,
    pub auto_start_sunshine: bool,
    pub file_mapping_menu_enabled: bool,
    pub notifications: bool,
    pub connection_notify: bool,
    pub dev_mode: bool,
    pub log_level: String,
}

impl Default for DesktopSettings {
    fn default() -> Self {
        Self {
            auto_start: true,
            start_minimized: true,
            auto_start_sunshine: true,
            file_mapping_menu_enabled: true,
            notifications: true,
            connection_notify: true,
            dev_mode: false,
            log_level: "info".to_string(),
        }
    }
}

fn settings_dir() -> Result<PathBuf, String> {
    dirs::config_dir()
        .map(|dir| dir.join("Sunshine GUI"))
        .ok_or_else(|| "Cannot resolve user config directory".to_string())
}

fn settings_path() -> Result<PathBuf, String> {
    Ok(settings_dir()?.join(SETTINGS_FILE))
}

fn normalize(settings: &mut DesktopSettings) {
    let level = settings.log_level.trim().to_ascii_lowercase();
    settings.log_level = match level.as_str() {
        "error" | "warn" | "info" | "debug" | "trace" => level,
        _ => "info".to_string(),
    };
}

pub fn load_desktop_settings_from_disk() -> DesktopSettings {
    let Ok(path) = settings_path() else {
        return DesktopSettings::default();
    };
    let Ok(text) = fs::read_to_string(path) else {
        return DesktopSettings::default();
    };
    let mut settings = serde_json::from_str::<DesktopSettings>(&text).unwrap_or_default();
    normalize(&mut settings);
    settings
}

fn save_desktop_settings_to_disk(settings: &DesktopSettings) -> Result<(), String> {
    let dir = settings_dir()?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let text = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(dir.join(SETTINGS_FILE), text).map_err(|e| e.to_string())
}

pub fn set_file_mapping_menu_enabled(enabled: bool) -> Result<(), String> {
    let _settings_guard = lock_desktop_settings();
    let mut settings = load_desktop_settings_from_disk();
    settings.file_mapping_menu_enabled = enabled;
    save_desktop_settings_to_disk(&settings)
}

#[cfg(target_os = "windows")]
fn startup_command(start_minimized: bool) -> Result<String, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    if start_minimized {
        Ok(format!("\"{}\" --hidden", exe.display()))
    } else {
        Ok(format!("\"{}\"", exe.display()))
    }
}

#[cfg(target_os = "windows")]
fn apply_auto_start(settings: &DesktopSettings) -> Result<(), String> {
    use winreg::RegKey;
    use winreg::enums::*;

    let managed_service_mode = managed_service_start_mode()?;
    let register_gui = should_register_gui_auto_start(settings.auto_start, managed_service_mode);
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (run_key, _) = hkcu
        .create_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run")
        .map_err(|e| e.to_string())?;
    let current = run_key.get_value::<String, _>(RUN_VALUE_NAME).ok();
    if register_gui {
        let command = startup_command(settings.start_minimized)?;
        // This reconciliation runs on every launch, so only touch the registry
        // when the stored command actually differs.
        if current.as_deref() != Some(command.as_str()) {
            run_key
                .set_value(RUN_VALUE_NAME, &command)
                .map_err(|e| e.to_string())?;
        }
    } else {
        let service_owns_startup = service_starts_at_boot(managed_service_mode);
        // Only drop the entry when the user turned auto-start off, or when the
        // service itself starts Sunshine at boot. Removing it in any other state
        // would delete the last remaining startup hook and undo a manual repair
        // on every single launch.
        if current.is_some() && (!settings.auto_start || service_owns_startup) {
            let _ = run_key.delete_value(RUN_VALUE_NAME);
            if settings.auto_start {
                log::debug!(
                    "Sunshine service starts at boot and owns the GUI agent; removed the redundant Run entry"
                );
            }
        }
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn apply_auto_start(_settings: &DesktopSettings) -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn is_auto_start_registered() -> bool {
    use winreg::RegKey;
    use winreg::enums::*;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    hkcu.open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run")
        .and_then(|key| key.get_value::<String, _>(RUN_VALUE_NAME))
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

#[cfg(not(target_os = "windows"))]
pub fn is_auto_start_registered() -> bool {
    false
}

#[cfg(target_os = "windows")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ServiceStartMode {
    Auto,
    DelayedAuto,
    Demand,
    Disabled,
}

#[cfg(target_os = "windows")]
fn sunshine_service_start_mode() -> Result<Option<ServiceStartMode>, String> {
    use winreg::RegKey;
    use winreg::enums::*;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let service = match hklm.open_subkey(r"SYSTEM\CurrentControlSet\Services\SunshineService") {
        Ok(service) => service,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    let start = service
        .get_value::<u32, _>("Start")
        .map_err(|error| error.to_string())?;
    match start {
        2 if service.get_value::<u32, _>("DelayedAutoStart").unwrap_or(0) != 0 => {
            Ok(Some(ServiceStartMode::DelayedAuto))
        }
        2 => Ok(Some(ServiceStartMode::Auto)),
        3 => Ok(Some(ServiceStartMode::Demand)),
        4 => Ok(Some(ServiceStartMode::Disabled)),
        value => Err(format!("Unsupported Sunshine service start type: {value}")),
    }
}

#[cfg(target_os = "windows")]
fn service_image_path_matches_install(image_path: &str, install_dir: &Path) -> bool {
    let image_path = image_path.trim().trim_matches('"');
    let expected = install_dir.join("tools").join("sunshinesvc.exe");

    image_path.eq_ignore_ascii_case(&expected.to_string_lossy())
}

#[cfg(target_os = "windows")]
fn sunshine_service_matches_install(install_dir: &Path) -> Result<bool, String> {
    use winreg::RegKey;
    use winreg::enums::*;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let service = match hklm.open_subkey(r"SYSTEM\CurrentControlSet\Services\SunshineService") {
        Ok(service) => service,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error.to_string()),
    };
    let image_path = service
        .get_value::<String, _>("ImagePath")
        .map_err(|error| error.to_string())?;

    Ok(service_image_path_matches_install(&image_path, install_dir))
}

#[cfg(target_os = "windows")]
fn managed_service_start_mode() -> Result<Option<ServiceStartMode>, String> {
    let install_dir = crate::sunshine::install_dir();
    if sunshine_service_matches_install(&install_dir)? {
        sunshine_service_start_mode()
    } else {
        Ok(None)
    }
}

/// Reports whether the SCM starts the managed service on its own at boot.
#[cfg(target_os = "windows")]
fn service_starts_at_boot(managed_service_mode: Option<ServiceStartMode>) -> bool {
    matches!(
        managed_service_mode,
        Some(ServiceStartMode::Auto | ServiceStartMode::DelayedAuto)
    )
}

#[cfg(target_os = "windows")]
fn should_register_gui_auto_start(
    requested: bool,
    managed_service_mode: Option<ServiceStartMode>,
) -> bool {
    // The mere existence of a service does not make it a startup hook: the SCM
    // never launches a Demand/Disabled service at boot, so the user-session Run
    // entry has to stay registered in that case. Only an Auto/DelayedAuto
    // service owns startup and makes the entry redundant.
    requested && !service_starts_at_boot(managed_service_mode)
}

#[cfg(target_os = "windows")]
fn combined_auto_start_ready(
    managed_service_mode: Option<ServiceStartMode>,
    gui_registered: bool,
) -> bool {
    // Mirror of `should_register_gui_auto_start`: whenever the service does not
    // start itself, the Run entry is the only hook that can bring Sunshine up.
    service_starts_at_boot(managed_service_mode) || gui_registered
}

#[cfg(target_os = "windows")]
pub fn is_combined_auto_start_enabled() -> bool {
    managed_service_start_mode()
        .map(|mode| combined_auto_start_ready(mode, is_auto_start_registered()))
        .unwrap_or(false)
}

#[cfg(not(target_os = "windows"))]
pub fn is_combined_auto_start_enabled() -> bool {
    false
}

#[cfg(target_os = "windows")]
fn configure_sunshine_service_start(mode: ServiceStartMode) -> Result<(), String> {
    let start_mode = match mode {
        ServiceStartMode::Auto => "auto",
        ServiceStartMode::DelayedAuto => "delayed-auto",
        ServiceStartMode::Demand => "demand",
        ServiceStartMode::Disabled => "disabled",
    };
    let script_path = std::env::temp_dir().join("sunshine-set-service-start.bat");
    let script = format!(
        "@echo off\r\nsc.exe config SunshineService start= {start_mode}\r\nexit /b %ERRORLEVEL%\r\n"
    );
    fs::write(&script_path, script).map_err(|e| e.to_string())?;
    let result = crate::bat_runner::run_elevated(&script_path, "service-start", &[]);
    let _ = fs::remove_file(&script_path);
    result
}

/// Picks the service start type that satisfies the requested auto-start
/// preference, or `None` when the current one already does. Returning `None`
/// keeps a plain settings save from raising a UAC prompt for a no-op change,
/// and keeps an explicitly disabled service disabled when auto-start is off.
#[cfg(target_os = "windows")]
fn desired_service_start_mode(
    enabled: bool,
    current: ServiceStartMode,
) -> Option<ServiceStartMode> {
    if enabled == service_starts_at_boot(Some(current)) {
        return None;
    }
    Some(if enabled {
        // Delayed start keeps boot fast and lets the GPU/driver stack settle
        // before capture starts.
        ServiceStartMode::DelayedAuto
    } else {
        ServiceStartMode::Demand
    })
}

pub fn set_combined_auto_start_enabled(enabled: bool) -> Result<(), String> {
    let previous = load_desktop_settings_from_disk();
    let mut settings = previous.clone();
    settings.auto_start = enabled;
    settings.auto_start_sunshine = enabled;

    #[cfg(target_os = "windows")]
    let previous_service_mode = managed_service_start_mode()?;
    #[cfg(target_os = "windows")]
    let mut service_changed = false;
    #[cfg(target_os = "windows")]
    if let Some(mode) = previous_service_mode
        && let Some(target_mode) = desired_service_start_mode(enabled, mode)
    {
        configure_sunshine_service_start(target_mode)?;
        service_changed = true;
    }

    let _settings_guard = lock_desktop_settings();
    if let Err(error) = save_desktop_settings_to_disk(&settings) {
        #[cfg(target_os = "windows")]
        if service_changed && let Some(mode) = previous_service_mode {
            let _ = configure_sunshine_service_start(mode);
        }
        return Err(error);
    }

    if let Err(error) = apply_auto_start(&settings) {
        let _ = save_desktop_settings_to_disk(&previous);
        #[cfg(target_os = "windows")]
        if service_changed && let Some(mode) = previous_service_mode {
            let _ = configure_sunshine_service_start(mode);
        }
        return Err(error);
    }

    Ok(())
}

pub fn apply_startup_settings(app: &AppHandle, settings: &DesktopSettings) {
    crate::logger::set_log_level(&settings.log_level);
    apply_dev_mode(app, settings.dev_mode);
    if let Err(e) = apply_auto_start(settings) {
        log::warn!("Failed to reconcile GUI auto-start registration: {}", e);
    }
    if settings.auto_start_sunshine {
        ensure_sunshine_started_async();
    }
}

fn apply_dev_mode(_app: &AppHandle, enabled: bool) {
    if !enabled {
        return;
    }

    #[cfg(debug_assertions)]
    {
        for label in ["main", "desktop", "log_console"] {
            if let Some(window) = _app.get_webview_window(label) {
                window.open_devtools();
            }
        }
    }
}

pub fn ensure_sunshine_started_async() {
    tauri::async_runtime::spawn(async {
        if let Err(e) = ensure_sunshine_started().await {
            log::warn!("Failed to auto-start Sunshine: {}", e);
        }
    });
}

async fn ensure_sunshine_started() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;

        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let already_running = tokio::task::spawn_blocking(|| {
            std::process::Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-Command",
                    "if (Get-Process sunshine -ErrorAction SilentlyContinue) { 'true' } else { 'false' }",
                ])
                .creation_flags(CREATE_NO_WINDOW)
                .output()
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
        .map(|output| String::from_utf8_lossy(&output.stdout).contains("true"))?;

        if already_running {
            return Ok(());
        }

        let install_dir = crate::sunshine::install_dir();
        let sunshine_exe = install_dir.join("sunshine.exe");
        if !sunshine_exe.exists() {
            return Err(format!(
                "sunshine.exe not found: {}",
                sunshine_exe.display()
            ));
        }

        if sunshine_service_matches_install(&install_dir)? {
            std::process::Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-Command",
                    "Start-Service -Name 'SunshineService'",
                ])
                .creation_flags(CREATE_NO_WINDOW)
                .spawn()
                .map_err(|e| e.to_string())?;
        } else {
            std::process::Command::new(&sunshine_exe)
                .current_dir(&install_dir)
                .creation_flags(CREATE_NO_WINDOW)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(())
    }
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;

    #[test]
    fn notification_preferences_are_enabled_by_default() {
        let settings = DesktopSettings::default();

        assert!(settings.notifications);
        assert!(settings.connection_notify);
    }

    #[test]
    fn missing_notification_preferences_migrate_to_enabled() {
        let settings: DesktopSettings = serde_json::from_str(r#"{"autoStart":false}"#)
            .expect("legacy desktop settings should deserialize");

        assert!(settings.notifications);
        assert!(settings.connection_notify);
    }

    #[test]
    fn boot_starting_service_owns_startup() {
        assert!(combined_auto_start_ready(
            Some(ServiceStartMode::Auto),
            false
        ));
        assert!(combined_auto_start_ready(
            Some(ServiceStartMode::DelayedAuto),
            false
        ));
        assert!(!should_register_gui_auto_start(
            true,
            Some(ServiceStartMode::Auto)
        ));
        assert!(!should_register_gui_auto_start(
            true,
            Some(ServiceStartMode::DelayedAuto)
        ));
    }

    #[test]
    fn demand_service_falls_back_to_gui_registration() {
        // The SCM never starts a Demand/Disabled service at boot, so the Run
        // entry stays the only startup hook and decides readiness.
        assert!(should_register_gui_auto_start(
            true,
            Some(ServiceStartMode::Demand)
        ));
        assert!(should_register_gui_auto_start(
            true,
            Some(ServiceStartMode::Disabled)
        ));
        assert!(combined_auto_start_ready(
            Some(ServiceStartMode::Demand),
            true
        ));
        assert!(!combined_auto_start_ready(
            Some(ServiceStartMode::Demand),
            false
        ));
        assert!(!combined_auto_start_ready(
            Some(ServiceStartMode::Disabled),
            false
        ));
    }

    #[test]
    fn service_free_startup_uses_gui_registration() {
        assert!(combined_auto_start_ready(None, true));
        assert!(!combined_auto_start_ready(None, false));
        assert!(should_register_gui_auto_start(true, None));
        assert!(!should_register_gui_auto_start(false, None));
    }

    #[test]
    fn service_start_mode_changes_only_when_needed() {
        assert_eq!(
            desired_service_start_mode(true, ServiceStartMode::Demand),
            Some(ServiceStartMode::DelayedAuto)
        );
        assert_eq!(
            desired_service_start_mode(true, ServiceStartMode::Disabled),
            Some(ServiceStartMode::DelayedAuto)
        );
        assert_eq!(
            desired_service_start_mode(true, ServiceStartMode::Auto),
            None
        );
        assert_eq!(
            desired_service_start_mode(true, ServiceStartMode::DelayedAuto),
            None
        );
        assert_eq!(
            desired_service_start_mode(false, ServiceStartMode::DelayedAuto),
            Some(ServiceStartMode::Demand)
        );
        assert_eq!(
            desired_service_start_mode(false, ServiceStartMode::Demand),
            None
        );
        assert_eq!(
            desired_service_start_mode(false, ServiceStartMode::Disabled),
            None
        );
    }

    #[test]
    fn service_must_belong_to_the_same_install() {
        let install_dir = Path::new(r"C:\Portable Sunshine");

        assert!(service_image_path_matches_install(
            r#""C:\Portable Sunshine\tools\sunshinesvc.exe""#,
            install_dir
        ));
        assert!(!service_image_path_matches_install(
            r#""C:\Program Files\Sunshine\tools\sunshinesvc.exe""#,
            install_dir
        ));
    }
}
