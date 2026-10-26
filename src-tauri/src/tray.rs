use log::{debug, error, info, warn};
use std::{
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};
use tauri::{
    AppHandle, Emitter, Manager, Runtime,
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
};

#[cfg(target_os = "windows")]
use crate::desktop_settings;
use crate::{proxy_server, sunshine, tray_config, utils, windows};

mod actions;
mod events;
mod icons;
mod main_panel;
mod menu;
mod notifications;
#[cfg(target_os = "windows")]
mod vdd_confirmation;

pub use actions::{cleanup_prevent_sleep, handle_tray_menu_event};
use menu::{build_tray_menu, tray_status_label};
#[cfg(test)]
use menu::{compact_menu_text, tray_notification_label};

#[cfg(target_os = "windows")]
pub(crate) fn default_window_icon_path() -> Result<std::path::PathBuf, String> {
    notifications::ensure_windows_notification_icon()
}

// 托盘图标 ID
const TRAY_ID: &str = "main-tray";

// 防止睡眠状态管理
static PREVENT_SLEEP_STATE: Mutex<bool> = Mutex::new(false);

// Sunshine 用户模式状态管理
#[cfg(target_os = "windows")]
static SUNSHINE_USER_MODE_STATE: Mutex<bool> = Mutex::new(false);

// Last icon name applied from the Sunshine core state. This avoids repeatedly
// decoding the same .ico file during the polling loop.
static CURRENT_CORE_ICON: Mutex<Option<String>> = Mutex::new(None);
static MONITORED_STATE_RECEIPT: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CoreConnectionState {
    Connecting,
    Connected,
    Disconnected,
}

#[derive(Default)]
struct RecoveryState {
    last_attempt: u64,
    active_attempt: Option<(u64, u64)>,
}

impl RecoveryState {
    fn begin(&mut self, monitored_state_baseline: u64) -> Option<u64> {
        if self.active_attempt.is_some() {
            return None;
        }
        self.last_attempt = self.last_attempt.wrapping_add(1).max(1);
        self.active_attempt = Some((self.last_attempt, monitored_state_baseline));
        Some(self.last_attempt)
    }

    fn finish(&mut self, attempt: u64) -> bool {
        if self.active_attempt.map(|active| active.0) != Some(attempt) {
            return false;
        }
        self.active_attempt = None;
        true
    }

    fn complete_from_monitored_state(&mut self, receipt: u64) -> bool {
        let Some((_, baseline)) = self.active_attempt else {
            return false;
        };
        if receipt <= baseline {
            return false;
        }
        self.active_attempt = None;
        true
    }

    fn is_in_progress(&self) -> bool {
        self.active_attempt.is_some()
    }
}

struct TrayRuntimeState {
    connection: CoreConnectionState,
    tray_state: Option<sunshine::TrayState>,
    recovery: RecoveryState,
}

impl TrayRuntimeState {
    fn menu_snapshot(&self) -> (Option<sunshine::TrayState>, CoreConnectionState, bool) {
        (
            self.tray_state.clone(),
            self.connection,
            self.recovery.is_in_progress(),
        )
    }

    fn begin_recovery_if_disconnected(&mut self, monitored_state_baseline: u64) -> Option<u64> {
        if self.connection != CoreConnectionState::Disconnected {
            return None;
        }
        self.recovery.begin(monitored_state_baseline)
    }

    fn apply_connected_state(
        &mut self,
        state: &sunshine::TrayState,
        monitored_state_receipt: Option<u64>,
    ) -> Option<(Option<sunshine::TrayState>, bool)> {
        if monitored_state_receipt.is_none() && self.connection != CoreConnectionState::Connected {
            return None;
        }
        // Only a monitor response accepted after recovery began is authoritative.
        // Action responses may already be in flight when the attempt starts.
        let recovery_completed = monitored_state_receipt
            .is_some_and(|receipt| self.recovery.complete_from_monitored_state(receipt));
        let connection_changed = self.connection != CoreConnectionState::Connected;
        self.connection = CoreConnectionState::Connected;

        let previous_state = self.tray_state.clone();
        let state_changed = self.tray_state.as_ref() != Some(state);
        self.tray_state = Some(state.clone());

        Some((
            previous_state,
            connection_changed || state_changed || recovery_completed,
        ))
    }

