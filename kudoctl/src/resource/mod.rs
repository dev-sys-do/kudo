pub mod workload;

use serde::{Deserialize, Serialize};

// Kind of a resource (workload, user, ...), internally tagged 
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Resource {
    Workload  (workload::Workload),
    // User,
}