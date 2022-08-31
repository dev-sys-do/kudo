use crate::external_api::{generic::model::Pagination, interface::ActixAppState};
use actix_web::{http::StatusCode, web, HttpResponse, Responder, Scope};

use super::{model::NodeError, service::NodeService};

pub struct NodeController {}

impl NodeController {
    pub fn services(&self) -> Scope {
        web::scope("/node")
            .service(web::resource("").route(web::get().to(NodeController::get_all_nodes)))
            .service(web::resource("/{node_id}").route(web::get().to(NodeController::get_node)))
    }

    /// It gets a node, and return it if it exists
    ///
    /// # Arguments:
    ///
    /// * id`: The node id to get
    ///
    /// # Returns:
    ///
    /// A HttpResponse with the node if it exists, or a 404 error if it doesn't.

    pub async fn get_node(id: web::Path<String>, data: web::Data<ActixAppState>) -> impl Responder {
        let mut node_service = NodeService::new(data.etcd_address)
            .await
            .map_err(|err| {
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body(format!(
                    "{{\"error\":\"Error creating node service, {:?}\"}}",
                    err
                ))
            })
            .unwrap();
        match node_service.get_node(&id).await {
            Ok(node) => {
                HttpResponse::build(StatusCode::OK).body(serde_json::to_string(&node).unwrap())
            }
            Err(err) => match err {
                NodeError::NodeNotFound => HttpResponse::build(StatusCode::NOT_FOUND)
                    .body("{\"error\":\"Node not found\"}"),
                _ => HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .body("{\"error\":\"Internal Server Error\"}"),
            },
        }
    }

    /// `get_all_nodes` is an async function that return all nodes from etcd
    /// # Description:
    /// * Get all nodes inside the cluster
    /// # Arguments:
    ///
    /// * `pagination`: The limit and offset used to paginate the vector of nodes
    /// # Returns:
    /// An HTTP response with the vector of nodes
    pub async fn get_all_nodes(
        pagination: Option<web::Query<Pagination>>,
        data: web::Data<ActixAppState>,
    ) -> impl Responder {
        let mut node_service = NodeService::new(data.etcd_address)
            .await
            .map_err(|err| {
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body(format!(
                    "{{\"error\":\"Error creating node service, {:?}\"}}",
                    err
                ))
            })
            .unwrap();
        match pagination {
            Some(pagination) => {
                let nodes = node_service
                    .get_all_nodes(pagination.limit, pagination.offset)
                    .await;
                web::Json(nodes)
            }
            None => {
                let nodes = node_service.get_all_nodes(0, 0).await;
                web::Json(nodes)
            }
        }

        // let nodes = node_service.get_all_nodes().await;
        // web::Json(nodes)
    }
}
