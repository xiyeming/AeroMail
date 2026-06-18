use tauri::State;

use crate::AppState;
use crate::models::account::{AccountConfig, AccountSummary};
use crate::models::error::ErrorPayload;

#[tauri::command]
pub async fn add_account(
    config: AccountConfig,
    state: State<'_, AppState>,
) -> Result<String, ErrorPayload> {
    let manager = state.account_manager.read().await;
    manager.add_account(config).map_err(|e| e.to_payload())
}

#[tauri::command]
pub async fn list_accounts(
    state: State<'_, AppState>,
) -> Result<Vec<AccountSummary>, ErrorPayload> {
    let manager = state.account_manager.read().await;
    manager.list_accounts().map_err(|e| e.to_payload())
}

#[tauri::command]
pub async fn delete_account(
    account_id: String,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    let manager = state.account_manager.read().await;
    manager.delete_account(&account_id).map_err(|e| e.to_payload())
}

#[tauri::command]
pub async fn test_account_connection(
    config: AccountConfig,
    state: State<'_, AppState>,
) -> Result<String, ErrorPayload> {
    let manager = state.account_manager.read().await;
    manager.test_connection(&config).await.map_err(|e| e.to_payload())
}
