mod config;
use chrono::{DateTime, Local, Utc};
use clap::{Parser, Subcommand};
use log::{debug, error, info, warn, LevelFilter};
use reqwest;
use std::io::Write;

/// Official CLI implementation for the kudo project
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Set verbosity level, can be 'debug', 'info', 'warn' or 'error'
    ///
    /// Default: 'info', if the flag is set but no level is given, 'debug' is used.
    #[clap(short, long)]
    verbosity: Option<Option<String>>,

    /// Set the controller adress
    ///
    /// This has priority over the config file and enviorment variable.
    #[clap(short, long)]
    host: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let mut global_config = config::read_config()?;

    if let Some(verbosity) = cli.verbosity {
        global_config.set_verbosity_level(verbosity.as_deref().unwrap_or("debug"));
    }

    env_logger::builder()
        .filter_level(global_config.verbosity_level)
        .try_init()?;

    if let Some(host) = cli.host.as_deref() {
        global_config.set_controller_url(host);
    }

    let client = reqwest::Client::new();

    Ok(())
}
