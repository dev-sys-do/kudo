use super::namespace;
use super::{instance, node, workload};
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpResponse, HttpServer};
use log::info;
use std::net::SocketAddr;

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
    ) -> Self {
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
                .service(node::controller::NodeController {}.services())
                .wrap(Logger::default())
        })
        .workers(num_workers)
        .bind(address)
        .unwrap()
        .run()
        .await
        .unwrap();

        Self {}
    }
}
