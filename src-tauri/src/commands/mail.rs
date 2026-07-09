#![allow(clippy::missing_errors_doc)]

use tauri::State;
use tracing::{debug, instrument, warn};

use crate::AppState;
use crate::error::AeroError;
use crate::models::error::ErrorPayload;
use crate::models::mail::{AttachmentInfo, FolderInfo, MailDetail, MailHeader};
use crate::services::imap_client;

#[tauri::command]
#[instrument(skip(state), fields(folder_id = %folder_id, limit, offset), err(Debug))]
pub async fn get_mail_list(
    folder_id: String,
    limit: u32,
    offset: u32,
    state: State<'_, AppState>,
) -> Result<Vec<MailHeader>, ErrorPayload> {
    debug!("listing mails");
    let db = &state.db;
    db.list_mails(&folder_id, limit, offset)
        .map_err(|e| e.to_payload())
}

#[tauri::command]
#[instrument(skip(state), fields(folder_id = %folder_id, limit, offset), err(Debug))]
pub async fn get_virtual_mail_list(
    folder_id: String,
    limit: u32,
    offset: u32,
    state: State<'_, AppState>,
) -> Result<Vec<MailHeader>, ErrorPayload> {
    debug!("listing virtual folder mails");
    let db = &state.db;
    db.list_virtual_mails(&folder_id, limit, offset)
        .map_err(|e| e.to_payload())
}

#[tauri::command]
#[instrument(skip(state), fields(account_ids, limit, offset), err(Debug))]
pub async fn get_inbox_mail_list(
    account_ids: Vec<String>,
    limit: u32,
    offset: u32,
    state: State<'_, AppState>,
) -> Result<Vec<MailHeader>, ErrorPayload> {
    debug!("listing inbox mails for accounts");
    state
        .db
        .list_inbox_mails(&account_ids, limit, offset)
        .map_err(|e| e.to_payload())
}

#[tauri::command]
#[instrument(skip(state), fields(mail_id = %mail_id), err(Debug))]
pub async fn archive_mail(
    mail_id: String,
    state: State<'_, AppState>,
) -> Result<bool, ErrorPayload> {
    let db = &state.db;
    db.set_mail_archived(&mail_id, true)
        .map_err(|e| e.to_payload())
}

#[tauri::command]
#[instrument(skip(state), fields(mail_id = %mail_id), err(Debug))]
pub async fn toggle_mail_spam(
    mail_id: String,
    state: State<'_, AppState>,
) -> Result<bool, ErrorPayload> {
    let db = &state.db;
    db.get_mail_detail(&mail_id)
        .map_err(|e| e.to_payload())?
        .ok_or_else(|| AeroError::MailNotFound(mail_id.clone()).to_payload())
        .map(|mail| !mail.is_spam)
        .and_then(|new_spam| {
            db.set_mail_spam(&mail_id, new_spam)
                .map_err(|e| e.to_payload())
        })
}

#[tauri::command]
#[instrument(skip(state), fields(folder_id = %folder_id), err(Debug))]
pub async fn get_virtual_unread_count(
    folder_id: String,
    state: State<'_, AppState>,
) -> Result<u32, ErrorPayload> {
    let db = &state.db;
    db.count_virtual_unread(&folder_id)
        .map_err(|e| e.to_payload())
}

#[tauri::command]
#[instrument(skip(state), fields(mail_id = %mail_id), err(Debug))]
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
#[instrument(skip(state), fields(mail_id = %mail_id, is_read), err(Debug))]
pub async fn mark_mail_read(
    mail_id: String,
    is_read: bool,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    let db = &state.db;
    let mail = db
        .get_mail_detail(&mail_id)
        .map_err(|e| e.to_payload())?
        .ok_or_else(|| AeroError::MailNotFound(mail_id.clone()).to_payload())?;

    sync_read_to_imap(&state, &mail.account_id, &mail.folder_id, mail.uid, is_read).await?;
    db.mark_mail_read(&mail_id, is_read)
        .map_err(|e| e.to_payload())?;
    Ok(())
}

