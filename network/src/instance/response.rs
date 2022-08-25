pub struct SetupInstanceResponse {
    pub interface_name: String,
    pub namespace_name: String,
}

impl SetupInstanceResponse {
    pub fn new(interface_name: String, namespace_name: String) -> Self {
        Self {
            interface_name,
            namespace_name,
        }
    }
}
