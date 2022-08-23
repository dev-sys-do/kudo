use std::fmt::Display;

use anyhow::{Context, Result};
use clap::ValueEnum;
use serde::Serialize;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum OutputFormat {
    /// Human readable format
    HumanReadable,
    /// JSON format
    Json,
    /// YAML format
    Yaml,
}

/// Formats the output of a `get` command.
pub fn format_output<T: Serialize + Display>(output: T, format: OutputFormat) -> Result<String> {
    match format {
        OutputFormat::HumanReadable => Ok(format!("{}", output)),
        OutputFormat::Json => serde_json::to_string(&output).map_err(anyhow::Error::from),
        OutputFormat::Yaml => serde_yaml::to_string(&output).map_err(anyhow::Error::from),
    }
    .context("Error formatting output")
}
