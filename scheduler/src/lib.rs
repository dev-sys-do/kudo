use proto::scheduler::{
    Instance, InstanceStatus, NodeRegisterRequest, NodeRegisterResponse, NodeStatus,
    NodeUnregisterRequest, NodeUnregisterResponse,
};
use tokio::sync::{mpsc, oneshot};
use tonic::Response;

pub mod instance_listener;
pub mod manager;
pub mod node_listener;
pub mod storage;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Node {
    id: String,
}

pub type NodeIdentifier = String;

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
        oneshot::Sender<Result<Response<NodeRegisterResponse>, tonic::Status>>,
    ),
    NodeUnregister(
        NodeUnregisterRequest,
        oneshot::Sender<Result<Response<NodeUnregisterResponse>, tonic::Status>>,
    ),
    NodeStatus(NodeStatus, mpsc::Sender<Result<(), tonic::Status>>),
}
