use tauri::State;

use crate::AppState;
use crate::models::error::ErrorPayload;

/// Sets a configuration setting.
///
/// # Errors
///
/// Returns an error if the database write fails.
#[allow(clippy::needless_pass_by_value)]
#[tauri::command]
pub fn set_setting(
    key: String,
    value: String,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    state
        .db
        .set_setting(&key, &value)
        .map_err(|e| e.to_payload())
}

/// Retrieves a configuration setting by key.
///
/// # Errors
///
/// Returns an error if the database read fails.
#[allow(clippy::needless_pass_by_value)]
#[tauri::command]
pub fn get_setting(
    key: String,
    state: State<'_, AppState>,
) -> Result<Option<String>, ErrorPayload> {
    state.db.get_setting(&key).map_err(|e| e.to_payload())
}
