use serde_derive::{Deserialize, Serialize};

///
/// NodeAgentConfig is a structure to define node agent configuration
///
/// server: node agent grpc server config
/// client: scheduler grpc server config
#[derive(Serialize, Deserialize, Debug)]
pub struct NodeAgentConfig {
    pub server: GrpcServerConfig,
    pub client: GrpcServerConfig,
}

///
/// GrpcServerConfig is a structure to define any grpc server configuration
///
/// host: ip or address of the grpc server
/// port: grpc server's port
#[derive(Serialize, Deserialize, Debug)]
pub struct GrpcServerConfig {
    pub host: String,
    pub port: u16,
}

/// The default configuration for node agent
impl Default for NodeAgentConfig {
    fn default() -> Self {
        NodeAgentConfig {
            server: GrpcServerConfig {
                host: "127.0.0.1".to_string(),
                port: 50053,
            },
            client: GrpcServerConfig {
                host: "127.0.0.1".to_string(),
                port: 50052,
            },
        }
    }
}

// / new function: allows you to define custom server and client configurations
// impl NodeAgentConfig {
//     pub fn new(
//         server_host: String,
//         server_port: u16,
//         client_host: String,
//         client_port: u16,
//     ) -> Self {
//         Self {
//             server: GrpcServerConfig {
//                 host: server_host,
//                 port: server_port,
//             },
//             client: GrpcServerConfig {
//                 host: client_host,
//                 port: client_port,
//             },
//         }
//     }
// }
