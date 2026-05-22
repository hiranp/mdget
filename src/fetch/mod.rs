// Phase 1: HTTP Core & Basic Extraction
// Public API and orchestrator for fetching URLs and converting to markdown

mod http;
mod extract;
mod convert;
mod envelope;

pub use http::{HttpClient, HttpResponse};
pub use extract::ExtractedArticle;
pub use envelope::{SuccessEnvelope, ErrorEnvelope};

use miette::Result;

pub struct FetchOptions {
    pub timeout_secs: u64,
    pub max_redirects: u32,
    pub user_agent: String,
}

impl Default for FetchOptions {
    fn default() -> Self {
        Self {
            timeout_secs: 30,
            max_redirects: 5,
            user_agent: "mdget/0.2.0".to_string(),
        }
    }
}

/// Main entry point: fetch a URL and return markdown output with YAML frontmatter.
/// Stub - implementation in Task 8
pub async fn fetch_url(_url: &str, _options: FetchOptions) -> Result<String> {
    todo!("Implement in Task 8")
}
