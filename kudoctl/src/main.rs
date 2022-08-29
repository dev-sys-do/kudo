mod config;
use chrono::Utc;
use clap::Parser;
mod client;
use log::LevelFilter;
mod resource;
use std::io::Write;
mod subcommands;

/// Official CLI implementation for the kudo project
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Set verbosity level, can be 'debug', 'info', 'warn' or 'error'
    ///
    /// Default: 'info', if the flag is set but no level is given, 'debug' is used.
    #[clap(short, long)]
    #[clap(short, long, global = true)]
    verbosity: Option<Option<String>>,

    /// Set the controller adress
    ///
    /// This has priority over the config file and enviorment variable.
    #[clap(short, long, global = true)]
    host: Option<String>,

    /// Define which namespace to target
    /// If not defined, the default namespace is used
    #[clap(short, long, global = true)]
    namespace: Option<String>,

    /// Execute a command on the connected cluster
    #[clap(subcommand)]
    command: subcommands::Subcommands,
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

    // Set namespace if defined
    if let Some(namespace) = cli.namespace {
        global_config.namespace = namespace;
    }

    subcommands::match_subcommand(cli.command, &global_config).await;

    Ok(())
}
