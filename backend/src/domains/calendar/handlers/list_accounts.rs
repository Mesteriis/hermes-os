use crate::app::handlers::{ApiError, AppState};
use crate::domains::calendar::events::CalendarAccountStore;
use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
#[derive(Deserialize)]
pub struct AccountQuery {
    pub provider: Option<String>,
}
pub async fn list_accounts(
    State(s): State<AppState>,
    Query(q): Query<AccountQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = CalendarAccountStore::new(pool)
        .list(q.provider.as_deref())
        .await?;
    Ok(Json(serde_json::to_value(items).unwrap_or_default()))
}
