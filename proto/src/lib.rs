pub mod network {
    #![allow(clippy::all)]
    tonic::include_proto!("network");
}

pub mod controller {
    #![allow(clippy::all)]
    tonic::include_proto!("controller");
}

pub mod agent {
    #![allow(non_camel_case_types)]
    #![allow(clippy::all)]
    tonic::include_proto!("agent");
}

pub mod scheduler {
    #![allow(clippy::all)]
    tonic::include_proto!("scheduler");
}
