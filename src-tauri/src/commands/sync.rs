#![allow(clippy::missing_errors_doc)]

use tauri::{AppHandle, State};

use crate::AppState;
use crate::models::error::ErrorPayload;

#[tauri::command]
pub async fn start_sync(
    account_id: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    let sync = state.sync_service.read().await;
    sync.start_sync(&account_id, app_handle)
        .await
        .map_err(|e| e.to_payload())
}

#[tauri::command]
pub async fn stop_sync(account_id: String, state: State<'_, AppState>) -> Result<(), ErrorPayload> {
    let sync = state.sync_service.read().await;
    sync.stop_sync(&account_id)
        .await
        .map_err(|e| e.to_payload())
}
