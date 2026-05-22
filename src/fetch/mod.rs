// Phase 1: HTTP Core & Basic Extraction
// Public API and orchestrator for fetching URLs and converting to markdown

mod convert;
mod envelope;
mod extract;
mod handlers;
mod http;
mod reduce;
pub mod request;
mod router;

pub use self::request::{OutputMode, RequestOptions};
use miette::Result;
use self::envelope::{ErrorEnvelope, SuccessEnvelope};
use self::http::HttpClient;
use self::reduce::reduce_markdown;

pub struct FetchOptions {
    pub timeout_secs: u64,
    pub max_redirects: u32,
    pub user_agent: String,
    pub compact: bool,
    pub max_body_words: Option<usize>,
    pub request: RequestOptions,
    #[allow(dead_code)]
    pub output_mode: OutputMode,
}

impl Default for FetchOptions {
    fn default() -> Self {
        Self {
            timeout_secs: 30,
            max_redirects: 5,
            user_agent: concat!("mdget/", env!("CARGO_PKG_VERSION")).to_string(),
            compact: false,
            max_body_words: None,
            request: RequestOptions::default(),
            output_mode: OutputMode::FrontmatterMarkdown,
        }
    }
}

/// Main entry point: fetch a URL and return markdown output with YAML frontmatter.
pub async fn fetch_url(url: &str, options: FetchOptions) -> Result<String> {
    // 1. Create HTTP client
    let client = HttpClient::new(options.timeout_secs, options.max_redirects, &options.user_agent);

    // 2. Fetch URL
    let response = match client.fetch(url, options.request.clone()).await {
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

    // 4. Route content type and dispatch to appropriate handler
    let handler_kind = router::route_content_type(response.content_type.as_deref());

    match handler_kind {
        router::HandlerKind::Text => {
            let charset_label = extract_charset_from_headers(response.content_type.as_deref())
                .unwrap_or_else(|| "utf-8".to_string());
            let encoding =
                encoding_rs::Encoding::for_label(charset_label.as_bytes()).unwrap_or(encoding_rs::UTF_8);
            let text = encoding.decode(&response.body).0.into_owned();

            let handler_res = handlers::text::handle(&text, options.max_body_words);

            let envelope = SuccessEnvelope {
                success: true,
                url: response.final_url,
                status: response.status,
                title: handler_res.title,
                word_count: handler_res.word_count,
                body_word_count: handler_res.body_word_count,
                render_mode: if options.compact { "compact".to_string() } else { "full".to_string() },
                body_word_limit: options.max_body_words,
                body_truncated: handler_res.truncated,
                fetched_at: chrono::Utc::now(),
                redirect_chain: if response.redirect_chain.is_empty() {
                    None
                } else {
                    Some(response.redirect_chain)
                },
            };

            envelope.to_output(&handler_res.body)
        }
        router::HandlerKind::Json => {
            let handler_res = handlers::json::handle(&response.body, response.content_type.as_deref());

            let envelope = SuccessEnvelope {
                success: true,
                url: response.final_url,
                status: response.status,
                title: handler_res.title,
                word_count: handler_res.word_count,
                body_word_count: handler_res.body_word_count,
                render_mode: if options.compact { "compact".to_string() } else { "full".to_string() },
                body_word_limit: options.max_body_words,
                body_truncated: handler_res.truncated,
                fetched_at: chrono::Utc::now(),
                redirect_chain: if response.redirect_chain.is_empty() {
                    None
                } else {
                    Some(response.redirect_chain)
                },
            };

            envelope.to_output(&handler_res.body)
        }
        router::HandlerKind::Feed => {
            let handler_res = handlers::feed::handle(&response.body, response.content_type.as_deref());

            let envelope = SuccessEnvelope {
                success: true,
                url: response.final_url,
                status: response.status,
                title: handler_res.title,
                word_count: handler_res.word_count,
                body_word_count: handler_res.body_word_count,
                render_mode: if options.compact { "compact".to_string() } else { "full".to_string() },
                body_word_limit: options.max_body_words,
                body_truncated: handler_res.truncated,
                fetched_at: chrono::Utc::now(),
                redirect_chain: if response.redirect_chain.is_empty() {
                    None
                } else {
                    Some(response.redirect_chain)
                },
            };

            envelope.to_output(&handler_res.body)
        }
        router::HandlerKind::Pdf => {
            let handler_res = match handlers::pdf::handle(&response.body) {
                Ok(res) => res,
                Err(msg) => {
                    return error_output(
                        response.final_url.clone(),
                        Some(response.status),
                        "pdf_extraction_failed",
                        msg,
                    );
                }
            };

            let envelope = SuccessEnvelope {
                success: true,
                url: response.final_url,
                status: response.status,
                title: handler_res.title,
                word_count: handler_res.word_count,
                body_word_count: handler_res.body_word_count,
                render_mode: if options.compact { "compact".to_string() } else { "full".to_string() },
                body_word_limit: options.max_body_words,
                body_truncated: handler_res.truncated,
                fetched_at: chrono::Utc::now(),
                redirect_chain: if response.redirect_chain.is_empty() {
                    None
                } else {
                    Some(response.redirect_chain)
                },
            };

            envelope.to_output(&handler_res.body)
        }
        router::HandlerKind::Html | router::HandlerKind::Unknown => {
            // Existing HTML pipeline (untouched for exact baseline compatibility)
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

            let reduced = reduce_markdown(&markdown, options.compact, options.max_body_words);

            // 7. Build SuccessEnvelope
            let envelope = SuccessEnvelope {
                success: true,
                url: response.final_url,
                status: response.status,
                title: article.title,
                word_count: article.word_count,
                body_word_count: reduced.body_word_count,
                render_mode: if options.compact { "compact".to_string() } else { "full".to_string() },
                body_word_limit: options.max_body_words,
                body_truncated: reduced.truncated,
                fetched_at: chrono::Utc::now(),
                redirect_chain: if response.redirect_chain.is_empty() {
                    None
                } else {
                    Some(response.redirect_chain)
                },
            };

            // 8. Return frontmatter + markdown
            envelope.to_output(&reduced.body)
        }
    }
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
        assert!(!opts.compact);
        assert_eq!(opts.max_body_words, None);
    }

    #[test]
    fn compact_reduction_keeps_headings_and_shortens_body() {
        let markdown = "# Title\n\nThis is the first paragraph with several words.\n\n## Details\n\nMore text here with another sentence.\n\n```rust\nfn example() {}\n```";
        let reduced = super::reduce::reduce_markdown(markdown, true, None);

        assert!(reduced.body.contains("# Title"));
        assert!(reduced.body.contains("## Details"));
        assert!(!reduced.body.contains("fn example"));
        assert!(reduced.body.split_whitespace().count() < markdown.split_whitespace().count());
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
