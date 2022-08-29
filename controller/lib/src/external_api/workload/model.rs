use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};

pub enum WorkloadError {
    WorkloadNotFound,
    Etcd(String),
    NameAlreadyExists(String),
    JsonToWorkload(String),
    WorkloadToJson(String),
}

impl WorkloadError {
    pub fn to_http(&self) -> HttpResponse {
        match self {
            WorkloadError::WorkloadNotFound => HttpResponse::NotFound().body("Workload not found"),
            WorkloadError::Etcd(err) => {
                HttpResponse::InternalServerError().body(format!("Etcd error: {} ", err))
            }
            WorkloadError::NameAlreadyExists(name) => {
                HttpResponse::Conflict().body(format!("Workload with name {} already exists", name))
            }
            WorkloadError::JsonToWorkload(err) => HttpResponse::InternalServerError().body(
                format!("Error while converting JSON string to workload : {}", err),
            ),
            WorkloadError::WorkloadToJson(err) => HttpResponse::InternalServerError().body(
                format!("Error while converting the workload to JSON: {}", err),
            ),
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
                "Error while converting the workload to json: {}",
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
        match serde_json::to_string(&self) {
            Ok(json) => HttpResponse::Ok().body(json),
            Err(err) => HttpResponse::InternalServerError().body(format!(
                "Error while converting the workload to json: {}",
                err
            )),
        }
    }
}
