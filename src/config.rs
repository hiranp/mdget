use config::{Config, Environment, File};
use directories::ProjectDirs;
use miette::{miette, IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use std::{env, path::PathBuf};

use crate::cli;

static QUALIFIER: &str = "";
static ORGANIZATION: &str = "";
static APPLICATION: &str = "mdget";

// default value <- global configuration file <- user configuration file <- environmen variables <- command line arguments
#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub log: Log,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Log {
    pub level: String,
    pub file: LogFile,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogFile {
    pub enabled: bool,
    pub path: String,
    pub level: String,
}

impl GlobalConfig {
    pub async fn new(cli: &cli::Cli) -> Result<GlobalConfig> {
        let project_dir = match ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION) {
            Some(v) => v,
            None => {
                return Err(miette!(
                    "error getting configurations path following XDG base directory"
                ))
            }
        };

        let home_path: PathBuf =
            cli.home
                .clone()
                .unwrap_or_else(|| match env::var("MDGET_HOME") {
                    Ok(v) => PathBuf::from(v),
                    Err(_) => project_dir.config_dir().to_path_buf(),
                });

        let system_config = format!("/etc/{APPLICATION}/config.toml");
        let user_config = home_path.join("config.toml");

        let mut config_builder = Config::builder()
            .set_default("log.level", "info")
            .into_diagnostic()?
            .set_default("log.file.enabled", false)
            .into_diagnostic()?
            .set_default(
                "log.file.path",
                project_dir
                    .cache_dir()
                    .to_str()
                    .ok_or(miette!("error getting cache path"))?,
            )
            .into_diagnostic()?
            .set_default("log.file.level", "info")
            .into_diagnostic()?
            .add_source(File::with_name(&system_config).required(false))
            .add_source(File::from(user_config).required(false))
            .add_source(
                Environment::with_prefix("MDGET")
                    .separator("_")
                    .ignore_empty(true),
            );

        config_builder = if let Some(log_level) = cli.log_level {
            config_builder
                .set_override("log.level", log_level.as_str())
                .into_diagnostic()?
        } else {
            config_builder
        };

        config_builder = if let Some(log_file_enable) = cli.log_file_enable {
            config_builder
                .set_override("log.file.enabled", log_file_enable)
                .into_diagnostic()?
        } else {
            config_builder
        };

        config_builder = if let Some(log_file_path) = cli.log_file_path.clone() {
            config_builder
                .set_override("log.file.path", log_file_path.to_str())
                .into_diagnostic()?
        } else {
            config_builder
        };

        config_builder = if let Some(log_file_level) = cli.log_file_level {
            config_builder
                .set_override("log.file.level", log_file_level.as_str())
                .into_diagnostic()?
        } else {
            config_builder
        };

        config_builder
            .build()
            .into_diagnostic()?
            .try_deserialize()
            .into_diagnostic()
    }
}
