use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;

use crate::app::{ApiError, AppState};
use crate::domains::api_support::telegram_store;
use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::client::commands::list_commands_filtered;
use crate::integrations::telegram::client::models::messages::TelegramCommandListResponse;

#[derive(Deserialize)]
pub(crate) struct TelegramCommandListQuery {
    pub(crate) account_id: Option<String>,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) provider_message_id: Option<String>,
    pub(crate) command_kinds: Option<String>,
    pub(crate) limit: Option<i64>,
}

pub(crate) async fn get_telegram_commands(
    State(state): State<AppState>,
    Query(query): Query<TelegramCommandListQuery>,
) -> Result<Json<TelegramCommandListResponse>, ApiError> {
    let account_id = query.account_id.ok_or_else(|| {
        ApiError::Telegram(TelegramError::InvalidRequest(
            "account_id is required".to_owned(),
        ))
    })?;
    let command_kinds = query
        .command_kinds
        .as_deref()
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let store = telegram_store(&state)?;
    let items = list_commands_filtered(
        store.pool(),
        &account_id,
        query.provider_chat_id.as_deref(),
        query.provider_message_id.as_deref(),
        &command_kinds,
        query.limit.unwrap_or(50),
    )
    .await?;
    Ok(Json(TelegramCommandListResponse { items }))
}
