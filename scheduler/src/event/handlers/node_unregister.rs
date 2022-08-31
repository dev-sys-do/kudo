use std::sync::Arc;

use crate::{orchestrator::Orchestrator, NodeIdentifier};
use anyhow::Result;
use proto::scheduler::NodeUnregisterResponse;
use tokio::sync::{oneshot, Mutex};
use tonic::Response;

pub struct NodeUnregisterHandler {}

impl NodeUnregisterHandler {
    pub async fn handle(
        orchestrator: Arc<Mutex<Orchestrator>>,
        id: NodeIdentifier,
        tx: oneshot::Sender<Result<Response<NodeUnregisterResponse>, tonic::Status>>,
    ) {
        match orchestrator
            .lock()
            .await
            .unregister_node(id.clone(), None)
            .await
        {
            Ok(_) => {
                log::info!("successfully unregistered node {:?}", id);

                let response = NodeUnregisterResponse {
                    code: 0,
                    description: "Bye from the cluster".to_string(),
                };

                tx.send(Ok(tonic::Response::new(response))).unwrap();
            }
            Err(err) => {
                log::error!("error while unregistering node : {:?} ({:?})", id, err);

                let response = NodeUnregisterResponse {
                    code: 1,
                    description: format!("Error thrown by the orchestrator: {:?}", err),
                };

                tx.send(Ok(tonic::Response::new(response))).unwrap();
            }
        };
    }
}
