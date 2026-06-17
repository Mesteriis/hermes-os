use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::Utc;
use serde_json::json;

use super::helpers::{
    AUDIT_ACTOR_ID, ensure_telegram_account_operation_allowed, publish_telegram_event,
    telegram_message_snapshot_payload, telegram_runtime_event_bridge_context,
    telegram_secret_store,
};
use crate::app::{ApiError, AppState};
use crate::domains::api_support::{
    TelegramListQuery, TelegramMessageListResponse, api_audit_log, communication_ingestion_store,
    telegram_store,
};
use crate::integrations::telegram::client::NewTelegramMessage;
use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::client::lifecycle;
use crate::integrations::telegram::client::models::messages::{
    TelegramDeleteRequest, TelegramEditRequest, TelegramForwardRequest, TelegramLifecycleResponse,
    TelegramManualSendRequest, TelegramManualSendResponse, TelegramMessageIngestResult,
    TelegramMessageTombstoneListResponse, TelegramMessageVersionListResponse, TelegramPinRequest,
    TelegramReplyRequest, TelegramRestoreVisibilityRequest,
};
use crate::integrations::telegram::runtime::TelegramRuntimeOperationContext;
use crate::platform::audit::NewApiAuditRecord;
use crate::platform::events::NewEventEnvelope;
use crate::platform::events::bus::telegram_event_types;

mod mark_read;
mod reactions;

pub(crate) use mark_read::post_telegram_message_mark_read;
pub(crate) use reactions::{
    delete_telegram_reaction, get_telegram_reactions, post_telegram_reaction,
};

pub(super) fn build_event(
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
        json!({"id": subject_id, "kind": "telegram_message"}),
    )
    .payload(payload)
    .build()
    .expect("event envelope must be valid")
}

fn build_command_event(
    account_id: &str,
    command_id: &str,
    command_kind: &str,
    provider_chat_id: &str,
    message_id: Option<&str>,
    provider_message_id: Option<&str>,
    status: &str,
    extra_payload: serde_json::Value,
) -> NewEventEnvelope {
    let mut payload = json!({
        "account_id": account_id,
        "command_id": command_id,
        "command_kind": command_kind,
        "action": command_kind,
        "provider_chat_id": provider_chat_id,
        "message_id": message_id,
        "provider_message_id": provider_message_id,
        "status": status,
    });
    if let (Some(payload_obj), Some(extra_obj)) =
        (payload.as_object_mut(), extra_payload.as_object())
    {
        for (key, value) in extra_obj {
            payload_obj.insert(key.clone(), value.clone());
        }
    }
    if let Some(payload_obj) = payload.as_object_mut() {
        payload_obj.insert("payload".to_owned(), extra_payload);
    }
    build_event(
        telegram_event_types::COMMAND_STATUS_CHANGED,
        account_id,
        command_id,
        payload,
    )
}

pub(crate) async fn post_telegram_fixture_message(
    State(state): State<AppState>,
    Json(request): Json<NewTelegramMessage>,
) -> Result<Json<TelegramMessageIngestResult>, ApiError> {
    let response = telegram_store(&state)?
        .ingest_fixture_message(&request)
        .await?;
    let store = telegram_store(&state)?;

    let event = build_event(
        telegram_event_types::MESSAGE_CREATED,
        &request.account_id,
        &response.message_id,
        telegram_message_snapshot_payload(
            &store,
            &response.message_id,
            json!({
            "provider_chat_id": &request.provider_chat_id,
            "provider_message_id": &request.provider_message_id,
            "delivery_state": request.delivery_state.as_str(),
            "runtime_kind": "fixture",
            }),
        )
        .await?,
    );
    publish_telegram_event(&state, event).await?;

    Ok(Json(response))
}

