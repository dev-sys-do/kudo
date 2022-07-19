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
fn default_config_file() -> PathBuf {
    dirs::home_dir()
        .unwrap_or(Path::new(".").to_path_buf())
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

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    #[serde(default = "default_controller_url")]
    controller_url: String,
    #[serde(default = "default_log_level_str")]
    verbosity_level: String,
}

pub struct Config {
    pub config_file: PathBuf,
    pub controller_url: String,
    pub verbosity_level: LevelFilter,
}

impl Config {
    pub fn set_verbosity_level(&mut self, verbosity_level_str: &str) {
        self.verbosity_level = get_verbosity_level_from_string(verbosity_level_str);
    }
    pub fn set_controller_url(&mut self, controller_url: &str) {
        self.controller_url = controller_url.to_string();
    }
}

// Read the config file and return a Config object.
// If the file does not exist, creates one with the default values.
fn read_config_file(path: &PathBuf) -> Result<ConfigFile, Box<dyn std::error::Error>> {
    if !path.exists() {
        let parent = path.parent();

        if let Some(parent) = parent {
            create_dir_all(parent)?;
        }

        let config_file = File::create(path)?;

        let default = ConfigFile {
            controller_url: default_controller_url(),
            verbosity_level: default_log_level_str(),
        };
        serde_yaml::to_writer(config_file, &default)?;
        Ok(default)
    } else {
        let file = File::open(path)?;

        let conf: ConfigFile = serde_yaml::from_reader(file)?;
        Ok(conf)
    }
}

// Read the configuration from the config file and the environment variables.
// The environment variables override the values in the config file.
pub fn read_config() -> Result<Config, Box<dyn std::error::Error>> {
    // Read the config file

    let file_path = if let Ok(path) = env::var("KUDO_CONFIG") {
        Path::new(&path).to_path_buf()
    } else {
        default_config_file()
    };
    let config_file = read_config_file(&file_path)?;

    // get the verbosity level

    let verbosity_level_str = {
        let env_verbosity_level = env::var("KUDO_VERBOSITY_LEVEL");

        if env_verbosity_level.is_ok() {
            env_verbosity_level.unwrap()
        } else {
            config_file.verbosity_level
        }
    };

    let verbosity_level = get_verbosity_level_from_string(&verbosity_level_str);

    // get the right controller url

    let controller_url = {
        let env_controller_url = env::var("KUDO_CONTROLLER_URL");

        if env_controller_url.is_ok() {
            env_controller_url.unwrap()
        } else {
            config_file.controller_url
        }
    };

    Ok(Config {
        config_file: file_path,
        controller_url,
        verbosity_level,
    })
}
