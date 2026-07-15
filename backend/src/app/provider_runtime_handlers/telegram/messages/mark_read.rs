use axum::Json;
use axum::extract::{Path, State};

use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::application::communication_provider_models::{
    TelegramMessageMarkReadRequest, TelegramMessageMarkReadResponse,
};

use super::super::helpers::ensure_telegram_account_operation_allowed;
use super::telegram_message_write_service;

pub(crate) async fn post_telegram_message_mark_read(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(request): Json<TelegramMessageMarkReadRequest>,
) -> Result<Json<TelegramMessageMarkReadResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "messages.mark_read")
        .await?;
    let response = telegram_message_write_service(&state)?
        .mark_message_read(&message_id, &request)
        .await?;
    Ok(Json(response))
}
