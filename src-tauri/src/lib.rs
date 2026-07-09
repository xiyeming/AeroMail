pub mod commands;
pub mod db;
pub mod error;
pub mod models;
pub mod services;

use commands::account::{
    add_account, delete_account, get_account_config, list_accounts, test_account_connection,
    update_account,
};
use commands::update::{check_for_updates, get_app_version, install_update};
use commands::ai::{
    ai_compose_assist, clear_chat_session, create_chat_session, delete_ai_mcp_server,
    delete_ai_provider, delete_ai_skill, delete_chat_session, extract_todos, get_ai_session_usage,
    get_ai_usage_summary, get_chat_messages, list_ai_mcp_servers, list_ai_provider_pricing,
    list_ai_providers, list_ai_skills, list_chat_sessions, quote_mail_to_chat, rename_chat_session,
    send_chat_message, set_chat_session_provider, summarize_mail, test_ai_provider,
    upsert_ai_mcp_server, upsert_ai_provider, upsert_ai_provider_pricing, upsert_ai_skill,
};
use commands::compose::{
    delete_draft, get_draft, get_drafts, prepare_reply, save_attachment, save_draft, send_mail,
    sync_draft_to_imap,
};
use commands::mail::{
    archive_mail, delete_mail, download_attachment, get_attachment_content, get_attachments,
    get_inbox_mail_list, get_mail_detail, get_mail_list, get_unread_count, get_virtual_mail_list,
    get_virtual_unread_count, list_folders, mark_mail_read, move_mail, toggle_mail_spam,
    toggle_mail_star, unarchive_mail,
};
use commands::search::{
    get_search_stats, index_pending_mails, search_mail_summaries, search_mails,
};
use commands::settings::{get_log_config, get_setting, set_log_config, set_setting};
use commands::sync::{fetch_older_mails, start_sync, stop_sync};
use commands::todo::{clear_completed_todos, delete_todo, list_todos, upsert_todo};
use commands::translation::{
    delete_translation_provider, get_cached_translation, list_translation_providers,
    translate_mail_text, translate_text, upsert_translation_provider,
};
use commands::window::{
    apply_window_decorations, close_main_window, confirmed_exit, hide_main_window,
    set_tray_menu_locale,
};
use db::pool::Database;
use services::account_manager::AccountManager;
use services::ai::AiService;
use services::ai::tools::ToolRegistry;
use services::logging::LogService;
use services::search::SearchService;
use services::sync::SyncService;
use services::translation::TranslationService;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Emitter;
use tauri::Manager;
use tauri::WindowEvent;
use tauri::Wry;
use tauri::image::Image;
use tauri::menu::{MenuBuilder, MenuItem, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tokio::sync::RwLock;

/// Ensures that the given path is an existing directory. If the path does not
/// exist, it is created. If it exists but is not a directory, an error is
/// returned.
fn ensure_directory(path: &PathBuf) -> Result<(), String> {
    if path.exists() {
        if path.is_dir() {
            return Ok(());
        }
        return Err(format!(
            "path exists but is not a directory: {}",
            path.display()
        ));
    }
    std::fs::create_dir_all(path).map_err(|e| format!("failed to create directory: {e}"))
}

pub struct AppState {
    pub account_manager: Arc<RwLock<AccountManager>>,
    pub ai_service: Arc<RwLock<AiService>>,
    pub tool_registry: Arc<RwLock<ToolRegistry>>,
    pub translation_service: TranslationService,
    pub sync_service: Arc<RwLock<SyncService>>,
    pub search_service: Arc<RwLock<SearchService>>,
    pub compose_service: Arc<RwLock<crate::services::compose::ComposeService>>,
    pub db: Arc<Database>,
    pub app_data_dir: PathBuf,
    pub log_service: Arc<LogService>,
}

impl AppState {
    /// Creates a new [`AppState`] by initializing the database, account manager,
    /// and AI service.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be opened or initialized.
    pub async fn new(app_handle: &tauri::AppHandle) -> Result<Self, crate::error::AeroError> {
        let db = Arc::new(Database::new(app_handle)?);
        let account_manager = Arc::new(RwLock::new(AccountManager::new(Arc::clone(&db))));
        let ai_service = Arc::new(RwLock::new(AiService::new(Arc::clone(&db))));
        let translation_service = TranslationService::new(Arc::clone(&db));

        let app_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| crate::error::AeroError::Internal(e.to_string()))?;

        let log_dir = app_dir.join("logs");
        let log_service = Arc::new(LogService::from_settings(&db, &log_dir)?);

        let attachments_dir = app_dir.join("attachments");
        if let Err(e) = ensure_directory(&attachments_dir) {
            return Err(crate::error::AeroError::Internal(format!(
                "failed to prepare attachments directory: {e}"
            )));
        }
        let sync_service = Arc::new(RwLock::new(SyncService::new(
            Arc::clone(&db),
            attachments_dir,
        )));

        let index_path = app_dir.join("tantivy_index");
        let search_service = Arc::new(RwLock::new(SearchService::new(
            Arc::clone(&db),
            &index_path,
        )?));

        let drafts_dir = app_dir.join("drafts");
        let attachments_dir_for_compose = app_dir.join("attachments");
        let compose_service = Arc::new(RwLock::new(crate::services::compose::ComposeService::new(
            Arc::clone(&db),
            drafts_dir,
            attachments_dir_for_compose,
            Arc::clone(&account_manager),
        )));

        let tool_registry = Arc::new(RwLock::new(ToolRegistry::new()));
        if let (Ok(servers), Ok(skills)) = (db.list_ai_mcp_servers(), db.list_ai_skills()) {
            let registry = Arc::clone(&tool_registry);
            registry.write().await.refresh(&servers, &skills).await;
        }

        Ok(Self {
            account_manager,
            ai_service,
            tool_registry,
            translation_service,
            sync_service,
            search_service,
            compose_service,
            db,
            app_data_dir: app_dir,
            log_service,
        })
    }
}

