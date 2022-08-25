pub struct Port {
    pub source: i32,
    pub destination: i32,
}

impl Port {
    pub fn new(source: i32, destination: i32) -> Self {
        Self {
            source,
            destination,
        }
    }
}
