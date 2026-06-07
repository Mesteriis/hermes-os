use super::dto::SearchQuery;
use crate::app::handlers::{ApiError, AppState};
use crate::domains::persons::enrichment::PersonEnrichmentStore;
use axum::Json;
use axum::extract::{Query, State};
pub async fn search(
    State(s): State<AppState>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        serde_json::to_value(
            PersonEnrichmentStore::new(pool)
                .search_persons(&q.q, q.limit.unwrap_or(20))
                .await?,
        )
        .unwrap_or_default(),
    ))
}
