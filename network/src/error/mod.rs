use std::fmt::Display;

/// Represent common errors raised by the Kudo's network crate
#[derive(Debug)]
pub enum KudoNetworkError {
    /// Error when executing the command
    CommandError(Box<dyn std::error::Error>),
    /// The command executed but the exit code was not 0
    CommandFailed(String),
    /// Failed to find the default network interface
    DefaultNetworkInterfaceError(String),
    /// Failed to enable route_localnet setting
    RouteLocalnetError(String),
    /// Failed to enable ip_forward setting
    IPForwardError(String),
}

impl Display for KudoNetworkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KudoNetworkError::CommandError(e) => write!(f, "Command error: {}", e),
            KudoNetworkError::CommandFailed(e) => write!(f, "Command failed: {}", e),
            KudoNetworkError::DefaultNetworkInterfaceError(e) => {
                write!(f, "Default network interface error: {}", e)
            }
            KudoNetworkError::RouteLocalnetError(e) => write!(f, "Route localnet error: {}", e),
            KudoNetworkError::IPForwardError(e) => write!(f, "IP forward error: {}", e),
        }
    }
}

impl std::error::Error for KudoNetworkError {}
