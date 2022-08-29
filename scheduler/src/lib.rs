use std::net::IpAddr;

use proto::scheduler::{
    Instance, InstanceStatus, NodeRegisterRequest, NodeRegisterResponse, NodeStatus,
    NodeUnregisterRequest, NodeUnregisterResponse,
};
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};
use tonic::Response;

pub mod config;
pub mod instance;
pub mod manager;
pub mod node;
pub mod orchestrator;
pub mod storage;

#[derive(Error, Debug)]
pub enum SchedulerError {
    #[error("unable to read the configuration file's path")]
    ConfigPathReadError(#[from] std::io::Error),
    #[error("unable to read the configuration file")]
    ConfigReadError(#[from] confy::ConfyError),
    #[error("invalid grpc address in configuration file")]
    InvalidGrpcAddress,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
    #[error("unknown scheduler error")]
    Unknown,
}

#[derive(Debug)]
pub enum ProxyError {
    TonicTransportError(tonic::transport::Error),
    TonicStatusError(tonic::Status),
    GrpcClientNotFound,
    ChannelSenderError,
    InvalidStatus,
}

pub type NodeIdentifier = String;
pub type InstanceIdentifier = String;

#[derive(Debug)]
pub enum Event {
    // Instance events
    InstanceCreate(
        Instance,
        mpsc::Sender<Result<InstanceStatus, tonic::Status>>,
    ),
    InstanceStop(
        NodeIdentifier,
        oneshot::Sender<Result<Response<()>, tonic::Status>>,
    ),
    InstanceDestroy(
        NodeIdentifier,
        oneshot::Sender<Result<Response<()>, tonic::Status>>,
    ),

    // Node events
    NodeRegister(
        NodeRegisterRequest,
        IpAddr,
        oneshot::Sender<Result<Response<NodeRegisterResponse>, tonic::Status>>,
    ),
    NodeUnregister(
        NodeUnregisterRequest,
        oneshot::Sender<Result<Response<NodeUnregisterResponse>, tonic::Status>>,
    ),
    NodeStatus(NodeStatus, mpsc::Sender<Result<(), tonic::Status>>),
}