pub struct TrayMenuState {
    pub show_item: MenuItem<Wry>,
    pub quit_item: MenuItem<Wry>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! You've been greeted from Rust.")
}

/// 在系统默认浏览器中打开指定 URL
#[tauri::command]
fn open_url(url: &str) -> Result<(), String> {
    tracing::info!("opening url: {url}");
    open::that(url).map_err(|e| format!("无法打开链接: {e}"))
}

#[must_use]
pub fn tray_labels(locale: &str) -> (&'static str, &'static str) {
    if locale.starts_with("zh") {
        ("显示 AeroMail", "退出")
    } else {
        ("Show AeroMail", "Quit")
    }
}

#[allow(clippy::missing_errors_doc)]
#[allow(clippy::too_many_lines)]
pub fn run() {
    let run_result = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let handle = app.handle().clone();

            // Hide to tray on minimize and ask before closing.
            // System tray icon with show/quit menu.
            let tray_menu_handle = app.handle();
            let initial_locale = Database::new(tray_menu_handle)
                .ok()
                .and_then(|db| db.get_setting("app.locale").ok().flatten())
                .unwrap_or_else(|| "en".to_string());
            let (show_label, quit_label) = tray_labels(&initial_locale);
            let show_item = MenuItemBuilder::with_id("show", show_label).build(tray_menu_handle)?;
            let quit_item = MenuItemBuilder::with_id("quit", quit_label).build(tray_menu_handle)?;
            let menu = MenuBuilder::new(tray_menu_handle)
                .items(&[&show_item, &quit_item])
                .build()?;

            app.manage(TrayMenuState {
                show_item,
                quit_item,
            });

            let icon = Image::from_bytes(include_bytes!("../icons/tray-icon.png"))?;

            TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            tauri::async_runtime::spawn(async move {
                match AppState::new(&handle).await {
                    Ok(state) => {
                        let system_title_bar = state
                            .db
                            .get_setting("app.systemTitleBar")
                            .ok()
                            .flatten()
                            .is_none_or(|v| v == "system");

                        handle.manage(state);

                        std::panic::set_hook(Box::new(|info| {
                            tracing::error!(panic = %info, "application panic");
                        }));

                        tracing::info!("AeroMail application state initialized");

                        // Apply decorations from the backend as well, with a short delay to work
                        // around Tauri v2 not always respecting runtime setDecorations on Linux.
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                        if let Some(window) = handle.get_webview_window("main") {
                            if let Err(e) = window.set_decorations(system_title_bar) {
                                tracing::warn!(error = %e, "failed to apply window decorations");
                            } else {
                                tracing::debug!(
                                    system_title_bar,
                                    "applied window decorations on startup"
                                );
                            }
                        }

                        // Start automatic sync for all configured accounts.
                        let sync_service = handle.state::<AppState>().sync_service.clone();
                        if let Err(e) = sync_service.read().await.start_all(handle.clone()).await {
                            tracing::error!(error = %e, "failed to start automatic sync");
                        }
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "failed to initialize app state");
                        handle.exit(1);
                    }
                }
            });
            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::Resized(size) if size.width == 0 && size.height == 0 => {
                let _ = window.hide();
            }
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = window.emit("app://close-requested", ());
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            add_account,
            list_accounts,
            get_account_config,
            update_account,
            delete_account,
            test_account_connection,
            set_setting,
            get_setting,
            get_log_config,
            set_log_config,
            list_ai_providers,
            upsert_ai_provider,
            delete_ai_provider,
            test_ai_provider,
            ai_compose_assist,
            create_chat_session,
            send_chat_message,
            summarize_mail,
            extract_todos,
            list_chat_sessions,
            get_chat_messages,
            delete_chat_session,
            clear_chat_session,
            rename_chat_session,
            quote_mail_to_chat,
            set_chat_session_provider,
            get_ai_usage_summary,
            get_ai_session_usage,
            upsert_ai_provider_pricing,
            list_ai_provider_pricing,
            list_ai_mcp_servers,
            upsert_ai_mcp_server,
            delete_ai_mcp_server,
            list_ai_skills,
            upsert_ai_skill,
            delete_ai_skill,
            list_translation_providers,
            upsert_translation_provider,
            delete_translation_provider,
            translate_mail_text,
            translate_text,
            get_cached_translation,
            start_sync,
            stop_sync,
            fetch_older_mails,
            list_todos,
            upsert_todo,
            delete_todo,
            clear_completed_todos,
            get_mail_list,
            get_virtual_mail_list,
            get_inbox_mail_list,
            get_mail_detail,
            mark_mail_read,
            toggle_mail_star,
            archive_mail,
            unarchive_mail,
            toggle_mail_spam,
            list_folders,
            get_unread_count,
            get_virtual_unread_count,
            delete_mail,
            move_mail,
            get_attachments,
            get_attachment_content,
            download_attachment,
            search_mails,
            index_pending_mails,
            search_mail_summaries,
            get_search_stats,
            save_draft,
            get_drafts,
            get_draft,
            delete_draft,
            send_mail,
            prepare_reply,
            sync_draft_to_imap,
            save_attachment,
            apply_window_decorations,
            hide_main_window,
            confirmed_exit,
            close_main_window,
            set_tray_menu_locale,
            open_url,
            check_for_updates,
            install_update,
            get_app_version,
        ])
        .run(tauri::generate_context!());

    if let Err(e) = run_result {
        eprintln!("error while running tauri application: {e}");
    }
}
