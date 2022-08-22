use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct IdResponse {
    pub id: String,
}
