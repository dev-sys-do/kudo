mod config;
use chrono::Utc;
use clap::Parser;
mod request;
use log::LevelFilter;
use reqwest;
mod resource;
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
        global_config.verbosity_level =
            config::get_verbosity_level_from_string(verbosity.as_deref().unwrap_or("debug"));
    }

    env_logger::builder()
        .format(move |buf, record| {
            let utc = Utc::now();

            match global_config.verbosity_level {
                // Write the file path and more time details if we are in trace mode
                LevelFilter::Trace => writeln!(
                    buf,
                    "{} - {}:{} [{}] {}",
                    utc,
                    record.file().unwrap_or(""),
                    record.line().unwrap_or(0),
                    record.level(),
                    record.args()
                ),
                _ => writeln!(
                    buf,
                    "{} [{:5}] {}",
                    utc.format("%F %T"),
                    record.level(),
                    record.args()
                ),
            }
        })
        .filter_level(global_config.verbosity_level)
        .try_init()?;

    if let Some(host) = cli.host.as_deref() {
        global_config.controller_url = host.to_string();
    }

    let client = reqwest::Client::new();

    Ok(())
}
