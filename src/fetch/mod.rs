// Phase 1: HTTP Core & Basic Extraction
// Public API and orchestrator for fetching URLs and converting to markdown

mod convert;
mod envelope;
mod extract;
mod http;

use miette::Result;

use self::envelope::{ErrorEnvelope, SuccessEnvelope};
use self::http::HttpClient;

pub struct FetchOptions {
    pub timeout_secs: u64,
    pub max_redirects: u32,
    pub user_agent: String,
}

impl Default for FetchOptions {
    fn default() -> Self {
        Self { timeout_secs: 30, max_redirects: 5, user_agent: "mdget/0.2.0".to_string() }
    }
}

/// Main entry point: fetch a URL and return markdown output with YAML frontmatter.
pub async fn fetch_url(url: &str, options: FetchOptions) -> Result<String> {
    // 1. Create HTTP client
    let client = HttpClient::new(options.timeout_secs, options.max_redirects, &options.user_agent);

    // 2. Fetch URL
    let response = match client.fetch(url).await {
        Ok(resp) => resp,
        Err(e) => {
            let (error, message) = classify_http_error(&e.to_string(), url);
            return error_output(url.to_string(), None, error, message);
        }
    };

    // 3. Check for HTTP errors
    if response.status >= 400 {
        let error_msg = match response.status {
            404 => "Not Found",
            401 | 403 => "Forbidden",
            500..=599 => "Server Error",
            _ => "HTTP Error",
        };

        return error_output(
            response.final_url.clone(),
            Some(response.status),
            format!("http_{}", response.status),
            format!("HTTP {}: {}", response.status, error_msg),
        );
    }

    // 4. Detect charset and convert to UTF-8
    let charset_label = extract_charset_from_headers(response.content_type.as_deref())
        .or_else(|| extract_charset_from_html(&response.body))
        .unwrap_or_else(|| "utf-8".to_string());

    let encoding =
        encoding_rs::Encoding::for_label(charset_label.as_bytes()).unwrap_or(encoding_rs::UTF_8);
    let html = encoding.decode(&response.body).0.into_owned();

    // 5. Extract article
    let article = match extract::extract_article(&html) {
        Ok(a) => a,
        Err(e) => {
            return error_output(
                response.final_url.clone(),
                Some(response.status),
                "extraction_failed",
                format!("Failed to extract article: {e}"),
            );
        }
    };

    // 6. Convert to markdown
    let markdown = match convert::html_to_markdown(&article.content_html, &response.final_url) {
        Ok(md) => md,
        Err(e) => {
            return error_output(
                response.final_url.clone(),
                Some(response.status),
                "conversion_failed",
                format!("Failed to convert to markdown: {e}"),
            );
        }
    };

    // 7. Build SuccessEnvelope
    let envelope = SuccessEnvelope::new(
        response.final_url,
        response.status,
        article.title,
        article.word_count,
        if response.redirect_chain.is_empty() { None } else { Some(response.redirect_chain) },
    );

    // 8. Return frontmatter + markdown
    envelope.to_output(&markdown)
}

fn extract_charset_from_headers(content_type: Option<&str>) -> Option<String> {
    content_type.and_then(|ct| {
        ct.split(';').find_map(|param| {
            let param = param.trim();
            let (key, value) = param.split_once('=')?;
            key.trim()
                .eq_ignore_ascii_case("charset")
                .then(|| value.trim().trim_matches('"').trim_matches('\'').to_string())
                .filter(|value| !value.is_empty())
        })
    })
}

fn extract_charset_from_html(body: &[u8]) -> Option<String> {
    // Pre-scan first 1KB for <meta charset="...">
    let scan_limit = std::cmp::min(1024, body.len());
    let html_start = String::from_utf8_lossy(&body[..scan_limit]);
    let html_lower = html_start.to_ascii_lowercase();

    // Look for <meta charset="..."> or <meta charset='...'>
    if let Some(pos) = html_lower.find("charset") {
        let snippet = &html_start[pos..std::cmp::min(pos + 50, html_start.len())];
        // Extract charset value
        if snippet.contains('=') {
            let after_eq = snippet.split('=').nth(1)?;
            let charset = after_eq
                .trim_matches(|c| c == '"' || c == '\'' || c == '>')
                .split(['"', '\'', '>'])
                .next()?;
            return (!charset.is_empty()).then(|| charset.to_string());
        }
    }

    None
}

fn error_output(
    url: String,
    status: Option<u16>,
    error: impl Into<String>,
    message: impl Into<String>,
) -> Result<String> {
    ErrorEnvelope::new(url, status, error, message).to_yaml()
}

fn classify_http_error(raw: &str, url: &str) -> (&'static str, String) {
    let lower = raw.to_ascii_lowercase();
    if lower.contains("timed out") || lower.contains("timeout") {
        ("http_timeout", format!("Request timed out: {url}"))
    } else if lower.contains("resolve host") || lower.contains("host") {
        ("http_dns_error", format!("Could not resolve host: {url}"))
    } else if lower.contains("invalid url") {
        ("invalid_url", format!("Invalid URL format: {url}"))
    } else if lower.contains("too many redirects") {
        ("too_many_redirects", format!("Too many redirects while fetching: {url}"))
    } else {
        ("http_error", format!("HTTP request failed: {raw}"))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FetchOptions, classify_http_error, extract_charset_from_headers, extract_charset_from_html,
    };

    #[test]
    fn default_fetch_options_are_agent_safe() {
        let opts = FetchOptions::default();
        assert_eq!(opts.timeout_secs, 30);
        assert_eq!(opts.max_redirects, 5);
        assert!(opts.user_agent.starts_with("mdget/"));
    }

    #[test]
    fn detects_charset_from_header() {
        let charset = extract_charset_from_headers(Some("text/html; charset=iso-8859-1"));
        assert_eq!(charset.as_deref(), Some("iso-8859-1"));
    }

    #[test]
    fn detects_charset_from_header_case_insensitive_and_quoted() {
        let charset =
            extract_charset_from_headers(Some("text/html; Charset=\"windows-1252\"; q=0.9"));
        assert_eq!(charset.as_deref(), Some("windows-1252"));
    }

    #[test]
    fn ignores_empty_charset_parameter() {
        let charset = extract_charset_from_headers(Some("text/html; charset=  \"\""));
        assert_eq!(charset, None);
    }

    #[test]
    fn detects_charset_from_html_meta() {
        let body = br#"<html><head><meta charset='windows-1252'></head></html>"#;
        let charset = extract_charset_from_html(body);
        assert_eq!(charset.as_deref(), Some("windows-1252"));
    }

    #[test]
    fn detects_charset_from_html_case_insensitive_meta() {
        let body = br#"<html><head><META CHARSET="ISO-8859-2"></head></html>"#;
        let charset = extract_charset_from_html(body);
        assert_eq!(charset.as_deref(), Some("ISO-8859-2"));
    }

    #[test]
    fn classify_http_error_maps_common_failures() {
        let (kind, msg) = classify_http_error("operation timed out", "https://example.com");
        assert_eq!(kind, "http_timeout");
        assert!(msg.contains("Request timed out"));

        let (kind, _) = classify_http_error("too many redirects", "https://example.com");
        assert_eq!(kind, "too_many_redirects");

        let (kind, _) = classify_http_error("resolve host failed", "https://example.com");
        assert_eq!(kind, "http_dns_error");

        let (kind, _) = classify_http_error("some other transport issue", "https://example.com");
        assert_eq!(kind, "http_error");
    }
}
