use std::sync::Arc;

use crate::external_api::interface::ActixAppState;

use super::super::workload::service::WorkloadService;
use super::model::{InstanceDTO, Pagination};
use super::service;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, Responder, Scope};
use tokio::sync::Mutex;
pub struct InstanceController {}

impl InstanceController {
    pub fn services(&self) -> Scope {
        web::scope("/instance")
            .service(
                web::resource("/{namespace}")
                    .route(web::get().to(InstanceController::get_instances))
                    .route(web::post().to(InstanceController::post_instance)),
            )
            .service(
                web::resource("/{namespace}/{name}")
                    .route(web::delete().to(InstanceController::delete_instance))
                    .route(web::get().to(InstanceController::get_instance)),
            )
    }

    /// `post_instance` is an async function that handle **/instance/\<namespace>** route (POST)
    /// # Description:
    /// * Create and start an instance
    /// # Arguments:
    ///
    /// * `namespace`: web::Path<String> - This is the namespace that the instance will be created in.
    /// * `body`: web::Json<InstanceDTO> - Contains the workload_name to create the instance from it.

    pub async fn post_instance(
        namespace: web::Path<String>,
        body: web::Json<InstanceDTO>,
        data: web::Data<ActixAppState>,
    ) -> impl Responder {
        let instance_service =
            service::InstanceService::new(&data.grpc_address, &data.etcd_address)
                .await
                .map_err(|err| {
                    HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(format!("Error creating instance service, {:?}", err))
                })
                .unwrap();
        let mut workload_service = WorkloadService::new(&data.etcd_address).await.unwrap();
        match workload_service
            .get_workload(&body.workload_name, &namespace)
            .await
        {
            Ok(workload) => {
                match super::service::InstanceService::retrieve_and_start_instance_from_workload(
                    Arc::new(Mutex::new(instance_service)),
                    &workload.id,
                )
                .await
                {
                    Ok(_) => HttpResponse::build(StatusCode::CREATED)
                        .body("Instance creating and starting..."),
                    Err(_) => HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                        .body("Internal Server Error"),
                }
            }
            Err(_) => HttpResponse::build(StatusCode::NOT_FOUND).body("Workload not found"),
        }
    }
    /// It deletes a instance from etcd and grpc.
    ///
    /// # Arguments:
    ///
    /// * `id`: The id of the instance to delete
    /// * `namespace`: The namespace of the workload
    ///
    /// # Returns:
    ///
    /// A Result<(), InstanceError>
    pub async fn delete_instance(
        params: web::Path<(String, String)>,
        data: web::Data<ActixAppState>,
    ) -> impl Responder {
        let mut instance_service =
            service::InstanceService::new(&data.grpc_address, &data.etcd_address)
                .await
                .unwrap();
        let (namespace, name) = params.into_inner();
        match instance_service
            .get_instance(format!("{}.{}", namespace, name).as_str())
            .await
        {
            Ok(instance) => match instance_service.delete_instance(instance).await {
                Ok(_) => HttpResponse::build(StatusCode::OK).body("Instance deleted"),
                Err(err) => HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(format!("Internal Server Error, {}", err)),
            },
            Err(_) => HttpResponse::build(StatusCode::NOT_FOUND).body("Instance not found"),
        }
    }

    /// It gets a instance from etcd, and if it exists, check if the namespace is the same.
    ///
    /// # Arguments:
    ///
    /// * `namespace`: The namespace of the instance
    /// * `name`: The workload name to get
    ///
    /// # Returns:
    ///
    /// A Result<String, InstanceError>
    pub async fn get_instance(
        params: web::Path<(String, String)>,
        data: web::Data<ActixAppState>,
    ) -> impl Responder {
        let mut instance_service =
            service::InstanceService::new(&data.grpc_address, &data.etcd_address)
                .await
                .map_err(|err| {
                    HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(format!("Error creating instance service, {:?}", err))
                })
                .unwrap();
        let (namespace, name) = params.into_inner();
        match instance_service
            .get_instance(format!("{}.{}", namespace, name).as_str())
            .await
        {
            Ok(instance) => match serde_json::to_string(&instance) {
                Ok(instance_str) => HttpResponse::build(StatusCode::OK).body(instance_str),
                Err(err) => HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(format!("Internal Server Error {}", err)),
            },
            Err(err) => HttpResponse::build(StatusCode::NOT_FOUND).body(err.to_string()),
        }
    }

    /// `get_instances` is an async function that handle **/instance/\<namespace>** route (GET)
    /// # Description:
    /// * Get all instances in the namespace
    /// # Arguments:
    ///
    /// * `namespace`: The namespace of the instance you want to retrieve.
    /// * `pagination`: Option<web::Query<Pagination>>
    pub async fn get_instances(
        namespace: web::Path<String>,
        pagination: Option<web::Query<Pagination>>,
        data: web::Data<ActixAppState>,
    ) -> impl Responder {
        let mut instance_service =
            service::InstanceService::new(&data.grpc_address, &data.etcd_address)
                .await
                .map_err(|err| {
                    HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(format!("Error creating instance service, {:?}", err))
                })
                .unwrap();

        match pagination {
            Some(pagination) => {
                let instances = instance_service
                    .get_instances(pagination.limit, pagination.offset, &namespace)
                    .await;
                web::Json(instances)
            }
            None => {
                let instances = instance_service.get_instances(0, 0, &namespace).await;
                web::Json(instances)
            }
        }
    }
}
