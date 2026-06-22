use std::path::PathBuf;
use tauri::State;

use crate::AppState;
use crate::error::AeroError;
use crate::models::error::ErrorPayload;
use crate::services::logging::{LogConfig, LogService};

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

/// Reads the persisted logging configuration.
///
/// # Errors
///
/// Returns an error if the database cannot be queried.
#[allow(clippy::needless_pass_by_value)]
#[tauri::command]
pub fn get_log_config(state: State<'_, AppState>) -> Result<LogConfig, ErrorPayload> {
    let default_dir = default_log_dir(&state);
    LogService::get_config(&state.db, &default_dir).map_err(|e| e.to_payload())
}

/// Updates and applies the logging configuration.
///
/// # Errors
///
/// Returns an error if persistence or layer reload fails.
#[allow(clippy::needless_pass_by_value)]
#[tauri::command]
pub fn set_log_config(config: LogConfig, state: State<'_, AppState>) -> Result<(), ErrorPayload> {
    if config.enabled {
        std::fs::create_dir_all(&config.dir).map_err(|e| {
            AeroError::Internal(format!("failed to create log dir: {e}")).to_payload()
        })?;
    }
    state
        .log_service
        .apply(&state.db, &config)
        .map_err(|e| e.to_payload())
}

fn default_log_dir(state: &State<'_, AppState>) -> PathBuf {
    state.app_data_dir.join("logs")
}
