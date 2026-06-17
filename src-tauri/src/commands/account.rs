use tauri::State;

use crate::models::account::{AccountConfig, AccountSummary};
use crate::AppState;

#[tauri::command]
pub async fn add_account(
    _config: AccountConfig,
    _state: State<'_, AppState>,
) -> Result<String, String> {
    Ok("account-id".to_string())
}

#[tauri::command]
pub async fn list_accounts(
    _state: State<'_, AppState>,
) -> Result<Vec<AccountSummary>, String> {
    Ok(vec![])
}

#[tauri::command]
pub async fn delete_account(
    _account_id: String,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn test_account_connection(
    _config: AccountConfig,
    _state: State<'_, AppState>,
) -> Result<String, String> {
    Ok("ok".to_string())
}
