// HTTP client wrapper using ureq (pure Rust, blocking)
// Implementation in Task 3

use miette::Result;

pub struct HttpClient {
    timeout_secs: u64,
    max_redirects: u32,
    user_agent: String,
}

pub struct HttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
    pub final_url: String,
    pub redirect_chain: Vec<String>,
    pub content_type: Option<String>,
}

impl HttpClient {
    pub fn new(timeout_secs: u64, max_redirects: u32, user_agent: &str) -> Self {
        Self {
            timeout_secs,
            max_redirects,
            user_agent: user_agent.to_string(),
        }
    }

    /// Fetch a URL. Runs ureq in `spawn_blocking` — never blocks the tokio runtime.
    pub async fn fetch(&self, _url: &str) -> Result<HttpResponse> {
        todo!("Implement in Task 3")
    }
}
