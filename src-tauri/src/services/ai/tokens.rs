use crate::services::ai::ChatMessage;

/// Result of counting tokens for a list of messages.
#[derive(Debug, Clone, Copy)]
pub struct TokenCount {
    pub total: u32,
    /// Whether the value was estimated rather than returned by the provider.
    pub estimated: bool,
}

/// Counts tokens for a list of chat messages.
///
/// Uses `tiktoken-rs` for GPT-family models and a character-count heuristic
/// for other models. The heuristic is intentionally conservative and is
/// marked as estimated.
#[must_use]
pub fn count_messages(messages: &[ChatMessage], model: &str) -> TokenCount {
    let model_lower = model.to_lowercase();
    if model_lower.contains("gpt") {
        if let Some(count) = count_with_tiktoken(messages, model) {
            return TokenCount {
                total: count,
                estimated: false,
            };
        }
    }

    let total = messages
        .iter()
        .map(|m| heuristic_token_count(&m.content) + 4)
        .sum();
    TokenCount {
        total,
        estimated: true,
    }
}

/// Counts tokens for a plain text string.
#[must_use]
pub fn count_text(text: &str, model: &str) -> TokenCount {
    let model_lower = model.to_lowercase();
    if model_lower.contains("gpt") {
        if let Ok(bpe) = tiktoken_rs::bpe_for_model(model) {
            let count = bpe.encode_with_special_tokens(text).len();
            return TokenCount {
                total: u32::try_from(count).unwrap_or(u32::MAX),
                estimated: false,
            };
        }
    }

    TokenCount {
        total: heuristic_token_count(text),
        estimated: true,
    }
}

fn count_with_tiktoken(messages: &[ChatMessage], model: &str) -> Option<u32> {
    let tiktoken_messages: Vec<tiktoken_rs::ChatCompletionRequestMessage> = messages
        .iter()
        .map(|m| tiktoken_rs::ChatCompletionRequestMessage {
            role: m.role.clone(),
            content: Some(m.content.clone()),
            name: None,
            function_call: None,
            tool_calls: Vec::new(),
            refusal: None,
        })
        .collect();

    tiktoken_rs::num_tokens_from_messages(model, &tiktoken_messages)
        .ok()
        .and_then(|n| u32::try_from(n).ok())
}

/// Calculates the cost of a completion from per-1k token prices.
#[must_use]
pub fn calculate_cost(
    prompt_tokens: u32,
    completion_tokens: u32,
    input_price_per_1k: f64,
    output_price_per_1k: f64,
) -> f64 {
    let prompt = f64::from(prompt_tokens) / 1000.0 * input_price_per_1k;
    let completion = f64::from(completion_tokens) / 1000.0 * output_price_per_1k;
    (prompt + completion).max(0.0)
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
fn heuristic_token_count(text: &str) -> u32 {
    // Use character count as a conservative upper-bound estimate for
    // unknown tokenizers. This is close enough for CJK and overestimates
    // English, which is acceptable for an estimated value.
    ((text.chars().count() as f64) * 0.75).ceil() as u32
}
