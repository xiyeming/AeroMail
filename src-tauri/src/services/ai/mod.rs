pub mod chat;
pub mod client;
pub mod providers;

use crate::db::pool::Database;
use crate::error::AeroError;
use crate::models::ai::{AiChatMessage, AiChatSession, AiProviderSummary};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

pub struct AiService {
    pub db: Arc<Database>,
    pub client: reqwest::Client,
}

impl AiService {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            client: reqwest::Client::new(),
        }
    }

    /// Lists all configured AI providers as summaries.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub fn list_providers(&self) -> Result<Vec<AiProviderSummary>, AeroError> {
        let providers = self.db.list_ai_providers()?;
        Ok(providers
            .into_iter()
            .map(|p| AiProviderSummary {
                id: p.id,
                name: p.name,
                kind: format!("{:?}", p.kind),
                model: p.model,
            })
            .collect())
    }

    /// Retrieves a chat session by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found or the query fails.
    pub fn get_session(&self, id: &str) -> Result<AiChatSession, AeroError> {
        self.db.get_chat_session(id)
    }

    /// Lists all chat sessions ordered by most recently updated.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub fn list_sessions(&self) -> Result<Vec<AiChatSession>, AeroError> {
        self.db.list_chat_sessions()
    }

    /// Retrieves all messages for a chat session.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub fn get_messages(&self, session_id: &str) -> Result<Vec<AiChatMessage>, AeroError> {
        self.db.get_chat_messages(session_id)
    }

    /// Sends a chat completion request to the specified provider.
    ///
    /// # Errors
    ///
    /// Returns an error if the provider is not found or the API request fails.
    pub async fn complete(
        &self,
        provider_id: &str,
        messages: Vec<ChatMessage>,
    ) -> Result<String, AeroError> {
        let provider = self.db.get_ai_provider(provider_id)?;
        let max_tokens = provider.max_tokens.unwrap_or(4096);
        client::chat_completion(&self.client, &provider, messages, max_tokens).await
    }
}