    fn mark_disconnected(&mut self) -> bool {
        let connection_changed = self.connection != CoreConnectionState::Disconnected;
        self.connection = CoreConnectionState::Disconnected;
        connection_changed || self.tray_state.take().is_some()
    }
}

static TRAY_RUNTIME_STATE: Mutex<TrayRuntimeState> = Mutex::new(TrayRuntimeState {
    connection: CoreConnectionState::Connecting,
    tray_state: None,
    recovery: RecoveryState {
        last_attempt: 0,
        active_attempt: None,
    },
});

/// Sunshine 核心是否处于活跃会话（推流中或暂停）。paused 会话仍持有资源，
/// 恢复瞬间不能处于效率模式节流之下。
pub(crate) fn is_tray_session_active() -> bool {
    TRAY_RUNTIME_STATE
        .lock()
        .map(|runtime| {
            runtime
                .tray_state
                .as_ref()
                .is_some_and(|state| matches!(state.status.as_str(), "streaming" | "paused"))
        })
        .unwrap_or(false)
}

static MAIN_PANEL_BRIDGE: main_panel::Bridge = main_panel::Bridge::new();

pub(crate) fn emit_message<R: Runtime>(app: &AppHandle<R>, msg_type: &str, message: &str) {
    let payload = serde_json::json!({
        "type": msg_type,
        "message": message
    });
    let mut has_visible_window = false;

    for label in ["main", "desktop"] {
        if let Some(window) = app.get_webview_window(label) {
            has_visible_window |= window.is_visible().unwrap_or(false);
            let _ = window.emit("show-message", &payload);
        }
    }

    if msg_type == "error" && !has_visible_window {
        use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

        app.dialog()
            .message(message)
            .title("Sunshine error")
            .kind(MessageDialogKind::Error)
            .show(|_| {});
    }
}

/// 托盘菜单翻译结构
struct TrayStrings {
    status_idle: &'static str,
    status_streaming: &'static str,
    status_paused: &'static str,
    status_pairing: &'static str,
    status_notification: &'static str,
    status_connecting: &'static str,
    status_disconnected: &'static str,
    notification: &'static str,
    client_connected: &'static str,
    client_connected_detail: &'static str,
    client_connected_named: &'static str,
    suspicious_client_warning: &'static str,
    client_disconnected: &'static str,
    client_disconnected_detail: &'static str,
    client_disconnected_named: &'static str,
    incoming_pairing: &'static str,
    pairing_instruction: &'static str,
    pairing_menu_instruction: &'static str,
    complete_pairing: &'static str,
    open_main_panel: &'static str,
    interfaces_menu: &'static str,
    display_menu: &'static str,
    tools_menu: &'static str,
    settings_menu: &'static str,
    advanced_menu: &'static str,
    help_menu: &'static str,
    open_sunshine: &'static str,
    vdd_settings: &'static str,
    vdd_create: &'static str,
    vdd_create_confirm: &'static str,
    vdd_close: &'static str,
    vdd_keep: &'static str,
    vdd_keep_confirm: &'static str,
    vdd_headless: &'static str,
    vdd_headless_confirm: &'static str,
    import_config: &'static str,
    export_config: &'static str,
    reset_config: &'static str,
    reset_config_confirm: &'static str,
    clear_cache: &'static str,
    clear_cache_confirm: &'static str,
    reset_display: &'static str,
    reset_display_confirm: &'static str,
    restart_user_mode: &'static str,
    prevent_sleep: &'static str,
    rtss_control: &'static str,
    host_performance: &'static str,
    log_console: &'static str,
    #[cfg(target_os = "windows")]
    auto_start: &'static str,
    #[cfg(any(debug_assertions, feature = "beta"))]
    web_stream: &'static str,
    about: &'static str,
    shutdown: &'static str,
    shutdown_message: &'static str,
    restart: &'static str,
    recover_service: &'static str,
    recovery_timeout: &'static str,
    tooltip: &'static str,
    tooltip_admin: &'static str,
}

