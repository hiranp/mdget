use crate::fetch::{FetchOptions, fetch_url, OutputMode, RequestOptions, request::parse_header};
use miette::{IntoDiagnostic, Result};
use std::path::PathBuf;
use tokio_graceful_shutdown::SubsystemHandle;

#[allow(clippy::too_many_arguments)]
pub async fn run(
    _subsys: &mut SubsystemHandle,
    url: String,
    output: Option<PathBuf>,
    compact: bool,
    max_body_words: Option<usize>,
    timeout: u64,
    max_redirects: u32,
    user_agent: String,
    headers: Vec<String>,
    cookies: Vec<String>,
    bearer: Option<String>,
    json: bool,
    no_frontmatter: bool,
) -> Result<()> {
    tracing::info!("Fetching URL: {}", url);

    let mut parsed_headers = Vec::new();
    for h in headers {
        let (name, value) = parse_header(&h)?;
        parsed_headers.push((name, value));
    }

    let output_mode = if json {
        OutputMode::JsonEnvelope
    } else if no_frontmatter {
        OutputMode::MarkdownOnly
    } else {
        OutputMode::FrontmatterMarkdown
    };

    let request_options = RequestOptions {
        headers: parsed_headers,
        cookies,
        bearer,
    };

    let options = FetchOptions {
        timeout_secs: timeout,
        max_redirects,
        user_agent,
        compact,
        max_body_words,
        request: request_options,
        output_mode,
    };

    let result = fetch_url(&url, options).await?;

    match output {
        Some(path) => {
            std::fs::write(&path, result).into_diagnostic().map_err(|e| {
                miette::miette!("Failed to write output to {}: {e}", path.display())
            })?;
            tracing::info!("Output written to: {}", path.display());
        }
        None => {
            println!("{}", result);
        }
    }

    Ok(())
}
