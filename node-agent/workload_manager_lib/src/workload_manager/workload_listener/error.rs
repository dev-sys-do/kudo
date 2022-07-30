use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct WorkloadListenerError {
    details: String,
}

impl WorkloadListenerError {
    pub fn new(msg: &str) -> Self {
        WorkloadListenerError{details: msg.to_string()}
    }
}

impl fmt::Display for WorkloadListenerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}", self.details)
    }
}

impl Error for WorkloadListenerError {
    fn description(&self) -> &str {
        &self.details.as_str()
    }
}