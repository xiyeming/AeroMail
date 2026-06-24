use tracing::{debug, instrument};

use crate::db::pool::Database;
use crate::error::AeroError;
use crate::models::ai::{AiChatMessage, AiChatSession, AiUsageLog};
use crate::services::ai::client::CompletionResult;
use crate::services::ai::tokens::{calculate_cost, count_messages, count_text};
use crate::services::ai::tools::ToolRegistry;
use crate::services::ai::{AiService, ChatMessage};

/// Extracts one or more `<think>...</think>` blocks from an AI reply.
///
/// Returns the cleaned reply content (with thinking blocks removed) and an
/// optional concatenated thinking process.
fn extract_thinking(reply: &str) -> (String, Option<String>) {
    let mut content = reply.to_string();
    let mut thinking_parts: Vec<String> = Vec::new();

    while let Some(start) = content.find("<think>") {
        let after_start = start + "<think>".len();
        match content[after_start..].find("</think>") {
            Some(end_offset) => {
                let inner = content[after_start..after_start + end_offset]
                    .trim()
                    .to_string();
                if !inner.is_empty() {
                    thinking_parts.push(inner);
                }
                let end = after_start + end_offset + "</think>".len();
                content.replace_range(start..end, "");
            }
            None => break,
        }
    }

    let thinking = if thinking_parts.is_empty() {
        None
    } else {
        Some(thinking_parts.join("\n\n"))
    };

    (content.trim().to_string(), thinking)
}

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
    let mut messages = vec![ChatMessage::new(
        "system",
        "You are a helpful email assistant. Be concise.",
    )];

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
        messages.push(ChatMessage::new(
            "system",
            format!("[Current Email Context]\nSubject: {subject}\nFrom: {from}\n\n{body}"),
        ));
    }

    for msg in history {
        messages.push(ChatMessage::new(msg.role.clone(), msg.content.clone()));
    }

    messages.push(ChatMessage::new("user", user_content));

    Ok(messages)
}

/// Quotes an email into an existing chat session by inserting a system message
/// containing the email subject, sender, and body text.
///
/// # Errors
///
/// Returns an error if the session or mail is not found, or the database write fails.
#[instrument(skip_all, fields(session_id = %session_id, mail_id = %mail_id), err(Debug))]
pub fn quote_mail_to_chat(
    db: &Database,
    session_id: &str,
    mail_id: &str,
) -> Result<AiChatMessage, AeroError> {
    debug!("quoting email into chat session");
    let subject = db.get_mail_subject(mail_id)?.unwrap_or_default();
    let from = db.get_mail_from_address(mail_id)?.unwrap_or_default();
    let body = db.get_mail_body_text(mail_id)?.unwrap_or_default();

    let content = format!("[Quoted Email]\nSubject: {subject}\nFrom: {from}\n\n{body}");

    db.insert_chat_message(session_id, "system", &content, None)
}

