use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProvider {
    pub id: String,
    pub name: String,
    pub kind: AiProviderKind,
    pub api_key_encrypted: Vec<u8>,
    pub base_url: Option<String>,
    pub model: String,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiProviderKind {
    OpenAI,
    Anthropic,
    Gemini,
    AzureOpenAI,
    DeepSeek,
    Moonshot,
    Qwen,
    Zhipu,
    MiniMax,
    Baichuan,
    CustomOpenAICompatible,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProviderSummary {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiChatSession {
    pub id: String,
    pub title: Option<String>,
    pub provider_id: String,
    pub model: String,
    pub context_mail_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiChatMessage {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub created_at: i64,
}
