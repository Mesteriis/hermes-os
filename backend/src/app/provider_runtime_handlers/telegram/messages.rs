use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::Utc;
use serde_json::json;

use super::helpers::{
    AUDIT_ACTOR_ID, ensure_telegram_account_operation_allowed, publish_telegram_event,
    telegram_message_snapshot_payload,
};
use crate::app::api_support::{
    TelegramListQuery, TelegramMessageListResponse, WhatsappWebMessageListResponse, api_audit_log,
    ensure_fixture_routes_enabled, event_store, telegram_runtime_use_case_context, telegram_store,
    whatsapp_web_store,
};
use crate::app::{ApiError, AppState};
use crate::application::communication_fixture_ingest::TelegramFixtureIngestApplicationService;
use crate::application::communication_provider_writes::{
    CommunicationConversationMessageRequest, CommunicationProviderMessageCommandResponse,
    TelegramMessageWriteApplicationService,
};
use crate::application::telegram_runtime;
use crate::integrations::telegram::client::NewTelegramMessage;
use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::client::lifecycle;
use crate::integrations::telegram::client::models::messages::{
    TelegramDeleteRequest, TelegramEditRequest, TelegramForwardRequest, TelegramLifecycleResponse,
    TelegramManualSendRequest, TelegramManualSendResponse, TelegramMessageIngestResult,
    TelegramMessageTombstoneListResponse, TelegramMessageVersionListResponse, TelegramPinRequest,
    TelegramReplyRequest, TelegramRestoreVisibilityRequest,
};
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

#[allow(clippy::too_many_arguments)]
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

fn telegram_message_write_service(
    state: &AppState,
) -> Result<TelegramMessageWriteApplicationService, ApiError> {
    Ok(TelegramMessageWriteApplicationService::new(
        telegram_store(state)?,
        api_audit_log(state)?,
        event_store(state)?,
        state.event_bus.clone(),
    ))
}

pub(crate) async fn post_telegram_fixture_message(
    State(state): State<AppState>,
    Json(request): Json<NewTelegramMessage>,
) -> Result<Json<TelegramMessageIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let Some(pool) = state.database.pool().cloned() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let response = TelegramFixtureIngestApplicationService::new(
        pool,
        telegram_store(&state)?,
        event_store(&state)?,
        state.event_bus.clone(),
    )
    .ingest_message(&request)
    .await?;
    Ok(Json(response))
}

pub(crate) async fn post_telegram_manual_send(
    State(state): State<AppState>,
    Json(request): Json<TelegramManualSendRequest>,
) -> Result<Json<TelegramManualSendResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "messages.send_text")
        .await?;
    let runtime_context = telegram_runtime_use_case_context(&state)?;
    let response = telegram_message_write_service(&state)?
        .send_manual_message(&runtime_context, &request)
        .await?;
    Ok(Json(response))
}

pub(crate) async fn post_communication_conversation_message(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
    Json(mut request): Json<CommunicationConversationMessageRequest>,
) -> Result<Json<CommunicationProviderMessageCommandResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "messages.send_text")
        .await?;
    let command_id = request
        .command_id
        .clone()
        .unwrap_or_else(lifecycle::new_command_id);
    request.command_id = Some(command_id.clone());
    let runtime_context = telegram_runtime_use_case_context(&state)?;
    let response = telegram_message_write_service(&state)?
        .send_conversation_message(&runtime_context, &conversation_id, request)
        .await?;
    Ok(Json(CommunicationProviderMessageCommandResponse::telegram(
        command_id, &response,
    )))
}

pub(crate) async fn post_telegram_message_reply(
    State(state): State<AppState>,
    Path(_message_id): Path<String>,
    Json(request): Json<TelegramReplyRequest>,
) -> Result<Json<TelegramManualSendResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "messages.reply")
        .await?;
    let runtime_context = telegram_runtime_use_case_context(&state)?;
    let response = telegram_message_write_service(&state)?
        .send_reply_message(&runtime_context, &request)
        .await?;
    Ok(Json(response))
}

pub(crate) async fn post_telegram_message_forward(
    State(state): State<AppState>,
    Path(_message_id): Path<String>,
    Json(request): Json<TelegramForwardRequest>,
) -> Result<Json<TelegramManualSendResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "messages.forward")
        .await?;
    let runtime_context = telegram_runtime_use_case_context(&state)?;
    let response = telegram_message_write_service(&state)?
        .send_forward_message(&runtime_context, &request)
        .await?;
    Ok(Json(response))
}

pub(crate) async fn get_telegram_messages(
    State(state): State<AppState>,
    Query(query): Query<TelegramListQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if matches!(
        query.provider.as_deref().map(str::trim),
        Some("whatsapp") | Some("whatsapp_web")
    ) {
        let items = whatsapp_web_store(&state)?
            .recent_messages(
                query.account_id.as_deref(),
                query.provider_chat_id.as_deref(),
                query.limit.unwrap_or(50),
            )
            .await?;
        return Ok(Json(json!(WhatsappWebMessageListResponse { items })));
    }
    if let Some(provider) = query.provider.as_deref().map(str::trim)
        && !provider.is_empty()
        && provider != "telegram"
        && provider != "telegram_user"
        && provider != "telegram_bot"
    {
        return Err(ApiError::InvalidCommunicationQuery(
            "unsupported communications message provider",
        ));
    }

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

    Ok(Json(json!(TelegramMessageListResponse { items })))
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
    let response = lifecycle::record_edit(&store, &request, &message_id, AUDIT_ACTOR_ID).await?;

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
        lifecycle::record_pin_state(&store, &request, &message_id, AUDIT_ACTOR_ID).await?;

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

/// GET /api/v1/communications/messages/{message_id}/reply-chain
pub(crate) async fn get_telegram_reply_chain(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<TelegramReplyChainResponse>, ApiError> {
    let store = telegram_store(&state)?;
    let chain = lifecycle::reply_chain(&store, &message_id).await?;
    Ok(Json(chain))
}

/// GET /api/v1/communications/messages/{message_id}/forward-chain
pub(crate) async fn get_telegram_forward_chain(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<TelegramForwardChainResponse>, ApiError> {
    let store = telegram_store(&state)?;
    let chain = lifecycle::forward_chain(&store, &message_id).await?;
    Ok(Json(chain))
}
