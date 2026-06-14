use axum::Json;
use axum::extract::State;

use crate::app::{ApiError, AppState};
use crate::domains::api_support::TelegramCapabilitiesResponse;

pub(crate) async fn get_telegram_capabilities(
    State(state): State<AppState>,
) -> Result<Json<TelegramCapabilitiesResponse>, ApiError> {
    Ok(Json(TelegramCapabilitiesResponse::current(&state.config)))
}
