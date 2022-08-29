use std::io;
use thiserror::Error;
use tokio::task::JoinError;

pub mod config;
pub mod event;
pub mod instance;
pub mod manager;
pub mod node;
pub mod orchestrator;
pub mod parser;
pub mod storage;

#[derive(Error, Debug)]
pub enum SchedulerError {
    #[error("unable to read the configuration file's path")]
    ConfigPathReadError(#[from] io::Error),
    #[error("unable to read the configuration file")]
    ConfigReadError(#[from] confy::ConfyError),
    #[error("invalid grpc address in configuration file")]
    InvalidGrpcAddress,
    #[error("proxy error: {0}")]
    ProxyError(#[from] ProxyError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
    #[error("unknown scheduler error")]
    Unknown,
}

#[derive(Error, Debug)]
pub enum ManagerError {
    #[error("unable to connect to the controller: {0}")]
    CannotConnectToController(#[from] ProxyError),
    #[error("an error occurred while joining the main thread: {0}")]
    FromTaskError(#[from] JoinError),
}

#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("an transport error occurred from tonic: {0}")]
    TonicTransportError(#[from] tonic::transport::Error),
    #[error("an status error occurred from tonic: {0}")]
    TonicStatusError(#[from] tonic::Status),
    #[error("the gRPC client was not found")]
    GrpcClientNotFound,
    #[error("the gRPC stream was not found")]
    GrpcStreamNotFound,
    #[error("an error occurred while sending a message to the channel")]
    ChannelSenderError,
}

pub type NodeIdentifier = String;
pub type InstanceIdentifier = String;