pub(crate) async fn post_telegram_manual_send(
    State(state): State<AppState>,
    Json(request): Json<TelegramManualSendRequest>,
) -> Result<Json<TelegramManualSendResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "messages.send_text")
        .await?;
    let communication_store = communication_ingestion_store(&state)?;
    let telegram_projection_store = telegram_store(&state)?;
    let secret_store = telegram_secret_store(&state)?;
    let context = TelegramRuntimeOperationContext {
        communication_store: &communication_store,
        telegram_store: &telegram_projection_store,
        secret_store: &secret_store,
        secret_resolver: &state.vault,
        config: &state.config,
        event_bridge: Some(telegram_runtime_event_bridge_context(&state)),
    };
    let response = state
        .telegram_runtime
        .send_manual_message(&context, &request)
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
    let store = telegram_store(&state)?;

    let event = build_event(
        telegram_event_types::MESSAGE_CREATED,
        &response.account_id,
        &response.message_id,
        telegram_message_snapshot_payload(
            &store,
            &response.message_id,
            json!({
            "provider_chat_id": &response.provider_chat_id,
            "delivery_state": &response.delivery_state,
            "runtime_kind": &response.runtime_kind,
            "status": &response.status,
            }),
        )
        .await?,
    );
    publish_telegram_event(&state, event).await?;

    let command_event = build_command_event(
        &response.account_id,
        &request.command_id,
        "send_text",
        &response.provider_chat_id,
        Some(&response.message_id),
        None,
        &response.status,
        json!({
            "telegram_message_id": &response.message_id,
            "runtime_kind": &response.runtime_kind,
        }),
    );
    publish_telegram_event(&state, command_event).await?;

    Ok(Json(response))
}

pub(crate) async fn post_telegram_message_reply(
    State(state): State<AppState>,
    Path(_message_id): Path<String>,
    Json(request): Json<TelegramReplyRequest>,
) -> Result<Json<TelegramManualSendResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "messages.reply")
        .await?;
    let communication_store = communication_ingestion_store(&state)?;
    let telegram_projection_store = telegram_store(&state)?;
    let secret_store = telegram_secret_store(&state)?;
    let context = TelegramRuntimeOperationContext {
        communication_store: &communication_store,
        telegram_store: &telegram_projection_store,
        secret_store: &secret_store,
        secret_resolver: &state.vault,
        config: &state.config,
        event_bridge: Some(telegram_runtime_event_bridge_context(&state)),
    };
    let response = state
        .telegram_runtime
        .send_reply_message(&context, &request)
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

    let store = telegram_store(&state)?;
    let event = build_event(
        telegram_event_types::MESSAGE_CREATED,
        &response.account_id,
        &response.message_id,
        telegram_message_snapshot_payload(
            &store,
            &response.message_id,
            json!({
                "provider_chat_id": &response.provider_chat_id,
                "delivery_state": &response.delivery_state,
                "runtime_kind": &response.runtime_kind,
                "status": &response.status,
            }),
        )
        .await?,
    );
    publish_telegram_event(&state, event).await?;

    let command_event = build_command_event(
        &response.account_id,
        &request.command_id,
        "reply",
        &response.provider_chat_id,
        Some(&response.message_id),
        None,
        &response.status,
        json!({
            "telegram_message_id": &response.message_id,
            "runtime_kind": &response.runtime_kind,
            "reply_to_provider_message_id": &request.reply_to_provider_message_id,
        }),
    );
    publish_telegram_event(&state, command_event).await?;

    Ok(Json(response))
}

pub(crate) async fn post_telegram_message_forward(
    State(state): State<AppState>,
    Path(_message_id): Path<String>,
    Json(request): Json<TelegramForwardRequest>,
) -> Result<Json<TelegramManualSendResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "messages.forward")
        .await?;
    let communication_store = communication_ingestion_store(&state)?;
    let telegram_projection_store = telegram_store(&state)?;
    let secret_store = telegram_secret_store(&state)?;
    let context = TelegramRuntimeOperationContext {
        communication_store: &communication_store,
        telegram_store: &telegram_projection_store,
        secret_store: &secret_store,
        secret_resolver: &state.vault,
        config: &state.config,
        event_bridge: Some(telegram_runtime_event_bridge_context(&state)),
    };
    let response = state
        .telegram_runtime
        .send_forward_message(&context, &request)
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

    let store = telegram_store(&state)?;
    let event = build_event(
        telegram_event_types::MESSAGE_CREATED,
        &response.account_id,
        &response.message_id,
        telegram_message_snapshot_payload(
            &store,
            &response.message_id,
            json!({
                "provider_chat_id": &response.provider_chat_id,
                "delivery_state": &response.delivery_state,
                "runtime_kind": &response.runtime_kind,
                "status": &response.status,
                "from_provider_chat_id": &request.from_provider_chat_id,
                "from_provider_message_id": &request.from_provider_message_id,
            }),
        )
        .await?,
    );
    publish_telegram_event(&state, event).await?;

    let command_event = build_command_event(
        &response.account_id,
        &request.command_id,
        "forward",
        &response.provider_chat_id,
        Some(&response.message_id),
        None,
        &response.status,
        json!({
            "telegram_message_id": &response.message_id,
            "runtime_kind": &response.runtime_kind,
            "from_provider_chat_id": &request.from_provider_chat_id,
            "from_provider_message_id": &request.from_provider_message_id,
        }),
    );
    publish_telegram_event(&state, command_event).await?;

    Ok(Json(response))
}

