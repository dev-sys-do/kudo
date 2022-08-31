use crate::etcd::EtcdClient;
use crate::external_api::generic::filter::FilterService;
use serde_json;
use std::net::SocketAddr;

use super::model::{Metadata, Namespace, NamespaceDTO, NamespaceError, NamespaceVector};

pub struct NamespaceService {
    etcd_service: EtcdClient,
    filter_service: FilterService,
}

impl NamespaceService {
    pub async fn new(etcd_address: &SocketAddr) -> Result<NamespaceService, NamespaceError> {
        let inner = NamespaceService {
            etcd_service: EtcdClient::new(etcd_address.to_string())
                .await
                .map_err(|err| NamespaceError::Etcd(err.to_string()))?,
            filter_service: FilterService::new(),
        };
        Ok(inner)
    }

    pub async fn namespace(&mut self, namespace_name: &str) -> Result<Namespace, NamespaceError> {
        let id = self.id(namespace_name);
        match self.etcd_service.get(&id).await {
            Some(namespace) => {
                let namespace: Namespace = serde_json::from_str(&namespace)
                    .map_err(|err| NamespaceError::JsonToNamespace(err.to_string()))?;
                Ok(namespace)
            }
            None => Err(NamespaceError::NotFound),
        }
    }

    pub async fn get_all_namespace(&mut self, limit: u32, offset: u32) -> NamespaceVector {
        let mut new_vec: Vec<Namespace> = Vec::new();
        match self.etcd_service.get_all().await {
            Some(namespaces) => {
                for namespace in namespaces {
                    // if namespace deserialize failed , we don't want to throw error , so we just don't add it to the vector
                    if let Ok(namespace) = serde_json::from_str::<Namespace>(&namespace) {
                        new_vec.push(namespace);
                    }
                }
                if offset > 0 {
                    match self.filter_service.offset(&new_vec, offset) {
                        Ok(namespaces) => new_vec = namespaces,
                        Err(_) => return NamespaceVector::new(vec![]),
                    }
                }
                if limit > 0 {
                    new_vec = self.filter_service.limit(&new_vec, limit);
                }
                NamespaceVector::new(new_vec)
            }
            None => NamespaceVector::new(vec![]),
        }
    }

    pub async fn create_namespace(
        &mut self,
        namespace_dto: NamespaceDTO,
    ) -> Result<Namespace, NamespaceError> {
        let id = self.id(&namespace_dto.name);
        match self.namespace(&namespace_dto.name).await {
            Ok(namespace) => Err(NamespaceError::NameAlreadyExists(namespace.name)),
            Err(err) => match err {
                NamespaceError::NotFound => {
                    let namespace = Namespace {
                        id: id.to_string(),
                        name: namespace_dto.name,
                        metadata: Metadata {},
                    };
                    let json = serde_json::to_string(&namespace)
                        .map_err(|err| NamespaceError::NamespaceToJson(err.to_string()))?;
                    self.etcd_service
                        .put(&id, &json)
                        .await
                        .map_err(|err| NamespaceError::Etcd(err.to_string()))?;
                    Ok(namespace)
                }
                _ => Err(err),
            },
        }
    }

    pub async fn update_namespace(
        &mut self,
        namespace_dto: NamespaceDTO,
        namespace_name: &str,
    ) -> Result<Namespace, NamespaceError> {
        // we get the id before update , and the new id after update
        let new_id = self.id(&namespace_dto.name);
        self.namespace(namespace_name).await?;
        let namespace = Namespace {
            id: new_id.to_string(),
            name: namespace_dto.name,
            metadata: Metadata {},
        };
        let json = serde_json::to_string(&namespace)
            .map_err(|err| NamespaceError::NamespaceToJson(err.to_string()))?;
        self.delete_namespace(namespace_name).await;
        self.etcd_service
            .put(&new_id, &json)
            .await
            .map_err(|err| NamespaceError::Etcd(err.to_string()))?;
        Ok(namespace)
    }

    pub async fn delete_namespace(&mut self, namespace_name: &str) {
        let id = self.id(namespace_name);
        _ = self.etcd_service.delete(&id).await;
    }
    pub fn id(&mut self, namespace_name: &str) -> String {
        format!("namespace.{}", namespace_name)
    }
}
