use crate::proxy_server;
use log::{debug, error, info, warn};
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use tauri::{
    AppHandle, Manager, PhysicalPosition, PhysicalSize, Runtime, WebviewWindow,
    webview::PageLoadEvent,
};

struct HeartbeatState {
    created_at: std::time::Instant,
    last_beat: Option<std::time::Instant>,
}

impl HeartbeatState {
    fn new() -> Self {
        Self {
            created_at: std::time::Instant::now(),
            last_beat: None,
        }
    }
}

/// WebView heartbeat state is scoped to one window lifetime. Reusing a stale
/// timestamp for a newly-created window with the same label can make the
/// recovery monitor reload that window while its first page is still loading.
static HEARTBEAT_MAP: Lazy<Mutex<HashMap<String, HeartbeatState>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Labels whose WebView2 render pipeline is suspended via ICoreWebView2_3::TrySuspend.
/// A suspended webview's JS timers are frozen, so the heartbeat monitor must not
/// treat it as crashed while it sits in this set.
static SUSPENDED_WEBVIEWS: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));

/// 心跳超时阈值（秒）：超过此时间未收到心跳则认为渲染进程可能崩溃
const HEARTBEAT_STALE_SECS: u64 = 30;
/// Cold WebView2 startup and the first local-proxy navigation can be slower
/// than a normal heartbeat interval. Recovery must not disrupt that startup.
const HEARTBEAT_STARTUP_GRACE_SECS: u64 = 90;

const MAIN_WINDOW_ID: &str = "main";
const ABOUT_WINDOW_ID: &str = "about";
const LOG_CONSOLE_WINDOW_ID: &str = "log_console";
const PIN_WINDOW_ID: &str = "pin_pairing";
const NR_OVERLAY_WINDOW_ID: &str = "nr_overlay";

#[cfg(target_os = "windows")]
pub fn register_agent_restart() {
    use windows::Win32::System::Recovery::{
        REGISTER_APPLICATION_RESTART_FLAGS, RegisterApplicationRestart,
    };
    use windows::core::w;

    if let Err(e) =
        unsafe { RegisterApplicationRestart(w!("--hidden"), REGISTER_APPLICATION_RESTART_FLAGS(0)) }
    {
        warn!("Failed to register GUI agent restart: {}", e);
    }
}

#[cfg(not(target_os = "windows"))]
pub fn register_agent_restart() {}

#[cfg(target_os = "windows")]
pub fn unregister_agent_restart() {
    use windows::Win32::System::Recovery::UnregisterApplicationRestart;

    if let Err(e) = unsafe { UnregisterApplicationRestart() } {
        debug!("Failed to unregister GUI agent restart: {}", e);
    }
}

#[cfg(not(target_os = "windows"))]
pub fn unregister_agent_restart() {}

fn clamp_axis(value: i32, min: i32, max: i32) -> i32 {
    if max < min {
        min
    } else {
        value.clamp(min, max)
    }
}

pub fn clamp_window_position_to_monitor(
    position: PhysicalPosition<i32>,
    monitor_position: PhysicalPosition<i32>,
    monitor_size: PhysicalSize<u32>,
    scale_factor: f64,
    logical_width: f64,
    logical_height: f64,
    margin_logical: f64,
) -> PhysicalPosition<i32> {
    let scale = if scale_factor.is_finite() && scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    };
    let margin = (margin_logical * scale).round() as i32;
    let width = (logical_width * scale).round() as i32;
    let height = (logical_height * scale).round() as i32;
    let min_x = monitor_position.x + margin;
    let min_y = monitor_position.y + margin;
    let max_x = monitor_position.x + monitor_size.width as i32 - width - margin;
    let max_y = monitor_position.y + monitor_size.height as i32 - height - margin;

    PhysicalPosition::new(
        clamp_axis(position.x, min_x, max_x),
        clamp_axis(position.y, min_y, max_y),
    )
}

fn constrain_tool_window_to_visible_area(
    window: &tauri::Window,
    logical_width: f64,
    logical_height: f64,
    margin_logical: f64,
) -> Result<(), String> {
    let Some(monitor) = window
        .current_monitor()
        .map_err(|e| format!("Failed to get current monitor: {}", e))?
    else {
        return Ok(());
    };
    let position = window
        .outer_position()
        .map_err(|e| format!("Failed to get window position: {}", e))?;
    let target_position = clamp_window_position_to_monitor(
        position,
        *monitor.position(),
        *monitor.size(),
        monitor.scale_factor(),
        logical_width,
        logical_height,
        margin_logical,
    );

    if target_position != position {
        window
            .set_position(target_position)
            .map_err(|e| format!("Failed to constrain tool window position: {}", e))?;
    }

    Ok(())
}

