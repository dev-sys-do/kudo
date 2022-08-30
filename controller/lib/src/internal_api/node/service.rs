use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use log::info;
use tokio::sync::Mutex;
use tokio::time;
use tonic::Streaming;

use crate::etcd::EtcdClient;
use crate::internal_api::node::model::InstanceIdentifier;

use super::super::super::external_api::instance::{
    model::Instance, model::InstanceError, service::InstanceService,
};
use super::model::NodeStatus;

#[derive(Debug)]
pub enum NodeServiceError {
    EtcdError(etcd_client::Error),
    SerdeError(serde_json::Error),
    StreamingClosed(tonic::Status),
    InstanceServiceError(InstanceError),
}

impl std::fmt::Display for NodeServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeServiceError::EtcdError(err) => write!(f, "EtcdError: {}", err),
            NodeServiceError::SerdeError(err) => write!(f, "SerdeError: {}", err),
            NodeServiceError::InstanceServiceError(_) => {
                write!(f, "InstanceServiceError")
            }
            NodeServiceError::StreamingClosed(err) => {
                write!(f, "StreamingClosed: {}", err)
            }
        }
    }
}

pub struct NodeService {
    etcd_interface: EtcdClient,
    instance_service: Arc<Mutex<InstanceService>>,
}

impl NodeService {
    pub async fn new(
        etcd_address: &SocketAddr,
        grpc_address: &str,
    ) -> Result<Self, NodeServiceError> {
        Ok(NodeService {
            etcd_interface: EtcdClient::new(etcd_address.to_string())
                .await
                .map_err(NodeServiceError::EtcdError)?,
            instance_service: Arc::new(Mutex::new(
                InstanceService::new(grpc_address, etcd_address)
                    .await
                    .map_err(NodeServiceError::InstanceServiceError)?,
            )),
        })
    }

    /// It receives a stream of `NodeStatus` messages from the scheduler, updates the node's status in etcd,
    /// and reschedules any instances that were running on the node if the stream is closed.
    ///
    /// Arguments:
    ///
    /// * `stream`: The stream of messages received from the scheduler.
    /// * `remote_address`: The address of the scheduler gRPC server that is sending the status update.
    ///
    /// Returns:
    ///
    /// A `Result` with an `Ok` value of `()` or an `Err` value of `NodeServiceError`.
    pub async fn update_node_status(
        &mut self,
        mut stream: Streaming<proto::controller::NodeStatus>,
        remote_address: String,
    ) -> Result<(), NodeServiceError> {
        let mut last_instances: Vec<Instance> = vec![];

        while let Some(node_status) = stream
            .message()
            .await
            .map_err(NodeServiceError::StreamingClosed)?
        {
            info!("{} \"update_node_status\" received chunk", remote_address);

            let node_status = NodeStatus::from(node_status);

            self.etcd_interface
                .put(
                    &node_status.id,
                    &serde_json::to_string(&node_status).map_err(NodeServiceError::SerdeError)?,
                )
                .await
                .map_err(NodeServiceError::EtcdError)?;

            let objects = self.etcd_interface.get_all().await.unwrap_or_default();

            for object in objects {
                if let Ok(instance) = serde_json::from_str::<Instance>(&object) {
                    if node_status.instances.contains(&InstanceIdentifier {
                        id: instance.clone().id,
                    }) {
                        last_instances.push(instance);
                    }
                }
            }
        }

        info!(
            "{} \"update_node_status\" streaming closed, rescheduling instances",
            remote_address.clone()
        );

        for instance in last_instances {
            InstanceService::schedule_instance(self.instance_service.clone(), instance)
        }

        //Delete Node after 5 min
        time::sleep(Duration::from_secs(300)).await;
        self.etcd_interface.delete(&remote_address).await;

        Ok(())
    }
}
