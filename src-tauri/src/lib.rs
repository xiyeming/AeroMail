pub mod commands;
pub mod db;
pub mod error;
pub mod models;
pub mod services;

use commands::account::{add_account, delete_account, list_accounts, test_account_connection};
use commands::ai::{
    create_chat_session, delete_ai_provider, delete_chat_session, get_chat_messages,
    list_ai_providers, list_chat_sessions, send_chat_message, test_ai_provider, upsert_ai_provider,
};
use commands::settings::{get_setting, set_setting};
use db::pool::Database;
use services::account_manager::AccountManager;
use services::ai::AiService;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::RwLock;

pub struct AppState {
    pub account_manager: Arc<RwLock<AccountManager>>,
    pub ai_service: Arc<RwLock<AiService>>,
    pub db: Arc<Database>,
}

impl AppState {
    /// Creates a new [`AppState`] by initializing the database, account manager,
    /// and AI service.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be opened or initialized.
    pub fn new(app_handle: &tauri::AppHandle) -> Result<Self, crate::error::AeroError> {
        let db = Arc::new(Database::new(app_handle)?);
        let account_manager = Arc::new(RwLock::new(AccountManager::new(Arc::clone(&db))));
        let ai_service = Arc::new(RwLock::new(AiService::new(Arc::clone(&db))));
        Ok(Self {
            account_manager,
            ai_service,
            db,
        })
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! You've been greeted from Rust.")
}

#[allow(clippy::expect_used, clippy::missing_panics_doc)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                match AppState::new(&handle) {
                    Ok(state) => {
                        handle.manage(state);
                    }
                    Err(e) => {
                        eprintln!("failed to initialize app state: {e}");
                        handle.exit(1);
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            add_account,
            list_accounts,
            delete_account,
            test_account_connection,
            set_setting,
            get_setting,
            list_ai_providers,
            upsert_ai_provider,
            delete_ai_provider,
            test_ai_provider,
            create_chat_session,
            send_chat_message,
            list_chat_sessions,
            get_chat_messages,
            delete_chat_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