const EN_STRINGS: TrayStrings = TrayStrings {
    status_idle: "Idle",
    status_streaming: "Streaming",
    status_paused: "Stream paused",
    status_pairing: "Pairing",
    status_notification: "New notification",
    status_connecting: "Connecting",
    status_disconnected: "Disconnected",
    notification: "Notification",
    client_connected: "Client connected",
    client_connected_detail: "A new streaming connection is ready.",
    client_connected_named: "“{name}” connected to this computer.",
    suspicious_client_warning: "Warning: You are very likely being affected by an unknown infringing client. This heuristic assessment may be inaccurate.",
    client_disconnected: "Client disconnected",
    client_disconnected_detail: "The streaming connection has ended.",
    client_disconnected_named: "“{name}” disconnected from this computer.",
    incoming_pairing: "Moonlight pairing request",
    pairing_instruction: "Click this notification to enter the 4-digit PIN shown in Moonlight.",
    pairing_menu_instruction: "Open “Complete pairing” from the Sunshine tray menu and enter the 4-digit PIN shown in Moonlight.",
    complete_pairing: "Complete pairing",
    open_main_panel: "Open main panel",
    interfaces_menu: "Other interfaces",
    display_menu: "Display",
    tools_menu: "Tools",
    settings_menu: "Settings",
    advanced_menu: "Advanced",
    help_menu: "Help",
    open_sunshine: "Web management UI",
    vdd_settings: "Virtual display settings",
    vdd_create: "Create virtual display",
    vdd_create_confirm: "This creates a virtual display and may change the Windows display layout. Continue?",
    vdd_close: "Close virtual display",
    vdd_keep: "Keep after streaming",
    vdd_keep_confirm: "Keep mode leaves the virtual display enabled after streaming ends and may change the Windows display layout. Continue?",
    vdd_headless: "Auto-create when headless",
    vdd_headless_confirm: "This experimental recovery feature creates a virtual display when no display is detected and may change the active display layout. Continue?",
    import_config: "Import config",
    export_config: "Export config",
    reset_config: "Reset config",
    reset_config_confirm: "This clears sunshine.conf and restores Sunshine defaults. Continue?",
    clear_cache: "Clear cache",
    clear_cache_confirm: "This terminates the running application so cached state can be rebuilt. Continue?",
    reset_display: "Reset display",
    reset_display_confirm: "This resets persisted display-device state. Continue?",
    restart_user_mode: "Run Sunshine in user mode",
    prevent_sleep: "Prevent system sleep",
    rtss_control: "RTSS control",
    host_performance: "Host performance",
    log_console: "Log console",
    #[cfg(target_os = "windows")]
    auto_start: "Run at startup",
    #[cfg(any(debug_assertions, feature = "beta"))]
    web_stream: "Web streaming",
    about: "About Sunshine",
    shutdown: "Exit Sunshine",
    shutdown_message: "This will stop the Sunshine service, GUI, and tray. Continue?",
    restart: "Restart Sunshine",
    recover_service: "Restart Sunshine service",
    recovery_timeout: "The Sunshine service did not recover in time. Check the service log and try again.",
    tooltip: "Sunshine GUI",
    tooltip_admin: "Sunshine GUI (admin)",
};

fn get_tray_strings() -> &'static TrayStrings {
    &EN_STRINGS
}

#[cfg(target_os = "windows")]
mod power {
    const ES_CONTINUOUS: u32 = 0x80000000;
    const ES_SYSTEM_REQUIRED: u32 = 0x00000001;
    const ES_AWAYMODE_REQUIRED: u32 = 0x00000040;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn SetThreadExecutionState(es_flags: u32) -> u32;
    }

    pub fn enable_prevent_sleep() -> Result<(), &'static str> {
        let flags = ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_AWAYMODE_REQUIRED;
        unsafe {
            if SetThreadExecutionState(flags) == 0 {
                return Err("SetThreadExecutionState failed");
            }
        }
        Ok(())
    }

    pub fn disable_prevent_sleep() -> Result<(), &'static str> {
        unsafe {
            if SetThreadExecutionState(ES_CONTINUOUS) == 0 {
                return Err("SetThreadExecutionState failed");
            }
        }
        Ok(())
    }
}

