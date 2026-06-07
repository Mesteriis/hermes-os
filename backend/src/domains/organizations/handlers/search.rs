use super::dto::SearchQuery;
use crate::app::handlers::{ApiError, AppState};
use crate::domains::organizations::api::{Organization, OrganizationStore};
use axum::Json;
use axum::extract::{Query, State};
pub(crate) async fn search(
    State(s): State<AppState>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<Vec<Organization>>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let all = OrganizationStore::new(pool).list(None, 200).await?;
    let term = q.q.to_lowercase();
    let items: Vec<_> = all
        .into_iter()
        .filter(|o| {
            o.display_name.to_lowercase().contains(&term)
                || o.legal_name
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&term)
        })
        .take(q.limit.unwrap_or(20).clamp(1, 100) as usize)
        .collect();
    Ok(Json(items))
}
