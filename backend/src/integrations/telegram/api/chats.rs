use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::Utc;
use serde_json::json;

use super::helpers::AUDIT_ACTOR_ID;
use super::helpers::{publish_telegram_event, telegram_secret_store};
use crate::app::{ApiError, AppState};
use crate::domains::api_support::{
    TelegramChatListResponse, TelegramListQuery, communication_ingestion_store, telegram_store,
};
use crate::integrations::telegram::client::{
    TelegramChat, TelegramChatGroupFilterListResponse, TelegramChatMember, TelegramError, lifecycle,
};
use crate::integrations::telegram::runtime::{TelegramChatSyncRequest, TelegramChatSyncResponse};
use crate::integrations::telegram::runtime::{
    TelegramHistorySyncRequest, TelegramHistorySyncResponse,
};
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

fn build_command_event(
    account_id: &str,
    command_id: &str,
    provider_chat_id: &str,
    telegram_chat_id: &str,
    action: &str,
    status: &str,
    chat: Option<&TelegramChat>,
) -> NewEventEnvelope {
    build_event(
        telegram_event_types::COMMAND_STATUS_CHANGED,
        account_id,
        command_id,
        json!({
            "command_id": command_id,
            "action": action,
            "provider_chat_id": provider_chat_id,
            "telegram_chat_id": telegram_chat_id,
            "status": status,
            "chat": chat,
        }),
    )
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
}

#[derive(serde::Deserialize)]
pub(crate) struct TelegramChatMembersQuery {
    pub(crate) limit: Option<i64>,
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
    let items = telegram_store(&state)?
        .list_chat_members(&telegram_chat_id, query.limit.unwrap_or(50))
        .await?;

    Ok(Json(TelegramChatMemberListResponse { items }))
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

