// Phase 1: HTTP Core & Basic Extraction
// Public API and orchestrator for fetching URLs and converting to markdown

mod convert;
mod envelope;
mod extract;
mod http;

pub use envelope::{ErrorEnvelope, SuccessEnvelope};
pub use extract::ExtractedArticle;
pub use http::{HttpClient, HttpResponse};

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
pub async fn fetch_url(url: &str, options: FetchOptions) -> Result<String> {
    // 1. Create HTTP client
    let client = HttpClient::new(
        options.timeout_secs,
        options.max_redirects,
        &options.user_agent,
    );

    // 2. Fetch URL
    let response = match client.fetch(url).await {
        Ok(resp) => resp,
        Err(e) => {
            let envelope = ErrorEnvelope {
                success: false,
                url: url.to_string(),
                status: None,
                error: "http_error".to_string(),
                message: format!("HTTP request failed: {}", e),
                fetched_at: chrono::Utc::now(),
            };
            return Ok(envelope.to_yaml()?);
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

        let envelope = ErrorEnvelope {
            success: false,
            url: response.final_url.clone(),
            status: Some(response.status),
            error: format!("http_{}", response.status),
            message: format!("HTTP {}: {}", response.status, error_msg),
            fetched_at: chrono::Utc::now(),
        };
        return Ok(envelope.to_yaml()?);
    }

    // 4. Detect charset and convert to UTF-8
    let charset_label = extract_charset_from_headers(&response.content_type)
        .or_else(|| extract_charset_from_html(&response.body))
        .unwrap_or_else(|| "utf-8".to_string());

    let encoding = encoding_rs::Encoding::for_label(charset_label.as_bytes())
        .unwrap_or(encoding_rs::UTF_8);
    let html = encoding.decode(&response.body).0.into_owned();

    // 5. Extract article
    let article = match extract::extract_article(&html) {
        Ok(a) => a,
        Err(e) => {
            let envelope = ErrorEnvelope {
                success: false,
                url: response.final_url.clone(),
                status: Some(response.status),
                error: "extraction_failed".to_string(),
                message: format!("Failed to extract article: {}", e),
                fetched_at: chrono::Utc::now(),
            };
            return Ok(envelope.to_yaml()?);
        }
    };

    // 6. Convert to markdown
    let markdown = match convert::html_to_markdown(&article.content_html, &response.final_url) {
        Ok(md) => md,
        Err(e) => {
            let envelope = ErrorEnvelope {
                success: false,
                url: response.final_url.clone(),
                status: Some(response.status),
                error: "conversion_failed".to_string(),
                message: format!("Failed to convert to markdown: {}", e),
                fetched_at: chrono::Utc::now(),
            };
            return Ok(envelope.to_yaml()?);
        }
    };

    // 7. Build SuccessEnvelope
    let envelope = SuccessEnvelope {
        success: true,
        url: response.final_url,
        status: response.status,
        title: article.title,
        word_count: article.word_count,
        fetched_at: chrono::Utc::now(),
        redirect_chain: if response.redirect_chain.is_empty() {
            None
        } else {
            Some(response.redirect_chain)
        },
    };

    // 8. Return frontmatter + markdown
    Ok(envelope.to_output(&markdown)?)
}

fn extract_charset_from_headers(content_type: &Option<String>) -> Option<String> {
    content_type.as_ref().and_then(|ct| {
        // Look for "charset=..." in Content-Type header
        ct.split(';').find_map(|param| {
            let param = param.trim();
            if param.starts_with("charset=") {
                Some(param[8..].trim_matches('"').to_string())
            } else {
                None
            }
        })
    })
}

fn extract_charset_from_html(body: &[u8]) -> Option<String> {
    // Pre-scan first 1KB for <meta charset="...">
    let scan_limit = std::cmp::min(1024, body.len());
    let html_start = String::from_utf8_lossy(&body[..scan_limit]);

    // Look for <meta charset="..."> or <meta charset='...'>
    if let Some(pos) = html_start.find("charset") {
        let snippet = &html_start[pos..std::cmp::min(pos + 50, html_start.len())];
        // Extract charset value
        if snippet.contains('=') {
            let after_eq = snippet.split('=').nth(1)?;
            let charset = after_eq
                .trim_matches(|c| c == '"' || c == '\'' || c == '>')
                .split(|c| c == '"' || c == '\'' || c == '>')
                .next()?
                .to_string();
            return if charset.is_empty() {
                None
            } else {
                Some(charset)
            };
        }
    }

    None
}
