#![allow(clippy::missing_errors_doc)]

use tauri::State;
use tracing::{debug, instrument};

use crate::AppState;
use crate::error::AeroError;
use crate::models::compose::{ComposeDraft, ComposeDraftSummary, ReplyKind, SendMailRequest};
use crate::models::error::ErrorPayload;

#[tauri::command]
#[instrument(skip(state, draft), fields(draft_id = %draft.id, account_id = ?draft.account_id), err(Debug))]
pub async fn save_draft(
    draft: ComposeDraft,
    state: State<'_, AppState>,
) -> Result<ComposeDraft, ErrorPayload> {
    let compose = state.compose_service.read().await;
    compose.save_draft(draft).map_err(|e| e.to_payload())
}

#[tauri::command]
#[instrument(skip(state), fields(account_id = ?account_id), err(Debug))]
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
#[instrument(skip(state), fields(draft_id = %draft_id), err(Debug))]
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
#[instrument(skip(state), fields(draft_id = %draft_id), err(Debug))]
pub async fn delete_draft(
    draft_id: String,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    let compose = state.compose_service.read().await;
    compose.delete_draft(&draft_id).map_err(|e| e.to_payload())
}

#[tauri::command]
#[instrument(skip(state), fields(draft_id = %draft_id), err(Debug))]
pub async fn send_mail(draft_id: String, state: State<'_, AppState>) -> Result<(), ErrorPayload> {
    debug!("sending mail");
    let compose = state.compose_service.read().await;
    compose
        .send_mail(SendMailRequest { draft_id })
        .await
        .map_err(|e| e.to_payload())
}

#[tauri::command]
#[instrument(skip(state), fields(mail_id = %mail_id, kind = ?kind), err(Debug))]
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
    debug!(account_id = %account_id, "preparing reply draft");
    let compose = state.compose_service.read().await;
    compose
        .prepare_reply(&account_id, &original, kind)
        .map_err(|e| e.to_payload())
}

#[tauri::command]
#[instrument(skip(state), fields(draft_id = %draft_id), err(Debug))]
pub async fn sync_draft_to_imap(
    draft_id: String,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    debug!("syncing draft to IMAP");
    let compose = state.compose_service.read().await;
    compose
        .sync_draft_to_imap(&draft_id)
        .await
        .map_err(|e| e.to_payload())
}

#[tauri::command]
#[instrument(skip(state, attachment, data), fields(draft_id = %draft_id, filename = %attachment.filename, size = data.len()), err(Debug))]
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
