use crate::fetch::{fetch_url, FetchOptions};
use miette::Result;
use std::path::PathBuf;
use tokio_graceful_shutdown::SubsystemHandle;

pub async fn run(
    _subsys: SubsystemHandle,
    url: String,
    output: Option<PathBuf>,
    timeout: u64,
    max_redirects: u32,
    user_agent: String,
) -> Result<()> {
    tracing::info!("Fetching URL: {}", url);

    let options = FetchOptions {
        timeout_secs: timeout,
        max_redirects,
        user_agent,
    };

    let result = fetch_url(&url, options).await?;

    match output {
        Some(path) => {
            std::fs::write(&path, result)?;
            tracing::info!("Output written to: {}", path.display());
        }
        None => {
            println!("{}", result);
        }
    }

    Ok(())
}
