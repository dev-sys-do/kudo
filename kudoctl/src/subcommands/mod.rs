use crate::config;
use clap::Subcommand;
use log::error;
mod apply;
mod delete;
mod get;

#[derive(Subcommand)]
pub enum Subcommands {
    Apply(apply::Apply),
    Get(get::GetSubcommand),
    Delete(delete::Subcommand),
}

/// Match the subcommand to execute
///
/// # Organisation
/// Every subcommand has its own module,
/// with a struct to configure the arguments and an execute function.
pub async fn match_subcommand(command: Subcommands, conf: &config::Config) {
    let result = match command {
        Subcommands::Apply(args) => apply::execute(args, conf).await,
        Subcommands::Get(args) => get::execute(args, conf).await,
        Subcommands::Delete(args) => delete::execute(args, conf).await,
    };

    // Print the result or the error
    match result {
        Ok(result) => {
            if !result.is_empty() {
                println!("{}", result);
            }
        }
        Err(err) => {
            error!("{}", err);
        }
    }
}
