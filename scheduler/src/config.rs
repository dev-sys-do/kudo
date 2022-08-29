use serde_derive::{Deserialize, Serialize};

/// `Config` is a struct that contains the configuration of the scheduler.
///
/// Properties:
///
/// * `host`: The hostname or IP address of the gRPC server.
/// * `port`: The port that the gRPC server will listen on.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ControllerConfig {
    pub host: String,
    pub port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            host: "127.0.0.1".to_string(),
            port: 50052,
        }
    }
}
