use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum WorkloadError {
    WorkloadNotFound,
    Etcd(String),
    NameAlreadyExists(String),
    JsonToWorkload(String),
    WorkloadToJson(String),
    NamespaceService,
    NamespaceNotFound,
}

impl WorkloadError {
    pub fn to_http(&self) -> HttpResponse {
        match self {
            WorkloadError::WorkloadNotFound => {
                HttpResponse::NotFound().body("{\"error\":\"Workload not found\"}")
            }
            WorkloadError::Etcd(err) => HttpResponse::InternalServerError()
                .body(format!("{{\"error\":\"Etcd error: {} \"}}", err)),
            WorkloadError::NameAlreadyExists(name) => HttpResponse::Conflict().body(format!(
                "{{\"error\":\"Workload with name {} already exists\"}}",
                name
            )),
            WorkloadError::JsonToWorkload(err) => {
                HttpResponse::InternalServerError().body(format!(
                    "{{\"error\":\"Error while converting JSON string to Workload : {}\"}}",
                    err
                ))
            }
            WorkloadError::WorkloadToJson(err) => {
                HttpResponse::InternalServerError().body(format!(
                    "{{\"error\":\"Error while converting the Workload to JSON: {}\"}}",
                    err
                ))
            }
            WorkloadError::NamespaceService => HttpResponse::InternalServerError()
                .body("{\"error\":\"Cannot create a NamespaceService instance\"}"),
            WorkloadError::NamespaceNotFound => {
                HttpResponse::NotFound().body("{\"error\":\"Namespace not found\"}")
            }
        }
    }
}
#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum Type {
    Container = 0,
}
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Ressources {
    pub cpu: u64,
    pub memory: u64,
    pub disk: u64,
}
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Ports {
    pub source: i32,
    pub destination: i32,
}
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Workload {
    pub id: String,
    pub name: String,
    pub workload_type: Type,
    pub uri: String,
    pub environment: Vec<String>,
    pub resources: Ressources,
    pub ports: Vec<Ports>,
    pub namespace: String,
}
impl Workload {
    pub fn to_http(&self) -> HttpResponse {
        match serde_json::to_string(&self) {
            Ok(json) => HttpResponse::Ok().body(json),
            Err(err) => HttpResponse::InternalServerError().body(format!(
                "{{\"error\":\"Error while converting the Workload to JSON: {}\"}}",
                err
            )),
        }
    }
}
#[derive(Deserialize, Serialize)]
pub struct WorkloadDTO {
    pub name: String,
    pub environment: Vec<String>,
    pub ports: Vec<Ports>,
    pub uri: String,
    pub resources: Ressources,
}
#[derive(Deserialize, Serialize)]
pub struct WorkloadVector {
    pub workloads: Vec<Workload>,
}
impl WorkloadVector {
    pub fn new(workloads: Vec<Workload>) -> WorkloadVector {
        WorkloadVector { workloads }
    }
    pub fn to_http(&self) -> HttpResponse {
        match serde_json::to_string(&self.workloads) {
            Ok(json) => HttpResponse::Ok().body(json),
            Err(err) => HttpResponse::InternalServerError().body(format!(
                "{{\"error\":\"Error while converting the Workload to JSON: {}\"}}",
                err
            )),
        }
    }
}
