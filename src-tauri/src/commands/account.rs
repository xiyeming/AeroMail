use tauri::{AppHandle, State};

use crate::AppState;
use crate::models::account::{AccountConfig, AccountSummary};
use crate::models::error::ErrorPayload;

/// Adds a new email account.
///
/// # Errors
///
/// Returns an error if the account configuration is invalid or the database write fails.
#[tauri::command]
#[tracing::instrument(skip(state, app), err(Debug))]
pub async fn add_account(
    config: AccountConfig,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<String, ErrorPayload> {
    let manager = state.account_manager.read().await;
    let id = manager.add_account(config).map_err(|e| e.to_payload())?;

    drop(manager);

    if let Err(e) = state.sync_service.read().await.start_sync(&id, app).await {
        tracing::warn!(error = %e, account_id = %id, "failed to start sync for new account");
    }

    Ok(id)
}

/// Lists all configured email accounts.
///
/// # Errors
///
/// Returns an error if the database read fails.
#[tauri::command]
#[tracing::instrument(skip(state), err(Debug))]
pub async fn list_accounts(
    state: State<'_, AppState>,
) -> Result<Vec<AccountSummary>, ErrorPayload> {
    let manager = state.account_manager.read().await;
    manager.list_accounts().map_err(|e| e.to_payload())
}

/// Retrieves the full configuration for an existing account.
///
/// # Errors
///
/// Returns an error if the account is not found.
#[tauri::command]
#[tracing::instrument(skip(state), err(Debug))]
pub async fn get_account_config(
    account_id: String,
    state: State<'_, AppState>,
) -> Result<AccountConfig, ErrorPayload> {
    let manager = state.account_manager.read().await;
    manager
        .get_account_config(&account_id)
        .map_err(|e| e.to_payload())
}

/// Updates an existing email account.
///
/// # Errors
///
/// Returns an error if the account is not found or the update fails.
#[tauri::command]
#[tracing::instrument(skip(state, password), err(Debug))]
pub async fn update_account(
    config: AccountConfig,
    password: Option<Vec<u8>>,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), ErrorPayload> {
    let account_id = config.id.clone();
    let manager = state.account_manager.read().await;
    manager
        .update_account(&config, password.as_deref())
        .map_err(|e| e.to_payload())?;

    drop(manager);

    // Restart sync so credential/config changes take effect immediately.
    let sync = state.sync_service.read().await;
    if let Err(e) = sync.stop_sync(&account_id).await {
        tracing::warn!(error = %e, account_id = %account_id, "failed to stop sync before restart");
    }
    if let Err(e) = sync.start_sync(&account_id, app).await {
        tracing::warn!(error = %e, account_id = %account_id, "failed to restart sync after account update");
    }
    drop(sync);

    Ok(())
}

/// Deletes an email account by ID.
///
/// # Errors
///
/// Returns an error if the account is not found or the database write fails.
#[tauri::command]
#[tracing::instrument(skip(state), err(Debug))]
pub async fn delete_account(
    account_id: String,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    let manager = state.account_manager.read().await;
    manager
        .delete_account(&account_id)
        .map_err(|e| e.to_payload())
}

/// Tests the connection to an email account.
///
/// # Errors
///
/// Returns an error if the connection test fails.
#[tauri::command]
#[tracing::instrument(skip(state), err(Debug))]
pub async fn test_account_connection(
    config: AccountConfig,
    state: State<'_, AppState>,
) -> Result<String, ErrorPayload> {
    let manager = state.account_manager.read().await;
    manager
        .test_connection(&config)
        .await
        .map_err(|e| e.to_payload())
}
