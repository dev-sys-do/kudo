use crate::external_api::generic::model::Pagination;
use crate::external_api::interface::ActixAppState;

use super::model::NamespaceDTO;
use super::service::NamespaceService;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, Responder, Scope};
pub struct NamespaceController {}
impl NamespaceController {
    pub fn services(&self) -> Scope {
        web::scope("/namespace")
            .service(
                web::resource("/{namespace_name}")
                    .route(web::delete().to(NamespaceController::delete_namespace))
                    .route(web::get().to(NamespaceController::namespace))
                    .route(web::patch().to(NamespaceController::patch_namespace)),
            )
            .service(
                web::resource("")
                    .route(web::put().to(NamespaceController::put_namespace))
                    .route(web::get().to(NamespaceController::get_all_namespace)),
            )
    }

    pub async fn namespace(
        params: web::Path<String>,
        data: web::Data<ActixAppState>,
    ) -> impl Responder {
        let mut namespace_service = match NamespaceService::new(&data.etcd_address).await {
            Ok(namespace) => namespace,
            Err(e) => return e.to_http(),
        };

        let namespace_name = params.into_inner();

        namespace_service
            .namespace(&namespace_name)
            .await
            .map_or_else(|e| e.to_http(), |w| w.to_http())
    }

    pub async fn put_namespace(
        body: web::Json<NamespaceDTO>,
        data: web::Data<ActixAppState>,
    ) -> impl Responder {
        let mut namespace_service = match NamespaceService::new(&data.etcd_address).await {
            Ok(namespace) => namespace,
            Err(e) => return e.to_http(),
        };
        let namespace_dto = body.into_inner();
        namespace_service
            .create_namespace(namespace_dto)
            .await
            .map_or_else(|e| e.to_http(), |w| w.to_http())
    }

    pub async fn get_all_namespace(
        pagination: Option<web::Query<Pagination>>,
        data: web::Data<ActixAppState>,
    ) -> impl Responder {
        let mut namespace_service = match NamespaceService::new(&data.etcd_address).await {
            Ok(namespace) => namespace,
            Err(e) => return e.to_http(),
        };

        match pagination {
            Some(pagination) => {
                let namespaces = namespace_service
                    .get_all_namespace(pagination.limit, pagination.offset)
                    .await;
                namespaces.to_http()
            }
            None => {
                let namespaces = namespace_service.get_all_namespace(0, 0).await;
                namespaces.to_http()
            }
        }
    }

    pub async fn patch_namespace(
        params: web::Path<String>,
        body: web::Json<NamespaceDTO>,
        data: web::Data<ActixAppState>,
    ) -> impl Responder {
        let mut namespace_service = match NamespaceService::new(&data.etcd_address).await {
            Ok(namespace) => namespace,
            Err(e) => return e.to_http(),
        };

        let namespace_name = params.into_inner();
        let namespace_dto = body.into_inner();

        namespace_service
            .update_namespace(namespace_dto, &namespace_name)
            .await
            .map_or_else(|e| e.to_http(), |w| w.to_http())
    }

    pub async fn delete_namespace(
        params: web::Path<String>,
        data: web::Data<ActixAppState>,
    ) -> impl Responder {
        let mut namespace_service = match NamespaceService::new(&data.etcd_address).await {
            Ok(namespace) => namespace,
            Err(e) => return e.to_http(),
        };

        let namespace_name = params.into_inner();

        namespace_service.delete_namespace(&namespace_name).await;
        HttpResponse::build(StatusCode::NO_CONTENT).body("Remove successfully")
    }
}
