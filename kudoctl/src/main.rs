mod config;
use clap::{Parser, Subcommand};
use log::{trace, LevelFilter};
use reqwest;

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

    let mut global_config = config::read_config();

    if let Some(verbosity) = cli.verbosity {
        // TODO: Alter the configuration when implemented
        println!(
            "Using verbosity : {}",
            verbosity.unwrap_or_else(|| "debug".to_string())
        );
    }

    if let Some(host) = cli.host.as_deref() {
        // TODO: Alter the configuration when implemented
        println!("Using host : {}", host);
    }

    let level = LevelFilter::Trace;

    env_logger::builder().filter_level(level);
    trace!("Logger initialized");

    let client = reqwest::Client::new();

    trace!("reqwest Client initialized");
    Ok(())
}
