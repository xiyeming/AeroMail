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
use commands::compose::{
    delete_draft, get_draft, get_drafts, prepare_reply, save_attachment, save_draft, send_mail,
    sync_draft_to_imap,
};
use commands::mail::{
    delete_mail, get_attachments, get_mail_detail, get_mail_list, get_unread_count, list_folders,
    mark_mail_read, move_mail, toggle_mail_star,
};
use commands::search::{get_search_stats, index_pending_mails, search_mails};
use commands::settings::{get_setting, set_setting};
use commands::sync::{start_sync, stop_sync};
use commands::translation::{
    delete_translation_provider, get_cached_translation, list_translation_providers,
    translate_mail_text, upsert_translation_provider,
};
use db::pool::Database;
use services::account_manager::AccountManager;
use services::ai::AiService;
use services::search::SearchService;
use services::sync::SyncService;
use services::translation::TranslationService;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::RwLock;

pub struct AppState {
    pub account_manager: Arc<RwLock<AccountManager>>,
    pub ai_service: Arc<RwLock<AiService>>,
    pub translation_service: TranslationService,
    pub sync_service: Arc<RwLock<SyncService>>,
    pub search_service: Arc<RwLock<SearchService>>,
    pub compose_service: Arc<RwLock<crate::services::compose::ComposeService>>,
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
        let translation_service = TranslationService::new(Arc::clone(&db));
        let sync_service = Arc::new(RwLock::new(SyncService::new(Arc::clone(&db))));

        // Initialize search service
        let app_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| crate::error::AeroError::Internal(e.to_string()))?;
        let index_path = app_dir.join("tantivy_index");
        let search_service = Arc::new(RwLock::new(SearchService::new(
            Arc::clone(&db),
            &index_path,
        )?));

        let drafts_dir = app_dir.join("drafts");
        let compose_service = Arc::new(RwLock::new(crate::services::compose::ComposeService::new(
            Arc::clone(&db),
            drafts_dir,
            Arc::clone(&account_manager),
        )));

        Ok(Self {
            account_manager,
            ai_service,
            translation_service,
            sync_service,
            search_service,
            compose_service,
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
            list_translation_providers,
            upsert_translation_provider,
            delete_translation_provider,
            translate_mail_text,
            get_cached_translation,
            start_sync,
            stop_sync,
            get_mail_list,
            get_mail_detail,
            mark_mail_read,
            toggle_mail_star,
            list_folders,
            get_unread_count,
            delete_mail,
            move_mail,
            get_attachments,
            search_mails,
            index_pending_mails,
            get_search_stats,
            save_draft,
            get_drafts,
            get_draft,
            delete_draft,
            send_mail,
            prepare_reply,
            sync_draft_to_imap,
            save_attachment,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