/// WebView 可见性控制初始化脚本
/// 窗口最小化或隐藏到托盘时，模拟 Page Visibility API 状态变化，触发 Chromium 内置节流：
/// - hidden 页面的 setTimeout/setInterval 最小间隔从 4ms 变为 1000ms
/// - requestAnimationFrame 停止执行
/// - CSS 动画暂停
/// 同时通过注入 CSS 暂停所有 CSS 动画/过渡，进一步降低 GPU 合成开销
const WEBVIEW_VISIBILITY_INIT_SCRIPT: &str = r#"
(function() {
    let _hidden = false;
    let _pauseStyleEl = null;

    Object.defineProperty(document, 'hidden', {
        get: function() { return _hidden; },
        configurable: true,
    });
    Object.defineProperty(document, 'visibilityState', {
        get: function() { return _hidden ? 'hidden' : 'visible'; },
        configurable: true,
    });

    function pauseAnimations() {
        if (!_pauseStyleEl) {
            _pauseStyleEl = document.createElement('style');
            _pauseStyleEl.id = '__gpu_pause_animations';
            _pauseStyleEl.textContent = '*, *::before, *::after { animation-play-state: paused !important; transition: none !important; }';
        }
        if (!_pauseStyleEl.parentNode) {
            (document.head || document.documentElement).appendChild(_pauseStyleEl);
        }
    }

    function resumeAnimations() {
        if (_pauseStyleEl && _pauseStyleEl.parentNode) {
            _pauseStyleEl.parentNode.removeChild(_pauseStyleEl);
        }
    }

    window.__setWebviewVisibility = function(visible) {
        var wasHidden = _hidden;
        _hidden = !visible;
        if (wasHidden !== _hidden) {
            console.log('[WebView Visibility] ' + (_hidden ? 'suspended' : 'active'));
            if (_hidden) {
                pauseAnimations();
            } else {
                resumeAnimations();
            }
            document.dispatchEvent(new Event('visibilitychange'));
        }
    };

    // WebView 心跳：每 10 秒向 Rust 后端发送心跳信号
    // 若渲染进程崩溃，心跳停止，后端检测到后可触发自动恢复
    setInterval(function() {
        try {
            if (window.__TAURI_INTERNALS__) {
                window.__TAURI_INTERNALS__.invoke('webview_heartbeat');
            }
        } catch(e) {}
    }, 10000);
    // 立即发送首次心跳
    try {
        if (window.__TAURI_INTERNALS__) {
            window.__TAURI_INTERNALS__.invoke('webview_heartbeat');
        }
    } catch(e) {}
})();
"#;

/// 禁用窗口的右键菜单（仅在生产环境）
#[cfg(not(debug_assertions))]
pub fn disable_context_menu<R: Runtime>(window: &WebviewWindow<R>) {
    const DISABLE_CONTEXT_MENU_SCRIPT: &str = r#"
        (function() {
            document.addEventListener('contextmenu', e => { e.preventDefault(); return false; }, true);
            document.addEventListener('keydown', e => {
                if (e.keyCode === 123 || // F12
                    (e.ctrlKey && e.shiftKey && (e.keyCode === 73 || e.keyCode === 74)) || // Ctrl+Shift+I/J
                    (e.ctrlKey && e.keyCode === 85)) { // Ctrl+U
                    e.preventDefault();
                    return false;
                }
            }, true);
        })();
    "#;

    let window_clone = window.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        match window_clone.eval(DISABLE_CONTEXT_MENU_SCRIPT) {
            Ok(_) => debug!("Disabled the context menu in the production build"),
            Err(e) => debug!(
                "Failed to inject the context-menu suppression script: {}",
                e
            ),
        }
    });
}

/// 开发环境不执行任何操作
#[cfg(debug_assertions)]
pub fn disable_context_menu<R: Runtime>(_window: &WebviewWindow<R>) {}

/// WebView 心跳命令：由前端 JS 定期调用，表明渲染进程仍然存活
#[tauri::command]
pub fn webview_heartbeat(webview: tauri::Webview) {
    let label = webview.label().to_string();
    let mut map = HEARTBEAT_MAP.lock().unwrap();
    let state = map.entry(label).or_insert_with(HeartbeatState::new);
    state.last_beat = Some(std::time::Instant::now());
}

fn begin_webview_heartbeat(window_id: &str) {
    // A window recreated under the same label must never inherit a suspended
    // state from its destroyed predecessor.
    SUSPENDED_WEBVIEWS.lock().unwrap().remove(window_id);
    HEARTBEAT_MAP
        .lock()
        .unwrap()
        .insert(window_id.to_string(), HeartbeatState::new());
}

fn end_webview_heartbeat(window_id: &str) {
    HEARTBEAT_MAP.lock().unwrap().remove(window_id);
}

