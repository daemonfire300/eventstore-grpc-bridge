use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Response {
    pub id: String,
    pub error: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Request {
    pub id: uuid::Uuid,
    pub namespace: String,
    pub data: Vec<u8>,
}
