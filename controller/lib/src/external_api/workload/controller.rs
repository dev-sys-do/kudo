use crate::external_api::interface::ActixAppState;

use super::model::WorkloadDTO;
use super::service::WorkloadService;
use crate::external_api::generic::model::Pagination;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, Responder, Scope};
pub struct WorkloadController {}
impl WorkloadController {
    pub fn services(&self) -> Scope {
        web::scope("/workload")
            .service(
                web::resource("/{namespace}/{workload_id}")
                    .route(web::delete().to(WorkloadController::delete_workload))
                    .route(web::get().to(WorkloadController::workload))
                    .route(web::patch().to(WorkloadController::patch_workload)),
            )
            .service(
                web::resource("/{namespace}")
                    .route(web::put().to(WorkloadController::put_workload))
                    .route(web::get().to(WorkloadController::get_all_workloads)),
            )
    }

    /// It gets a workload from etcd, and if it exists, check if the namespace is the same.
    ///
    /// # Arguments:
    ///
    /// * `workload_id`: The workload id to get
    /// * `namespace`: The namespace of the workload
    ///
    /// # Returns:
    ///
    /// A Result<String, WorkloadError>

    pub async fn workload(
        params: web::Path<(String, String)>,
        data: web::Data<ActixAppState>,
    ) -> impl Responder {
        let (namespace, workload_id) = params.into_inner();

        let mut workload_service = match WorkloadService::new(&data.etcd_address).await {
            Ok(workload) => workload,
            Err(e) => return e.to_http(),
        };

        workload_service
            .get_workload(&workload_id, &namespace)
            .await
            .map_or_else(|e| e.to_http(), |w| w.to_http())
    }

    /// `put_workload` is an async function that handle **/workload/\<namespace>** route (PUT)
    /// # Description:
    /// * Create a new workload
    /// # Arguments:
    ///
    /// * `namespace`: web::Path<String> - This is the namespace that the workload will be created in.
    /// * `body`: web::Json<WorkloadDTO> - Contain all information required to create the workload.

    pub async fn put_workload(
        namespace: web::Path<String>,
        body: web::Json<WorkloadDTO>,
        data: web::Data<ActixAppState>,
    ) -> impl Responder {
        let mut workload_service = match WorkloadService::new(&data.etcd_address).await {
            Ok(workload) => workload,
            Err(e) => return e.to_http(),
        };
        let workload_dto = body.into_inner();
        workload_service
            .create_workload(workload_dto, &namespace)
            .await
            .map_or_else(|e| e.to_http(), |w| w.to_http())
    }

    /// `get_all_workloads` is an async function that handle **/workload/\<namespace>** route (GET)
    /// # Description:
    /// * Get all workload in the namespace
    /// # Arguments:
    ///
    /// * `namespace`: The namespace of the workloads you want to retrieve.
    /// * `pagination`: Option<web::Query<Pagination>>

    pub async fn get_all_workloads(
        namespace: web::Path<String>,
        pagination: Option<web::Query<Pagination>>,
        data: web::Data<ActixAppState>,
    ) -> impl Responder {
        let mut workload_service = match WorkloadService::new(&data.etcd_address).await {
            Ok(workload) => workload,
            Err(e) => return e.to_http(),
        };

        match pagination {
            Some(pagination) => {
                let workloads = workload_service
                    .get_all_workloads(pagination.limit, pagination.offset, &namespace)
                    .await;
                workloads.to_http()
            }
            None => {
                let workloads = workload_service.get_all_workloads(0, 0, &namespace).await;
                workloads.to_http()
            }
        }
    }

    /// `patch_workload` is an asynchronous function that handle **/workload/\<namespace>/<workload_id>** route (PATCH)
    /// # Description:
    /// * Update a workload
    /// # Arguments:
    ///
    /// * `params`: web::Path<(String, String)> - The first Path parameter is the namespace and the second the workload id.
    /// * `body`: web::Json<WorkloadDTO> - Contain all information required to create the workload.

    pub async fn patch_workload(
        params: web::Path<(String, String)>,
        body: web::Json<WorkloadDTO>,
        data: web::Data<ActixAppState>,
    ) -> impl Responder {
        let mut workload_service = match WorkloadService::new(&data.etcd_address).await {
            Ok(workload) => workload,
            Err(e) => return e.to_http(),
        };

        let (namespace, workload_id) = params.into_inner();
        let workload_dto = body.into_inner();

        workload_service
            .update_workload(workload_dto, &workload_id, &namespace)
            .await
            .map_or_else(|e| e.to_http(), |w| w.to_http())
    }

    /// It deletes a workload from etcd
    ///
    /// # Arguments:
    ///
    /// * `id`: The id of the workload to delete
    /// * `namespace`: The namespace of the workload
    ///
    /// # Returns:
    ///
    /// A Result<(), WorkloadError>

    pub async fn delete_workload(
        params: web::Path<(String, String)>,
        data: web::Data<ActixAppState>,
    ) -> impl Responder {
        let mut workload_service = match WorkloadService::new(&data.etcd_address).await {
            Ok(workload) => workload,
            Err(e) => return e.to_http(),
        };

        let (namespace, workload_id) = params.into_inner();

        workload_service
            .delete_workload(&workload_id, &namespace)
            .await;
        HttpResponse::build(StatusCode::NO_CONTENT).body("Remove successfully")
    }
}