/// Resize the About window to match its web content height.
#[tauri::command]
pub fn resize_about_window(window: tauri::Window, height: f64) -> Result<(), String> {
    if window.label() != ABOUT_WINDOW_ID {
        return Err("resize_about_window can only be called from the About window".to_string());
    }

    if !height.is_finite() {
        return Err("Invalid about window height".to_string());
    }

    let scale_factor = window
        .scale_factor()
        .map_err(|e| format!("Failed to get window scale factor: {}", e))?;
    let current_size = window
        .inner_size()
        .map_err(|e| format!("Failed to get window size: {}", e))?;
    let current_width = current_size.width as f64 / scale_factor;
    let current_height = current_size.height as f64 / scale_factor;
    let max_height = window
        .current_monitor()
        .ok()
        .flatten()
        .map(|monitor| (monitor.size().height as f64 / scale_factor - 120.0).max(420.0))
        .unwrap_or(900.0);
    let target_height = height.clamp(420.0, max_height);

    if (current_height - target_height).abs() < 1.0 {
        return Ok(());
    }

    window
        .set_size(tauri::Size::Logical(tauri::LogicalSize::new(
            current_width,
            target_height,
        )))
        .map_err(|e| format!("Failed to resize about window: {}", e))
}

/// Resize the floating tool window to match its web content size.
// These synchronous commands execute on the GUI thread. Do not resize/reposition
// a caption while Windows owns its native move loop (there are no DOM pointers).
fn native_window_move_active(window: &tauri::Window) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::WindowsAndMessaging::{
            GUI_INMOVESIZE, GUITHREADINFO, GetGUIThreadInfo, GetWindowThreadProcessId,
        };
        let hwnd = window.hwnd().map_err(|e| e.to_string())?;
        let mut info = GUITHREADINFO {
            cbSize: std::mem::size_of::<GUITHREADINFO>() as u32,
            ..Default::default()
        };
        unsafe {
            GetGUIThreadInfo(
                GetWindowThreadProcessId(windows::Win32::Foundation::HWND(hwnd.0), None),
                &mut info,
            )
        }
        .map_err(|e| e.to_string())?;
        return Ok(info.flags.0 & GUI_INMOVESIZE.0 != 0);
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = window;
        Ok(false)
    }
}

#[tauri::command]
pub fn resize_tool_window(window: tauri::Window, width: f64, height: f64) -> Result<bool, String> {
    if window.label() != NR_OVERLAY_WINDOW_ID {
        return Err("resize_tool_window can only be called from an overlay".to_string());
    }
    if !width.is_finite() || !height.is_finite() {
        return Err("Invalid tool window size".to_string());
    }
    if native_window_move_active(&window)? {
        return Ok(false);
    }
    let scale_factor = window.scale_factor().map_err(|e| e.to_string())?;
    let current_size = window.inner_size().map_err(|e| e.to_string())?;
    let min_height: f64 = 32.0;
    let max_height = window
        .current_monitor()
        .ok()
        .flatten()
        .map(|monitor| (monitor.size().height as f64 / scale_factor - 32.0).max(min_height))
        .unwrap_or(900.0);
    let target_width = width.clamp(120.0, 560.0);
    let target_height = height.clamp(min_height, max_height);
    if (current_size.width as f64 / scale_factor - target_width).abs() >= 1.0
        || (current_size.height as f64 / scale_factor - target_height).abs() >= 1.0
    {
        // Read the current anchor here, not in an earlier frontend IPC request.
        let position = window.outer_position().map_err(|e| e.to_string())?;
        window
            .set_size(tauri::Size::Logical(tauri::LogicalSize::new(
                target_width,
                target_height,
            )))
            .map_err(|e| e.to_string())?;
        window
            .set_position(tauri::PhysicalPosition::new(
                position.x + current_size.width as i32
                    - (target_width * scale_factor).round() as i32,
                position.y,
            ))
            .map_err(|e| e.to_string())?;
    }
    constrain_tool_window_to_visible_area(&window, target_width, target_height, 0.0)?;
    Ok(true)
}

/// 通过 WebView2 COM API 强制重新加载页面（在渲染进程崩溃后仍可工作）
#[cfg(target_os = "windows")]
fn reload_webview_via_com<R: Runtime>(window: &WebviewWindow<R>) {
    let label = window.label().to_string();
    let _ = window.with_webview(move |webview| {
        let controller = webview.controller();
        unsafe {
            if let Ok(core_webview) = controller.CoreWebView2() {
                match core_webview.Reload() {
                    Ok(_) => log::info!(
                        "Triggered a reload to recover the WebView renderer [{}]",
                        label
                    ),
                    Err(e) => log::warn!("Failed to reload the WebView [{}]: {:?}", label, e),
                }
            }
        }
    });
}

