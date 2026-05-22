use crate::config::Log;
use miette::{IntoDiagnostic, Result};
use std::fs;
use std::path::Path;
use std::str::FromStr;
use tracing::warn;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_log::LogTracer;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::prelude::*;

pub async fn configure_log(log_config: &Log) -> Result<Option<WorkerGuard>> {
    LogTracer::init().into_diagnostic()?;

    let mut is_fall_back = false;

    let stdout_subscriber = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_filter(LevelFilter::from_str(&log_config.level).unwrap_or_else(|_| {
            is_fall_back = true;
            LevelFilter::INFO
        }));

    if !log_config.file.enabled {
        let subscriber = tracing_subscriber::registry().with(stdout_subscriber);

        tracing::subscriber::set_global_default(subscriber).into_diagnostic()?;

        if is_fall_back {
            warn!("invalid log level '{}', fall back to info level", &log_config.level)
        }

        return Ok(None);
    }

    let log_dir = Path::new(&log_config.file.path);
    fs::create_dir_all(log_dir).into_diagnostic()?;

    let file_appender = tracing_appender::rolling::daily(&log_config.file.path, "rolling.log");

    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    let file_subscriber = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_ansi(false)
        .with_writer(file_writer)
        .with_filter(LevelFilter::from_str(&log_config.file.level).unwrap_or_else(|_| {
            is_fall_back = true;
            LevelFilter::INFO
        }));

    let subscriber = tracing_subscriber::registry().with(stdout_subscriber).with(file_subscriber);

    tracing::subscriber::set_global_default(subscriber).into_diagnostic()?;

    if is_fall_back {
        warn!("invalid log level '{}', fall back to info level", &log_config.level)
    }

    Ok(Some(guard))
}
