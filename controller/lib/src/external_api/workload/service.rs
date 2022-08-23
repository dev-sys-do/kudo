use super::model::WorkloadInfo;

pub fn get_workloads(ids: &[String]) -> String {
    format!("get_workloads {}", ids.join(","))
}

pub fn create_workload(workload: &WorkloadInfo) -> String {
    format!("create_workload {}", workload.name)
}

pub fn update_workload(id: &String, workload: &WorkloadInfo) -> String {
    format!("update_workload {}, id {}", workload.name, id)
}

pub fn delete_workload(id: &String) -> String {
    format!("delete_workload {}", id)
}