/// 创建系统托盘
pub fn create_system_tray<R: Runtime + 'static>(app: &AppHandle<R>) -> tauri::Result<()> {
    #[cfg(target_os = "windows")]
    init_sunshine_user_mode_state(app);

    if let Err(error) = notifications::initialize(app) {
        warn!("{}", error);
    }

    build_owned_system_tray(app)?;
    events::start_tray_state_monitoring(app);

    Ok(())
}

fn build_owned_system_tray<R: Runtime + 'static>(app: &AppHandle<R>) -> tauri::Result<()> {
    if app.tray_by_id(TRAY_ID).is_some() {
        return Ok(());
    }
    let menu = build_tray_menu(app)?;
    let tooltip = default_tray_tooltip();
    let tray_icon = icons::load_initial_tray_icon(app);

    TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .icon(tray_icon)
        .tooltip(tooltip)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| handle_tray_menu_event(app, event.id().as_ref()))
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                ..
            } => handle_tray_click(tray.app_handle()),
            TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } => handle_tray_double_click(tray.app_handle()),
            _ => {}
        })
        .build(app)?;

    info!("GUI tray created for the current user session");

    Ok(())
}

fn open_main_panel_from_tray<R: Runtime>(app: &AppHandle<R>, source: &str) {
    if let Some(window) = app.get_webview_window("main") {
        windows::show_and_activate_window(&window);
        return;
    }

    if let Err(e) = windows::create_main_window(app) {
        error!("Failed to open main panel from tray {}: {}", source, e);
    }
}

pub fn mark_main_panel_loading() {
    MAIN_PANEL_BRIDGE.mark_loading();
}

fn emit_to_main_when_ready<R: Runtime>(
    app: &AppHandle<R>,
    event: &str,
    payload: serde_json::Value,
) {
    MAIN_PANEL_BRIDGE.emit_or_queue(app, event, payload);
}

#[tauri::command]
pub fn main_panel_loading() {
    mark_main_panel_loading();
}

#[tauri::command]
pub fn main_panel_ready(app: AppHandle) {
    MAIN_PANEL_BRIDGE.mark_ready(&app);
}

fn default_tray_tooltip() -> &'static str {
    let s = get_tray_strings();
    if utils::is_running_as_admin().unwrap_or(false) {
        s.tooltip_admin
    } else {
        s.tooltip
    }
}

