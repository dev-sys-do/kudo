use serde::{Deserialize, Serialize};
pub enum FilterError {
    OutOfRange,
}

#[derive(Deserialize, Serialize)]
pub struct Pagination {
    pub limit: u32,
    pub offset: u32,
}
