use crate::etcd::EtcdClient;
use crate::external_api::namespace::model::{Metadata, Namespace};

use super::namespace;
use super::{instance, workload};
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpResponse, HttpServer};
use log::info;
use std::net::SocketAddr;

#[derive(Debug)]
pub enum ExternalAPIInterfaceError {
    EtcdError(etcd_client::Error),
    SerdeError(serde_json::Error),
}

pub struct ExternalAPIInterface {}

pub struct ActixAppState {
    pub etcd_address: SocketAddr,
    pub grpc_address: String,
}

impl ExternalAPIInterface {
    pub async fn new(
        address: SocketAddr,
        num_workers: usize,
        etcd_address: SocketAddr,
        grpc_address: String,
    ) -> Result<Self, ExternalAPIInterfaceError> {
        let mut etcd_client = EtcdClient::new(etcd_address.to_string())
            .await
            .map_err(ExternalAPIInterfaceError::EtcdError)?;

        if etcd_client.get("namespace.default").await.is_none() {
            info!("Creating default namespace");

            let namespace = Namespace {
                id: "namespace.default".to_string(),
                name: "default".to_string(),
                metadata: Metadata {},
            };
            etcd_client
                .put(
                    &namespace.id,
                    &serde_json::to_string(&namespace)
                        .map_err(ExternalAPIInterfaceError::SerdeError)?,
                )
                .await
                .map_err(ExternalAPIInterfaceError::EtcdError)?;
        }

        info!(
            "Starting {} HTTP worker(s) listening on {}",
            num_workers, address
        );

        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(ActixAppState {
                    etcd_address,
                    grpc_address: grpc_address.clone(),
                }))
                .route("/health", web::get().to(HttpResponse::Ok))
                .service(workload::controller::WorkloadController {}.services())
                .service(namespace::controller::NamespaceController {}.services())
                .service(instance::controller::InstanceController {}.services())
                .wrap(Logger::default())
        })
        .workers(num_workers)
        .bind(address)
        .unwrap()
        .run()
        .await
        .unwrap();

        Ok(Self {})
    }
}
