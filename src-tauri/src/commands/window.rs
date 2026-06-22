use tauri::{AppHandle, Manager, Runtime, WebviewWindow};
use tracing::{debug, instrument};

/// Applies window decorations from the backend with a short delay.
///
/// Tauri v2 on Linux does not always respect runtime `set_decorations` calls
/// immediately, so the actual change is deferred slightly.
///
/// # Errors
///
/// Returns an error if the main window exists but its decorations cannot be set.
#[tauri::command]
#[instrument(skip(app))]
pub async fn apply_window_decorations<R: Runtime>(
    app: AppHandle<R>,
    enabled: bool,
) -> Result<(), String> {
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    if let Some(window) = app.get_webview_window("main") {
        window
            .set_decorations(enabled)
            .map_err(|e| format!("failed to set decorations: {e}"))?;
        debug!(enabled, "applied window decorations from command");
    }
    Ok(())
}

/// Hides the main window so the application continues running in the tray.
///
/// # Errors
///
/// Returns an error if the main window exists but cannot be hidden.
#[tauri::command]
pub async fn hide_main_window<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Closes the main window after the user confirmed exit via the frontend dialog.
#[tauri::command]
pub async fn confirmed_exit<R: Runtime>(app: AppHandle<R>) {
    app.exit(0);
}

/// Programmatically closes the main window; used when the frontend has already
/// shown a confirmation dialog.
///
/// # Errors
///
/// Returns an error if the window cannot be closed.
#[tauri::command]
pub async fn close_main_window<R: Runtime>(window: WebviewWindow<R>) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
}
