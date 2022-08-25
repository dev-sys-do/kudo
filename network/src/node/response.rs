pub struct SetupNodeResponse {
    pub interface_name: String,
}

impl SetupNodeResponse {
    pub fn new(interface_name: String) -> Self {
        Self { interface_name }
    }
}