    let secret_store = telegram_secret_store(&state)?;
    let response = match state
        .telegram_runtime
        .sync_chats(
            &communication_ingestion_store(&state)?,
            &telegram_store(&state)?,
            &secret_store,
            &state.vault,
            &state.config,
            &request,
        )
        .await
    {
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

    let secret_store = telegram_secret_store(&state)?;
    let response = match state
        .telegram_runtime
        .sync_history(
            &communication_ingestion_store(&state)?,
            &telegram_store(&state)?,
            &secret_store,
            &state.vault,
            &state.config,
            &request,
        )
        .await
    {
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

// ---------------------------------------------------------------------------
// Dialog management endpoints (ADR-0091)
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub(crate) struct TelegramChatActionRequest {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
}

#[derive(serde::Serialize)]
pub(crate) struct TelegramChatActionResponse {
    pub(crate) telegram_chat_id: String,
    pub(crate) action: String,
    pub(crate) status: String,
    pub(crate) metadata: serde_json::Value,
}

async fn record_dialog_command(
    state: &AppState,
    telegram_chat_id: &str,
    request: &TelegramChatActionRequest,
    command_kind: &str,
    action_class: &str,
) -> Result<String, ApiError> {
    let store = telegram_store(state)?;
    let command_id = lifecycle::new_command_id();

    let _cmd = lifecycle::insert_command(
        store.pool(),
        &command_id,
        &request.account_id,
        command_kind,
        &format!(
            "{command_kind}:{}:{}",
            telegram_chat_id,
            Utc::now().timestamp_millis()
        ),
        &request.provider_chat_id,
        None,
        "available",
        action_class,
        "confirmed",
        AUDIT_ACTOR_ID,
        json!({}),
        json!({"telegram_chat_id": telegram_chat_id}),
        json!({}),
    )
    .await?;

    let chat = store.telegram_chat_by_id(telegram_chat_id).await?;
    let command_event = build_command_event(
        &request.account_id,
        &command_id,
        &request.provider_chat_id,
        telegram_chat_id,
        command_kind,
        "queued",
        chat.as_ref(),
    );
    publish_telegram_event(state, command_event).await?;

    Ok(command_id)
}

pub(crate) async fn post_telegram_chat_pin(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Json(request): Json<TelegramChatActionRequest>,
) -> Result<Json<TelegramChatActionResponse>, ApiError> {
    let store = telegram_store(&state)?;
    let metadata = store
        .set_chat_metadata_bool(&telegram_chat_id, "is_pinned", true)
        .await?;
    let _command_id =
        record_dialog_command(&state, &telegram_chat_id, &request, "pin", "provider_write").await?;

    Ok(Json(TelegramChatActionResponse {
        telegram_chat_id,
        action: "pin".to_owned(),
        status: "pinned".to_owned(),
        metadata,
    }))
}

pub(crate) async fn post_telegram_chat_unpin(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Json(request): Json<TelegramChatActionRequest>,
) -> Result<Json<TelegramChatActionResponse>, ApiError> {
    let store = telegram_store(&state)?;
    let metadata = store
        .set_chat_metadata_bool(&telegram_chat_id, "is_pinned", false)
        .await?;
    let _command_id = record_dialog_command(
        &state,
        &telegram_chat_id,
        &request,
        "unpin",
        "provider_write",
    )
    .await?;

    Ok(Json(TelegramChatActionResponse {
        telegram_chat_id,
        action: "unpin".to_owned(),
        status: "unpinned".to_owned(),
        metadata,
    }))
}

pub(crate) async fn post_telegram_chat_archive(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Json(request): Json<TelegramChatActionRequest>,
) -> Result<Json<TelegramChatActionResponse>, ApiError> {
    let store = telegram_store(&state)?;
    let metadata = store
        .set_chat_metadata_bool(&telegram_chat_id, "is_archived", true)
        .await?;
    let _command_id = record_dialog_command(
        &state,
        &telegram_chat_id,
        &request,
        "archive",
        "provider_write",
    )
    .await?;

    Ok(Json(TelegramChatActionResponse {
        telegram_chat_id,
        action: "archive".to_owned(),
        status: "archived".to_owned(),
        metadata,
    }))
}

pub(crate) async fn post_telegram_chat_unarchive(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Json(request): Json<TelegramChatActionRequest>,
) -> Result<Json<TelegramChatActionResponse>, ApiError> {
    let store = telegram_store(&state)?;
    let metadata = store
        .set_chat_metadata_bool(&telegram_chat_id, "is_archived", false)
        .await?;
    let _command_id = record_dialog_command(
        &state,
        &telegram_chat_id,
        &request,
        "unarchive",
        "provider_write",
    )
    .await?;

    Ok(Json(TelegramChatActionResponse {
        telegram_chat_id,
        action: "unarchive".to_owned(),
        status: "unarchived".to_owned(),
        metadata,
    }))
}

pub(crate) async fn post_telegram_chat_mute(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Json(request): Json<TelegramChatActionRequest>,
) -> Result<Json<TelegramChatActionResponse>, ApiError> {
    let store = telegram_store(&state)?;
    let metadata = store
        .set_chat_metadata_bool(&telegram_chat_id, "is_muted", true)
        .await?;
    let _command_id = record_dialog_command(
        &state,
        &telegram_chat_id,
        &request,
        "mute",
        "provider_write",
    )
    .await?;

    Ok(Json(TelegramChatActionResponse {
        telegram_chat_id,
        action: "mute".to_owned(),
        status: "muted".to_owned(),
        metadata,
    }))
}

pub(crate) async fn post_telegram_chat_unmute(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Json(request): Json<TelegramChatActionRequest>,
) -> Result<Json<TelegramChatActionResponse>, ApiError> {
    let store = telegram_store(&state)?;
    let metadata = store
        .set_chat_metadata_bool(&telegram_chat_id, "is_muted", false)
        .await?;
    let _command_id = record_dialog_command(
        &state,
        &telegram_chat_id,
        &request,
        "unmute",
        "provider_write",
    )
    .await?;

    Ok(Json(TelegramChatActionResponse {
        telegram_chat_id,
        action: "unmute".to_owned(),
        status: "unmuted".to_owned(),
        metadata,
    }))
}

pub(crate) async fn post_telegram_chat_mark_read(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Json(request): Json<TelegramChatActionRequest>,
) -> Result<Json<TelegramChatActionResponse>, ApiError> {
    let store = telegram_store(&state)?;
    store
        .set_chat_last_read_at(&telegram_chat_id, Some(Utc::now()))
        .await?;
    let metadata = store.recompute_chat_unread_count(&telegram_chat_id).await?;
    let _command_id = record_dialog_command(
        &state,
        &telegram_chat_id,
        &request,
        "mark_read",
        "local_write",
    )
    .await?;

    Ok(Json(TelegramChatActionResponse {
        telegram_chat_id,
        action: "mark_read".to_owned(),
        status: "read".to_owned(),
        metadata,
    }))
}

pub(crate) async fn post_telegram_chat_mark_unread(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Json(request): Json<TelegramChatActionRequest>,
) -> Result<Json<TelegramChatActionResponse>, ApiError> {
    let store = telegram_store(&state)?;
    store.set_chat_last_read_at(&telegram_chat_id, None).await?;
    let metadata = store.recompute_chat_unread_count(&telegram_chat_id).await?;
    let _command_id = record_dialog_command(
        &state,
        &telegram_chat_id,
        &request,
        "mark_unread",
        "local_write",
    )
    .await?;

    Ok(Json(TelegramChatActionResponse {
        telegram_chat_id,
        action: "mark_unread".to_owned(),
        status: "unread".to_owned(),
        metadata,
    }))
}
