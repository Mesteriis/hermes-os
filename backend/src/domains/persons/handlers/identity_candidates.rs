use axum::extract::{Query, State}; use axum::Json;
use serde::Deserialize;
use crate::app::handlers::{ApiError, AppState};
use crate::domains::persons::identity::PersonIdentityStore;

#[derive(Deserialize)] pub struct CandidateQuery { pub limit: Option<i64> }
pub(crate) async fn identity_candidates(State(state): State<AppState>, Query(q): Query<CandidateQuery>) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    Ok(Json(serde_json::to_value(PersonIdentityStore::new(pool).list_candidates(q.limit).await?).unwrap_or_default()))
}
