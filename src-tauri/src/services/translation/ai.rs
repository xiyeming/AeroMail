use crate::error::AeroError;
use crate::models::ai::AiProvider;
use crate::services::ai::ChatMessage;

/// Translates `source_text` into `target_lang` via an AI provider.
///
/// Uses the async `chat_completion` client directly.
///
/// # Errors
///
/// Returns an error if the API call fails.
pub async fn translate(
    ai_provider: &AiProvider,
    source_text: &str,
    target_lang: &str,
) -> Result<String, AeroError> {
    let prompt = format!(
        "Translate the following text to {target_lang}. \
         Only output the translation, no explanation:\n\n{source_text}"
    );

    let messages = vec![ChatMessage::new("user", prompt)];

    let client = reqwest::Client::new();
    let max_tokens = ai_provider.max_tokens.unwrap_or(2048);

    crate::services::ai::client::chat_completion(&client, ai_provider, &messages, max_tokens, None)
        .await
        .map(|result| result.content)
}
