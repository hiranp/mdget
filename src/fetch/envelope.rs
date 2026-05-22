// YAML frontmatter structures and serialization
// Implementation in Task 4

use miette::Result;
use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorEnvelope {
    pub success: bool,                          // always false
    pub url: String,
    pub status: Option<u16>,
    pub error: String,                          // machine-readable: "http_timeout", etc.
    pub message: String,                        // human-readable
    pub fetched_at: chrono::DateTime<chrono::Utc>,
}

impl ErrorEnvelope {
    pub fn to_yaml(&self) -> Result<String> {
        todo!("Implement in Task 4")
    }
}

#[derive(Serialize)]
pub struct SuccessEnvelope {
    pub success: bool,                          // always true
    pub url: String,
    pub status: u16,
    pub title: Option<String>,
    pub word_count: usize,
    pub fetched_at: chrono::DateTime<chrono::Utc>,
    pub redirect_chain: Option<Vec<String>>,
}

impl SuccessEnvelope {
    pub fn to_output(&self, _markdown_body: &str) -> Result<String> {
        todo!("Implement in Task 7")
    }
}

pub fn yaml_frontmatter<T: Serialize>(_data: &T) -> Result<String> {
    todo!("Implement in Task 4")
}
