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
pub mod parser;
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

#[derive(Debug)]
#[allow(dead_code)]
pub struct Node {
    id: String,
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
    InstanceStart(
        NodeIdentifier,
        oneshot::Sender<Result<Response<()>, tonic::Status>>,
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
