#![allow(clippy::missing_errors_doc)]

use tauri::{AppHandle, State};
use tracing::instrument;

use crate::AppState;
use crate::models::error::ErrorPayload;

#[tauri::command]
#[instrument(skip(state, app_handle), fields(account_id = %account_id), err(Debug))]
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
#[instrument(skip(state), fields(account_id = %account_id), err(Debug))]
pub async fn stop_sync(account_id: String, state: State<'_, AppState>) -> Result<(), ErrorPayload> {
    let sync = state.sync_service.read().await;
    sync.stop_sync(&account_id)
        .await
        .map_err(|e| e.to_payload())
}

#[tauri::command]
#[instrument(skip(state), fields(folder_id = %folder_id, limit), err(Debug))]
pub async fn fetch_older_mails(
    folder_id: String,
    limit: u32,
    state: State<'_, AppState>,
) -> Result<u32, ErrorPayload> {
    let sync = state.sync_service.read().await;
    let fetched = sync
        .fetch_older_mails(&folder_id, limit)
        .await
        .map_err(|e| e.to_payload())?;
    drop(sync);

    if fetched > 0 {
        let index_result = state.search_service.read().await.index_pending_mails();
        if let Err(e) = index_result {
            tracing::warn!(error = %e, "failed to index fetched older mails");
        }
    }

    Ok(fetched)
}
