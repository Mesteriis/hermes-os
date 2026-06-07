use super::dto::SearchQuery;
use crate::app::handlers::{ApiError, AppState};
use crate::domains::graph::core::{GraphNode, GraphStore};
use axum::Json;
use axum::extract::{Query, State};
pub async fn search(
    State(state): State<AppState>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<Vec<GraphNode>>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let term = q.q.as_deref().unwrap_or_default().trim();
    if term.is_empty() {
        return Err(ApiError::InvalidGraphQuery("q must not be empty"));
    }
    let limit = q.limit.unwrap_or(20).clamp(1, 50);
    Ok(Json(GraphStore::new(pool).search_nodes(term, limit).await?))
}