/// 检查指定窗口的心跳是否已超时，若超时则触发 COM 级重新加载
#[cfg(target_os = "windows")]
fn check_and_recover_webview<R: Runtime>(window: &WebviewWindow<R>) {
    let label = window.label().to_string();
    let is_stale = {
        let map = HEARTBEAT_MAP.lock().unwrap();
        match map.get(&label) {
            Some(state) if state.created_at.elapsed().as_secs() >= HEARTBEAT_STARTUP_GRACE_SECS => {
                state
                    .last_beat
                    .is_some_and(|last_beat| last_beat.elapsed().as_secs() > HEARTBEAT_STALE_SECS)
            }
            Some(_) | None => false,
        }
    };

    if is_stale {
        warn!(
            "WebView heartbeat timed out [{}]; attempting renderer recovery",
            label
        );
        reload_webview_via_com(window);
        // 重置心跳时间，给恢复留出时间
        HEARTBEAT_MAP
            .lock()
            .unwrap()
            .insert(label, HeartbeatState::new());
    }
}

/// 启动 WebView 心跳监控后台任务
/// 定期检查可见窗口的心跳状态，若检测到崩溃则自动恢复
pub fn start_heartbeat_monitor(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        // 等待应用完全启动后再开始监控
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;

        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;

            // The slim control panel has one primary window.
            for label in &[MAIN_WINDOW_ID] {
                // 挂起中的 WebView JS 定时器被冻结，心跳缺失是预期行为，
                // 不能按崩溃恢复（否则会在挂起期间强制重载页面）。
                if SUSPENDED_WEBVIEWS.lock().unwrap().contains(*label) {
                    continue;
                }
                if let Some(win) = app.get_webview_window(label) {
                    let is_visible = win.is_visible().unwrap_or(false);
                    let is_minimized = win.is_minimized().unwrap_or(true);

                    // 仅当窗口可见且未最小化时检查心跳
                    if is_visible && !is_minimized {
                        #[cfg(target_os = "windows")]
                        check_and_recover_webview(&win);
                    }
                }
            }
        }
    });
}

/// 显示并激活窗口（解决权限隔离问题）
pub fn show_and_activate_window<R: Runtime>(window: &WebviewWindow<R>) {
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();

    // 恢复 WebView 活跃状态（引擎级 + JS 级）
    resume_webview(window);
    set_webview_window_visibility(window, true);

    // 重置代理快速失败状态，确保恢复后首次请求不被拦截
    proxy_server::reset_fast_fail();

    // 检查 WebView 心跳，若渲染进程已崩溃则自动恢复
    #[cfg(target_os = "windows")]
    check_and_recover_webview(window);

    #[cfg(target_os = "windows")]
    force_activate_window_win32(window);

    #[cfg(target_os = "windows")]
    refresh_webview_surface(window);
}

/// 使用 Windows API 强制激活窗口
#[cfg(target_os = "windows")]
fn force_activate_window_win32<R: Runtime>(window: &WebviewWindow<R>) {
    use log::warn;
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::Threading::GetCurrentThreadId;
    use windows::Win32::UI::WindowsAndMessaging::{
        ASFW_ANY, AllowSetForegroundWindow, BringWindowToTop, GetWindowThreadProcessId, SW_RESTORE,
        SW_SHOW, SetForegroundWindow, ShowWindow,
    };

    // 直接从 Tauri 窗口获取 HWND，避免使用 FindWindowW（对隐藏窗口不可靠）
    let hwnd = match window.window_handle() {
        Ok(handle) => match handle.as_raw() {
            RawWindowHandle::Win32(win32_handle) => HWND(win32_handle.hwnd.get() as *mut _),
            _ => {
                error!("Could not get a Win32 window handle: this is not a Win32 window");
                return;
            }
        },
        Err(e) => {
            error!("Failed to get the window handle: {:?}", e);
            return;
        }
    };

    if hwnd.0.is_null() {
        error!("The window handle is null");
        return;
    }

    debug!("Obtained the window handle: {:?}", hwnd);

    unsafe {
        // 获取目标窗口的线程ID用于诊断
        let mut target_pid: u32 = 0;
        let target_tid = GetWindowThreadProcessId(hwnd, Some(&mut target_pid));
        let current_tid = GetCurrentThreadId();
        debug!(
            "Current thread: {}, target thread: {}, target process: {}",
            current_tid, target_tid, target_pid
        );

        // 允许任何进程设置前台窗口
        let _ = AllowSetForegroundWindow(ASFW_ANY);

        // 确保窗口可见并恢复
        let _ = ShowWindow(hwnd, SW_RESTORE);
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = BringWindowToTop(hwnd);

        // 尝试设置为前台窗口
        let result = SetForegroundWindow(hwnd);

        if result.as_bool() {
            debug!("Activated the window with the Windows API");
        } else {
            warn!("SetForegroundWindow returned FALSE; the window may not be in the foreground");
        }
    }
}

