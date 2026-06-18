use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ErrorPayload {
    pub code: String,
    pub args: Vec<String>,
}
