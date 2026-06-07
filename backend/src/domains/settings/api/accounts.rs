use super::dto::AccountsResponse;
use crate::app::handlers::{ApiError, AppState};
use crate::domains::mail::core::CommunicationIngestionStore;
use axum::Json;
use axum::extract::State;

pub(crate) async fn accounts(
    State(state): State<AppState>,
) -> Result<Json<AccountsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = CommunicationIngestionStore::new(pool)
        .list_provider_accounts()
        .await?;
    Ok(Json(AccountsResponse { items }))
}
