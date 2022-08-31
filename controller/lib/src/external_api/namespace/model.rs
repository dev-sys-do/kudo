use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};

pub enum NamespaceError {
    NotFound,
    Etcd(String),
    NameAlreadyExists(String),
    JsonToNamespace(String),
    NamespaceToJson(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Metadata {}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Namespace {
    pub id: String,
    pub name: String,
    pub metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NamespaceDTO {
    pub name: String,
}

impl NamespaceError {
    pub fn to_http(&self) -> HttpResponse {
        match self {
            NamespaceError::NotFound => {
                HttpResponse::NotFound().body("{\"error\":\"Namespace not found\"}")
            }
            NamespaceError::Etcd(err) => HttpResponse::InternalServerError()
                .body(format!("{{\"error\":\"Etcd error: {} \"}}", err)),
            NamespaceError::NameAlreadyExists(name) => HttpResponse::Conflict().body(format!(
                "{{\"error\":\"Namespace with name {} already exists\"}}",
                name
            )),
            NamespaceError::JsonToNamespace(err) => {
                HttpResponse::InternalServerError().body(format!(
                    "{{\"error\":\"Error while converting JSON string to Namespace : {}\"}}",
                    err
                ))
            }
            NamespaceError::NamespaceToJson(err) => {
                HttpResponse::InternalServerError().body(format!(
                    "{{\"error\":\"Error while converting the Namespace to JSON: {}\"}}",
                    err
                ))
            }
        }
    }
}
impl Namespace {
    pub fn to_http(&self) -> HttpResponse {
        match serde_json::to_string(&self) {
            Ok(json) => HttpResponse::Ok().body(json),
            Err(err) => HttpResponse::InternalServerError().body(format!(
                "{{\"error\":\"Error while converting the Namespace to JSON: {}\"}}",
                err
            )),
        }
    }
}
#[derive(Deserialize, Serialize)]
pub struct NamespaceVector {
    pub namespaces: Vec<Namespace>,
}
impl NamespaceVector {
    pub fn new(namespaces: Vec<Namespace>) -> NamespaceVector {
        NamespaceVector { namespaces }
    }
    pub fn to_http(&self) -> HttpResponse {
        match serde_json::to_string(&self.namespaces) {
            Ok(json) => HttpResponse::Ok().body(json),
            Err(err) => HttpResponse::InternalServerError().body(format!(
                "{{\"error\":\"Error while converting the Namespace to JSON: {}\"}}",
                err
            )),
        }
    }
}