#[tauri::command]
#[instrument(skip(state), fields(mail_id = %mail_id), err(Debug))]
pub async fn toggle_mail_star(
    mail_id: String,
    state: State<'_, AppState>,
) -> Result<bool, ErrorPayload> {
    let db = &state.db;
    let mail = db
        .get_mail_detail(&mail_id)
        .map_err(|e| e.to_payload())?
        .ok_or_else(|| AeroError::MailNotFound(mail_id.clone()).to_payload())?;

    let new_starred = !mail.is_starred;
    sync_star_to_imap(
        &state,
        &mail.account_id,
        &mail.folder_id,
        mail.uid,
        new_starred,
    )
    .await?;
    db.toggle_mail_star(&mail_id).map_err(|e| e.to_payload())
}

#[tauri::command]
#[instrument(skip(state), fields(account_id = %account_id), err(Debug))]
pub async fn list_folders(
    account_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<FolderInfo>, ErrorPayload> {
    let db = &state.db;
    db.list_folders(&account_id).map_err(|e| e.to_payload())
}

#[tauri::command]
#[instrument(skip(state), fields(account_id = %account_id), err(Debug))]
pub async fn get_unread_count(
    account_id: String,
    state: State<'_, AppState>,
) -> Result<u32, ErrorPayload> {
    let db = &state.db;
    db.count_unread(&account_id).map_err(|e| e.to_payload())
}

fn is_trash_folder(folder: &crate::models::mail::FolderInfo) -> bool {
    let lower_path = folder.path.to_lowercase();
    let lower_name = folder.name.to_lowercase();
    [
        "trash",
        "deleted",
        "deleted items",
        "deleted messages",
        "bin",
        "[gmail]/trash",
        "[gmail]/bin",
    ]
    .iter()
    .any(|name| lower_path == *name || lower_name == *name)
}

fn find_trash_folder_id(
    db: &crate::db::pool::Database,
    account_id: &str,
) -> Result<Option<String>, crate::error::AeroError> {
    let folders = db.list_folders(account_id)?;
    Ok(folders.into_iter().find(is_trash_folder).map(|f| f.id))
}

#[tauri::command]
#[instrument(skip(state), fields(mail_id = %mail_id), err(Debug))]
pub async fn delete_mail(mail_id: String, state: State<'_, AppState>) -> Result<(), ErrorPayload> {
    let db = &state.db;
    let mail = db
        .get_mail_detail(&mail_id)
        .map_err(|e| e.to_payload())?
        .ok_or_else(|| AeroError::MailNotFound(mail_id.clone()).to_payload())?;

    let current_folder = db
        .get_folder_by_id(&mail.folder_id)
        .map_err(|e| e.to_payload())?
        .ok_or_else(|| AeroError::MailNotFound(mail.folder_id.clone()).to_payload())?;

    // If the mail is already in a trash folder, delete it permanently.
    if is_trash_folder(&current_folder) {
        debug!(account_id = %mail.account_id, folder_id = %mail.folder_id, uid = mail.uid, "permanently deleting mail on server");
        sync_delete_to_imap(&state, &mail.account_id, &mail.folder_id, mail.uid).await?;
        db.delete_mail(&mail_id).map_err(|e| e.to_payload())?;
        return Ok(());
    }

    // Otherwise move the mail to the account's trash folder.
    let Some(trash_folder_id) =
        find_trash_folder_id(db, &mail.account_id).map_err(|e| e.to_payload())?
    else {
        warn!(account_id = %mail.account_id, "no trash folder found; falling back to permanent delete");
        sync_delete_to_imap(&state, &mail.account_id, &mail.folder_id, mail.uid).await?;
        db.delete_mail(&mail_id).map_err(|e| e.to_payload())?;
        return Ok(());
    };

    debug!(account_id = %mail.account_id, folder_id = %mail.folder_id, uid = mail.uid, target_folder_id = %trash_folder_id, "moving mail to trash");
    sync_move_to_imap(
        &state,
        &mail.account_id,
        &mail.folder_id,
        mail.uid,
        &trash_folder_id,
    )
    .await?;

    db.move_mail(&mail_id, &trash_folder_id)
        .map_err(|e| e.to_payload())
}

#[tauri::command]
#[instrument(skip(state), fields(mail_id = %mail_id, target_folder_id = %target_folder_id), err(Debug))]
pub async fn move_mail(
    mail_id: String,
    target_folder_id: String,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    let db = &state.db;
    let mail = db
        .get_mail_detail(&mail_id)
        .map_err(|e| e.to_payload())?
        .ok_or_else(|| AeroError::MailNotFound(mail_id.clone()).to_payload())?;

    debug!(account_id = %mail.account_id, folder_id = %mail.folder_id, uid = mail.uid, target_folder_id = %target_folder_id, "moving mail on server");
    sync_move_to_imap(
        &state,
        &mail.account_id,
        &mail.folder_id,
        mail.uid,
        &target_folder_id,
    )
    .await?;

    db.move_mail(&mail_id, &target_folder_id)
        .map_err(|e| e.to_payload())
}

#[tauri::command]
#[instrument(skip(state), fields(mail_id = %mail_id), err(Debug))]
pub async fn get_attachments(
    mail_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<AttachmentInfo>, ErrorPayload> {
    let db = &state.db;
    db.get_attachments(&mail_id).map_err(|e| e.to_payload())
}

#[tauri::command]
#[instrument(skip(state), fields(attachment_id = %attachment_id), err(Debug))]
pub async fn get_attachment_content(
    attachment_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<u8>, ErrorPayload> {
    let db = &state.db;
    let local_path = db
        .get_attachment_path(&attachment_id)
        .map_err(|e| e.to_payload())?
        .ok_or_else(|| AeroError::AttachmentNotFound(attachment_id.clone()).to_payload())?;

    debug!(path = %local_path, "reading attachment from disk");
    std::fs::read(&local_path).map_err(|e| {
        AeroError::Internal(format!("failed to read attachment {attachment_id}: {e}")).to_payload()
    })
}

#[tauri::command]
#[instrument(skip(state), fields(attachment_id = %attachment_id), err(Debug))]
pub async fn download_attachment(
    attachment_id: String,
    target_path: std::path::PathBuf,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    // Validate target path: must be absolute and not contain path traversal
    if !target_path.is_absolute() {
        return Err(AeroError::Internal("target path must be absolute".to_string()).to_payload());
    }
    if target_path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return Err(AeroError::Internal("target path must not contain '..'".to_string()).to_payload());
    }

    let db = &state.db;
    let local_path = db
        .get_attachment_path(&attachment_id)
        .map_err(|e| e.to_payload())?
        .ok_or_else(|| AeroError::AttachmentNotFound(attachment_id.clone()).to_payload())?;

    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            AeroError::Internal(format!("failed to create target directory: {e}")).to_payload()
        })?;
    }

    std::fs::copy(&local_path, &target_path).map_err(|e| {
        AeroError::Internal(format!(
            "failed to copy attachment to {}: {e}",
            target_path.display()
        ))
        .to_payload()
    })?;

    Ok(())
}

