use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::Utc;
use serde_json::json;

use super::helpers::{
    AUDIT_ACTOR_ID, ensure_telegram_account_operation_allowed, publish_telegram_event,
};
use crate::app::api_support::{
    TelegramChatListResponse, TelegramListQuery, api_audit_log, telegram_runtime_use_case_context,
    telegram_store,
};
use crate::app::{ApiError, AppState};
use crate::application::telegram_runtime;
use crate::integrations::telegram::client::{
    TelegramChat, TelegramChatGroupFilterListResponse, TelegramChatMember, TelegramError,
};
use crate::integrations::telegram::runtime::{TelegramChatSyncRequest, TelegramChatSyncResponse};
use crate::integrations::telegram::runtime::{
    TelegramHistorySyncRequest, TelegramHistorySyncResponse,
};
use crate::platform::audit::NewApiAuditRecord;
use crate::platform::events::NewEventEnvelope;
use crate::platform::events::bus::telegram_event_types;

fn build_event(
    event_type: &str,
    account_id: &str,
    subject_id: &str,
    payload: serde_json::Value,
) -> NewEventEnvelope {
    let now = Utc::now();
    NewEventEnvelope::builder(
        format!("evt_{}", now.timestamp_nanos_opt().unwrap_or(0)),
        event_type.to_owned(),
        now,
        json!({"channel": "telegram", "account_id": account_id}),
        json!({"id": subject_id, "kind": "telegram_sync"}),
    )
    .payload(payload)
    .build()
    .expect("event envelope must be valid")
}

pub(crate) async fn get_telegram_chats(
    State(state): State<AppState>,
    Query(query): Query<TelegramListQuery>,
) -> Result<Json<TelegramChatListResponse>, ApiError> {
    let items = telegram_store(&state)?
        .list_chats(query.account_id.as_deref(), query.limit.unwrap_or(50))
        .await?;

    Ok(Json(TelegramChatListResponse { items }))
}

pub(crate) async fn get_telegram_folders(
    State(state): State<AppState>,
    Query(query): Query<TelegramListQuery>,
) -> Result<Json<TelegramChatGroupFilterListResponse>, ApiError> {
    let items = telegram_store(&state)?
        .list_chat_group_filters(query.account_id.as_deref())
        .await?;

    Ok(Json(TelegramChatGroupFilterListResponse { items }))
}

#[derive(serde::Serialize)]
pub(crate) struct TelegramChatDetailResponse {
    pub(crate) item: TelegramChat,
}

#[derive(serde::Serialize)]
pub(crate) struct TelegramChatMemberListResponse {
    pub(crate) items: Vec<TelegramChatMember>,
    pub(crate) next_cursor: Option<String>,
}

#[derive(serde::Deserialize)]
pub(crate) struct TelegramChatMembersQuery {
    pub(crate) query: Option<String>,
    pub(crate) role: Option<String>,
    pub(crate) limit: Option<i64>,
    pub(crate) cursor: Option<String>,
}

#[derive(serde::Serialize)]
pub(crate) struct TelegramChatMembersSyncResponse {
    pub(crate) telegram_chat_id: String,
    pub(crate) synced_count: usize,
    pub(crate) items: Vec<TelegramChatMember>,
}

pub(crate) async fn get_telegram_chat_detail(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
) -> Result<Json<TelegramChatDetailResponse>, ApiError> {
    let item = telegram_store(&state)?
        .telegram_chat_by_id(&telegram_chat_id)
        .await?
        .ok_or_else(|| {
            ApiError::Telegram(TelegramError::InvalidRequest(format!(
                "Telegram chat `{telegram_chat_id}` was not found"
            )))
        })?;

    Ok(Json(TelegramChatDetailResponse { item }))
}

pub(crate) async fn get_telegram_chat_members(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Query(query): Query<TelegramChatMembersQuery>,
) -> Result<Json<TelegramChatMemberListResponse>, ApiError> {
    let limit = query.limit.unwrap_or(50);
    let items = telegram_store(&state)?
        .list_chat_members(
            &telegram_chat_id,
            query.query.as_deref(),
            query.role.as_deref(),
            limit,
            query.cursor.as_deref(),
        )
        .await?;
    let next_cursor = if items.len() >= limit as usize {
        let offset = query
            .cursor
            .as_deref()
            .unwrap_or("0")
            .parse::<i64>()
            .unwrap_or(0)
            .max(0)
            + limit;
        Some(offset.to_string())
    } else {
        None
    };

    Ok(Json(TelegramChatMemberListResponse { items, next_cursor }))
}

