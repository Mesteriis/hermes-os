use serde::{Deserialize, Serialize};
#[derive(Serialize)]
pub struct PersonList {
    pub items: Vec<serde_json::Value>,
}
#[derive(Deserialize)]
pub struct ListQuery {
    pub limit: Option<i64>,
}
#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<i64>,
}
#[derive(Deserialize)]
pub struct NotesRequest {
    pub notes: String,
}
