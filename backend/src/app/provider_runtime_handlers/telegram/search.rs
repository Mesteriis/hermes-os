use axum::Json;
use axum::extract::{Path, Query, State};
use serde::{Deserialize, Serialize};

use crate::app::api_support::{telegram_runtime_use_case_context, telegram_store};
use crate::app::{ApiError, AppState};
use crate::application::telegram_runtime;
use crate::integrations::telegram::client::models::TelegramChat;

#[derive(Deserialize)]
pub(crate) struct TelegramMessageSearchQuery {
    pub(crate) q: String,
    pub(crate) account_id: Option<String>,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Deserialize)]
pub(crate) struct TelegramProviderSearchCommand {
    pub(crate) account_id: String,
    pub(crate) q: String,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Deserialize)]
pub(crate) struct TelegramMediaSearchQuery {
    pub(crate) q: Option<String>,
    pub(crate) account_id: Option<String>,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) kind: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Deserialize)]
pub(crate) struct TelegramChatSearchQuery {
    pub(crate) q: String,
    pub(crate) account_id: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Deserialize)]
pub(crate) struct TelegramPinnedMessagesQuery {
    pub(crate) limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct TelegramSearchResponse {
    pub(crate) query: String,
    pub(crate) items: Vec<crate::integrations::telegram::client::models::messages::TelegramMessage>,
    pub(crate) total: usize,
}

#[derive(Serialize)]
pub(crate) struct TelegramProviderSearchResponse {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) query: String,
    pub(crate) limit: i32,
    pub(crate) status: String,
    pub(crate) error: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct TelegramChatSearchResponse {
    pub(crate) query: String,
    pub(crate) items: Vec<TelegramChat>,
    pub(crate) total: usize,
}

#[derive(Serialize)]
pub(crate) struct TelegramMediaItem {
    pub(crate) message_id: String,
    pub(crate) provider_message_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) file_name: String,
    pub(crate) kind: String,
    pub(crate) mime_type: Option<String>,
    pub(crate) size_bytes: Option<i64>,
    pub(crate) occurred_at: Option<String>,
    pub(crate) download_state: String,
    pub(crate) tdlib_file_id: Option<i64>,
    pub(crate) provider_attachment_id: Option<String>,
    pub(crate) local_path: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct TelegramMediaSearchResponse {
    pub(crate) query: Option<String>,
    pub(crate) source: String,
    pub(crate) provider_search_attempted: bool,
    pub(crate) provider_search_error: Option<String>,
    pub(crate) items: Vec<TelegramMediaItem>,
}

/// GET /api/v1/communications/search/messages?q=&account_id=&provider_chat_id=&limit=
pub(crate) async fn search_telegram_messages(
    State(state): State<AppState>,
    Query(query): Query<TelegramMessageSearchQuery>,
) -> Result<Json<TelegramSearchResponse>, ApiError> {
    let store = telegram_store(&state)?;
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let search_q = query.q.trim().to_owned();

    if search_q.is_empty() {
        return Err(ApiError::Telegram(
            crate::integrations::telegram::client::TelegramError::InvalidRequest(
                "search query `q` is required".to_owned(),
            ),
        ));
    }

    let items = store
        .search_messages(
            query.account_id.as_deref(),
            query.provider_chat_id.as_deref(),
            &search_q,
            limit,
        )
        .await?;

    Ok(Json(TelegramSearchResponse {
        query: search_q,
        total: items.len(),
        items,
    }))
}

/// POST /api/v1/integrations/telegram/provider-search
pub(crate) async fn post_telegram_provider_search(
    State(state): State<AppState>,
    Json(payload): Json<TelegramProviderSearchCommand>,
) -> Result<Json<TelegramProviderSearchResponse>, ApiError> {
    let limit = payload.limit.unwrap_or(50).clamp(1, 200);
    let search_q = payload.q.trim().to_owned();
    let account_id = payload.account_id.trim();

    if account_id.is_empty() {
        return Err(ApiError::Telegram(
            crate::integrations::telegram::client::TelegramError::InvalidRequest(
                "search payload account_id is required".to_owned(),
            ),
        ));
    }

    if search_q.is_empty() {
        return Err(ApiError::Telegram(
            crate::integrations::telegram::client::TelegramError::InvalidRequest(
                "search query `q` is required".to_owned(),
            ),
        ));
    }

    let runtime_context = telegram_runtime_use_case_context(&state)?;
    let result = telegram_runtime::refresh_provider_search(
        &runtime_context,
        account_id.to_owned(),
        payload.provider_chat_id.clone(),
        search_q.clone(),
        limit as i32,
    )
    .await;

    let (status, error) = match result {
        Ok(()) => ("queued".to_owned(), None),
        Err(error) => {
            tracing::debug!(
                error = %error,
                account_id = %account_id,
                "post_telegram_provider_search: TDLib provider search failed"
            );
            ("failed".to_owned(), Some(error.to_string()))
        }
    };

    Ok(Json(TelegramProviderSearchResponse {
        account_id: account_id.to_owned(),
        provider_chat_id: payload.provider_chat_id,
        query: search_q,
        limit: limit as i32,
        status,
        error,
    }))
}

/// GET /api/v1/communications/conversations/search?q=&account_id=&limit=
pub(crate) async fn search_telegram_chats(
    State(state): State<AppState>,
    Query(query): Query<TelegramChatSearchQuery>,
) -> Result<Json<TelegramChatSearchResponse>, ApiError> {
    let store = telegram_store(&state)?;
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let search_q = query.q.trim().to_owned();

    if search_q.is_empty() {
        return Err(ApiError::Telegram(
            crate::integrations::telegram::client::TelegramError::InvalidRequest(
                "search query `q` is required".to_owned(),
            ),
        ));
    }

    let items = store
        .search_chats(query.account_id.as_deref(), &search_q, limit)
        .await?;

    Ok(Json(TelegramChatSearchResponse {
        query: search_q,
        total: items.len(),
        items,
    }))
}

/// GET /api/v1/communications/conversations/{conversation_id}/pinned-messages?limit=
pub(crate) async fn get_telegram_pinned_messages(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
    Query(query): Query<TelegramPinnedMessagesQuery>,
) -> Result<Json<crate::app::api_support::TelegramMessageListResponse>, ApiError> {
    let store = telegram_store(&state)?;
    let items = store
        .pinned_messages(&conversation_id, query.limit.unwrap_or(100).clamp(1, 200))
        .await?;

    Ok(Json(crate::app::api_support::TelegramMessageListResponse {
        items,
    }))
}

/// GET /api/v1/communications/search/media?account_id=&provider_chat_id=&kind=&limit=
pub(crate) async fn search_telegram_media(
    State(state): State<AppState>,
    Query(query): Query<TelegramMediaSearchQuery>,
) -> Result<Json<TelegramMediaSearchResponse>, ApiError> {
    use crate::integrations::telegram::client::models::messages::TelegramMessage as DomainMsg;

    let store = telegram_store(&state)?;
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let search_q = query
        .q
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let messages = store
        .recent_messages(
            query.account_id.as_deref(),
            query.provider_chat_id.as_deref(),
            limit,
        )
        .await?;

    let mut items = Vec::new();
    for msg in &messages {
        if let Some(arr) = msg
            .metadata
            .get("attachments")
            .or(msg.metadata.get("files"))
            .and_then(|v| v.as_array())
        {
            for att in arr {
                let kind = att
                    .get("attachment_type")
                    .or(att.get("kind"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("file");
                if query.kind.as_deref().is_some_and(|fk| kind != fk) {
                    continue;
                }
                let file_name = att
                    .get("filename")
                    .or(att.get("file_name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_owned();
                let mime_type = att
                    .get("content_type")
                    .or(att.get("mime_type"))
                    .and_then(|v| v.as_str())
                    .map(ToOwned::to_owned);
                if let Some(search_q) = search_q {
                    let search_q = search_q.to_lowercase();
                    let mut haystack = vec![
                        file_name.to_lowercase(),
                        kind.to_lowercase(),
                        msg.provider_message_id.to_lowercase(),
                    ];
                    if let Some(mime) = mime_type.as_deref() {
                        haystack.push(mime.to_lowercase());
                    }
                    if !haystack.into_iter().any(|value| value.contains(&search_q)) {
                        continue;
                    }
                }
                items.push(TelegramMediaItem {
                    message_id: msg.message_id.clone(),
                    provider_message_id: msg.provider_message_id.clone(),
                    provider_chat_id: msg.provider_chat_id.clone().unwrap_or_default(),
                    file_name,
                    kind: kind.to_owned(),
                    mime_type,
                    size_bytes: att
                        .get("size")
                        .or(att.get("size_bytes"))
                        .and_then(|v| v.as_i64()),
                    occurred_at: msg.occurred_at.map(|t| t.to_rfc3339()),
                    download_state: att
                        .get("download_state")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_owned(),
                    tdlib_file_id: att
                        .get("tdlib_file_id")
                        .or_else(|| {
                            att.get("metadata")
                                .and_then(|value| value.get("tdlib_file_id"))
                        })
                        .and_then(|v| v.as_i64()),
                    provider_attachment_id: att
                        .get("attachment_id")
                        .or_else(|| att.get("provider_attachment_id"))
                        .and_then(|v| v.as_str())
                        .map(ToOwned::to_owned),
                    local_path: att
                        .get("local_path")
                        .and_then(|v| v.as_str())
                        .map(ToOwned::to_owned),
                });
            }
        }
    }

    Ok(Json(TelegramMediaSearchResponse {
        query: search_q.map(ToOwned::to_owned),
        source: "projection".to_owned(),
        provider_search_attempted: false,
        provider_search_error: None,
        items,
    }))
}
