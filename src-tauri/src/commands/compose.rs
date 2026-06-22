#![allow(clippy::missing_errors_doc)]

use tauri::State;

use crate::AppState;
use crate::error::AeroError;
use crate::models::compose::{ComposeDraft, ComposeDraftSummary, ReplyKind, SendMailRequest};
use crate::models::error::ErrorPayload;

#[tauri::command]
pub async fn save_draft(
    draft: ComposeDraft,
    state: State<'_, AppState>,
) -> Result<ComposeDraft, ErrorPayload> {
    let compose = state.compose_service.read().await;
    compose.save_draft(draft).map_err(|e| e.to_payload())
}

#[tauri::command]
pub async fn get_drafts(
    account_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<ComposeDraftSummary>, ErrorPayload> {
    let compose = state.compose_service.read().await;
    compose
        .list_drafts(account_id.as_deref())
        .map_err(|e| e.to_payload())
}

#[tauri::command]
pub async fn get_draft(
    draft_id: String,
    state: State<'_, AppState>,
) -> Result<ComposeDraft, ErrorPayload> {
    let compose = state.compose_service.read().await;
    compose
        .get_draft(&draft_id)
        .map_err(|e| e.to_payload())?
        .ok_or_else(|| AeroError::DraftNotFound(draft_id).to_payload())
}

#[tauri::command]
pub async fn delete_draft(
    draft_id: String,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    let compose = state.compose_service.read().await;
    compose.delete_draft(&draft_id).map_err(|e| e.to_payload())
}

#[tauri::command]
pub async fn send_mail(
    req: SendMailRequest,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    let compose = state.compose_service.read().await;
    compose.send_mail(req).await.map_err(|e| e.to_payload())
}

#[tauri::command]
pub async fn prepare_reply(
    mail_id: String,
    kind: ReplyKind,
    state: State<'_, AppState>,
) -> Result<ComposeDraft, ErrorPayload> {
    let db = &state.db;
    let original = db
        .get_mail_detail(&mail_id)
        .map_err(|e| e.to_payload())?
        .ok_or_else(|| AeroError::MailNotFound(mail_id.clone()).to_payload())?;

    let account_id = original.account_id.clone();
    let compose = state.compose_service.read().await;
    compose
        .prepare_reply(&account_id, &original, kind)
        .map_err(|e| e.to_payload())
}

#[tauri::command]
pub async fn sync_draft_to_imap(
    draft_id: String,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    let compose = state.compose_service.read().await;
    compose
        .sync_draft_to_imap(&draft_id)
        .await
        .map_err(|e| e.to_payload())
}

#[tauri::command]
pub async fn save_attachment(
    draft_id: String,
    attachment: crate::models::compose::AttachmentDraft,
    data: Vec<u8>,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    let compose = state.compose_service.read().await;
    if draft_id.is_empty() {
        return Err(AeroError::DraftNotFound("empty".to_string()).to_payload());
    }
    compose
        .draft_service
        .write_attachment(&draft_id, &attachment, &data)
        .map(|_| ())
        .map_err(|e| e.to_payload())
}