fn tray_tooltip_from_state(state: &sunshine::TrayState) -> String {
    let tooltip = state.tooltip.trim();
    if !(tooltip.is_empty() || state.status == "idle" && tooltip == "Sunshine") {
        return tooltip.to_string();
    }

    match state.status.as_str() {
        "idle" => "Sunshine - Idle".to_string(),
        "streaming" if !state.app_name.is_empty() => format!("Streaming {}", state.app_name),
        "streaming" => "Streaming".to_string(),
        "paused" if !state.app_name.is_empty() => format!("Stream paused: {}", state.app_name),
        "paused" => "Stream paused".to_string(),
        "pairing" if !state.pairing_client_name.is_empty() => {
            format!("Pairing request: {}", state.pairing_client_name)
        }
        "pairing" => "Pairing request".to_string(),
        "notification" if !state.notification.title.trim().is_empty() => {
            state.notification.title.trim().to_string()
        }
        "notification" if !state.notification.message.trim().is_empty() => {
            state.notification.message.trim().to_string()
        }
        "notification" => "Sunshine notification".to_string(),
        _ => default_tray_tooltip().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tray_state(status: &str) -> sunshine::TrayState {
        sunshine::TrayState {
            status: status.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn idle_tooltip_shows_explicit_state() {
        let mut state = tray_state("idle");
        state.tooltip = "Sunshine".to_string();

        assert_eq!(tray_tooltip_from_state(&state), "Sunshine - Idle");
    }

    #[test]
    fn explicit_tooltip_is_preserved() {
        let mut state = tray_state("idle");
        state.tooltip = "Sunshine - Ready".to_string();

        assert_eq!(tray_tooltip_from_state(&state), "Sunshine - Ready");
    }

    #[test]
    fn streaming_tooltip_includes_app_name() {
        let mut state = tray_state("streaming");
        state.app_name = "Steam Big Picture".to_string();

        assert_eq!(
            tray_tooltip_from_state(&state),
            "Streaming Steam Big Picture"
        );
    }

    #[test]
    fn paused_tooltip_includes_app_name() {
        let mut state = tray_state("paused");
        state.app_name = "Desktop".to_string();

        assert_eq!(tray_tooltip_from_state(&state), "Stream paused: Desktop");
    }

    #[test]
    fn pairing_tooltip_includes_client_name() {
        let mut state = tray_state("pairing");
        state.pairing_client_name = "Moonlight".to_string();

        assert_eq!(
            tray_tooltip_from_state(&state),
            "Pairing request: Moonlight"
        );
    }

    #[test]
    fn notification_tooltip_prefers_title_then_message() {
        let mut state = tray_state("notification");
        state.notification.title = "Update available".to_string();
        state.notification.message = "A new Sunshine build is ready".to_string();
        assert_eq!(tray_tooltip_from_state(&state), "Update available");

        state.notification.title.clear();
        assert_eq!(
            tray_tooltip_from_state(&state),
            "A new Sunshine build is ready"
        );
    }

    #[test]
    fn pairing_notification_label_names_the_client() {
        let mut state = tray_state("pairing");
        state.pairing_client_name = "Moonlight Client".to_string();
        state.notification.active = true;
        state.notification.action = "open_pin".to_string();

        assert_eq!(
            tray_notification_label(&EN_STRINGS, &state),
            "Complete pairing: Moonlight Client"
        );
    }

    #[test]
    fn menu_status_includes_streamed_app() {
        let mut state = tray_state("streaming");
        state.app_name = "Desktop".to_string();

        assert_eq!(
            tray_status_label(&EN_STRINGS, Some(&state), CoreConnectionState::Connected),
            "Sunshine · Streaming: Desktop"
        );
    }

    #[test]
    fn menu_status_uses_connecting_without_core_state() {
        assert_eq!(
            tray_status_label(&EN_STRINGS, None, CoreConnectionState::Connecting),
            "Sunshine · Connecting"
        );
    }

    #[test]
    fn menu_status_distinguishes_disconnected_core() {
        assert_eq!(
            tray_status_label(&EN_STRINGS, None, CoreConnectionState::Disconnected),
            "Sunshine · Disconnected"
        );
    }

    #[test]
    fn primary_actions_have_clear_english_labels() {
        #[cfg(target_os = "windows")]
        assert_eq!(EN_STRINGS.auto_start, "Run at startup");
        assert_eq!(EN_STRINGS.shutdown, "Exit Sunshine");
    }

    #[test]
    fn native_confirmations_use_english_tray_text() {
        assert!(EN_STRINGS.vdd_create_confirm.contains("virtual display"));
        assert!(EN_STRINGS.vdd_keep_confirm.contains("streaming ends"));
        assert!(EN_STRINGS.vdd_headless_confirm.contains("experimental"));
        assert!(
            EN_STRINGS
                .reset_config_confirm
                .contains("Sunshine defaults")
        );
        assert!(EN_STRINGS.clear_cache_confirm.contains("cached state"));
        assert!(EN_STRINGS.reset_display_confirm.contains("display-device"));
    }

    #[test]
    fn long_dynamic_menu_text_is_compacted() {
        assert_eq!(compact_menu_text("1234567890", 8), "12345...");
        assert_eq!(compact_menu_text("  short  ", 8), "short");
    }

    #[test]
    fn recovery_state_rejects_duplicate_and_stale_completion() {
        let mut recovery = RecoveryState::default();

        let first = recovery.begin(7).expect("first recovery should start");
        assert!(recovery.is_in_progress());
        assert_eq!(recovery.begin(7), None);
        assert!(!recovery.finish(first + 1));
        assert!(recovery.finish(first));
        assert!(!recovery.is_in_progress());

        let second = recovery.begin(11).expect("second recovery should start");
        assert_ne!(second, first);
        assert!(!recovery.finish(first));
        assert!(!recovery.complete_from_monitored_state(11));
        assert!(recovery.complete_from_monitored_state(12));
        assert!(!recovery.complete_from_monitored_state(13));
    }

    #[test]
    fn runtime_state_applies_connection_transitions_atomically() {
        let mut runtime = TrayRuntimeState {
            connection: CoreConnectionState::Connecting,
            tray_state: None,
            recovery: RecoveryState::default(),
        };
        assert_eq!(runtime.begin_recovery_if_disconnected(3), None);

        assert!(runtime.mark_disconnected());
        assert!(!runtime.mark_disconnected());
        assert!(runtime.begin_recovery_if_disconnected(3).is_some());

        let state = tray_state("idle");
        assert!(runtime.apply_connected_state(&state, None).is_none());
        let (previous, should_rebuild) = runtime
            .apply_connected_state(&state, Some(4))
            .expect("fresh monitor state should be accepted");
        assert!(previous.is_none());
        assert!(should_rebuild);

        let (_, connection, recovery_in_progress) = runtime.menu_snapshot();
        assert_eq!(connection, CoreConnectionState::Connected);
        assert!(!recovery_in_progress);
        assert!(
            !runtime
                .apply_connected_state(&state, Some(5))
                .expect("connected monitor state should be accepted")
                .1
        );
    }
}

fn apply_tray_state<R: Runtime + 'static>(
    app: &AppHandle<R>,
    state: &sunshine::TrayState,
    monitored_state_receipt: Option<u64>,
) {
    let applied = {
        let mut runtime = TRAY_RUNTIME_STATE.lock().unwrap();
        runtime.apply_connected_state(state, monitored_state_receipt)
    };
    crate::power::refresh_ecoqos_state(app);
    let Some((previous_state, should_rebuild_menu)) = applied else {
        debug!("Ignored non-monitor tray state while Core is unavailable");
        return;
    };

    if state.owner != "gui" {
        if app.remove_tray_by_id(TRAY_ID).is_some() {
            info!(
                "Removed GUI tray because core tray owner is '{}'",
                state.owner
            );
            *CURRENT_CORE_ICON.lock().unwrap() = None;
        }
        return;
    }

    notifications::show_connection_change_if_any(app, previous_state.as_ref(), state);

    if let Err(e) = build_owned_system_tray(app) {
        error!("Failed to create GUI tray for core owner: {}", e);
        return;
    }

    notifications::show_core_notification_if_new(app, state);

    if should_rebuild_menu {
        rebuild_tray_menu(app);
    }

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let tooltip = tray_tooltip_from_state(state);
        if let Err(e) = tray.set_tooltip(Some(tooltip.as_str())) {
            debug!("Failed to update tray tooltip from core state: {}", e);
        }
    }
    icons::apply_tray_icon(app, state.icon.as_str());
}

fn apply_core_disconnected<R: Runtime + 'static>(app: &AppHandle<R>) {
    let should_rebuild_menu = TRAY_RUNTIME_STATE.lock().unwrap().mark_disconnected();
    crate::power::refresh_ecoqos_state(app);

    if let Err(e) = build_owned_system_tray(app) {
        error!(
            "Failed to keep GUI tray available while Core is unavailable: {}",
            e
        );
        return;
    }
    if should_rebuild_menu {
        rebuild_tray_menu(app);
    }
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let tooltip =
            tray_status_label(get_tray_strings(), None, CoreConnectionState::Disconnected);
        let _ = tray.set_tooltip(Some(tooltip.as_str()));
    }
    icons::apply_disconnected_tray_icon(app);
}

