use crate::app::handlers::{ApiError, AppState};
use crate::domains::graph::core::{GraphStore, GraphSummary};
use axum::Json;
use axum::extract::State;
pub async fn summary(State(state): State<AppState>) -> Result<Json<GraphSummary>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(GraphStore::new(pool).summary().await?))
}
