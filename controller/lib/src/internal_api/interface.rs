use std::net::SocketAddr;

use super::node::controller::NodeController;
use log::info;
use proto::controller::node_service_server::NodeServiceServer;
use tonic::transport::Server;

#[derive(Debug)]
pub enum InternalAPIInterfaceError {
    NodeControllerError(super::node::controller::NodeControllerError),
}

pub struct InternalAPIInterface {}

impl InternalAPIInterface {
    pub fn new(address: SocketAddr, etcd_address: SocketAddr, grpc_address: String) -> Self {
        info!("Starting gRPC server listening on {}", address);

        tokio::spawn(async move {
            Server::builder()
                .add_service(NodeServiceServer::new(
                    NodeController::new(&etcd_address, &grpc_address)
                        .await
                        .map_err(InternalAPIInterfaceError::NodeControllerError)
                        .unwrap(),
                ))
                .serve(address)
                .await
                .unwrap();
        });

        Self {}
    }
}