#[instrument(skip(state), fields(account_id = %account_id, folder_id = %folder_id, uid, is_read), err(Debug))]
async fn sync_read_to_imap(
    state: &State<'_, AppState>,
    account_id: &str,
    folder_id: &str,
    uid: u32,
    is_read: bool,
) -> Result<(), ErrorPayload> {
    if uid == 0 {
        return Ok(());
    }

    let account_manager = state.account_manager.read().await;
    let config = account_manager
        .get_account_config_with_refresh(account_id)
        .await
        .map_err(|e| e.to_payload())?;
    drop(account_manager);

    let folder = state
        .db
        .get_folder_by_id(folder_id)
        .map_err(|e| e.to_payload())?
        .ok_or_else(|| AeroError::MailNotFound(folder_id.to_string()).to_payload())?;

    debug!(folder_path = %folder.path, uid, is_read, "issuing IMAP seen update");
    let mut session = imap_client::connect_imap(&config)
        .await
        .map_err(|e| e.to_payload())?;
    imap_client::set_seen_on_server(&mut session, &folder.path, uid, is_read)
        .await
        .map_err(|e| e.to_payload())?;
    session
        .logout()
        .await
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()).to_payload())?;

    Ok(())
}

#[instrument(skip(state), fields(account_id = %account_id, folder_id = %folder_id, uid, is_starred), err(Debug))]
async fn sync_star_to_imap(
    state: &State<'_, AppState>,
    account_id: &str,
    folder_id: &str,
    uid: u32,
    is_starred: bool,
) -> Result<(), ErrorPayload> {
    if uid == 0 {
        return Ok(());
    }

    let account_manager = state.account_manager.read().await;
    let config = account_manager
        .get_account_config_with_refresh(account_id)
        .await
        .map_err(|e| e.to_payload())?;
    drop(account_manager);

    let folder = state
        .db
        .get_folder_by_id(folder_id)
        .map_err(|e| e.to_payload())?
        .ok_or_else(|| AeroError::MailNotFound(folder_id.to_string()).to_payload())?;

    debug!(folder_path = %folder.path, uid, is_starred, "issuing IMAP flagged update");
    let mut session = imap_client::connect_imap(&config)
        .await
        .map_err(|e| e.to_payload())?;
    imap_client::set_flagged_on_server(&mut session, &folder.path, uid, is_starred)
        .await
        .map_err(|e| e.to_payload())?;
    session
        .logout()
        .await
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()).to_payload())?;

    Ok(())
}