pub(crate) async fn get_telegram_messages(
    State(state): State<AppState>,
    Query(query): Query<TelegramListQuery>,
) -> Result<Json<TelegramMessageListResponse>, ApiError> {
    let store = telegram_store(&state)?;
    let mut items = store
        .recent_messages(
            query.account_id.as_deref(),
            query.provider_chat_id.as_deref(),
            query.limit.unwrap_or(50),
        )
        .await?;

    for message in &mut items {
        let summary = lifecycle::reaction_summary(store.pool(), &message.message_id).await?;
        if summary.active_reactions > 0 {
            let metadata = message.metadata.as_object_mut().ok_or_else(|| {
                ApiError::Telegram(TelegramError::InvalidRequest(
                    "telegram message metadata must be an object".to_owned(),
                ))
            })?;
            metadata.insert(
                "reaction_summary".to_owned(),
                json!({
                    "message_id": summary.message_id,
                    "total_reactions": summary.total_reactions,
                    "active_reactions": summary.active_reactions,
                    "reactions": summary.reactions,
                }),
            );
        }
    }

    Ok(Json(TelegramMessageListResponse { items }))
}

// ---------------------------------------------------------------------------
// Lifecycle endpoints (ADR-0091)
// ---------------------------------------------------------------------------

pub(crate) async fn post_telegram_message_edit(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(request): Json<TelegramEditRequest>,
) -> Result<Json<TelegramLifecycleResponse>, ApiError> {
    request.validate()?;
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "messages.edit").await?;
    let store = telegram_store(&state)?;
    let response =
        lifecycle::record_edit(store.pool(), &request, &message_id, AUDIT_ACTOR_ID).await?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_message_edit(
            AUDIT_ACTOR_ID,
            &message_id,
            &request.account_id,
            &request.provider_chat_id,
        ))
        .await?;

    // Emit realtime event
    let event = build_event(
        telegram_event_types::MESSAGE_UPDATED,
        &request.account_id,
        &message_id,
        telegram_message_snapshot_payload(
            &store,
            &message_id,
            json!({
            "provider_chat_id": &request.provider_chat_id,
            "provider_message_id": &request.provider_message_id,
            "version_number": response.version_number,
            }),
        )
        .await?,
    );
    publish_telegram_event(&state, event).await?;

    let command_event = build_command_event(
        &request.account_id,
        &request.command_id,
        "edit",
        &request.provider_chat_id,
        Some(&message_id),
        Some(&request.provider_message_id),
        "queued",
        json!({
            "telegram_message_id": &message_id,
            "new_text": &request.new_text,
        }),
    );
    publish_telegram_event(&state, command_event).await?;

    Ok(Json(response))
}

pub(crate) async fn post_telegram_message_delete(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(request): Json<TelegramDeleteRequest>,
) -> Result<Json<TelegramLifecycleResponse>, ApiError> {
    request.validate()?;
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "messages.delete")
        .await?;
    let store = telegram_store(&state)?;
    let response =
        lifecycle::record_delete(store.pool(), &request, &message_id, AUDIT_ACTOR_ID).await?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_message_delete(
            AUDIT_ACTOR_ID,
            &message_id,
            &request.account_id,
            &request.provider_chat_id,
        ))
        .await?;

    // Emit realtime event
    let event = build_event(
        telegram_event_types::MESSAGE_DELETED,
        &request.account_id,
        &message_id,
        telegram_message_snapshot_payload(
            &store,
            &message_id,
            json!({
            "provider_chat_id": &request.provider_chat_id,
            "provider_message_id": &request.provider_message_id,
            "reason_class": &request.reason_class,
            "tombstone_id": &response.tombstone_id,
            }),
        )
        .await?,
    );
    publish_telegram_event(&state, event).await?;

    let command_event = build_command_event(
        &request.account_id,
        &request.command_id,
        "delete",
        &request.provider_chat_id,
        Some(&message_id),
        Some(&request.provider_message_id),
        "queued",
        json!({
            "telegram_message_id": &message_id,
            "reason_class": &request.reason_class,
            "actor_class": &request.actor_class,
            "is_provider_delete": request.is_provider_delete,
            "tombstone_id": &response.tombstone_id,
        }),
    );
    publish_telegram_event(&state, command_event).await?;

    Ok(Json(response))
}

