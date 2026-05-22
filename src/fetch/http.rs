// HTTP client wrapper using ureq (pure Rust, blocking)

use miette::{miette, Result};
use std::io::Read;
use std::time::Duration;

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
    ///
    /// # Thread-boundary note
    /// ureq is blocking, so we must run it inside `spawn_blocking`. ureq is `Send` and
    /// thread-safe, so no issues with move semantics. Redirect history is captured via
    /// `response.history()` which returns an iterator over each redirect step.
    pub async fn fetch(&self, url: &str) -> Result<HttpResponse> {
        let url = url.to_string();
        let timeout = self.timeout_secs;
        let max_redirects = self.max_redirects;
        let user_agent = self.user_agent.clone();

        tokio::task::spawn_blocking(move || {
            Self::fetch_blocking(&url, timeout, max_redirects, &user_agent)
        })
        .await
        .map_err(|e| miette!("HTTP fetch task panicked: {e}"))?
    }

    fn fetch_blocking(
        url: &str,
        timeout_secs: u64,
        max_redirects: u32,
        user_agent: &str,
    ) -> Result<HttpResponse> {
        // Create ureq agent with timeout and max redirects
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(timeout_secs))
            .redirects(max_redirects)
            .build();

        // Send request
        let response = agent
            .get(url)
            .set("User-Agent", user_agent)
            .call()
            .map_err(|e| match e {
                ureq::Error::Status(code, _resp) => {
                    miette!("HTTP error {}: {}", code, url)
                }
                ureq::Error::Transport(t) => {
                    miette!("HTTP transport error: {}", t)
                }
            })?;

        // Collect redirect history
        let redirect_chain: Vec<String> = response
            .history()
            .iter()
            .map(|r| r.get_url().to_string())
            .collect();

        // Get final URL and status
        let final_url = response.get_url().to_string();
        let status = response.status();
        let content_type = response.header("content-type").map(|s| s.to_string());

        // Read body as bytes
        let mut body = Vec::new();
        response
            .into_reader()
            .read_to_end(&mut body)
            .map_err(|e| miette!("Failed to read response body: {}", e))?;

        Ok(HttpResponse {
            status,
            body,
            final_url,
            redirect_chain,
            content_type,
        })
    }
}
