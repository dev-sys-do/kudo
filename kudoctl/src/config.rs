use std::{
    env,
    fs::{create_dir_all, File},
    path::{Path, PathBuf},
};

use log::LevelFilter;
use serde::{Deserialize, Serialize};

// Defaults configuration values

fn default_log_level_str() -> String {
    "info".to_string()
}
fn default_controller_url() -> String {
    "http://localhost:8080".to_string()
}
fn default_config_file_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| Path::new(".").to_path_buf())
        .join(".kudo")
        .join("config.yaml")
}

// Returns the right LevelFilter for the given log level string.
pub fn get_verbosity_level_from_string(verbosity_level_str: &str) -> LevelFilter {
    match verbosity_level_str {
        "off" => LevelFilter::Off,
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        _ => LevelFilter::Info,
    }
}

// Serializable configuration struct
#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    #[serde(default = "default_controller_url")]
    controller_url: String,
    #[serde(default = "default_log_level_str")]
    verbosity_level: String,
}

impl Default for ConfigFile {
    fn default() -> Self {
        ConfigFile {
            controller_url: default_controller_url(),
            verbosity_level: default_log_level_str(),
        }
    }
}

// This struct contains the configuration for the application.
pub struct Config {
    #[allow(dead_code)]
    config_file: PathBuf,
    pub controller_url: String,
    pub verbosity_level: LevelFilter,
}

// Read the config file and return a Config object.
// If the file does not exist, creates one with the default values.
fn read_config_file(path: &PathBuf) -> Result<ConfigFile, Box<dyn std::error::Error>> {
    let config_file: ConfigFile = if !path.exists() {
        let parent = path.parent();

        // Create the config directory if it doesn't exist.
        if let Some(parent) = parent {
            create_dir_all(parent)?;
        }

        let file_handler = File::create(path)?;

        let default = Default::default();
        serde_yaml::to_writer(file_handler, &default)?;
        default
    } else {
        let file = File::open(path)?;
        serde_yaml::from_reader(file)?
    };
    Ok(config_file)
}

// Read the configuration from the config file and the environment variables.
// The environment variables override the values in the config file.
pub fn read_config() -> Result<Config, Box<dyn std::error::Error>> {
    // Read the config file

    let file_path = env::var("KUDO_CONFIG")
        .map(|path| Path::new(&path).to_path_buf())
        .unwrap_or_else(|_| default_config_file_path());

    let config_file = read_config_file(&file_path)?;

    // get the verbosity level

    let verbosity_level_string =
        check_env_override("KUDO_VERBOSITY_LEVEL", &config_file.verbosity_level);

    let verbosity_level = get_verbosity_level_from_string(&verbosity_level_string);

    // get the right controller url

    let controller_url = check_env_override("KUDO_CONTROLLER_URL", &config_file.controller_url);

    Ok(Config {
        config_file: file_path,
        controller_url,
        verbosity_level,
    })
}

// Reads the environment variable and returns the value if it is set, returns the `config_var` otherwise.
fn check_env_override(env_var: &str, config_var: &str) -> String {
    env::var(env_var).unwrap_or_else(|_| config_var.to_string())
}