pub(crate) async fn post_telegram_chat_members_sync(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
) -> Result<Json<TelegramChatMembersSyncResponse>, ApiError> {
    let telegram_store = telegram_store(&state)?;
    let chat = telegram_store
        .telegram_chat_by_id(&telegram_chat_id)
        .await?
        .ok_or_else(|| {
            ApiError::Telegram(TelegramError::InvalidRequest(format!(
                "Telegram chat `{telegram_chat_id}` was not found"
            )))
        })?;
    ensure_telegram_account_operation_allowed(&state, &chat.account_id, "participants.sync")
        .await?;
    let started = build_event(
        telegram_event_types::SYNC_STARTED,
        &chat.account_id,
        &telegram_chat_id,
        json!({
            "scope": "members",
            "provider_chat_id": &chat.provider_chat_id,
        }),
    );
    publish_telegram_event(&state, started).await?;

    let runtime_context = telegram_runtime_use_case_context(&state)?;
    let items = match telegram_runtime::sync_chat_members(&runtime_context, &telegram_chat_id).await
    {
        Ok(items) => items,
        Err(error) => {
            let failed = build_event(
                telegram_event_types::SYNC_FAILED,
                &chat.account_id,
                &telegram_chat_id,
                json!({
                    "scope": "members",
                    "provider_chat_id": &chat.provider_chat_id,
                    "status": "failed",
                }),
            );
            publish_telegram_event(&state, failed).await?;
            return Err(error.into());
        }
    };

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_participants_sync(
            AUDIT_ACTOR_ID,
            &telegram_chat_id,
            &chat.account_id,
            &chat.provider_chat_id,
            items.len() as i64,
        ))
        .await?;

    let progress = build_event(
        telegram_event_types::SYNC_PROGRESS,
        &chat.account_id,
        &telegram_chat_id,
        json!({
            "scope": "members",
            "provider_chat_id": &chat.provider_chat_id,
            "synced_count": items.len(),
            "status": "completed",
        }),
    );
    publish_telegram_event(&state, progress).await?;

    let completed = build_event(
        telegram_event_types::SYNC_COMPLETED,
        &chat.account_id,
        &telegram_chat_id,
        json!({
            "scope": "members",
            "provider_chat_id": &chat.provider_chat_id,
            "synced_count": items.len(),
            "status": "completed",
        }),
    );
    publish_telegram_event(&state, completed).await?;

    Ok(Json(TelegramChatMembersSyncResponse {
        telegram_chat_id,
        synced_count: items.len(),
        items,
    }))
}

pub(crate) async fn post_telegram_sync_chats(
    State(state): State<AppState>,
    Json(request): Json<TelegramChatSyncRequest>,
) -> Result<Json<TelegramChatSyncResponse>, ApiError> {
    let started = build_event(
        telegram_event_types::SYNC_STARTED,
        &request.account_id,
        &request.account_id,
        json!({
            "scope": "chats",
        }),
    );
    publish_telegram_event(&state, started).await?;

    let runtime_context = telegram_runtime_use_case_context(&state)?;
    let response = match telegram_runtime::sync_chats(&runtime_context, &request).await {
        Ok(response) => response,
        Err(error) => {
            let failed = build_event(
                telegram_event_types::SYNC_FAILED,
                &request.account_id,
                &request.account_id,
                json!({
                    "scope": "chats",
                    "status": "failed",
                }),
            );
            publish_telegram_event(&state, failed).await?;
            return Err(error.into());
        }
    };

    let progress = build_event(
        telegram_event_types::SYNC_PROGRESS,
        &request.account_id,
        &request.account_id,
        json!({
            "scope": "chats",
            "synced_count": response.synced_count,
            "status": &response.status,
        }),
    );
    publish_telegram_event(&state, progress).await?;

    let completed = build_event(
        telegram_event_types::SYNC_COMPLETED,
        &request.account_id,
        &request.account_id,
        json!({
            "scope": "chats",
            "synced_count": response.synced_count,
            "status": &response.status,
        }),
    );
    publish_telegram_event(&state, completed).await?;

    Ok(Json(response))
}

pub(crate) async fn post_telegram_sync_history(
    State(state): State<AppState>,
    Json(request): Json<TelegramHistorySyncRequest>,
) -> Result<Json<TelegramHistorySyncResponse>, ApiError> {
    let started = build_event(
        telegram_event_types::SYNC_STARTED,
        &request.account_id,
        &request.provider_chat_id,
        json!({
            "scope": "history",
            "provider_chat_id": &request.provider_chat_id,
            "mode": &request.mode,
        }),
    );
    publish_telegram_event(&state, started).await?;

    let runtime_context = telegram_runtime_use_case_context(&state)?;
    let response = match telegram_runtime::sync_history(&runtime_context, &request).await {
        Ok(response) => response,
        Err(error) => {
            let failed = build_event(
                telegram_event_types::SYNC_FAILED,
                &request.account_id,
                &request.provider_chat_id,
                json!({
                    "scope": "history",
                    "provider_chat_id": &request.provider_chat_id,
                    "mode": &request.mode,
                    "status": "failed",
                }),
            );
            publish_telegram_event(&state, failed).await?;
            return Err(error.into());
        }
    };

    let progress = build_event(
        telegram_event_types::SYNC_PROGRESS,
        &request.account_id,
        &request.provider_chat_id,
        json!({
            "scope": "history",
            "provider_chat_id": &request.provider_chat_id,
            "mode": &request.mode,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
            "status": &response.status,
        }),
    );
    publish_telegram_event(&state, progress).await?;

    let completed = build_event(
        telegram_event_types::SYNC_COMPLETED,
        &request.account_id,
        &request.provider_chat_id,
        json!({
            "scope": "history",
            "provider_chat_id": &request.provider_chat_id,
            "mode": &request.mode,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
            "status": &response.status,
        }),
    );
    publish_telegram_event(&state, completed).await?;

    Ok(Json(response))
}