fn apply_tray_state_on_main_thread<R: Runtime + 'static>(
    app: &AppHandle<R>,
    state: sunshine::TrayState,
) {
    let app_handle = app.clone();
    if let Err(e) = app.run_on_main_thread(move || {
        apply_tray_state(&app_handle, &state, None);
    }) {
        debug!("Failed to schedule tray state update on main thread: {}", e);
    }
}

fn apply_monitored_tray_state_on_main_thread<R: Runtime + 'static>(
    app: &AppHandle<R>,
    state: sunshine::TrayState,
) {
    let receipt = MONITORED_STATE_RECEIPT.fetch_add(1, Ordering::AcqRel) + 1;
    let app_handle = app.clone();
    if let Err(e) = app.run_on_main_thread(move || {
        apply_tray_state(&app_handle, &state, Some(receipt));
    }) {
        debug!(
            "Failed to schedule monitored tray state update on main thread: {}",
            e
        );
    }
}

/// 初始化 Sunshine 用户模式状态（仅 Windows）
#[cfg(target_os = "windows")]
fn init_sunshine_user_mode_state<R: Runtime + 'static>(app: &AppHandle<R>) {
    // 使用默认值 false，避免阻塞启动
    *SUNSHINE_USER_MODE_STATE.lock().unwrap() = false;

    // 异步更新 Sunshine 用户模式状态（不阻塞启动；阻塞的 sc/tasklist 放在 spawn_blocking 中）
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(500)).await;

        match tokio::task::spawn_blocking(crate::sunshine::is_sunshine_running_in_user_mode_impl)
            .await
        {
            Ok(Ok(is_user_mode)) => {
                let changed = {
                    let mut state = SUNSHINE_USER_MODE_STATE.lock().unwrap();
                    let changed = *state != is_user_mode;
                    *state = is_user_mode;
                    changed
                };
                debug!(
                    "Sunshine user-mode status updated asynchronously: {}",
                    is_user_mode
                );
                if changed {
                    let rebuild_handle = app_handle.clone();
                    let _ = app_handle.run_on_main_thread(move || {
                        rebuild_tray_menu(&rebuild_handle);
                    });
                }
            }
            Ok(Err(e)) => {
                debug!("Failed to check Sunshine user-mode status: {}", e);
            }
            Err(e) => {
                debug!("spawn_blocking failed while checking user mode: {}", e);
            }
        }
    });
}

