use crate::windows;
use log::debug;
use tauri::{AppHandle, Manager, Runtime};

const WINDOW_ID: &str = "nr_overlay";
const WIDTH: f64 = 190.0;
const HEIGHT: f64 = 48.0;

/// Create the one floating surface that still has a functional reason to be
/// separate from the main panel: the in-stream noise-reduction overlay.
pub fn create_nr_overlay_window<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    crate::nr_overlay::request_visibility(app, true);

    if let Some(window) = app.get_webview_window(WINDOW_ID) {
        return window.show().map_err(|error| error.to_string());
    }

    let window = tauri::WebviewWindowBuilder::new(
        app,
        WINDOW_ID,
        tauri::WebviewUrl::App("sunshine-frame.html?mode=overlay".into()),
    )
    .title("Sunshine noise reduction")
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .focused(false)
    .visible(false)
    .inner_size(WIDTH, HEIGHT)
    .resizable(false)
    .build()
    .map_err(|error| format!("Failed to create the noise-reduction overlay: {error}"))?;

    windows::disable_context_menu(&window);
    #[cfg(target_os = "windows")]
    windows::configure_webview_security(&window);
    crate::power::refresh_ecoqos_state(app);

    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        if let Ok(Some(monitor)) = window.current_monitor() {
            let scale = monitor.scale_factor();
            let margin = (16.0 * scale).round() as i32;
            let width = (WIDTH * scale).round() as i32;
            let position = tauri::PhysicalPosition::new(
                monitor.position().x + monitor.size().width as i32 - width - margin,
                monitor.position().y + margin,
            );
            let _ = window.set_position(position);
        }
        if crate::nr_overlay::visibility_requested() {
            let _ = window.show();
            debug!("Noise-reduction overlay shown");
        }
    });

    Ok(())
}
