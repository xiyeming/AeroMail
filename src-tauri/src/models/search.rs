use serde::{Deserialize, Serialize};

/// Search result returned from full-text search.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub mail_id: String,
    pub score: f32,
    pub snippet: Option<String>,
}

/// Search query parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    pub query: String,
    pub folder_id: Option<String>,
    pub account_id: Option<String>,
    pub date_from: Option<i64>,
    pub date_to: Option<i64>,
    pub has_attachment: Option<bool>,
    pub is_read: Option<bool>,
}

/// Search index statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchStats {
    pub total_indexed: u64,
    pub last_index_time: Option<String>,
}
