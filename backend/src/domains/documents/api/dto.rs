use crate::domains::documents::processing::DocumentProcessingJob;
use serde::{Deserialize, Serialize};
#[derive(Serialize)]
pub struct JobsResponse {
    pub items: Vec<DocumentProcessingJob>,
}
#[derive(Deserialize)]
pub struct JobsQuery {
    pub limit: Option<i64>,
}
#[derive(Deserialize)]
pub struct RetryRequest {
    pub actor_id: Option<String>,
}
#[derive(Serialize)]
pub struct RetryResponse {
    pub job_id: String,
    pub status: String,
}
