use axum::Json;
use axum::extract::{Path, Query, State};
use serde::{Deserialize, Serialize};

use crate::app::{ApiError, AppState};
use crate::domains::api_support::{
    TelegramMessageListResponse, communication_ingestion_store, telegram_store,
};
use crate::integrations::telegram::client::{TelegramTopic, TelegramTopicListResponse};

use super::helpers::telegram_secret_store;

#[derive(Deserialize)]
pub(crate) struct TelegramTopicsQuery {
    pub(crate) limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct TelegramTopicListApiResponse {
    pub(crate) telegram_chat_id: String,
    pub(crate) items: Vec<TelegramTopic>,
}

#[derive(Deserialize)]
pub(crate) struct TelegramTopicMessagesQuery {
    pub(crate) limit: Option<i64>,
}

/// GET /api/v1/telegram/chats/{telegram_chat_id}/topics
///
/// Attempts a live TDLib fetch to refresh the topic projection before serving DB rows.
/// Falls back to the DB projection if TDLib is unavailable or the account is in fixture mode.
pub(crate) async fn get_telegram_topics(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Query(query): Query<TelegramTopicsQuery>,
) -> Result<Json<TelegramTopicListApiResponse>, ApiError> {
    let store = telegram_store(&state)?;
    let limit = query.limit.unwrap_or(100).clamp(1, 200);

    let secret_store = telegram_secret_store(&state)?;
    if let Err(error) = state
        .telegram_runtime
        .sync_forum_topics(
            &communication_ingestion_store(&state)?,
            &store,
            &secret_store,
            &state.vault,
            &state.config,
            &telegram_chat_id,
        )
        .await
    {
        tracing::debug!(
            error = %error,
            telegram_chat_id = %telegram_chat_id,
            "get_telegram_topics: TDLib live sync failed, serving DB projection"
        );
    }

    let items = crate::integrations::telegram::client::topics::list_topics(
        store.pool(),
        &telegram_chat_id,
        limit,
    )
    .await?;

    Ok(Json(TelegramTopicListApiResponse {
        telegram_chat_id,
        items,
    }))
}

/// GET /api/v1/telegram/topics/{topic_id}
pub(crate) async fn get_telegram_topic_detail(
    State(state): State<AppState>,
    Path(topic_id): Path<String>,
) -> Result<Json<TelegramTopic>, ApiError> {
    let store = telegram_store(&state)?;
    let topic = crate::integrations::telegram::client::topics::get_topic(store.pool(), &topic_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(topic))
}

/// GET /api/v1/telegram/topics/{topic_id}/messages
/// Returns messages whose metadata.forum_topic_id matches topic_id.
pub(crate) async fn get_telegram_topic_messages(
    State(state): State<AppState>,
    Path(topic_id): Path<String>,
    Query(query): Query<TelegramTopicMessagesQuery>,
) -> Result<Json<TelegramMessageListResponse>, ApiError> {
    let store = telegram_store(&state)?;
    let limit = query.limit.unwrap_or(50).clamp(1, 200);

    let message_ids = crate::integrations::telegram::client::topics::list_topic_message_ids(
        store.pool(),
        &topic_id,
        limit,
    )
    .await?;

    if message_ids.is_empty() {
        return Ok(Json(TelegramMessageListResponse { items: vec![] }));
    }

    // Fetch full message projections for the matching IDs
    let items = store.messages_by_ids(&message_ids).await?;

    Ok(Json(TelegramMessageListResponse { items }))
}
