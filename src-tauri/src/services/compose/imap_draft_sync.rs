use std::sync::Arc;

use futures::TryStreamExt;
use tracing::warn;

use crate::db::pool::Database;
use crate::error::AeroError;
use crate::models::account::AccountConfig;
use crate::models::compose::ComposeDraft;
use crate::services::imap_client;

/// Syncs a draft to the IMAP Drafts folder.
///
/// If the draft has an existing `remote_uid`, the old draft is deleted first.
/// Returns the new remote UID assigned by the IMAP server, or `0` if unavailable.
pub async fn sync_draft_to_imap(
    config: &AccountConfig,
    draft: &ComposeDraft,
    message_bytes: &[u8],
    _db: &Arc<Database>,
) -> Result<u32, AeroError> {
    let mut session = imap_client::connect_imap(config).await?;

    let drafts_folder = imap_client::find_drafts_folder(&mut session).await?;

    if let Some(uid) = draft.remote_uid {
        if let Err(e) = delete_uid(&mut session, &drafts_folder, uid).await {
            warn!("Failed to delete old IMAP draft {uid} in {drafts_folder}: {e}");
        }
    }

    imap_client::append_message(&mut session, &drafts_folder, Some("\\Draft"), message_bytes)
        .await?;

    let _ = session.logout().await;

    // async-imap append returns no UID, so return 0 to indicate unknown.
    Ok(0)
}

pub async fn delete_uid(
    session: &mut imap_client::ImapSession,
    folder: &str,
    uid: u32,
) -> Result<(), AeroError> {
    session
        .select(folder)
        .await
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;
    let _deleted: Vec<_> = session
        .uid_store(format!("{uid}"), "+FLAGS (\\Deleted)")
        .await
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?
        .try_collect()
        .await
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;
    session
        .expunge()
        .await
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?
        .try_collect::<Vec<_>>()
        .await
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;
    Ok(())
}
