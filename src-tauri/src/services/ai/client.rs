use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::AeroError;
use crate::models::ai::{AiProvider, AiProviderKind};
use crate::services::ai::ChatMessage;

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessagePayload>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<serde_json::Value>>,
}

#[derive(Serialize, Deserialize)]
struct ChatMessagePayload {
    role: String,
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct Choice {
    message: MessagePayload,
}

#[derive(Deserialize, Debug, Clone)]
struct MessagePayload {
    content: Option<String>,
    #[serde(default)]
    reasoning_content: Option<String>,
    #[serde(default)]
    thinking: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<ToolCall>>,
}

/// A tool invocation returned by the model.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

/// Token usage reported by the provider, if available.
#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}

/// Result of a chat-completion request.
#[derive(Debug, Clone)]
pub struct CompletionResult {
    pub content: String,
    pub reasoning: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub usage: Option<Usage>,
}

/// Sends a chat completion request to an OpenAI-compatible API.
///
/// # Errors
///
/// Returns an error if the API key is invalid, the HTTP request fails,
/// or the response cannot be parsed.
pub async fn chat_completion(
    client: &Client,
    provider: &AiProvider,
    messages: &[ChatMessage],
    max_tokens: u32,
    tools: Option<&[serde_json::Value]>,
) -> Result<CompletionResult, AeroError> {
    let api_key_bytes = crate::services::crypto::decrypt_password(&provider.api_key_encrypted)
        .map_err(|_| AeroError::AiApiError("API key decryption failed".to_string()))?;
    let api_key = String::from_utf8(api_key_bytes)
        .map_err(|_| AeroError::AiApiError("API key is not valid UTF-8".to_string()))?;
    let base_url = provider
        .base_url
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| {
            AeroError::AiApiError("AI provider base_url is not configured".to_string())
        })?;
    let url = match provider.kind {
        AiProviderKind::Anthropic => format!("{}/messages", base_url.trim_end_matches('/')),
        _ => format!("{}/chat/completions", base_url.trim_end_matches('/')),
    };

    let payload_messages: Vec<ChatMessagePayload> = messages
        .iter()
        .map(|m| ChatMessagePayload {
            role: m.role.clone(),
            content: if m.content.is_empty() {
                None
            } else {
                Some(m.content.clone())
            },
            tool_calls: m.tool_calls.clone(),
            tool_call_id: m.tool_call_id.clone(),
        })
        .collect();

    let body = ChatCompletionRequest {
        model: provider.model.clone(),
        messages: payload_messages,
        max_tokens,
        tools: tools.map(<[serde_json::Value]>::to_vec),
    };

    let request_builder = match provider.kind {
        AiProviderKind::Anthropic => client
            .post(&url)
            .header("x-api-key", &api_key)
            .header("anthropic-version", "2023-06-01"),
        _ => client
            .post(&url)
            .header("Authorization", format!("Bearer {api_key}")),
    };

    let res = request_builder
        .json(&body)
        .send()
        .await
        .map_err(|e| AeroError::AiApiError(e.to_string()))?;

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        if status.as_u16() == 429 {
            return Err(AeroError::AiRateLimited);
        }
        return Err(AeroError::AiApiError(format!("HTTP {status}: {text}")));
    }

    let data: ChatCompletionResponse = res
        .json()
        .await
        .map_err(|e| AeroError::AiApiError(e.to_string()))?;

    let choice = data
        .choices
        .into_iter()
        .next()
        .ok_or_else(|| AeroError::AiApiError("empty response".to_string()))?;
    let message = choice.message;

    let content = message.content.unwrap_or_default();
    let reasoning = message.reasoning_content.or(message.thinking);

    Ok(CompletionResult {
        content,
        reasoning,
        tool_calls: message.tool_calls,
        usage: data.usage,
    })
}