/// Keep native windows square on Windows 11 without transparent WebView composition.
#[cfg(target_os = "windows")]
fn apply_dwm_square_corners<R: Runtime>(window: &WebviewWindow<R>) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dwm::DwmSetWindowAttribute;

    let hwnd = match window.window_handle() {
        Ok(handle) => match handle.as_raw() {
            RawWindowHandle::Win32(h) => HWND(h.hwnd.get() as *mut _),
            _ => return,
        },
        Err(_) => return,
    };

    unsafe {
        // DWMWA_WINDOW_CORNER_PREFERENCE = 33; DWMWCP_DONOTROUND = 1.
        let preference: u32 = 1;
        let attr = windows::Win32::Graphics::Dwm::DWMWINDOWATTRIBUTE(33i32);
        match DwmSetWindowAttribute(
            hwnd,
            attr,
            &preference as *const u32 as *const std::ffi::c_void,
            std::mem::size_of::<u32>() as u32,
        ) {
            Ok(_) => debug!("Applied square DWM corners"),
            Err(e) => debug!("Could not apply square DWM corners: {:?}", e),
        }
    }
}

/// 获取或创建窗口的辅助函数
fn get_or_create_window<R: Runtime, F>(
    app: &AppHandle<R>,
    window_id: &str,
    builder_fn: F,
) -> Result<WebviewWindow<R>, String>
where
    F: FnOnce(&AppHandle<R>) -> Result<WebviewWindow<R>, tauri::Error>,
{
    if let Some(window) = app.get_webview_window(window_id) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        crate::power::refresh_ecoqos_state(app);
        return Ok(window);
    }

    let window = builder_fn(app).map_err(|e| format!("Could not create window: {}", e))?;
    crate::power::refresh_ecoqos_state(app);
    Ok(window)
}

/// 打开关于窗口（单例模式）
pub fn open_about_window<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let window = get_or_create_window(app, ABOUT_WINDOW_ID, |app| {
        tauri::WebviewWindowBuilder::new(
            app,
            ABOUT_WINDOW_ID,
            tauri::WebviewUrl::App("sunshine-frame.html?mode=about".into()),
        )
        .title("About Sunshine Control Panel")
        .inner_size(540.0, 620.0)
        .resizable(false)
        .maximizable(false)
        .minimizable(true)
        .decorations(true)
        .center()
        .build()
    })?;

    #[cfg(target_os = "windows")]
    configure_webview_security(&window);

    disable_context_menu(&window);
    debug!("Opened the About window");
    Ok(())
}

/// 打开日志控制台窗口（单例模式）
pub fn open_log_console<R: Runtime>(app: &AppHandle<R>) {
    match get_or_create_window(app, LOG_CONSOLE_WINDOW_ID, |app| {
        tauri::WebviewWindowBuilder::new(
            app,
            LOG_CONSOLE_WINDOW_ID,
            tauri::WebviewUrl::App("sunshine-frame.html?mode=console".into()),
        )
        .title("Log console")
        .inner_size(1000.0, 700.0)
        .resizable(true)
        .maximizable(true)
        .minimizable(true)
        .decorations(true)
        .center()
        .build()
    }) {
        Ok(window) => {
            #[cfg(target_os = "windows")]
            configure_webview_security(&window);
            disable_context_menu(&window);
            debug!("Opened the log console window");
        }
        Err(e) => error!("{}", e),
    }
}

pub fn open_pin_window<R: Runtime>(app: &AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = app.get_webview_window(PIN_WINDOW_ID) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        debug!("Activated the PIN window");
        crate::power::refresh_ecoqos_state(app);
        return Ok(());
    }

    let window = tauri::WebviewWindowBuilder::new(
        app,
        PIN_WINDOW_ID,
        tauri::WebviewUrl::App("sunshine-frame.html?mode=pin".into()),
    )
    .title("PIN pairing")
    .fullscreen(true)
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .skip_taskbar(true)
    .visible(false)
    .build()
    .map_err(|e| format!("Failed to create the PIN window: {}", e))?;

    disable_context_menu(&window);

    // 禁用自动填充和密码保存提示
    #[cfg(target_os = "windows")]
    configure_webview_security(&window);

    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let _ = window.show();
        let _ = window.set_focus();
    });

    debug!("Created the PIN pairing window");
    crate::power::refresh_ecoqos_state(app);
    Ok(())
}

/// 创建主窗口
pub fn create_main_window<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<(), Box<dyn std::error::Error>> {
    create_main_window_internal(app, true)
}

/// 创建隐藏的主窗口
pub fn create_main_window_hidden<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<(), Box<dyn std::error::Error>> {
    create_main_window_internal(app, false)
}

