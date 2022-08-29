use std::net::SocketAddr;

use super::filter::FilterService;
use super::model::{Ressources, Type, Workload, WorkloadDTO, WorkloadError, WorkloadVector};
use crate::etcd::EtcdClient;
use serde_json;

pub struct WorkloadService {
    etcd_service: EtcdInterface,
    filter_service: FilterService,
}

impl WorkloadService {
    pub async fn new(etcd_address: &SocketAddr) -> Result<WorkloadService, WorkloadError> {
        let inner = WorkloadService {
            etcd_service: EtcdClient::new(etcd_address.to_string())
                .await
                .map_err(|err| WorkloadError::Etcd(err.to_string()))?,
            filter_service: FilterService::new(),
        };
        Ok(inner)
    }
    pub async fn get_workload(
        &mut self,
        workload_name: &str,
        namespace: &str,
    ) -> Result<Workload, WorkloadError> {
        let id = self.id(workload_name, namespace);
        match self.etcd_service.get(&id).await {
            Some(workload) => {
                let workload: Workload = serde_json::from_str(&workload)
                    .map_err(|err| WorkloadError::JsonToWorkload(err.to_string()))?;
                if workload.namespace == namespace {
                    Ok(serde_json::to_string(&workload).unwrap())
                } else {
                    Err(WorkloadError::WorkloadNotFound)
                }
            }
            None => Err(WorkloadError::WorkloadNotFound),
        }
    }

    pub async fn get_all_workloads(
        &mut self,
        limit: u32,
        offset: u32,
        namespace: &str,
    ) -> WorkloadVector {
        let mut new_vec: Vec<Workload> = Vec::new();
        match self.etcd_service.get_all().await {
            Some(workloads) => {
                for workload in workloads {
                    // if workload deserialize failed , we don't want to throw error , so we just don't add it to the vector
                    if let Ok(workload) = serde_json::from_str::<Workload>(&workload) {
                        if workload.namespace == namespace {
                            new_vec.push(workload);
                        }
                    }
                }
                if offset > 0 {
                    match self.filter_service.offset(&new_vec, offset) {
                        Ok(workloads) => new_vec = workloads,
                        Err(_) => return WorkloadVector::new(vec![]),
                    }
                }
                if limit > 0 {
                    new_vec = self.filter_service.limit(&new_vec, limit);
                }
                WorkloadVector::new(new_vec)
            }
            None => WorkloadVector::new(vec![]),
        }
    }

    pub async fn create_workload(
        &mut self,
        workload_dto: WorkloadDTO,
        namespace: &str,
    ) -> Result<Workload, WorkloadError> {
        let new_id = self.id(&workload_dto.name, namespace);
        match self.get_workload(&workload_dto.name, namespace).await {
            Ok(workload) => Err(WorkloadError::NameAlreadyExists(workload.name)),
            Err(err) => match err {
                WorkloadError::WorkloadNotFound => {
                    let workload = Workload {
                        id: new_id.to_string(),
                        name: workload_dto.name,
                        workload_type: Type::Container,
                        uri: workload_dto.uri,
                        environment: workload_dto.environment,
                        resources: Ressources {
                            cpu: 0,
                            memory: 0,
                            disk: 0,
                        },
                        ports: workload_dto.ports,
                        namespace: namespace.to_string(),
                    };
                    let json = serde_json::to_string(&workload)
                        .map_err(|err| WorkloadError::WorkloadToJson(err.to_string()))?;
                    self.etcd_service
                        .put(&new_id, &json)
                        .await
                        .map_err(|err| WorkloadError::Etcd(err.to_string()))?;
                    Ok(workload)
                }
                _ => Err(err),
            },
        }
    }

    pub async fn update_workload(
        &mut self,
        workload_dto: WorkloadDTO,
        workload_name: &str,
        namespace: &str,
    ) -> Result<Workload, WorkloadError> {
        // we get the id before update , and the new id after update
        let new_id = self.id(&workload_dto.name, namespace);
        self.get_workload(workload_name, namespace).await?;
        let workload = Workload {
            id: new_id.to_string(),
            name: workload_dto.name,
            workload_type: Type::Container,
            uri: workload_dto.uri,
            environment: workload_dto.environment.to_vec(),
            resources: Ressources {
                cpu: 0,
                memory: 0,
                disk: 0,
            },
            ports: workload_dto.ports.to_vec(),
            namespace: namespace.to_string(),
        };
        let json = serde_json::to_string(&workload)
            .map_err(|err| WorkloadError::WorkloadToJson(err.to_string()))?;
        self.etcd_service
            .put(&new_id, &json)
            .await
            .map_err(|err| WorkloadError::Etcd(err.to_string()))?;
        Ok(workload)
    }

    pub async fn delete_workload(&mut self, workload_name: &str, namespace: &str) {
        let id = self.id(workload_name, namespace);
        _ = self.etcd_service.delete(&id).await;
    }

    pub fn id(&mut self, name: &str, namespace: &str) -> String {
        format!("{}.{}", namespace, name)
    }
}