pub(crate) async fn post_telegram_message_restore_visibility(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(request): Json<TelegramRestoreVisibilityRequest>,
) -> Result<Json<TelegramLifecycleResponse>, ApiError> {
    request.validate()?;
    ensure_telegram_account_operation_allowed(
        &state,
        &request.account_id,
        "messages.restore_visibility",
    )
    .await?;
    let store = telegram_store(&state)?;
    let response =
        lifecycle::record_restore_visibility(store.pool(), &request, &message_id, AUDIT_ACTOR_ID)
            .await?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_message_restore_visibility(
            AUDIT_ACTOR_ID,
            &message_id,
            &request.account_id,
            &request.provider_chat_id,
        ))
        .await?;

    // Emit realtime event
    let event = build_event(
        telegram_event_types::MESSAGE_VISIBILITY_RESTORED,
        &request.account_id,
        &message_id,
        telegram_message_snapshot_payload(
            &store,
            &message_id,
            json!({
            "provider_chat_id": &request.provider_chat_id,
            "provider_message_id": &request.provider_message_id,
            "tombstone_id": &response.tombstone_id,
            }),
        )
        .await?,
    );
    publish_telegram_event(&state, event).await?;

    let command_event = build_command_event(
        &request.account_id,
        &request.command_id,
        "restore_visibility",
        &request.provider_chat_id,
        Some(&message_id),
        Some(&request.provider_message_id),
        "queued",
        json!({
            "telegram_message_id": &message_id,
            "reason": &request.reason,
            "tombstone_id": &response.tombstone_id,
        }),
    );
    publish_telegram_event(&state, command_event).await?;

    Ok(Json(response))
}

pub(crate) async fn post_telegram_message_pin(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(request): Json<TelegramPinRequest>,
) -> Result<Json<TelegramLifecycleResponse>, ApiError> {
    request.validate()?;
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "messages.pin").await?;
    let store = telegram_store(&state)?;
    let response =
        lifecycle::record_pin_state(store.pool(), &request, &message_id, AUDIT_ACTOR_ID).await?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_message_pin(
            AUDIT_ACTOR_ID,
            &message_id,
            &request.account_id,
            &request.provider_chat_id,
            request.is_pinned,
        ))
        .await?;

    let event = build_event(
        telegram_event_types::MESSAGE_UPDATED,
        &request.account_id,
        &message_id,
        telegram_message_snapshot_payload(
            &store,
            &message_id,
            json!({
            "provider_chat_id": &request.provider_chat_id,
            "provider_message_id": &request.provider_message_id,
            "is_pinned": request.is_pinned,
            "status": &response.status,
            }),
        )
        .await?,
    );
    publish_telegram_event(&state, event).await?;

    let command_event = build_command_event(
        &request.account_id,
        &request.command_id,
        "pin",
        &request.provider_chat_id,
        Some(&message_id),
        Some(&request.provider_message_id),
        "queued",
        json!({
            "telegram_message_id": &message_id,
            "is_pinned": request.is_pinned,
        }),
    );
    publish_telegram_event(&state, command_event).await?;

    Ok(Json(response))
}

pub(crate) async fn get_telegram_message_versions(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<TelegramMessageVersionListResponse>, ApiError> {
    let store = telegram_store(&state)?;
    let versions = lifecycle::list_message_versions(store.pool(), &message_id).await?;
    Ok(Json(TelegramMessageVersionListResponse {
        message_id,
        versions,
    }))
}

pub(crate) async fn get_telegram_message_tombstones(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<TelegramMessageTombstoneListResponse>, ApiError> {
    let store = telegram_store(&state)?;
    let tombstones = lifecycle::list_tombstones(store.pool(), &message_id).await?;
    Ok(Json(TelegramMessageTombstoneListResponse {
        message_id,
        tombstones,
    }))
}

use crate::integrations::telegram::client::models::messages::{
    TelegramForwardChainResponse, TelegramReplyChainResponse,
};

/// GET /api/v1/telegram/messages/{message_id}/reply-chain
pub(crate) async fn get_telegram_reply_chain(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<TelegramReplyChainResponse>, ApiError> {
    let store = telegram_store(&state)?;
    let chain = lifecycle::reply_chain(store.pool(), &message_id).await?;
    Ok(Json(chain))
}

/// GET /api/v1/telegram/messages/{message_id}/forward-chain
pub(crate) async fn get_telegram_forward_chain(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<TelegramForwardChainResponse>, ApiError> {
    let store = telegram_store(&state)?;
    let chain = lifecycle::forward_chain(store.pool(), &message_id).await?;
    Ok(Json(chain))
}
