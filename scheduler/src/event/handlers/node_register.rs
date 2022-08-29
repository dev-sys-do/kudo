use std::{net::IpAddr, sync::Arc};

use crate::{node::Node, orchestrator::Orchestrator};
use anyhow::Result;
use proto::{
    controller::node_service_client::NodeServiceClient,
    scheduler::{NodeRegisterResponse, Status},
};
use tokio::sync::{oneshot, Mutex};
use tonic::Response;
use uuid::Uuid;

pub struct NodeRegisterHandler {}

impl NodeRegisterHandler {
    pub async fn handle(
        orchestrator: Arc<Mutex<Orchestrator>>,
        addr: IpAddr,
        controller_client: Arc<Mutex<Option<NodeServiceClient<tonic::transport::Channel>>>>,
        tx: oneshot::Sender<Result<Response<NodeRegisterResponse>, tonic::Status>>,
    ) {
        // todo: parse certificate and get the node information
        let node = Node {
            id: Uuid::new_v4().to_string(),
            status: Status::Starting,
            resource: None,
        };

        match orchestrator
            .lock()
            .await
            .register_node(node.clone(), addr, controller_client.clone())
            .await
        {
            Ok(_) => {
                log::info!("successfully registered node: {:?}", node.id);

                let response = NodeRegisterResponse {
                    code: 0,
                    description: "Welcome to the cluster".to_string(),
                    subnet: "".to_string(),
                };

                tx.send(Ok(tonic::Response::new(response))).unwrap();
            }
            Err(err) => {
                log::error!("error while registering node : {:?} ({:?})", node.id, err);

                let response = NodeRegisterResponse {
                    code: 1,
                    description: format!("Error thrown by the orchestrator: {:?}", err),
                    subnet: "".to_string(),
                };

                tx.send(Ok(tonic::Response::new(response))).unwrap();
            }
        };
    }
}
