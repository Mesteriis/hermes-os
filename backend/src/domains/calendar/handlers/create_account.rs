use crate::app::handlers::{ApiError, AppState};
use crate::domains::calendar::events::CalendarAccountStore;
use axum::Json;
use axum::extract::State;
use serde::Deserialize;
#[derive(Deserialize)]
pub struct NewAccount {
    pub provider: String,
    pub account_name: String,
    pub email: Option<String>,
}
pub(crate) async fn create_account(
    State(s): State<AppState>,
    Json(r): Json<NewAccount>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let account = CalendarAccountStore::new(pool)
        .create(&r.provider, &r.account_name, r.email.as_deref())
        .await?;
    Ok(Json(serde_json::to_value(account).unwrap_or_default()))
}
