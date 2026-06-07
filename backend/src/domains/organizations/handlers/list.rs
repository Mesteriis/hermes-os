use super::dto::{ListQuery, OrgList};
use crate::app::handlers::{ApiError, AppState};
use crate::domains::organizations::api::OrganizationStore;
use axum::Json;
use axum::extract::{Query, State};
pub(crate) async fn list(
    State(s): State<AppState>,
    Query(_q): Query<ListQuery>,
) -> Result<Json<OrgList>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(OrgList {
        items: OrganizationStore::new(pool).list(None, 200).await?,
    }))
}
