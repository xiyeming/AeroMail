use tracing::{debug, instrument};

use crate::db::pool::Database;
use crate::error::AeroError;
use crate::models::ai::{AiChatMessage, AiChatSession};
use crate::services::ai::{AiService, ChatMessage};

/// Builds the full message list for a chat completion request, including
/// system prompt, optional email context, conversation history, and the new user message.
///
/// # Errors
///
/// Returns an error if the database query for mail context fails.
#[instrument(skip_all, fields(session_id = %session.id, has_context = session.context_mail_id.is_some(), history_len = history.len(), content_len = user_content.len()), err(Debug))]
pub fn build_messages(
    db: &Database,
    session: &AiChatSession,
    history: &[AiChatMessage],
    user_content: &str,
) -> Result<Vec<ChatMessage>, AeroError> {
    let mut messages = vec![ChatMessage {
        role: "system".to_string(),
        content: "You are a helpful email assistant. Be concise.".to_string(),
    }];

    if let Some(ref mail_id) = session.context_mail_id {
        debug!(mail_id = %mail_id, "adding email context to chat messages");
        let subject = db
            .get_mail_subject(mail_id)?
            .ok_or(AeroError::AiContextMailNotFound)?;
        let from = db
            .get_mail_from_address(mail_id)?
            .ok_or(AeroError::AiContextMailNotFound)?;
        let body = db
            .get_mail_body_text(mail_id)?
            .ok_or(AeroError::AiContextMailNotFound)?;
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: format!("[Current Email Context]\nSubject: {subject}\nFrom: {from}\n\n{body}"),
        });
    }

    for msg in history {
        messages.push(ChatMessage {
            role: msg.role.clone(),
            content: msg.content.clone(),
        });
    }

    messages.push(ChatMessage {
        role: "user".to_string(),
        content: user_content.to_string(),
    });

    Ok(messages)
}

/// Sends a user message in a chat session, builds context, calls the AI provider,
/// and persists both the user message and the assistant reply.
///
/// # Errors
///
/// Returns an error if the session is not found, the database operations fail,
/// or the AI API request fails.
#[instrument(skip_all, fields(session_id = %session_id, content_len = user_content.len()), err(Debug))]
pub async fn send_message(
    ai: &AiService,
    session_id: &str,
    user_content: &str,
) -> Result<AiChatMessage, AeroError> {
    debug!("loading chat session and history");
    let session = ai.db.get_chat_session(session_id)?;
    let history = ai.db.get_chat_messages(session_id)?;
    let messages = build_messages(&ai.db, &session, &history, user_content)?;
    debug!(provider_id = %session.provider_id, message_count = messages.len(), "calling AI provider");
    let reply = ai.complete(&session.provider_id, messages).await?;

    debug!("persisting chat messages");
    let mut conn = ai.db.connection()?;
    let tx = conn.transaction()?;
    let user_id = uuid::Uuid::new_v4().to_string();
    let assistant_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    tx.execute(
        "INSERT INTO ai_chat_messages (id, session_id, role, content, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        (&user_id, session_id, "user", user_content, now),
    )?;
    tx.execute(
        "INSERT INTO ai_chat_messages (id, session_id, role, content, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        (&assistant_id, session_id, "assistant", &reply, now),
    )?;
    tx.execute(
        "UPDATE ai_chat_sessions SET updated_at = ?1 WHERE id = ?2",
        (now, session_id),
    )?;
    tx.commit()?;
    drop(conn);

    Ok(AiChatMessage {
        id: assistant_id,
        session_id: session_id.to_string(),
        role: "assistant".to_string(),
        content: reply,
        created_at: now,
    })
}
