use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "lowercase")]
pub enum AiProviderKind {
    OpenAI,
    Anthropic,
    Gemini,
    #[serde(rename = "azure_openai")]
    AzureOpenAI,
    DeepSeek,
    Moonshot,
    Qwen,
    Zhipu,
    MiniMax,
    Baichuan,
    #[serde(rename = "custom_openai_compatible")]
    CustomOpenAICompatible,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiProviderSummary {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct AiChatMessage {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub thinking: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiUsageLog {
    pub id: String,
    pub session_id: Option<String>,
    pub provider_id: String,
    pub model: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub estimated: bool,
    pub cost: Option<f64>,
    pub currency: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiProviderPricing {
    pub id: String,
    pub provider_id: String,
    pub model: String,
    pub input_price_per_1k: f64,
    pub output_price_per_1k: f64,
    pub currency: String,
    pub effective_from: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiMcpServer {
    pub id: String,
    pub name: String,
    pub transport: AiMcpTransport,
    /// For stdio transport: the executable to spawn.
    pub command: Option<String>,
    /// For stdio transport: arguments passed to the executable.
    pub args: Option<Vec<String>>,
    /// For sse transport: the server URL.
    pub url: Option<String>,
    pub env_json: Option<String>,
    pub is_enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiMcpTransport {
    Stdio,
    Sse,
}

impl std::fmt::Display for AiMcpTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stdio => write!(f, "stdio"),
            Self::Sse => write!(f, "sse"),
        }
    }
}

impl rusqlite::types::ToSql for AiMcpTransport {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::Owned(
            rusqlite::types::Value::Text(self.to_string()),
        ))
    }
}

impl rusqlite::types::FromSql for AiMcpTransport {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value.as_str()? {
            "stdio" => Ok(Self::Stdio),
            "sse" => Ok(Self::Sse),
            other => Err(rusqlite::types::FromSqlError::Other(
                format!("unknown transport: {other}").into(),
            )),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    /// OpenAPI-compatible JSON schema object describing the skill parameters.
    pub input_schema_json: String,
    /// Executable command to run.
    pub command: String,
    /// Arguments appended to the command. The raw JSON arguments object is
    /// always written to the process stdin.
    pub args: Option<Vec<String>>,
    pub working_dir: Option<String>,
    pub timeout_seconds: Option<u32>,
    pub is_enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// A tool exposed to the LLM, produced by either an MCP server or a skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDefinition {
    pub name: String,
    pub description: Option<String>,
    /// OpenAPI-compatible parameter schema as a JSON value.
    pub input_schema: serde_json::Value,
    pub source: ToolSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolSource {
    Mcp { server_id: String },
    Skill { skill_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiUsageSummary {
    pub provider_id: String,
    pub model: String,
    pub total_prompt_tokens: u64,
    pub total_completion_tokens: u64,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub currency: String,
}
