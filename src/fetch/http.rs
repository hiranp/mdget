// HTTP client wrapper using ureq (pure Rust, blocking)

use std::time::Duration;

use miette::{Result, miette};
use ureq::ResponseExt;

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
        Self { timeout_secs, max_redirects, user_agent: user_agent.to_string() }
    }

    /// Fetch a URL. Runs ureq in `spawn_blocking` — never blocks the tokio runtime.
    ///
    /// # Thread-boundary note
    /// ureq is blocking, so we must run it inside `spawn_blocking`. ureq is `Send` and
    /// thread-safe, so no issues with move semantics. Redirect history is captured via
    /// `response.history()` which returns an iterator over each redirect step.
    pub async fn fetch(&self, url: &str) -> Result<HttpResponse> {
        // Validate before crossing the thread boundary
        url::Url::parse(url).map_err(|_| miette!("Invalid URL format: {url}"))?;

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
        // Build agent with explicit limits and redirect history enabled.
        let config = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .timeout_global(Some(Duration::from_secs(timeout_secs)))
            .max_redirects(max_redirects)
            .save_redirect_history(true)
            .user_agent(user_agent)
            .build();
        let agent = ureq::Agent::new_with_config(config);

        let response = agent.get(url).call().map_err(|e| match e {
            ureq::Error::Timeout(_) => miette!("Request timed out: {url}"),
            ureq::Error::HostNotFound => miette!("Could not resolve host: {url}"),
            ureq::Error::TooManyRedirects => miette!("Too many redirects: {url}"),
            other => miette!("HTTP transport error: {other}"),
        })?;

        let status = response.status().as_u16();
        let final_url = response.get_uri().to_string();
        let redirect_chain = response
            .get_redirect_history()
            .map(|history| history.iter().map(ToString::to_string).collect())
            .unwrap_or_default();
        let content_type = response
            .headers()
            .get(ureq::http::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(ToOwned::to_owned);

        let mut body_reader = response.into_body();
        let body =
            body_reader.read_to_vec().map_err(|e| miette!("Failed to read response body: {e}"))?;

        Ok(HttpResponse { status, body, final_url, redirect_chain, content_type })
    }
}