fn create_main_window_internal<R: Runtime>(
    app: &AppHandle<R>,
    visible: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    proxy_server::ensure_started();
    if app.get_webview_window(MAIN_WINDOW_ID).is_some() {
        debug!("The main window already exists; skipping creation");
        return Ok(());
    }

    let visibility_desc = if visible { "visible" } else { "hidden" };
    info!("Creating the {visibility_desc} main window");

    crate::tray::mark_main_panel_loading();
    begin_webview_heartbeat(MAIN_WINDOW_ID);

    let window = tauri::WebviewWindowBuilder::new(
        app,
        MAIN_WINDOW_ID,
        tauri::WebviewUrl::App("sunshine-frame.html".into()),
    )
    .title("Sunshine Control Panel")
    .inner_size(1280.0, 800.0)
    .min_inner_size(1024.0, 600.0)
    .center()
    // Native Windows title bar. The custom chrome this replaces needed a
    // full-width `data-tauri-drag-region` strip across the top of the shell,
    // which sat above the embedded web UI and swallowed clicks on its
    // navigation links. The OS title bar also gives us correct snap, shake,
    // double-click-to-maximise and system-menu behaviour for free.
    .decorations(true)
    // Keep WebView2 opaque to avoid stale transparent composition surfaces
    // after restore.
    .transparent(false)
    .shadow(false)
    .visible(false)
    .disable_drag_drop_handler()
    .initialization_script(WEBVIEW_VISIBILITY_INIT_SCRIPT)
    .on_page_load(move |window, payload| {
        if visible && payload.event() == PageLoadEvent::Finished {
            tauri::async_runtime::spawn(async move {
                // WebView2 can report a completed DOM before its compositor
                // has submitted the first frame. Avoid exposing that black
                // frame, then present the built-in loading shell or panel.
                tokio::time::sleep(std::time::Duration::from_millis(120)).await;
                show_and_activate_window(&window);
            });
        }
    })
    .build()
    .map_err(|e| {
        end_webview_heartbeat(MAIN_WINDOW_ID);
        format!("Failed to create the {visibility_desc} main window: {e}")
    })?;

    // Force square DWM corners to match the panel's visual language.
    #[cfg(target_os = "windows")]
    apply_dwm_square_corners(&window);

    // 禁用自动填充和密码保存提示
    #[cfg(target_os = "windows")]
    configure_webview_security(&window);

    disable_context_menu(&window);
    info!("Created the {visibility_desc} main window");
    crate::power::refresh_ecoqos_state(app);
    Ok(())
}

/// 激活主窗口
pub fn activate_main_window(app: &tauri::AppHandle, target_url: Option<String>) {
    debug!(
        "activate_main_window called with target_url: {:?}",
        target_url
    );

    let created = if app.get_webview_window(MAIN_WINDOW_ID).is_none() {
        info!("The main window does not exist; creating it from the user-session agent");
        if let Err(e) = create_main_window(app) {
            error!("Failed to create the main window on demand: {}", e);
            return;
        }
        true
    } else {
        false
    };

    // Cold windows are revealed by the page-load hook after WebView2 has a
    // composited first frame. The startup page reads any runtime URL itself.
    if created {
        return;
    }

    let Some(window) = app.get_webview_window(MAIN_WINDOW_ID) else {
        error!("The main window is still unavailable after creation");
        return;
    };

    debug!("Activating the main window");

    let is_visible = window.is_visible().unwrap_or(false);
    let is_minimized = window.is_minimized().unwrap_or(false);

    if is_minimized {
        let _ = window.unminimize();
    }
    if !is_visible {
        let _ = window.show();
    }
    let _ = window.set_focus();

    // 恢复 WebView 活跃状态（引擎级 + JS 级）
    resume_webview(&window);
    set_webview_window_visibility(&window, true);
    crate::power::refresh_ecoqos_state(app);

    // 重置代理快速失败状态
    proxy_server::reset_fast_fail();

    #[cfg(target_os = "windows")]
    force_activate_window_win32(&window);

    #[cfg(target_os = "windows")]
    refresh_webview_surface(&window);

    if let Some(url) = target_url {
        navigate_to_url(&window, &url);
    }

    // 短暂置顶以强制显示在前台
    let _ = window.set_always_on_top(true);
    let window_clone = window.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        let _ = window_clone.set_always_on_top(false);
    });

    debug!("Window activation completed");
}

/// 导航到指定 URL
fn navigate_to_url(window: &WebviewWindow, url: &str) {
    debug!("Navigating to: {}", url);

    let Ok(parsed_url) = url::Url::parse(url) else {
        error!("Failed to parse URL: {}", url);
        return;
    };

    let path = format!(
        "{}{}",
        parsed_url.path(),
        parsed_url
            .query()
            .map(|q| format!("?{}", q))
            .unwrap_or_default()
    );

    if path.contains("/pin") {
        debug!("Skipping navigation for the /pin path");
        return;
    }

    let proxy_url = proxy_server::get_proxy_url();
    let script = format!(
        r#"(function(){{ const iframe = document.querySelector('.sunshine-iframe'); if (iframe) iframe.src = '{}{}'; }})();"#,
        proxy_url, path
    );

    let _ = window.eval(&script);
    debug!("Sent the navigation command");
}

