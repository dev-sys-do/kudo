use super::{model::WorkloadInfo, service};
use actix_web::{delete, get, patch, put, web, HttpResponse, Responder, Scope};

use super::service::WorkloadService;
use super::{model::Pagination, model::WorkloadDTO};
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
