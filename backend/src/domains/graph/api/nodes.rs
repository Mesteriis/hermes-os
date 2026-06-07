use super::dto::NodesQuery;
use crate::app::handlers::{ApiError, AppState};
use crate::domains::graph::core::{GraphNode, GraphStore};
use axum::Json;
use axum::extract::{Query, State};
pub(crate) async fn nodes(
    State(state): State<AppState>,
    Query(q): Query<NodesQuery>,
) -> Result<Json<Vec<GraphNode>>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let limit = q.limit.unwrap_or(20).clamp(1, 50);
    Ok(Json(
        GraphStore::new(pool).list_nodes_for_picker(limit).await?,
    ))
}