/// 配置 WebView2 安全设置（禁用浏览器自动填充和密码自动保存）
///
/// WebView2 默认会弹出自动填充/密码保存提示，在嵌入式管理面板中会干扰用户操作。
/// 通过 ICoreWebView2Settings4 接口在引擎级别禁用，比 HTML autocomplete="off" 更可靠。
#[cfg(target_os = "windows")]
pub(crate) fn configure_webview_security<R: Runtime>(window: &WebviewWindow<R>) {
    let label = window.label().to_string();
    let _ = window.with_webview(move |webview| {
        // 引入 webview2-com 的 COM 接口类型
        use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Settings4;
        // 引入 windows-core 0.61 的 Interface trait（与 webview2-com 版本一致）
        use wv2_windows_core::Interface;

        let controller = webview.controller();
        unsafe {
            let Ok(core_webview) = controller.CoreWebView2() else {
                log::warn!("Could not get ICoreWebView2 [{}]", label);
                return;
            };
            let Ok(settings) = core_webview.Settings() else {
                log::warn!("Could not get ICoreWebView2Settings [{}]", label);
                return;
            };

            // Cast 到 Settings4（WebView2 SDK 1.0.902+，所有现代版本均支持）
            match settings.cast::<ICoreWebView2Settings4>() {
                Ok(settings4) => {
                    // 禁用地址/联系人等通用自动填充
                    let _ = settings4.SetIsGeneralAutofillEnabled(false);
                    // 禁用密码自动保存提示
                    let _ = settings4.SetIsPasswordAutosaveEnabled(false);
                    log::debug!("Disabled WebView2 autofill and password saving [{}]", label);
                }
                Err(e) => {
                    log::warn!(
                        "Could not get ICoreWebView2Settings4; WebView2 may be outdated [{}]: {:?}",
                        label,
                        e
                    );
                }
            }
        }
    });
}

/// 设置 WebView 前端的可见性状态。
fn set_webview_visibility(window: &tauri::Window, visible: bool) {
    let label = window.label().to_string();
    if let Some(webview_window) = window.app_handle().get_webview_window(&label) {
        set_webview_window_visibility(&webview_window, visible);
    }
}

/// Re-submit the existing WebView2 bounds after a native window restore.
/// This invalidates stale composition tiles without navigating or changing
/// any page state.
#[cfg(target_os = "windows")]
fn refresh_webview_surface<R: Runtime>(window: &WebviewWindow<R>) {
    let label = window.label().to_string();
    let _ = window.with_webview(move |webview| {
        let controller = webview.controller();
        unsafe {
            let mut bounds = Default::default();
            if controller.Bounds(&mut bounds).is_ok()
                && let Err(error) = controller.SetBounds(bounds)
            {
                log::debug!("Failed to refresh WebView bounds [{}]: {:?}", label, error);
            }
            let _ = controller.NotifyParentWindowPositionChanged();
        }
    });
}

/// 挂起 WebView2 渲染管线（窗口最小化/隐藏时调用）。
///
/// TrySuspend 要求 controller 不可见，而 wry 最小化时不会自动隐藏 controller，
/// 因此先 SetIsVisible(false) 再挂起；恢复时由 resume_webview 按相反顺序还原。
#[cfg(target_os = "windows")]
fn suspend_webview<R: Runtime>(ww: &WebviewWindow<R>) {
    use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2_3;
    use webview2_com::TrySuspendCompletedHandler;
    use wv2_windows_core::Interface;

    let is_visible = ww.is_visible().unwrap_or(true);
    let is_minimized = ww.is_minimized().unwrap_or(false);
    if is_visible && !is_minimized {
        // 200ms 延迟期内窗口已经恢复，不挂起。
        return;
    }

    let label = ww.label().to_string();
    if !SUSPENDED_WEBVIEWS.lock().unwrap().insert(label.clone()) {
        return;
    }

    let _ = ww.with_webview(move |webview| {
        let controller = webview.controller();
        unsafe {
            let mut started = true;
            if let Err(e) = controller.SetIsVisible(false) {
                log::warn!("SetIsVisible(false) failed [{}]: {}", label, e);
                started = false;
            }

            let webview3 = started
                .then(|| controller.CoreWebView2())
                .and_then(|core| core.ok())
                .map(|core| core.cast::<ICoreWebView2_3>());

            match webview3 {
                Some(Ok(webview3)) => {
                    let cb_label = label.clone();
                    let cb_controller = controller.clone();
                    let handler =
                        TrySuspendCompletedHandler::create(Box::new(move |result, suspended| {
                            if result.is_ok() && suspended {
                                debug!("WebView2 rendering suspended [{}]", cb_label);
                            } else {
                                debug!("WebView2 suspension did not take effect [{}]: {:?}", cb_label, result.err());
                                SUSPENDED_WEBVIEWS.lock().unwrap().remove(&cb_label);
                                // 异步失败时外层闭包已返回，必须在此自行还原
                                // controller 可见性，否则面板会停留在隐形状态。
                                if let Err(e) = cb_controller.SetIsVisible(true) {
                                    log::warn!(
                                        "Could not restore visibility after suspension failed [{}]: {}",
                                        cb_label,
                                        e
                                    );
                                }
                            }
                            Ok(())
                        }));
                    if let Err(e) = webview3.TrySuspend(&handler) {
                        log::warn!("TrySuspend failed [{}]: {}", label, e);
                        started = false;
                    }
                }
                Some(Err(e)) => {
                    log::warn!("Could not acquire ICoreWebView2_3 [{}]: {}", label, e);
                    started = false;
                }
                None => {}
            }

            if !started {
                // 还原 controller 可见性，避免留下"隐形但仍活着"的 WebView。
                SUSPENDED_WEBVIEWS.lock().unwrap().remove(&label);
                if let Err(e) = controller.SetIsVisible(true) {
                    log::warn!("Could not restore visibility after suspension failed [{}]: {}", label, e);
                }
            }
        }
    });
}

