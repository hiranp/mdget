mod cli;
mod commands;
mod config;
mod log;

use clap::Parser;
use cli::{Cli, Commands};
use commands::completion;
use miette::Result;
use tokio::time::Duration;
use tokio_graceful_shutdown::{SubsystemBuilder, SubsystemHandle, Toplevel};

use crate::commands::command1;
use crate::commands::command2;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let global_config = config::GlobalConfig::new(&cli).await?;
    let _guard = log::configure_log(&global_config.log).await?;

    match cli.command {
        // NOTE: see also https://github.com/Finomnis/tokio-graceful-shutdown/tree/main/examples
        Commands::Fetch {
            url,
            output,
            compact,
            max_body_words,
            timeout,
            max_redirects,
            user_agent,
            header,
            cookie,
            bearer,
            json,
            no_frontmatter,
        } => Toplevel::new(move |s: &mut SubsystemHandle| {
            s.start(SubsystemBuilder::new(
                "fetch",
                async move |subsys: &mut SubsystemHandle| -> Result<()> {
                    crate::commands::fetch::run(
                        subsys,
                        url.clone(),
                        output.clone(),
                        compact,
                        max_body_words,
                        timeout,
                        max_redirects,
                        user_agent.clone(),
                        header.clone(),
                        cookie.clone(),
                        bearer.clone(),
                        json,
                        no_frontmatter,
                    )
                    .await
                },
            ));
            async {}
        })
        .catch_signals()
        .handle_shutdown_requests(Duration::from_millis(1000))
        .await
        .map_err(Into::into),
        Commands::Command1 => Toplevel::new(|s: &mut SubsystemHandle| {
            s.start(SubsystemBuilder::new("command1", command1::run));
            async {}
        })
        .catch_signals()
        .handle_shutdown_requests(Duration::from_millis(1000))
        .await
        .map_err(Into::into),
        Commands::Command2 { arg } => Toplevel::new(move |s: &mut SubsystemHandle| {
            s.start(SubsystemBuilder::new(
                "command2",
                async move |subsys: &mut SubsystemHandle| -> Result<()> {
                    command2::run(subsys, arg).await
                },
            ));
            async {}
        })
        .catch_signals()
        .handle_shutdown_requests(Duration::from_millis(1000))
        .await
        .map_err(Into::into),
        Commands::Completion { shell } => {
            completion::run(shell).await;
            Ok(())
        }
    }
}
