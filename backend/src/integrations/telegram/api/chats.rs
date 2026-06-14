use axum::Json;
use axum::extract::{Query, State};

use super::helpers::telegram_secret_store;
use crate::app::{ApiError, AppState};
use crate::domains::api_support::{
    TelegramChatListResponse, TelegramListQuery, communication_ingestion_store, telegram_store,
};
use crate::integrations::telegram::runtime::{TelegramChatSyncRequest, TelegramChatSyncResponse};
use crate::integrations::telegram::runtime::{
    TelegramHistorySyncRequest, TelegramHistorySyncResponse,
};

pub(crate) async fn get_telegram_chats(
    State(state): State<AppState>,
    Query(query): Query<TelegramListQuery>,
) -> Result<Json<TelegramChatListResponse>, ApiError> {
    let items = telegram_store(&state)?
        .list_chats(query.account_id.as_deref(), query.limit.unwrap_or(50))
        .await?;

    Ok(Json(TelegramChatListResponse { items }))
}

pub(crate) async fn post_telegram_sync_chats(
    State(state): State<AppState>,
    Json(request): Json<TelegramChatSyncRequest>,
) -> Result<Json<TelegramChatSyncResponse>, ApiError> {
    let secret_store = telegram_secret_store(&state)?;
    Ok(Json(
        state
            .telegram_runtime
            .sync_chats(
                &communication_ingestion_store(&state)?,
                &telegram_store(&state)?,
                &secret_store,
                &state.vault,
                &state.config,
                &request,
            )
            .await?,
    ))
}

pub(crate) async fn post_telegram_sync_history(
    State(state): State<AppState>,
    Json(request): Json<TelegramHistorySyncRequest>,
) -> Result<Json<TelegramHistorySyncResponse>, ApiError> {
    let secret_store = telegram_secret_store(&state)?;
    Ok(Json(
        state
            .telegram_runtime
            .sync_history(
                &communication_ingestion_store(&state)?,
                &telegram_store(&state)?,
                &secret_store,
                &state.vault,
                &state.config,
                &request,
            )
            .await?,
    ))
}
