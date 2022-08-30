use proto::scheduler::{Resource, Status};

pub mod listener;
pub mod registered;

#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub status: Status,
    pub resource: Option<Resource>,
}