/// Sends a user message in a chat session, builds context, calls the AI provider,
/// and persists both the user message and the assistant reply.
///
/// # Errors
///
/// Returns an error if the session is not found, the database operations fail,
/// or the AI API request fails.
#[instrument(skip_all, fields(session_id = %session_id, content_len = user_content.len()), err(Debug))]
#[allow(clippy::too_many_lines)]
pub async fn send_message(
    ai: &AiService,
    session_id: &str,
    user_content: &str,
    tool_registry: &ToolRegistry,
) -> Result<AiChatMessage, AeroError> {
    debug!("loading chat session and history");
    let session = ai.db.get_chat_session(session_id)?;
    let history = ai.db.get_chat_messages(session_id)?;
    let mut messages = build_messages(&ai.db, &session, &history, user_content)?;

    let tools = tool_registry.list_openai_tools();
    let tools_opt = if tools.is_empty() {
        None
    } else {
        Some(tools.as_slice())
    };

    let mut last_result: Option<CompletionResult> = None;
    let mut assistant_content = String::new();
    let mut thinking: Option<String> = None;
    const MAX_TOOL_ROUNDS: usize = 5;

    for round in 0..MAX_TOOL_ROUNDS {
        debug!(provider_id = %session.provider_id, message_count = messages.len(), round, "calling AI provider");
        let result = ai
            .complete(&session.provider_id, &messages, tools_opt)
            .await?;
        last_result = Some(result.clone());

        let (round_content, round_thinking) = if let Some(reasoning) = result.reasoning.clone() {
            (extract_thinking(&result.content).0, Some(reasoning))
        } else {
            extract_thinking(&result.content)
        };

        if let Some(calls) = result.tool_calls.as_ref().filter(|c| !c.is_empty()) {
            messages.push(ChatMessage {
                role: "assistant".to_string(),
                content: round_content.clone(),
                tool_calls: Some(calls.clone()),
                tool_call_id: None,
            });
            for call in calls {
                let tool_result = tool_registry
                    .execute(call)
                    .await
                    .unwrap_or_else(|e| format!("Error: {e}"));
                messages.push(ChatMessage {
                    role: "tool".to_string(),
                    content: tool_result,
                    tool_calls: None,
                    tool_call_id: Some(call.id.clone()),
                });
            }
            assistant_content = round_content;
            thinking = round_thinking;
            continue;
        }

        assistant_content = round_content;
        thinking = round_thinking;
        break;
    }

    let last_result = last_result
        .ok_or_else(|| AeroError::AiApiError("no response from AI provider".to_string()))?;

    debug!("persisting chat messages");
    let prompt_tokens = last_result
        .usage
        .as_ref()
        .and_then(|u| u.prompt_tokens)
        .unwrap_or_else(|| count_messages(&messages, &session.model).total);
    let completion_tokens = last_result
        .usage
        .as_ref()
        .and_then(|u| u.completion_tokens)
        .unwrap_or_else(|| count_text(&assistant_content, &session.model).total);
    let total_tokens = last_result
        .usage
        .as_ref()
        .and_then(|u| u.total_tokens)
        .unwrap_or(prompt_tokens + completion_tokens);
    let estimated = last_result.usage.is_none();

    let cost = ai
        .db
        .get_ai_provider_pricing(&session.provider_id, &session.model)
        .ok()
        .flatten()
        .map(|pricing| {
            calculate_cost(
                prompt_tokens,
                completion_tokens,
                pricing.input_price_per_1k,
                pricing.output_price_per_1k,
            )
        });

    let mut conn = ai.db.connection()?;
    let tx = conn.transaction()?;
    let user_id = uuid::Uuid::new_v4().to_string();
    let assistant_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    tx.execute(
        "INSERT INTO ai_chat_messages (id, session_id, role, content, thinking, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (&user_id,
            session_id,
            "user",
            user_content,
            None::<&str>,
            now,
        ),
    )?;
    tx.execute(
        "INSERT INTO ai_chat_messages (id, session_id, role, content, thinking, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (&assistant_id,
            session_id,
            "assistant",
            &assistant_content,
            thinking.as_deref(),
            now,
        ),
    )?;
    tx.execute(
        "UPDATE ai_chat_sessions SET updated_at = ?1 WHERE id = ?2",
        (now, session_id),
    )?;
    tx.commit()?;
    drop(conn);

    let usage_log = AiUsageLog {
        id: uuid::Uuid::new_v4().to_string(),
        session_id: Some(session_id.to_string()),
        provider_id: session.provider_id.clone(),
        model: session.model.clone(),
        prompt_tokens,
        completion_tokens,
        total_tokens,
        estimated,
        cost,
        currency: "USD".to_string(),
        created_at: now,
    };
    if let Err(e) = ai.db.insert_ai_usage_log(&usage_log) {
        debug!(error = %e, "failed to insert ai usage log");
    }

    Ok(AiChatMessage {
        id: assistant_id,
        session_id: session_id.to_string(),
        role: "assistant".to_string(),
        content: assistant_content,
        thinking,
        created_at: now,
    })
}
