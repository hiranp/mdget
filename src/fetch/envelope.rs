// YAML frontmatter structures and serialization

use miette::Result;
use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorEnvelope {
    pub success: bool, // always false
    pub url: String,
    pub status: Option<u16>,
    pub error: String,   // machine-readable: "http_timeout", "http_not_found", etc.
    pub message: String, // human-readable: "Request timed out after 30s"
    pub fetched_at: chrono::DateTime<chrono::Utc>,
}

impl ErrorEnvelope {
    pub fn new(
        url: String,
        status: Option<u16>,
        error: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            success: false,
            url,
            status,
            error: error.into(),
            message: message.into(),
            fetched_at: chrono::Utc::now(),
        }
    }

    pub fn to_yaml(&self) -> Result<String> {
        yaml_frontmatter(self)
    }
}

#[derive(Serialize)]
pub struct SuccessEnvelope {
    pub success: bool, // always true
    pub url: String,
    pub status: u16,
    pub title: Option<String>,
    pub word_count: usize,
    pub body_word_count: usize,
    pub render_mode: String,
    pub body_word_limit: Option<usize>,
    pub body_truncated: bool,
    pub fetched_at: chrono::DateTime<chrono::Utc>,
    pub redirect_chain: Option<Vec<String>>,
}

impl SuccessEnvelope {
    pub fn to_output(&self, markdown_body: &str) -> Result<String> {
        let frontmatter = yaml_frontmatter(self)?;
        Ok(format!("{}\n{}", frontmatter, markdown_body))
    }
}

pub fn yaml_frontmatter<T: Serialize>(data: &T) -> Result<String> {
    let yaml = serde_yml::to_string(data)
        .map_err(|e| miette::miette!("YAML serialization failed: {}", e))?;
    let cleaned = yaml.trim().trim_start_matches("---").trim();
    Ok(format!("---\n{}\n---\n", cleaned))
}
