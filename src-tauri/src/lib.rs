pub mod commands;
pub mod db;
pub mod error;
pub mod models;
pub mod services;

use commands::account::{add_account, delete_account, list_accounts, test_account_connection};
use db::pool::Database;
use services::account_manager::AccountManager;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::RwLock;

pub struct AppState {
    pub account_manager: Arc<RwLock<AccountManager>>,
    pub db: Arc<Database>,
}

impl AppState {
    pub async fn new(app_handle: &tauri::AppHandle) -> Result<Self, crate::error::AeroError> {
        let db = Arc::new(Database::new(app_handle).await?);
        let account_manager = Arc::new(RwLock::new(AccountManager::new(Arc::clone(&db))));
        Ok(Self {
            account_manager,
            db,
        })
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! You've been greeted from Rust.")
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = AppState::new(&handle)
                    .await
                    .expect("failed to initialize app state");
                handle.manage(state);
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            add_account,
            list_accounts,
            delete_account,
            test_account_connection
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
