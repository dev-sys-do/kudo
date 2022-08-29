mod config;
use clap::Parser;
mod client;
use logger::setup_env_logger;
mod logger;
mod resource;
mod subcommands;

/// Official CLI implementation for the kudo project
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Set verbosity level, can be 'trace', 'debug', 'info', 'warn' or 'error'
    ///
    /// Default: 'info', if the flag is set but no level is given, 'debug' is used.
    /// Logs are written to stderr, the only stdout output is the data returned by the command.
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
    // Parse the CLI arguments

    let cli = Cli::parse();

    // parse config

    let mut global_config = config::read_config()?;

    // set verbosity level

    if let Some(verbosity) = cli.verbosity {
        global_config.verbosity_level =
            config::get_verbosity_level_from_string(verbosity.as_deref().unwrap_or("debug"));
    }

    setup_env_logger(global_config.verbosity_level)?;

    if let Some(host) = cli.host.as_deref() {
        global_config.controller_url = host.to_string();
    }

    global_config.namespace = cli.namespace.unwrap_or_else(|| "default".to_string());

    subcommands::match_subcommand(cli.command, &global_config).await;

    Ok(())
}
