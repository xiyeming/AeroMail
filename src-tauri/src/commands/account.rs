use tauri::State;

use crate::models::account::{AccountConfig, AccountSummary};
use crate::AppState;

#[tauri::command]
pub async fn add_account(
    config: AccountConfig,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let manager = state.account_manager.read().await;
    manager.add_account(config).map_err(String::from)
}

#[tauri::command]
pub async fn list_accounts(
    state: State<'_, AppState>,
) -> Result<Vec<AccountSummary>, String> {
    let manager = state.account_manager.read().await;
    manager.list_accounts().map_err(String::from)
}

#[tauri::command]
pub async fn delete_account(
    account_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let manager = state.account_manager.read().await;
    manager.delete_account(&account_id).map_err(String::from)
}

#[tauri::command]
pub async fn test_account_connection(
    config: AccountConfig,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let manager = state.account_manager.read().await;
    manager.test_connection(&config).await.map_err(String::from)
}
