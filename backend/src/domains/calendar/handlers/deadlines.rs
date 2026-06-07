use crate::app::handlers::{ApiError, AppState};
use crate::domains::calendar::scheduling::DeadlineStore;
use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
#[derive(Deserialize)]
pub struct DeadlineQuery {
    pub status: Option<String>,
    pub limit: Option<i64>,
}
pub(crate) async fn deadlines(
    State(s): State<AppState>,
    Query(q): Query<DeadlineQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = DeadlineStore::new(pool)
        .list(q.status.as_deref(), q.limit.unwrap_or(50))
        .await?;
    Ok(Json(serde_json::to_value(items).unwrap_or_default()))
}
