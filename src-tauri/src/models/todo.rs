use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoItem {
    pub id: String,
    pub text: String,
    pub done: bool,
    pub mail_id: Option<String>,
    pub created_at: i64,
    pub completed_at: Option<i64>,
    pub reminder_at: Option<i64>,
    pub notified_at: Option<i64>,
    #[serde(default)]
    pub completion_log: Vec<i64>,
}