#[cfg(not(target_os = "windows"))]
fn suspend_webview<R: Runtime>(_ww: &WebviewWindow<R>) {}

/// 恢复挂起的 WebView2（窗口重新可见/获得焦点时调用）。幂等：未挂起时为 no-op。
#[cfg(target_os = "windows")]
fn resume_webview<R: Runtime>(ww: &WebviewWindow<R>) {
    use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2_3;
    use wv2_windows_core::Interface;

    let label = ww.label().to_string();
    if !SUSPENDED_WEBVIEWS.lock().unwrap().remove(&label) {
        return;
    }

    debug!("Resuming WebView2 rendering [{}]", label);
    let _ = ww.with_webview(move |webview| {
        let controller = webview.controller();
        unsafe {
            // SetIsVisible(true) 本身会隐式恢复，但显式 Resume 语义更明确。
            if let Ok(core) = controller.CoreWebView2()
                && let Ok(webview3) = core.cast::<ICoreWebView2_3>()
                && let Err(e) = webview3.Resume()
            {
                log::warn!("Resume failed [{}]: {}", label, e);
            }
            if let Err(e) = controller.SetIsVisible(true) {
                log::warn!("SetIsVisible(true) failed [{}]: {}", label, e);
            }
        }
    });
}

#[cfg(not(target_os = "windows"))]
fn resume_webview<R: Runtime>(_ww: &WebviewWindow<R>) {}

/// Notify the page so it can pause animations and suspend expensive content.
/// The native window already controls WebView2 composition. Toggling the
/// controller's IsVisible property separately can leave transparent WebViews
/// with a stale surface after restore.
fn set_webview_window_visibility<R: Runtime>(ww: &WebviewWindow<R>, visible: bool) {
    let label = ww.label();
    let js = format!(
        "if(window.__setWebviewVisibility)window.__setWebviewVisibility({})",
        visible
    );
    if let Err(e) = ww.eval(&js) {
        debug!("Failed to set WebView visibility [{}]: {}", label, e);
    }
}

/// 处理窗口事件
pub fn handle_window_event(window: &tauri::Window, event: &tauri::WindowEvent) {
    match event {
        tauri::WindowEvent::CloseRequested { .. } => {
            end_webview_heartbeat(window.label());
            // 窗口正在销毁：清掉挂起标记即可，无需 Resume。
            SUSPENDED_WEBVIEWS.lock().unwrap().remove(window.label());
            match window.label() {
                "main" => {
                    // Let the WebView close so the user agent returns to its
                    // headless baseline. The tray and session services keep
                    // the application event loop alive.
                    set_webview_visibility(window, false);
                    crate::tray::mark_main_panel_loading();
                }
                _ => {}
            }
        }
        tauri::WindowEvent::Focused(focused) => {
            // Only the primary window uses the visibility initialization script.
            let label = window.label();
            if label != MAIN_WINDOW_ID {
                return;
            }

            if *focused {
                // 窗口获得焦点时恢复 WebView 活跃状态
                set_webview_visibility(window, true);
                if let Some(ww) = window.app_handle().get_webview_window(label) {
                    resume_webview(&ww);
                }
                // 重置代理快速失败状态
                proxy_server::reset_fast_fail();
            } else {
                // 失去焦点时，延迟检查是否处于最小化或隐藏状态
                let app_handle = window.app_handle().clone();
                let label = window.label().to_string();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                    if let Some(ww) = app_handle.get_webview_window(&label) {
                        let is_minimized = ww.is_minimized().unwrap_or(false);
                        let is_visible = ww.is_visible().unwrap_or(true);
                        if is_minimized || !is_visible {
                            set_webview_window_visibility(&ww, false);
                            suspend_webview(&ww);
                            debug!(
                                "WebView entered suspension [{}]: minimized={}, visible={}",
                                label, is_minimized, is_visible
                            );
                        }
                    }
                });
            }
        }
        tauri::WindowEvent::Destroyed => {
            // 延迟重估，等窗口从 Tauri 注册表移除后再判断是否回到托盘常驻态。
            let app_handle = window.app_handle().clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                crate::power::refresh_ecoqos_state(&app_handle);
            });
        }
        _ => {}
    }
}
