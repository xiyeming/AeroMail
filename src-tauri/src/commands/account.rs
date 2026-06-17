use tauri::command;
use crate::error::AeroError;
use crate::models::account::Account;
use crate::AppState;

#[command]
pub async fn add_account(
    _state: tauri::State<'_, AppState>,
    _account: Account,
) -> Result<String, AeroError> {
    Ok("account added".to_string())
}

#[command]
pub async fn list_accounts(
    _state: tauri::State<'_, AppState>,
) -> Result<Vec<Account>, AeroError> {
    Ok(vec![])
}

#[command]
pub async fn delete_account(
    _state: tauri::State<'_, AppState>,
    _id: String,
) -> Result<(), AeroError> {
    Ok(())
}

#[command]
pub async fn test_account_connection(
    _state: tauri::State<'_, AppState>,
    _account: Account,
) -> Result<bool, AeroError> {
    Ok(true)
}