/// 处理托盘单击事件
pub fn handle_tray_click<R: Runtime>(app: &AppHandle<R>) {
    open_main_panel_from_tray(app, "click");
}
/// 处理托盘双击事件
pub fn handle_tray_double_click<R: Runtime>(app: &AppHandle<R>) {
    open_main_panel_from_tray(app, "double click");
}

/// 重建托盘菜单（语言切换后调用）
fn rebuild_tray_menu<R: Runtime>(app: &AppHandle<R>) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        match build_tray_menu(app) {
            Ok(menu) => {
                if let Err(e) = tray.set_menu(Some(menu)) {
                    error!("Failed to rebuild the tray menu: {}", e);
                }
                // 更新 tooltip
                let tooltip = {
                    let runtime = TRAY_RUNTIME_STATE.lock().unwrap();
                    if runtime.connection == CoreConnectionState::Connected {
                        runtime
                            .tray_state
                            .as_ref()
                            .map(tray_tooltip_from_state)
                            .unwrap_or_else(|| default_tray_tooltip().to_string())
                    } else {
                        tray_status_label(get_tray_strings(), None, runtime.connection)
                    }
                };
                let _ = tray.set_tooltip(Some(tooltip));
            }
            Err(e) => error!("Failed to build the tray menu: {}", e),
        }
    }
}

/// Tauri 命令：前端通知 tray 同步语言
pub fn refresh_menu<R: Runtime>(app: &AppHandle<R>) {
    rebuild_tray_menu(app);
}