#[instrument(skip(state), fields(account_id = %account_id, folder_id = %folder_id, uid), err(Debug))]
async fn sync_delete_to_imap(
    state: &State<'_, AppState>,
    account_id: &str,
    folder_id: &str,
    uid: u32,
) -> Result<(), ErrorPayload> {
    if uid == 0 {
        return Ok(());
    }

    let account_manager = state.account_manager.read().await;
    let config = account_manager
        .get_account_config_with_refresh(account_id)
        .await
        .map_err(|e| e.to_payload())?;
    drop(account_manager);

    let folder = state
        .db
        .get_folder_by_id(folder_id)
        .map_err(|e| e.to_payload())?
        .ok_or_else(|| AeroError::MailNotFound(folder_id.to_string()).to_payload())?;

    debug!(folder_path = %folder.path, uid, "issuing IMAP delete");
    let mut session = imap_client::connect_imap(&config)
        .await
        .map_err(|e| e.to_payload())?;
    imap_client::delete_mail_on_server(&mut session, &folder.path, uid)
        .await
        .map_err(|e| e.to_payload())?;
    session
        .logout()
        .await
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()).to_payload())?;

    Ok(())
}

#[instrument(skip(state), fields(account_id = %account_id, folder_id = %folder_id, target_folder_id = %target_folder_id, uid), err(Debug))]
async fn sync_move_to_imap(
    state: &State<'_, AppState>,
    account_id: &str,
    folder_id: &str,
    uid: u32,
    target_folder_id: &str,
) -> Result<(), ErrorPayload> {
    if uid == 0 {
        return Ok(());
    }

    let account_manager = state.account_manager.read().await;
    let config = account_manager
        .get_account_config_with_refresh(account_id)
        .await
        .map_err(|e| e.to_payload())?;
    drop(account_manager);

    let source_folder = state
        .db
        .get_folder_by_id(folder_id)
        .map_err(|e| e.to_payload())?
        .ok_or_else(|| AeroError::MailNotFound(folder_id.to_string()).to_payload())?;
    let target_folder = state
        .db
        .get_folder_by_id(target_folder_id)
        .map_err(|e| e.to_payload())?
        .ok_or_else(|| AeroError::MailNotFound(target_folder_id.to_string()).to_payload())?;

    debug!(source_path = %source_folder.path, target_path = %target_folder.path, uid, "issuing IMAP move");
    let mut session = imap_client::connect_imap(&config)
        .await
        .map_err(|e| e.to_payload())?;
    imap_client::move_mail_on_server(&mut session, &source_folder.path, uid, &target_folder.path)
        .await
        .map_err(|e| e.to_payload())?;
    session
        .logout()
        .await
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()).to_payload())?;

    Ok(())
}
