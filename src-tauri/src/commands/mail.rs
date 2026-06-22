#![allow(clippy::missing_errors_doc)]

use tauri::State;

use crate::AppState;
use crate::error::AeroError;
use crate::models::error::ErrorPayload;
use crate::models::mail::{AttachmentInfo, FolderInfo, MailDetail, MailHeader};

#[tauri::command]
pub async fn get_mail_list(
    folder_id: String,
    limit: u32,
    offset: u32,
    state: State<'_, AppState>,
) -> Result<Vec<MailHeader>, ErrorPayload> {
    let db = &state.db;
    db.list_mails(&folder_id, limit, offset)
        .map_err(|e| e.to_payload())
}

#[tauri::command]
pub async fn get_mail_detail(
    mail_id: String,
    state: State<'_, AppState>,
) -> Result<MailDetail, ErrorPayload> {
    let db = &state.db;
    db.get_mail_detail(&mail_id)
        .map_err(|e| e.to_payload())?
        .ok_or_else(|| AeroError::MailNotFound(mail_id).to_payload())
}

#[tauri::command]
pub async fn mark_mail_read(
    mail_id: String,
    is_read: bool,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    let db = &state.db;
    db.mark_mail_read(&mail_id, is_read)
        .map_err(|e| e.to_payload())
}

#[tauri::command]
pub async fn toggle_mail_star(
    mail_id: String,
    state: State<'_, AppState>,
) -> Result<bool, ErrorPayload> {
    let db = &state.db;
    db.toggle_mail_star(&mail_id).map_err(|e| e.to_payload())
}

#[tauri::command]
pub async fn list_folders(
    account_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<FolderInfo>, ErrorPayload> {
    let db = &state.db;
    db.list_folders(&account_id).map_err(|e| e.to_payload())
}

#[tauri::command]
pub async fn get_unread_count(
    account_id: String,
    state: State<'_, AppState>,
) -> Result<u32, ErrorPayload> {
    let db = &state.db;
    db.count_unread(&account_id).map_err(|e| e.to_payload())
}

#[tauri::command]
pub async fn delete_mail(mail_id: String, state: State<'_, AppState>) -> Result<(), ErrorPayload> {
    let db = &state.db;
    db.delete_mail(&mail_id).map_err(|e| e.to_payload())
}

#[tauri::command]
pub async fn move_mail(
    mail_id: String,
    target_folder_id: String,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    let db = &state.db;
    db.move_mail(&mail_id, &target_folder_id)
        .map_err(|e| e.to_payload())
}

#[tauri::command]
pub async fn get_attachments(
    mail_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<AttachmentInfo>, ErrorPayload> {
    let db = &state.db;
    db.get_attachments(&mail_id).map_err(|e| e.to_payload())
}
