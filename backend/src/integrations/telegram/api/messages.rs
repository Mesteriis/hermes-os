use axum::Json;
use axum::extract::{Query, State};

use super::helpers::{AUDIT_ACTOR_ID, telegram_secret_store};
use crate::app::{ApiError, AppState};
use crate::domains::api_support::{
    TelegramListQuery, TelegramMessageListResponse, api_audit_log, communication_ingestion_store,
    telegram_store,
};
use crate::integrations::telegram::client::{
    NewTelegramMessage, TelegramManualSendRequest, TelegramManualSendResponse,
    TelegramMessageIngestResult,
};
use crate::platform::audit::NewApiAuditRecord;

pub(crate) async fn post_telegram_fixture_message(
    State(state): State<AppState>,
    Json(request): Json<NewTelegramMessage>,
) -> Result<Json<TelegramMessageIngestResult>, ApiError> {
    Ok(Json(
        telegram_store(&state)?
            .ingest_fixture_message(&request)
            .await?,
    ))
}

pub(crate) async fn post_telegram_manual_send(
    State(state): State<AppState>,
    Json(request): Json<TelegramManualSendRequest>,
) -> Result<Json<TelegramManualSendResponse>, ApiError> {
    let secret_store = telegram_secret_store(&state)?;
    let response = state
        .telegram_runtime
        .send_manual_message(
            &communication_ingestion_store(&state)?,
            &telegram_store(&state)?,
            &secret_store,
            &state.vault,
            &state.config,
            &request,
        )
        .await?;
    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_message_send(
            AUDIT_ACTOR_ID,
            &response.message_id,
            &response.account_id,
            &response.provider_chat_id,
            &response.rendered_preview_hash,
        ))
        .await?;

    Ok(Json(response))
}

pub(crate) async fn get_telegram_messages(
    State(state): State<AppState>,
    Query(query): Query<TelegramListQuery>,
) -> Result<Json<TelegramMessageListResponse>, ApiError> {
    let items = telegram_store(&state)?
        .recent_messages(
            query.account_id.as_deref(),
            query.provider_chat_id.as_deref(),
            query.limit.unwrap_or(50),
        )
        .await?;

    Ok(Json(TelegramMessageListResponse { items }))
}
