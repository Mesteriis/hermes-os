use crate::domains::tasks::api::Task;
use serde::{Deserialize, Serialize};
#[derive(Serialize)]
pub struct TaskList {
    pub items: Vec<Task>,
}
#[derive(Deserialize)]
pub struct ListQuery {
    pub limit: Option<i64>,
}
#[derive(Deserialize)]
pub struct NewTask {
    pub title: String,
    pub description: Option<String>,
}
#[derive(Deserialize)]
pub struct TaskUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
}
#[derive(Deserialize)]
pub struct StatusUpdate {
    pub status: String,
}
