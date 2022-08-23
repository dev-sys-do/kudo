use serde::{Deserialize, Serialize};

// Workload definition (serialized to YAML)
#[derive(Serialize, Deserialize, Debug)]
pub struct Workload {
    pub name: String,
    // uri of the workload vm/container image to execute
    pub uri: String,
    pub resources: Resources,
    pub ports: Option<Vec<String>>,
    /// environment variables to set on the workload
    pub env: Option<Vec<String>>,
}

// Resources assigned to a workload 
#[derive(Serialize, Deserialize, Debug)]
pub struct Resources {
    // CPU in milliCPU
    pub cpu: u64,
    // Memory in MB
    pub memory: u64,
    // Storage in GB
    pub disk: u64,
}

