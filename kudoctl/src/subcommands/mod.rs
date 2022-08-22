use clap::Subcommand;
use log::info;

use crate::config;

mod apply;

#[derive(Subcommand)]
pub enum Subcommands {
    Apply(apply::Apply),
}

pub async fn match_subcommand(command: Subcommands, conf: &config::Config) {
    let result = match command {
        Subcommands::Apply(args) => apply::execute(args, conf),
    }
    .await;

    if result.is_err() {
        info!("{}", result.unwrap_err());
    }
}
