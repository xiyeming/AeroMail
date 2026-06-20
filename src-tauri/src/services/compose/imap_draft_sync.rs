use std::net::TcpStream;
use std::sync::Arc;

use imap::Session;
use native_tls::TlsStream;

use crate::db::pool::Database;
use crate::error::AeroError;
use crate::models::account::{AccountConfig, AuthConfig};
use crate::models::compose::ComposeDraft;

/// Syncs a draft to the IMAP Drafts folder.
///
/// If the draft has an existing `remote_uid`, the old draft is deleted first.
/// Returns the new remote UID assigned by the IMAP server, or `0` if unavailable.
pub fn sync_draft_to_imap(
    config: &AccountConfig,
    draft: &ComposeDraft,
    message_bytes: &[u8],
    _db: &Arc<Database>,
) -> Result<u32, AeroError> {
    let mut session = connect_blocking(config)?;

    let drafts_folder = find_drafts_folder(&mut session)?;

    // Delete old draft if exists
    if let Some(uid) = draft.remote_uid {
        let _ = delete_uid(&mut session, &drafts_folder, uid);
    }

    // APPEND new draft with \Draft flag
    session
        .select(&drafts_folder)
        .map_err(|e| AeroError::ImapAppendFailed(e.to_string()))?;

    let flags = [imap::types::Flag::Draft];
    let result = session
        .append_with_flags(&drafts_folder, message_bytes, &flags)
        .map_err(|e| AeroError::ImapAppendFailed(e.to_string()))?;

    let _ = session.logout();

    // The imap crate append returns (), so we cannot extract a UID.
    // Return 0 to indicate the UID is unknown.
    let _ = result;
    Ok(0)
}

fn connect_blocking(
    config: &AccountConfig,
) -> Result<Session<TlsStream<TcpStream>>, AeroError> {
    let mut tls_builder = native_tls::TlsConnector::builder();
    if !config.advanced.verify_certificate {
        tls_builder.danger_accept_invalid_certs(true);
    }
    if let Some(ref cert_path) = config.advanced.ca_cert_path {
        tls_builder.add_root_certificate(
            native_tls::Certificate::from_pem(
                &std::fs::read(cert_path)
                    .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?,
            )
            .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?,
        );
    }
    let tls = tls_builder
        .build()
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;

    let client = imap::connect(
        format!("{}:{}", config.imap.host, config.imap.port),
        &config.imap.host,
        &tls,
    )
    .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;

    let session = match &config.auth {
        AuthConfig::Password { password_encrypted } => {
            let password = String::from_utf8_lossy(password_encrypted);
            client
                .login(&config.name, &password)
                .map_err(|e| AeroError::ImapAuthFailed(e.0.to_string()))?
        }
        AuthConfig::OAuth2 { .. } => {
            return Err(AeroError::InvalidConfig(
                "OAuth2 IMAP not yet implemented".into(),
            ));
        }
    };

    Ok(session)
}

fn find_drafts_folder(
    session: &mut Session<TlsStream<TcpStream>>,
) -> Result<String, AeroError> {
    let mailboxes = session
        .list(None, Some("*"))
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;

    let candidates = ["Drafts", "Draft", "[Gmail]/Drafts", "草稿箱", "\\u8349\\u7a3f"];
    for candidate in &candidates {
        if let Some(mb) = mailboxes
            .iter()
            .find(|m| m.name().eq_ignore_ascii_case(candidate))
        {
            return Ok(mb.name().to_string());
        }
    }
    // Fallback to first drafts-like folder
    for mb in mailboxes.iter() {
        if mb.name().to_lowercase().contains("draft") {
            return Ok(mb.name().to_string());
        }
    }
    Err(AeroError::ImapAppendFailed(
        "Drafts folder not found".to_string(),
    ))
}

fn delete_uid(
    session: &mut Session<TlsStream<TcpStream>>,
    folder: &str,
    uid: u32,
) -> Result<(), AeroError> {
    session
        .select(folder)
        .map_err(|e| AeroError::ImapAppendFailed(e.to_string()))?;
    session
        .uid_store(format!("{uid}"), "+FLAGS (\\Deleted)")
        .map_err(|e| AeroError::ImapAppendFailed(e.to_string()))?;
    session
        .expunge()
        .map_err(|e| AeroError::ImapAppendFailed(e.to_string()))?;
    Ok(())
}
