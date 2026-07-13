use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use hermes_communications_api::accounts::ProviderAccount;
use hermes_events_api::NewEventEnvelope;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::Row;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::app::api_support::{
    automation_calls::*,
    communications::*,
    ensure_fixture_routes_enabled,
    messaging_integrations::*,
    platform_dtos::*,
    query_parsing::{communication::*, documents::*, graph::*, personas::*, projects::*, tasks::*},
    review_commands::*,
    review_lists::*,
    stores::{ai_runtime::*, domain_stores::*, integration_stores::*, settings_vault::*},
    telegram_capabilities::*,
    whatsapp_capabilities::*,
};
use crate::app::signal_hub_support::{
    provider_account_or_not_found, remove_provider_account_signal_connection,
    sync_provider_account_signal_connection, sync_whatsapp_runtime_signal_connection,
};
use crate::app::{ApiError, AppState};
use crate::application::communication_fixture_ingest::WhatsappFixtureIngestApplicationService;
use crate::domains::communications::storage::AttachmentSafetyScanStatus;
use crate::integrations::whatsapp::client::errors::WhatsappWebError;
use crate::integrations::whatsapp::client::models::{
    NewWhatsappWebCall, NewWhatsappWebDialog, NewWhatsappWebMedia, NewWhatsappWebMessage,
    NewWhatsappWebMessageDelete, NewWhatsappWebMessageUpdate, NewWhatsappWebParticipant,
    NewWhatsappWebPresence, NewWhatsappWebReaction, NewWhatsappWebReceipt,
    NewWhatsappWebRuntimeEvent, NewWhatsappWebStatus, NewWhatsappWebStatusDelete,
    NewWhatsappWebStatusView, WhatsappLiveAccountSetupRequest, WhatsappWebAccountSetupRequest,
    WhatsappWebAccountSetupResponse, WhatsappWebCallIngestResult, WhatsappWebDeliveryState,
    WhatsappWebDialogIngestResult, WhatsappWebMediaIngestResult,
    WhatsappWebMessageDeleteIngestResult, WhatsappWebMessageIngestResult,
    WhatsappWebMessageUpdateIngestResult, WhatsappWebParticipantIngestResult,
    WhatsappWebPresenceIngestResult, WhatsappWebReactionIngestResult,
    WhatsappWebReceiptIngestResult, WhatsappWebRuntimeEventIngestResult,
    WhatsappWebStatusDeleteIngestResult, WhatsappWebStatusIngestResult,
    WhatsappWebStatusViewIngestResult,
};
use crate::integrations::whatsapp::runtime::contracts::{
    WhatsAppAuthorizedSessionCredentialWrite, WhatsAppCommandDeadLetterRequest,
    WhatsAppCredentialBinding, WhatsAppMediaDownloadRequest, WhatsAppMediaUploadRequest,
    WhatsAppPairCodeSession, WhatsAppPairCodeStartRequest, WhatsAppProviderCommand,
    WhatsAppProviderCommandListResponse, WhatsAppProviderCommandResponse,
    WhatsAppProviderRuntimeShape, WhatsAppQrLinkSession, WhatsAppQrLinkStartRequest,
    WhatsAppRuntimeHealth, WhatsAppRuntimeRelinkRequest, WhatsAppRuntimeRemoveRequest,
    WhatsAppRuntimeRemoveResponse, WhatsAppRuntimeRevokeRequest, WhatsAppRuntimeStartRequest,
    WhatsAppRuntimeStatus, WhatsAppRuntimeStopRequest, WhatsAppVoiceNoteSendRequest,
};
use crate::platform::events::bus::{sanitize_event_payload, whatsapp_event_types};
use hermes_observations_api::models::ObservationOriginKind;

pub(crate) mod accounts;
pub(crate) mod conversations;
pub(crate) mod media;
pub(crate) mod messages;
pub(crate) mod statuses;
pub(crate) mod sync_chats;
pub(crate) mod sync_history;
pub(crate) mod sync_statuses;

const AUDIT_ACTOR_ID: &str = "hermes-frontend";
static WHATSAPP_EVENT_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Deserialize)]
pub(crate) struct WhatsAppCommandListQuery {
    pub(crate) account_id: Option<String>,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) provider_message_id: Option<String>,
    pub(crate) command_kinds: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppChatSyncRequest {
    pub(crate) account_id: String,
    pub(crate) limit: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppChatSyncItem {
    pub(crate) conversation_id: String,
    pub(crate) account_id: String,
    pub(crate) channel_kind: String,
    pub(crate) provider_chat_id: String,
    pub(crate) title: String,
    pub(crate) chat_kind: Option<String>,
    pub(crate) is_archived: bool,
    pub(crate) is_pinned: bool,
    pub(crate) is_muted: bool,
    pub(crate) is_unread: bool,
    pub(crate) unread_count: Option<i64>,
    pub(crate) participant_count: Option<i64>,
    pub(crate) community_parent_chat_id: Option<String>,
    pub(crate) community_parent_title: Option<String>,
    pub(crate) invite_link: Option<String>,
    pub(crate) is_community_root: bool,
    pub(crate) is_broadcast: bool,
    pub(crate) is_newsletter: bool,
    pub(crate) avatar_metadata: Value,
    pub(crate) provider_labels: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppChatSyncResponse {
    pub(crate) account_id: String,
    pub(crate) runtime_kind: String,
    pub(crate) status: String,
    pub(crate) synced_count: usize,
    pub(crate) items: Vec<WhatsAppChatSyncItem>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppHistorySyncRequest {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) limit: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppHistorySyncResponse {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) runtime_kind: String,
    pub(crate) status: String,
    pub(crate) synced_count: usize,
    pub(crate) has_more: bool,
    pub(crate) items: Vec<crate::integrations::whatsapp::client::models::WhatsappWebMessage>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppMembersSyncRequest {
    pub(crate) account_id: String,
    pub(crate) limit: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppMembersSyncItem {
    pub(crate) participant_id: String,
    pub(crate) conversation_id: String,
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) provider_member_id: String,
    pub(crate) provider_identity_id: Option<String>,
    pub(crate) sender_display_name: Option<String>,
    pub(crate) role: String,
    pub(crate) status: Option<String>,
    pub(crate) identity_kind: Option<String>,
    pub(crate) address: Option<String>,
    pub(crate) is_admin: bool,
    pub(crate) is_owner: bool,
    pub(crate) participant_metadata: Value,
    pub(crate) identity_metadata: Value,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppMembersSyncResponse {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) runtime_kind: String,
    pub(crate) status: String,
    pub(crate) synced_count: usize,
    pub(crate) has_more: bool,
    pub(crate) items: Vec<WhatsAppMembersSyncItem>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppStatusSyncRequest {
    pub(crate) account_id: String,
    pub(crate) limit: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppStatusSyncResponse {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) runtime_kind: String,
    pub(crate) status: String,
    pub(crate) synced_count: usize,
    pub(crate) has_more: bool,
    pub(crate) items: Vec<crate::integrations::whatsapp::client::models::WhatsappWebMessage>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppPresenceSyncRequest {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppPresenceSyncItem {
    pub(crate) identity_id: String,
    pub(crate) account_id: String,
    pub(crate) channel_kind: String,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) provider_identity_id: String,
    pub(crate) identity_kind: String,
    pub(crate) display_name: Option<String>,
    pub(crate) address: Option<String>,
    pub(crate) presence_state: String,
    pub(crate) last_seen_at: Option<String>,
    pub(crate) observed_at: Option<String>,
    pub(crate) identity_metadata: Value,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppPresenceSyncResponse {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) runtime_kind: String,
    pub(crate) status: String,
    pub(crate) synced_count: usize,
    pub(crate) has_more: bool,
    pub(crate) items: Vec<WhatsAppPresenceSyncItem>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppCallsSyncRequest {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppCallsSyncItem {
    pub(crate) call_id: String,
    pub(crate) account_id: String,
    pub(crate) provider_call_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) direction: String,
    pub(crate) call_state: String,
    pub(crate) started_at: Option<String>,
    pub(crate) ended_at: Option<String>,
    pub(crate) observed_at: Option<String>,
    pub(crate) metadata: Value,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppCallsSyncResponse {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) runtime_kind: String,
    pub(crate) status: String,
    pub(crate) synced_count: usize,
    pub(crate) has_more: bool,
    pub(crate) items: Vec<WhatsAppCallsSyncItem>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppContactsSyncRequest {
    pub(crate) account_id: String,
    pub(crate) limit: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppContactsSyncItem {
    pub(crate) identity_id: String,
    pub(crate) account_id: String,
    pub(crate) channel_kind: String,
    pub(crate) provider_identity_id: String,
    pub(crate) identity_kind: String,
    pub(crate) display_name: Option<String>,
    pub(crate) address: Option<String>,
    pub(crate) push_name: Option<String>,
    pub(crate) business_profile: Value,
    pub(crate) profile_photo_ref: Value,
    pub(crate) display_name_history: Vec<String>,
    pub(crate) identity_metadata: Value,
    pub(crate) whatsapp_trace_metadata: Value,
    pub(crate) phone_trace_metadata: Value,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppContactsSyncResponse {
    pub(crate) account_id: String,
    pub(crate) runtime_kind: String,
    pub(crate) status: String,
    pub(crate) synced_count: usize,
    pub(crate) has_more: bool,
    pub(crate) items: Vec<WhatsAppContactsSyncItem>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppMediaSyncRequest {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) content_type: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppMediaSyncItem {
    pub(crate) attachment_id: String,
    pub(crate) message_id: String,
    pub(crate) raw_record_id: String,
    pub(crate) account_id: String,
    pub(crate) channel_kind: String,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) provider_message_id: String,
    pub(crate) provider_attachment_id: String,
    pub(crate) filename: Option<String>,
    pub(crate) content_type: String,
    pub(crate) size_bytes: i64,
    pub(crate) sha256: String,
    pub(crate) scan_status: String,
    pub(crate) storage_kind: String,
    pub(crate) storage_path: String,
    pub(crate) message_subject: String,
    pub(crate) sender: String,
    pub(crate) sender_display_name: Option<String>,
    pub(crate) occurred_at: Option<String>,
    pub(crate) created_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppMediaSyncResponse {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) content_type: Option<String>,
    pub(crate) runtime_kind: String,
    pub(crate) status: String,
    pub(crate) synced_count: usize,
    pub(crate) has_more: bool,
    pub(crate) items: Vec<WhatsAppMediaSyncItem>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppRuntimeBridgeMediaLifecycleRequest {
    pub(crate) account_id: String,
    pub(crate) command_id: String,
    pub(crate) media_direction: String,
    pub(crate) lifecycle_phase: String,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) provider_message_id: Option<String>,
    pub(crate) provider_media_id: Option<String>,
    pub(crate) attachment_id: Option<String>,
    pub(crate) blob_id: Option<String>,
    pub(crate) progress_percent: Option<u8>,
    pub(crate) content_type: Option<String>,
    pub(crate) filename: Option<String>,
    pub(crate) error_code: Option<String>,
    pub(crate) error_message: Option<String>,
    pub(crate) runtime_blockers: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppRuntimeBridgeSyncLifecycleRequest {
    pub(crate) account_id: String,
    pub(crate) scope: String,
    pub(crate) phase: String,
    pub(crate) subject_id: Option<String>,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) synced_count: Option<i64>,
    pub(crate) has_more: Option<bool>,
    pub(crate) error_code: Option<String>,
    pub(crate) error_message: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppRuntimeBridgeClaimCommandsRequest {
    pub(crate) account_id: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppRuntimeBridgeCommandFailedRequest {
    pub(crate) error_message: String,
    pub(crate) error_code: Option<String>,
    pub(crate) retry_after_seconds: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppRuntimeBridgeExecutableCommand {
    pub(crate) command_id: String,
    pub(crate) account_id: String,
    pub(crate) command_kind: String,
    pub(crate) provider_kind: String,
    pub(crate) provider_shape: String,
    pub(crate) runtime_kind: String,
    pub(crate) lifecycle_state: Option<String>,
    pub(crate) session_restore_available: bool,
    pub(crate) runtime_blockers: Vec<String>,
    pub(crate) provider_chat_id: String,
    pub(crate) provider_message_id: Option<String>,
    pub(crate) idempotency_key: String,
    pub(crate) capability_state: String,
    pub(crate) action_class: String,
    pub(crate) confirmation_decision: String,
    pub(crate) status: String,
    pub(crate) retry_count: i32,
    pub(crate) max_retries: i32,
    pub(crate) payload: Value,
    pub(crate) target_ref: Value,
    pub(crate) audit_metadata: Value,
    pub(crate) provider_state: Value,
    pub(crate) result_payload: Value,
    pub(crate) next_attempt_at: Option<chrono::DateTime<Utc>>,
    pub(crate) last_attempt_at: Option<chrono::DateTime<Utc>>,
    pub(crate) created_at: chrono::DateTime<Utc>,
    pub(crate) updated_at: chrono::DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppRuntimeBridgeClaimCommandsResponse {
    pub(crate) items: Vec<WhatsAppRuntimeBridgeExecutableCommand>,
}

pub(crate) async fn post_whatsapp_sync_members(
    State(state): State<AppState>,
    Path(provider_chat_id): Path<String>,
    Json(request): Json<WhatsAppMembersSyncRequest>,
) -> Result<Json<WhatsAppMembersSyncResponse>, ApiError> {
    let account_id = required_string("account_id", &request.account_id)?;
    ensure_whatsapp_sync_supported(&state, &account_id, "sync_members").await?;
    let provider_chat_id = required_string("provider_chat_id", &provider_chat_id)?;
    let limit = request.limit.unwrap_or(50).clamp(1, 200);
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &provider_chat_id,
        "members",
        "started",
        json!({"scope": "members", "provider_chat_id": provider_chat_id}),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_STARTED,
        &account_id,
        &provider_chat_id,
        json!({"scope": "members", "provider_chat_id": provider_chat_id}),
    )
    .await?;
    let runtime_kind = current_whatsapp_runtime_kind(&state, &account_id).await?;
    let items =
        match list_whatsapp_sync_members(&state, &account_id, &provider_chat_id, limit).await {
            Ok(items) => items,
            Err(error) => {
                capture_whatsapp_sync_runtime_signal(
                    &state,
                    &account_id,
                    &provider_chat_id,
                    "members",
                    "failed",
                    json!({
                        "scope": "members",
                        "provider_chat_id": provider_chat_id,
                        "status": "failed",
                    }),
                )
                .await?;
                publish_whatsapp_sync_event(
                    &state,
                    whatsapp_event_types::SYNC_FAILED,
                    &account_id,
                    &provider_chat_id,
                    json!({
                        "scope": "members",
                        "provider_chat_id": provider_chat_id,
                        "status": "failed",
                    }),
                )
                .await?;
                return Err(error);
            }
        };
    let response = WhatsAppMembersSyncResponse {
        account_id: account_id.clone(),
        provider_chat_id: provider_chat_id.clone(),
        runtime_kind,
        status: "synced".to_owned(),
        synced_count: items.len(),
        has_more: items.len() as i64 >= limit,
        items,
    };
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &provider_chat_id,
        "members",
        "progress",
        json!({
            "scope": "members",
            "provider_chat_id": provider_chat_id,
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_PROGRESS,
        &account_id,
        &provider_chat_id,
        json!({
            "scope": "members",
            "provider_chat_id": provider_chat_id,
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &provider_chat_id,
        "members",
        "completed",
        json!({
            "scope": "members",
            "provider_chat_id": provider_chat_id,
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_COMPLETED,
        &account_id,
        &provider_chat_id,
        json!({
            "scope": "members",
            "provider_chat_id": provider_chat_id,
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    Ok(Json(response))
}

pub(crate) async fn post_whatsapp_sync_presence(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppPresenceSyncRequest>,
) -> Result<Json<WhatsAppPresenceSyncResponse>, ApiError> {
    let account_id = required_string("account_id", &request.account_id)?;
    ensure_whatsapp_sync_supported(&state, &account_id, "sync_presence").await?;
    let limit = request.limit.unwrap_or(50).clamp(1, 200);
    let provider_chat_id = request
        .provider_chat_id
        .as_deref()
        .map(|value| required_string("provider_chat_id", value))
        .transpose()?;
    let subject_id = provider_chat_id
        .clone()
        .unwrap_or_else(|| account_id.clone());
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &subject_id,
        "presence",
        "started",
        json!({
            "scope": "presence",
            "provider_chat_id": provider_chat_id,
        }),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_STARTED,
        &account_id,
        &subject_id,
        json!({
            "scope": "presence",
            "provider_chat_id": provider_chat_id,
        }),
    )
    .await?;
    let runtime_kind = current_whatsapp_runtime_kind(&state, &account_id).await?;
    let items =
        match list_whatsapp_sync_presence(&state, &account_id, provider_chat_id.as_deref(), limit)
            .await
        {
            Ok(items) => items,
            Err(error) => {
                capture_whatsapp_sync_runtime_signal(
                    &state,
                    &account_id,
                    &subject_id,
                    "presence",
                    "failed",
                    json!({
                        "scope": "presence",
                        "provider_chat_id": provider_chat_id,
                        "status": "failed",
                    }),
                )
                .await?;
                publish_whatsapp_sync_event(
                    &state,
                    whatsapp_event_types::SYNC_FAILED,
                    &account_id,
                    &subject_id,
                    json!({
                        "scope": "presence",
                        "provider_chat_id": provider_chat_id,
                        "status": "failed",
                    }),
                )
                .await?;
                return Err(error);
            }
        };
    let response = WhatsAppPresenceSyncResponse {
        account_id: account_id.clone(),
        provider_chat_id: provider_chat_id.clone(),
        runtime_kind,
        status: "synced".to_owned(),
        synced_count: items.len(),
        has_more: items.len() as i64 >= limit,
        items,
    };
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &subject_id,
        "presence",
        "progress",
        json!({
            "scope": "presence",
            "provider_chat_id": provider_chat_id,
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_PROGRESS,
        &account_id,
        &subject_id,
        json!({
            "scope": "presence",
            "provider_chat_id": provider_chat_id,
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &subject_id,
        "presence",
        "completed",
        json!({
            "scope": "presence",
            "provider_chat_id": provider_chat_id,
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_COMPLETED,
        &account_id,
        &subject_id,
        json!({
            "scope": "presence",
            "provider_chat_id": provider_chat_id,
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    Ok(Json(response))
}

pub(crate) async fn post_whatsapp_sync_calls(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppCallsSyncRequest>,
) -> Result<Json<WhatsAppCallsSyncResponse>, ApiError> {
    let account_id = required_string("account_id", &request.account_id)?;
    ensure_whatsapp_sync_supported(&state, &account_id, "sync_calls").await?;
    let limit = request.limit.unwrap_or(50).clamp(1, 200);
    let provider_chat_id = request
        .provider_chat_id
        .as_deref()
        .map(|value| required_string("provider_chat_id", value))
        .transpose()?;
    let subject_id = provider_chat_id
        .clone()
        .unwrap_or_else(|| account_id.clone());
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &subject_id,
        "calls",
        "started",
        json!({
            "scope": "calls",
            "provider_chat_id": provider_chat_id,
        }),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_STARTED,
        &account_id,
        &subject_id,
        json!({
            "scope": "calls",
            "provider_chat_id": provider_chat_id,
        }),
    )
    .await?;
    let runtime_kind = current_whatsapp_runtime_kind(&state, &account_id).await?;
    let items =
        match list_whatsapp_sync_calls(&state, &account_id, provider_chat_id.as_deref(), limit)
            .await
        {
            Ok(items) => items,
            Err(error) => {
                capture_whatsapp_sync_runtime_signal(
                    &state,
                    &account_id,
                    &subject_id,
                    "calls",
                    "failed",
                    json!({
                        "scope": "calls",
                        "provider_chat_id": provider_chat_id,
                        "status": "failed",
                    }),
                )
                .await?;
                publish_whatsapp_sync_event(
                    &state,
                    whatsapp_event_types::SYNC_FAILED,
                    &account_id,
                    &subject_id,
                    json!({
                        "scope": "calls",
                        "provider_chat_id": provider_chat_id,
                        "status": "failed",
                    }),
                )
                .await?;
                return Err(error);
            }
        };
    let response = WhatsAppCallsSyncResponse {
        account_id: account_id.clone(),
        provider_chat_id: provider_chat_id.clone(),
        runtime_kind,
        status: "synced".to_owned(),
        synced_count: items.len(),
        has_more: items.len() as i64 >= limit,
        items,
    };
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &subject_id,
        "calls",
        "progress",
        json!({
            "scope": "calls",
            "provider_chat_id": provider_chat_id,
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_PROGRESS,
        &account_id,
        &subject_id,
        json!({
            "scope": "calls",
            "provider_chat_id": provider_chat_id,
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &subject_id,
        "calls",
        "completed",
        json!({
            "scope": "calls",
            "provider_chat_id": provider_chat_id,
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_COMPLETED,
        &account_id,
        &subject_id,
        json!({
            "scope": "calls",
            "provider_chat_id": provider_chat_id,
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    Ok(Json(response))
}

pub(crate) async fn post_whatsapp_sync_contacts(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppContactsSyncRequest>,
) -> Result<Json<WhatsAppContactsSyncResponse>, ApiError> {
    let account_id = required_string("account_id", &request.account_id)?;
    ensure_whatsapp_sync_supported(&state, &account_id, "sync_contacts").await?;
    let limit = request.limit.unwrap_or(50).clamp(1, 200);
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &account_id,
        "contacts",
        "started",
        json!({
            "scope": "contacts",
        }),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_STARTED,
        &account_id,
        &account_id,
        json!({
            "scope": "contacts",
        }),
    )
    .await?;
    let runtime_kind = current_whatsapp_runtime_kind(&state, &account_id).await?;
    let items = match list_whatsapp_sync_contacts(&state, &account_id, limit).await {
        Ok(items) => items,
        Err(error) => {
            capture_whatsapp_sync_runtime_signal(
                &state,
                &account_id,
                &account_id,
                "contacts",
                "failed",
                json!({
                    "scope": "contacts",
                    "status": "failed",
                }),
            )
            .await?;
            publish_whatsapp_sync_event(
                &state,
                whatsapp_event_types::SYNC_FAILED,
                &account_id,
                &account_id,
                json!({
                    "scope": "contacts",
                    "status": "failed",
                }),
            )
            .await?;
            return Err(error);
        }
    };
    let response = WhatsAppContactsSyncResponse {
        account_id: account_id.clone(),
        runtime_kind,
        status: "synced".to_owned(),
        synced_count: items.len(),
        has_more: items.len() as i64 >= limit,
        items,
    };
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &account_id,
        "contacts",
        "progress",
        json!({
            "scope": "contacts",
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_PROGRESS,
        &account_id,
        &account_id,
        json!({
            "scope": "contacts",
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &account_id,
        "contacts",
        "completed",
        json!({
            "scope": "contacts",
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_COMPLETED,
        &account_id,
        &account_id,
        json!({
            "scope": "contacts",
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    Ok(Json(response))
}

pub(crate) async fn post_whatsapp_sync_media(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppMediaSyncRequest>,
) -> Result<Json<WhatsAppMediaSyncResponse>, ApiError> {
    let account_id = required_string("account_id", &request.account_id)?;
    ensure_whatsapp_sync_supported(&state, &account_id, "sync_media").await?;
    let limit = request.limit.unwrap_or(50).clamp(1, 200);
    let provider_chat_id = request
        .provider_chat_id
        .as_deref()
        .map(|value| required_string("provider_chat_id", value))
        .transpose()?;
    let content_type = request
        .content_type
        .as_deref()
        .map(|value| required_string("content_type", value))
        .transpose()?;
    let subject_id = provider_chat_id
        .clone()
        .unwrap_or_else(|| account_id.clone());
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &subject_id,
        "media",
        "started",
        json!({
            "scope": "media",
            "provider_chat_id": provider_chat_id,
            "content_type": content_type,
        }),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_STARTED,
        &account_id,
        &subject_id,
        json!({
            "scope": "media",
            "provider_chat_id": provider_chat_id,
            "content_type": content_type,
        }),
    )
    .await?;
    let runtime_kind = current_whatsapp_runtime_kind(&state, &account_id).await?;
    let items = match list_whatsapp_sync_media(
        &state,
        &account_id,
        provider_chat_id.as_deref(),
        content_type.as_deref(),
        limit,
    )
    .await
    {
        Ok(items) => items,
        Err(error) => {
            capture_whatsapp_sync_runtime_signal(
                &state,
                &account_id,
                &subject_id,
                "media",
                "failed",
                json!({
                    "scope": "media",
                    "provider_chat_id": provider_chat_id,
                    "content_type": content_type,
                    "status": "failed",
                }),
            )
            .await?;
            publish_whatsapp_sync_event(
                &state,
                whatsapp_event_types::SYNC_FAILED,
                &account_id,
                &subject_id,
                json!({
                    "scope": "media",
                    "provider_chat_id": provider_chat_id,
                    "content_type": content_type,
                    "status": "failed",
                }),
            )
            .await?;
            return Err(error);
        }
    };
    let response = WhatsAppMediaSyncResponse {
        account_id: account_id.clone(),
        provider_chat_id: provider_chat_id.clone(),
        content_type: content_type.clone(),
        runtime_kind,
        status: "synced".to_owned(),
        synced_count: items.len(),
        has_more: items.len() as i64 >= limit,
        items,
    };
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &subject_id,
        "media",
        "progress",
        json!({
            "scope": "media",
            "provider_chat_id": provider_chat_id,
            "content_type": content_type,
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_PROGRESS,
        &account_id,
        &subject_id,
        json!({
            "scope": "media",
            "provider_chat_id": provider_chat_id,
            "content_type": content_type,
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &subject_id,
        "media",
        "completed",
        json!({
            "scope": "media",
            "provider_chat_id": provider_chat_id,
            "content_type": content_type,
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_COMPLETED,
        &account_id,
        &subject_id,
        json!({
            "scope": "media",
            "provider_chat_id": provider_chat_id,
            "content_type": content_type,
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    Ok(Json(response))
}

pub(crate) async fn get_whatsapp_commands(
    State(state): State<AppState>,
    Query(query): Query<WhatsAppCommandListQuery>,
) -> Result<Json<WhatsAppProviderCommandListResponse>, ApiError> {
    let account_id = query.account_id.ok_or_else(|| {
        ApiError::WhatsappWeb(WhatsappWebError::InvalidRequest(
            "account_id is required".to_owned(),
        ))
    })?;
    let command_kinds = query
        .command_kinds
        .as_deref()
        .map(parse_command_kinds)
        .unwrap_or_default();
    Ok(Json(
        whatsapp_provider_runtime_service(&state)?
            .list_provider_commands(
                &account_id,
                query.provider_chat_id.as_deref(),
                query.provider_message_id.as_deref(),
                &command_kinds,
                query.limit.unwrap_or(50),
            )
            .await?,
    ))
}

pub(crate) async fn post_whatsapp_command_retry(
    State(state): State<AppState>,
    Path(command_id): Path<String>,
) -> Result<Json<WhatsAppProviderCommand>, ApiError> {
    let command = whatsapp_provider_runtime_service(&state)?
        .manual_retry_provider_command(&command_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    publish_whatsapp_command_record_event(&state, &command, "manual_retry").await?;
    Ok(Json(command))
}

pub(crate) async fn post_whatsapp_command_dead_letter(
    State(state): State<AppState>,
    Path(command_id): Path<String>,
    Json(request): Json<WhatsAppCommandDeadLetterRequest>,
) -> Result<Json<WhatsAppProviderCommand>, ApiError> {
    let command = whatsapp_provider_runtime_service(&state)?
        .dead_letter_provider_command(&command_id, &request.reason)
        .await?
        .ok_or(ApiError::NotFound)?;
    publish_whatsapp_command_record_event(&state, &command, "manual_dead_letter").await?;
    Ok(Json(command))
}

pub(crate) async fn post_whatsapp_fixture_account(
    State(state): State<AppState>,
    Json(request): Json<WhatsappWebAccountSetupRequest>,
) -> Result<Json<WhatsappWebAccountSetupResponse>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let response = whatsapp_provider_runtime_service(&state)?
        .setup_fixture_account(&request)
        .await?;
    let account = provider_account_or_not_found(&state, &response.account_id).await?;
    sync_provider_account_signal_connection(&state, &account, None).await?;
    Ok(Json(response))
}

pub(crate) async fn post_whatsapp_fixture_message(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebMessage>,
) -> Result<Json<WhatsappWebMessageIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_message(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::MESSAGE_CREATED,
        "whatsapp_message",
        &result.message_id,
        Some(&request.provider_chat_id),
        Some(&request.provider_message_id),
        request.occurred_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "provider_chat_id": request.provider_chat_id,
            "provider_message_id": request.provider_message_id,
            "delivery_state": request.delivery_state.as_str(),
            "sender_id": request.sender_id,
            "sender_display_name": request.sender_display_name,
            "occurred_at": request.occurred_at,
            "source": "fixture_message_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_fixture_reaction(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebReaction>,
) -> Result<Json<WhatsappWebReactionIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_reaction(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::REACTION_CHANGED,
        "whatsapp_message",
        &result.message_id,
        Some(&request.provider_chat_id),
        Some(&request.provider_message_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "reaction_id": result.reaction_id,
            "provider_chat_id": request.provider_chat_id,
            "provider_message_id": request.provider_message_id,
            "provider_actor_id": request.provider_actor_id,
            "sender_display_name": request.sender_display_name,
            "reaction": request.reaction,
            "is_active": request.is_active,
            "observed_at": request.observed_at,
            "source": "fixture_reaction_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_fixture_message_update(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebMessageUpdate>,
) -> Result<Json<WhatsappWebMessageUpdateIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_message_update(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::MESSAGE_UPDATED,
        "whatsapp_message",
        &result.message_id,
        Some(&request.provider_chat_id),
        Some(&request.provider_message_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "version_id": result.version_id,
            "provider_chat_id": request.provider_chat_id,
            "provider_message_id": request.provider_message_id,
            "observed_at": request.observed_at,
            "source": "fixture_message_update_ingest",
            "edited": true,
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_fixture_message_delete(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebMessageDelete>,
) -> Result<Json<WhatsappWebMessageDeleteIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_message_delete(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::MESSAGE_DELETED,
        "whatsapp_message",
        &result.message_id,
        Some(&request.provider_chat_id),
        Some(&request.provider_message_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "tombstone_id": result.tombstone_id,
            "provider_chat_id": request.provider_chat_id,
            "provider_message_id": request.provider_message_id,
            "reason_class": request.reason_class,
            "actor_class": request.actor_class,
            "observed_at": request.observed_at,
            "source": "fixture_message_delete_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_fixture_receipt(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebReceipt>,
) -> Result<Json<WhatsappWebReceiptIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_receipt(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::RECEIPT_CHANGED,
        "whatsapp_message",
        &result.message_id,
        Some(&request.provider_chat_id),
        Some(&request.provider_message_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "provider_chat_id": request.provider_chat_id,
            "provider_message_id": request.provider_message_id,
            "delivery_state": result.delivery_state,
            "observed_at": request.observed_at,
            "source": "fixture_receipt_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_fixture_media(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebMedia>,
) -> Result<Json<WhatsappWebMediaIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    Ok(Json(
        whatsapp_fixture_ingest_service(&state)?
            .ingest_media(&request)
            .await?,
    ))
}

pub(crate) async fn post_whatsapp_fixture_status(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebStatus>,
) -> Result<Json<WhatsappWebStatusIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_status(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::STATUS_UPDATED,
        "whatsapp_status",
        &result.message_id,
        Some(&format!("whatsapp_status_feed:{}", request.account_id)),
        Some(&request.provider_status_id),
        request.occurred_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "provider_status_id": request.provider_status_id,
            "sender_id": request.sender_id,
            "sender_display_name": request.sender_display_name,
            "sender_identity_kind": request.sender_identity_kind,
            "sender_address": request.sender_address,
            "sender_push_name": request.sender_push_name,
            "occurred_at": request.occurred_at,
            "status_state": "posted",
            "source": "fixture_status_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_fixture_status_view(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebStatusView>,
) -> Result<Json<WhatsappWebStatusViewIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_status_view(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::STATUS_UPDATED,
        "whatsapp_status",
        &result.message_id,
        Some(&format!("whatsapp_status_feed:{}", request.account_id)),
        Some(&request.provider_status_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "provider_status_id": request.provider_status_id,
            "viewer_id": request.viewer_id,
            "viewer_display_name": request.viewer_display_name,
            "observed_at": request.observed_at,
            "status_state": "viewed",
            "source": "fixture_status_view_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_fixture_status_delete(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebStatusDelete>,
) -> Result<Json<WhatsappWebStatusDeleteIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_status_delete(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::STATUS_DELETED,
        "whatsapp_status",
        &result.message_id,
        Some(&format!("whatsapp_status_feed:{}", request.account_id)),
        Some(&request.provider_status_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "tombstone_id": result.tombstone_id,
            "provider_status_id": request.provider_status_id,
            "actor_class": request.actor_class,
            "reason_class": request.reason_class,
            "observed_at": request.observed_at,
            "status_state": "deleted",
            "source": "fixture_status_delete_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_fixture_presence(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebPresence>,
) -> Result<Json<WhatsappWebPresenceIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_presence(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::PRESENCE_CHANGED,
        "whatsapp_identity",
        result
            .identity_id
            .as_deref()
            .unwrap_or(request.provider_identity_id.as_str()),
        Some(&request.provider_chat_id),
        None,
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "identity_id": result.identity_id,
            "raw_record_id": result.raw_record_id,
            "provider_chat_id": request.provider_chat_id,
            "provider_identity_id": request.provider_identity_id,
            "identity_kind": request.identity_kind,
            "display_name": request.display_name,
            "presence_state": request.presence_state,
            "last_seen_at": request.last_seen_at,
            "observed_at": request.observed_at,
            "source": "fixture_presence_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_fixture_call(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebCall>,
) -> Result<Json<WhatsappWebCallIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_call(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::CALL_UPDATED,
        "whatsapp_call",
        &result.call_id,
        Some(&request.provider_chat_id),
        Some(&request.provider_call_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "call_id": result.call_id,
            "raw_record_id": result.raw_record_id,
            "provider_call_id": request.provider_call_id,
            "provider_chat_id": request.provider_chat_id,
            "direction": request.direction,
            "call_state": request.call_state,
            "started_at": request.started_at,
            "ended_at": request.ended_at,
            "source": "fixture_call_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_fixture_runtime_event(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebRuntimeEvent>,
) -> Result<Json<WhatsappWebRuntimeEventIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_runtime_event(&request)
        .await?;
    let mut metadata_keys = request
        .metadata
        .as_object()
        .map(|metadata| metadata.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    metadata_keys.sort();
    publish_whatsapp_runtime_event(
        &state,
        &request.account_id,
        &request.provider_event_id,
        &request.runtime_event_kind,
        request.effective_runtime_status(),
        request.effective_lifecycle_state(),
        request.effective_severity(),
        metadata_keys,
        request.observed_at,
    )
    .await?;
    project_runtime_bridge_lifecycle_state(&state, &request).await?;
    Ok(Json(result))
}

async fn project_runtime_bridge_lifecycle_state(
    state: &AppState,
    request: &NewWhatsappWebRuntimeEvent,
) -> Result<(), ApiError> {
    let Some(lifecycle_state) = request.effective_lifecycle_state() else {
        return Ok(());
    };
    if !matches!(
        lifecycle_state,
        "linked"
            | "available"
            | "syncing"
            | "degraded"
            | "blocked"
            | "revoked"
            | "removed"
            | "qr_pending"
            | "pair_code_pending"
            | "created"
    ) {
        return Ok(());
    }
    communication_provider_account_store(state)?
        .update_whatsapp_lifecycle_state(&request.account_id, lifecycle_state)
        .await
        .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?;
    Ok(())
}

pub(crate) async fn post_whatsapp_fixture_dialog(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebDialog>,
) -> Result<Json<WhatsappWebDialogIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_dialog(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::DIALOG_UPDATED,
        "whatsapp_dialog",
        &result.conversation_id,
        Some(&request.provider_chat_id),
        None,
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "conversation_id": result.conversation_id,
            "channel_id": result.channel_id,
            "raw_record_id": result.raw_record_id,
            "provider_chat_id": request.provider_chat_id,
            "chat_title": request.chat_title,
            "chat_kind": request.chat_kind,
            "is_pinned": request.is_pinned,
            "is_archived": request.is_archived,
            "is_muted": request.is_muted,
            "is_unread": request.is_unread,
            "unread_count": request.unread_count,
            "participant_count": request.participant_count,
            "community_parent_chat_id": request.community_parent_chat_id,
            "community_parent_title": request.community_parent_title,
            "invite_link": request.invite_link,
            "is_community_root": request.is_community_root,
            "is_broadcast": request.is_broadcast,
            "is_newsletter": request.is_newsletter,
            "avatar_metadata": request.avatar_metadata,
            "provider_labels": request.provider_labels,
            "observed_at": request.observed_at,
            "source": "fixture_dialog_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_fixture_participant(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebParticipant>,
) -> Result<Json<WhatsappWebParticipantIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_participant(&request)
        .await?;
    let provider_member_id = request.effective_provider_member_id();
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::PARTICIPANT_CHANGED,
        "whatsapp_participant",
        &result.participant_id,
        Some(&request.provider_chat_id),
        Some(provider_member_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "conversation_id": result.conversation_id,
            "participant_id": result.participant_id,
            "identity_id": result.identity_id,
            "raw_record_id": result.raw_record_id,
            "provider_chat_id": request.provider_chat_id,
            "provider_member_id": provider_member_id,
            "provider_identity_id": request.provider_identity_id,
            "display_name": request.display_name,
            "role": result.current_role,
            "status": result.current_status,
            "previous_role": result.previous_role,
            "previous_status": result.previous_status,
            "role_changed": result.role_changed,
            "membership_changed": result.membership_changed,
            "observed_at": request.observed_at,
            "source": "fixture_participant_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_fixture_authorized_session(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppAuthorizedSessionCredentialWrite>,
) -> Result<Json<WhatsAppCredentialBinding>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let provider_account = provider_account_or_not_found(&state, &request.account_id).await?;
    let lifecycle_source = authorized_session_lifecycle_source(&state, &request.account_id).await?;
    let binding = whatsapp_provider_runtime_service(&state)?
        .store_authorized_session_credential(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request,
        )
        .await?;
    let status = whatsapp_provider_runtime_service(&state)?
        .runtime_status(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request.account_id,
        )
        .await?;
    sync_whatsapp_runtime_signal_connection(&state, &provider_account, &status).await?;
    capture_whatsapp_runtime_lifecycle_signal(&state, &status, lifecycle_source).await?;
    publish_whatsapp_runtime_status_event(&state, &status, lifecycle_source).await?;
    publish_whatsapp_session_link_state_event(
        &state,
        &status.account_id,
        &status.provider_shape,
        &status.runtime_kind,
        &status.status,
        lifecycle_source,
        status.updated_at,
    )
    .await?;
    Ok(Json(binding))
}

pub(crate) async fn post_whatsapp_runtime_bridge_message(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebMessage>,
) -> Result<Json<WhatsappWebMessageIngestResult>, ApiError> {
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_runtime_bridge_message(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::MESSAGE_CREATED,
        "whatsapp_message",
        &result.message_id,
        Some(&request.provider_chat_id),
        Some(&request.provider_message_id),
        request.occurred_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "provider_chat_id": request.provider_chat_id,
            "provider_message_id": request.provider_message_id,
            "delivery_state": request.delivery_state.as_str(),
            "sender_id": request.sender_id,
            "sender_display_name": request.sender_display_name,
            "occurred_at": request.occurred_at,
            "source": "runtime_bridge_message_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_runtime_bridge_reaction(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebReaction>,
) -> Result<Json<WhatsappWebReactionIngestResult>, ApiError> {
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_runtime_bridge_reaction(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::REACTION_CHANGED,
        "whatsapp_message",
        &result.message_id,
        Some(&request.provider_chat_id),
        Some(&request.provider_message_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "reaction_id": result.reaction_id,
            "provider_chat_id": request.provider_chat_id,
            "provider_message_id": request.provider_message_id,
            "provider_actor_id": request.provider_actor_id,
            "sender_display_name": request.sender_display_name,
            "reaction": request.reaction,
            "is_active": request.is_active,
            "observed_at": request.observed_at,
            "source": "runtime_bridge_reaction_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_runtime_bridge_message_update(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebMessageUpdate>,
) -> Result<Json<WhatsappWebMessageUpdateIngestResult>, ApiError> {
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_runtime_bridge_message_update(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::MESSAGE_UPDATED,
        "whatsapp_message",
        &result.message_id,
        Some(&request.provider_chat_id),
        Some(&request.provider_message_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "version_id": result.version_id,
            "provider_chat_id": request.provider_chat_id,
            "provider_message_id": request.provider_message_id,
            "observed_at": request.observed_at,
            "source": "runtime_bridge_message_update_ingest",
            "edited": true,
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_runtime_bridge_message_delete(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebMessageDelete>,
) -> Result<Json<WhatsappWebMessageDeleteIngestResult>, ApiError> {
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_runtime_bridge_message_delete(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::MESSAGE_DELETED,
        "whatsapp_message",
        &result.message_id,
        Some(&request.provider_chat_id),
        Some(&request.provider_message_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "tombstone_id": result.tombstone_id,
            "provider_chat_id": request.provider_chat_id,
            "provider_message_id": request.provider_message_id,
            "reason_class": request.reason_class,
            "actor_class": request.actor_class,
            "observed_at": request.observed_at,
            "source": "runtime_bridge_message_delete_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_runtime_bridge_receipt(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebReceipt>,
) -> Result<Json<WhatsappWebReceiptIngestResult>, ApiError> {
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_runtime_bridge_receipt(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::RECEIPT_CHANGED,
        "whatsapp_message",
        &result.message_id,
        Some(&request.provider_chat_id),
        Some(&request.provider_message_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "provider_chat_id": request.provider_chat_id,
            "provider_message_id": request.provider_message_id,
            "delivery_state": result.delivery_state,
            "observed_at": request.observed_at,
            "source": "runtime_bridge_receipt_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_runtime_bridge_media(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebMedia>,
) -> Result<Json<WhatsappWebMediaIngestResult>, ApiError> {
    Ok(Json(
        whatsapp_fixture_ingest_service(&state)?
            .ingest_runtime_bridge_media(&request)
            .await?,
    ))
}

pub(crate) async fn post_whatsapp_runtime_bridge_media_lifecycle(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppRuntimeBridgeMediaLifecycleRequest>,
) -> Result<StatusCode, ApiError> {
    let event_type = whatsapp_runtime_bridge_media_event_type(
        &request.media_direction,
        &request.lifecycle_phase,
    )?;
    let progress_percent = match request.lifecycle_phase.as_str() {
        "requested" => None,
        "failed" => request.progress_percent,
        "started" => Some(request.progress_percent.unwrap_or(0)),
        "progress" | "completed" => Some(request.progress_percent.unwrap_or(100)),
        _ => None,
    };
    let payload = json!({
        "account_id": request.account_id,
        "command_id": request.command_id,
        "provider_chat_id": request.provider_chat_id,
        "provider_message_id": request.provider_message_id,
        "provider_media_id": request.provider_media_id,
        "attachment_id": request.attachment_id,
        "blob_id": request.blob_id,
        "content_type": request.content_type,
        "filename": request.filename,
        "progress_percent": progress_percent,
        "error_code": request.error_code,
        "error_message": request.error_message,
        "runtime_blockers": request.runtime_blockers.unwrap_or_default(),
        "source": "runtime_bridge_media_lifecycle",
    });
    publish_whatsapp_media_event(&state, event_type, &request.command_id, payload).await?;
    Ok(StatusCode::ACCEPTED)
}

pub(crate) async fn post_whatsapp_runtime_bridge_sync_lifecycle(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppRuntimeBridgeSyncLifecycleRequest>,
) -> Result<StatusCode, ApiError> {
    let scope = match request.scope.as_str() {
        "chats" | "history" | "members" | "statuses" | "presence" | "calls" | "contacts"
        | "media" => request.scope.as_str(),
        _ => {
            return Err(WhatsappWebError::InvalidRequest(format!(
                "unsupported runtime bridge sync scope `{}`",
                request.scope
            ))
            .into());
        }
    };
    let phase = match request.phase.as_str() {
        "started" | "progress" | "completed" | "failed" => request.phase.as_str(),
        _ => {
            return Err(WhatsappWebError::InvalidRequest(format!(
                "unsupported runtime bridge sync phase `{}`",
                request.phase
            ))
            .into());
        }
    };
    let subject_id = request
        .subject_id
        .clone()
        .or_else(|| request.provider_chat_id.clone())
        .unwrap_or_else(|| request.account_id.clone());
    let payload = json!({
        "scope": scope,
        "status": phase,
        "subject_id": subject_id,
        "provider_chat_id": request.provider_chat_id,
        "synced_count": request.synced_count,
        "has_more": request.has_more,
        "error_code": request.error_code,
        "error_message": request.error_message,
        "source": "runtime_bridge_sync_lifecycle",
    });
    capture_whatsapp_sync_runtime_signal(
        &state,
        &request.account_id,
        &subject_id,
        scope,
        phase,
        payload.clone(),
    )
    .await?;
    let event_type = match phase {
        "started" => whatsapp_event_types::SYNC_STARTED,
        "progress" => whatsapp_event_types::SYNC_PROGRESS,
        "completed" => whatsapp_event_types::SYNC_COMPLETED,
        "failed" => whatsapp_event_types::SYNC_FAILED,
        _ => unreachable!(),
    };
    publish_whatsapp_sync_event(
        &state,
        event_type,
        &request.account_id,
        &subject_id,
        payload,
    )
    .await?;
    Ok(StatusCode::ACCEPTED)
}

pub(crate) async fn post_whatsapp_runtime_bridge_claim_commands(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppRuntimeBridgeClaimCommandsRequest>,
) -> Result<Json<WhatsAppRuntimeBridgeClaimCommandsResponse>, ApiError> {
    let limit = request.limit.unwrap_or(20).clamp(1, 100);
    let account_id = request
        .account_id
        .as_deref()
        .map(|value| required_string("account_id", value))
        .transpose()?;
    let pool = state
        .database
        .pool()
        .expect("database pool configured")
        .clone();
    let now = Utc::now();
    let imported = crate::integrations::whatsapp::runtime::import_canonical_provider_commands(
        &pool, now, limit,
    )
    .await?;
    for command in &imported {
        publish_whatsapp_command_record_event(
            &state,
            &command.clone().into(),
            "canonical_provider_command_import",
        )
        .await?;
    }
    let recovered = crate::integrations::whatsapp::runtime::recover_stale_live_executing_commands(
        &pool,
        now,
        account_id.as_deref(),
    )
    .await?;
    for command in &recovered {
        publish_whatsapp_command_record_event(&state, &command.clone().into(), "stale_recovery")
            .await?;
    }
    let claimed = crate::integrations::whatsapp::runtime::claim_due_live_commands_for_execution(
        &pool,
        now,
        limit,
        account_id.as_deref(),
    )
    .await?;
    let mut items = Vec::with_capacity(claimed.len());
    for command in claimed {
        let account = provider_account_or_not_found(&state, &command.account_id).await?;
        let runtime_status = whatsapp_provider_runtime_service(&state)?
            .runtime_status(
                &whatsapp_secret_reference_store(&state)?,
                &state.vault,
                &command.account_id,
            )
            .await?;
        publish_whatsapp_command_record_event(
            &state,
            &command.clone().into(),
            "runtime_bridge_claim",
        )
        .await?;
        items.push(WhatsAppRuntimeBridgeExecutableCommand {
            command_id: command.command_id,
            account_id: command.account_id,
            command_kind: command.command_kind,
            provider_kind: account.provider_kind.as_str().to_owned(),
            provider_shape: runtime_status.provider_shape,
            runtime_kind: runtime_status.runtime_kind,
            lifecycle_state: account
                .config
                .get("lifecycle_state")
                .and_then(Value::as_str)
                .map(str::to_owned),
            session_restore_available: runtime_status.session_restore_available,
            runtime_blockers: runtime_status.runtime_blockers,
            provider_chat_id: command.provider_chat_id,
            provider_message_id: command.provider_message_id,
            idempotency_key: command.idempotency_key,
            capability_state: command.capability_state,
            action_class: command.action_class,
            confirmation_decision: command.confirmation_decision,
            status: command.status,
            retry_count: command.retry_count,
            max_retries: command.max_retries,
            payload: command.payload,
            target_ref: command.target_ref,
            audit_metadata: command.audit_metadata,
            provider_state: command.provider_state,
            result_payload: command.result_payload,
            next_attempt_at: command.next_attempt_at,
            last_attempt_at: command.last_attempt_at,
            created_at: command.created_at,
            updated_at: command.updated_at,
        });
    }
    Ok(Json(WhatsAppRuntimeBridgeClaimCommandsResponse { items }))
}

pub(crate) async fn post_whatsapp_runtime_bridge_command_failed(
    State(state): State<AppState>,
    Path(command_id): Path<String>,
    Json(request): Json<WhatsAppRuntimeBridgeCommandFailedRequest>,
) -> Result<Json<WhatsAppProviderCommand>, ApiError> {
    let pool = state
        .database
        .pool()
        .expect("database pool configured")
        .clone();
    let updated = crate::integrations::whatsapp::runtime::reschedule_failed_command(
        &pool,
        &command_id,
        Utc::now(),
        &required_string("error_message", &request.error_message)?,
        request
            .error_code
            .as_deref()
            .map(|value| required_string("error_code", value))
            .transpose()?
            .as_deref(),
        request.retry_after_seconds,
    )
    .await?
    .ok_or(ApiError::NotFound)?;
    let command: WhatsAppProviderCommand = updated.into();
    publish_whatsapp_command_record_event(&state, &command, "runtime_bridge_failed").await?;
    Ok(Json(command))
}

pub(crate) async fn post_whatsapp_runtime_bridge_status(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebStatus>,
) -> Result<Json<WhatsappWebStatusIngestResult>, ApiError> {
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_runtime_bridge_status(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::STATUS_UPDATED,
        "whatsapp_status",
        &result.message_id,
        Some(&format!("whatsapp_status_feed:{}", request.account_id)),
        Some(&request.provider_status_id),
        request.occurred_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "provider_status_id": request.provider_status_id,
            "sender_id": request.sender_id,
            "sender_display_name": request.sender_display_name,
            "sender_identity_kind": request.sender_identity_kind,
            "sender_address": request.sender_address,
            "sender_push_name": request.sender_push_name,
            "occurred_at": request.occurred_at,
            "status_state": "posted",
            "source": "runtime_bridge_status_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_runtime_bridge_status_view(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebStatusView>,
) -> Result<Json<WhatsappWebStatusViewIngestResult>, ApiError> {
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_runtime_bridge_status_view(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::STATUS_UPDATED,
        "whatsapp_status",
        &result.message_id,
        Some(&format!("whatsapp_status_feed:{}", request.account_id)),
        Some(&request.provider_status_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "provider_status_id": request.provider_status_id,
            "viewer_id": request.viewer_id,
            "viewer_display_name": request.viewer_display_name,
            "observed_at": request.observed_at,
            "status_state": "viewed",
            "source": "runtime_bridge_status_view_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_runtime_bridge_status_delete(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebStatusDelete>,
) -> Result<Json<WhatsappWebStatusDeleteIngestResult>, ApiError> {
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_runtime_bridge_status_delete(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::STATUS_DELETED,
        "whatsapp_status",
        &result.message_id,
        Some(&format!("whatsapp_status_feed:{}", request.account_id)),
        Some(&request.provider_status_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "tombstone_id": result.tombstone_id,
            "provider_status_id": request.provider_status_id,
            "actor_class": request.actor_class,
            "reason_class": request.reason_class,
            "observed_at": request.observed_at,
            "status_state": "deleted",
            "source": "runtime_bridge_status_delete_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_runtime_bridge_presence(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebPresence>,
) -> Result<Json<WhatsappWebPresenceIngestResult>, ApiError> {
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_runtime_bridge_presence(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::PRESENCE_CHANGED,
        "whatsapp_identity",
        result
            .identity_id
            .as_deref()
            .unwrap_or(request.provider_identity_id.as_str()),
        Some(&request.provider_chat_id),
        None,
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "identity_id": result.identity_id,
            "raw_record_id": result.raw_record_id,
            "provider_chat_id": request.provider_chat_id,
            "provider_identity_id": request.provider_identity_id,
            "identity_kind": request.identity_kind,
            "display_name": request.display_name,
            "presence_state": request.presence_state,
            "last_seen_at": request.last_seen_at,
            "observed_at": request.observed_at,
            "source": "runtime_bridge_presence_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_runtime_bridge_call(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebCall>,
) -> Result<Json<WhatsappWebCallIngestResult>, ApiError> {
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_runtime_bridge_call(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::CALL_UPDATED,
        "whatsapp_call",
        &result.call_id,
        Some(&request.provider_chat_id),
        Some(&request.provider_call_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "call_id": result.call_id,
            "raw_record_id": result.raw_record_id,
            "provider_call_id": request.provider_call_id,
            "provider_chat_id": request.provider_chat_id,
            "direction": request.direction,
            "call_state": request.call_state,
            "started_at": request.started_at,
            "ended_at": request.ended_at,
            "source": "runtime_bridge_call_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_runtime_bridge_runtime_event(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebRuntimeEvent>,
) -> Result<Json<WhatsappWebRuntimeEventIngestResult>, ApiError> {
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_runtime_bridge_runtime_event(&request)
        .await?;
    let mut metadata_keys = request
        .metadata
        .as_object()
        .map(|metadata| metadata.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    metadata_keys.sort();
    publish_whatsapp_runtime_event(
        &state,
        &request.account_id,
        &request.provider_event_id,
        &request.runtime_event_kind,
        request.effective_runtime_status(),
        request.effective_lifecycle_state(),
        request.effective_severity(),
        metadata_keys,
        request.observed_at,
    )
    .await?;
    project_runtime_bridge_lifecycle_state(&state, &request).await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_runtime_bridge_dialog(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebDialog>,
) -> Result<Json<WhatsappWebDialogIngestResult>, ApiError> {
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_runtime_bridge_dialog(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::DIALOG_UPDATED,
        "whatsapp_dialog",
        &result.conversation_id,
        Some(&request.provider_chat_id),
        None,
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "conversation_id": result.conversation_id,
            "channel_id": result.channel_id,
            "raw_record_id": result.raw_record_id,
            "provider_chat_id": request.provider_chat_id,
            "chat_title": request.chat_title,
            "chat_kind": request.chat_kind,
            "is_pinned": request.is_pinned,
            "is_archived": request.is_archived,
            "is_muted": request.is_muted,
            "is_unread": request.is_unread,
            "unread_count": request.unread_count,
            "participant_count": request.participant_count,
            "community_parent_chat_id": request.community_parent_chat_id,
            "community_parent_title": request.community_parent_title,
            "invite_link": request.invite_link,
            "is_community_root": request.is_community_root,
            "is_broadcast": request.is_broadcast,
            "is_newsletter": request.is_newsletter,
            "avatar_metadata": request.avatar_metadata,
            "provider_labels": request.provider_labels,
            "observed_at": request.observed_at,
            "source": "runtime_bridge_dialog_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_runtime_bridge_participant(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebParticipant>,
) -> Result<Json<WhatsappWebParticipantIngestResult>, ApiError> {
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_runtime_bridge_participant(&request)
        .await?;
    let provider_member_id = request.effective_provider_member_id();
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::PARTICIPANT_CHANGED,
        "whatsapp_participant",
        &result.participant_id,
        Some(&request.provider_chat_id),
        Some(provider_member_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "conversation_id": result.conversation_id,
            "participant_id": result.participant_id,
            "identity_id": result.identity_id,
            "raw_record_id": result.raw_record_id,
            "provider_chat_id": request.provider_chat_id,
            "provider_member_id": provider_member_id,
            "provider_identity_id": request.provider_identity_id,
            "display_name": request.display_name,
            "role": result.current_role,
            "status": result.current_status,
            "previous_role": result.previous_role,
            "previous_status": result.previous_status,
            "role_changed": result.role_changed,
            "membership_changed": result.membership_changed,
            "observed_at": request.observed_at,
            "source": "runtime_bridge_participant_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_runtime_bridge_authorized_session(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppAuthorizedSessionCredentialWrite>,
) -> Result<Json<WhatsAppCredentialBinding>, ApiError> {
    let provider_account = provider_account_or_not_found(&state, &request.account_id).await?;
    let lifecycle_source = authorized_session_lifecycle_source(&state, &request.account_id).await?;
    let binding = whatsapp_provider_runtime_service(&state)?
        .store_authorized_session_credential(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request,
        )
        .await?;
    let status = whatsapp_provider_runtime_service(&state)?
        .runtime_status(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request.account_id,
        )
        .await?;
    sync_whatsapp_runtime_signal_connection(&state, &provider_account, &status).await?;
    capture_whatsapp_runtime_lifecycle_signal(&state, &status, lifecycle_source).await?;
    publish_whatsapp_runtime_status_event(&state, &status, lifecycle_source).await?;
    publish_whatsapp_session_link_state_event(
        &state,
        &status.account_id,
        &status.provider_shape,
        &status.runtime_kind,
        &status.status,
        lifecycle_source,
        status.updated_at,
    )
    .await?;
    Ok(Json(binding))
}

pub(crate) async fn publish_whatsapp_command_event(
    state: &AppState,
    response: &WhatsAppProviderCommandResponse,
) -> Result<(), ApiError> {
    let event = build_whatsapp_command_event(response);
    event_store(state)?.append(&event).await?;
    let _ = state.event_bus.broadcast(event);
    Ok(())
}

async fn publish_whatsapp_command_record_event(
    state: &AppState,
    command: &WhatsAppProviderCommand,
    source: &str,
) -> Result<(), ApiError> {
    let event = build_whatsapp_command_record_event(command, source);
    event_store(state)?.append(&event).await?;
    let _ = state.event_bus.broadcast(event);
    Ok(())
}

pub(crate) async fn publish_whatsapp_media_event(
    state: &AppState,
    event_type: &str,
    command_id: &str,
    payload: serde_json::Value,
) -> Result<(), ApiError> {
    let now = Utc::now();
    if let Some(account_id) = payload.get("account_id").and_then(Value::as_str) {
        let _ = whatsapp_fixture_ingest_service(state)?
            .capture_media_lifecycle_event(
                account_id,
                command_id,
                event_type,
                payload.clone(),
                &format!("media_{}", event_type.replace('.', "_")),
                now,
            )
            .await?;
    }
    let event = NewEventEnvelope::builder(
        whatsapp_event_id("media", command_id, now),
        event_type.to_owned(),
        now,
        json!({
            "channel": "whatsapp",
            "actor_id": AUDIT_ACTOR_ID,
            "kind": "whatsapp_provider_commands",
            "source_id": command_id,
        }),
        json!({
            "id": command_id,
            "entity_id": command_id,
            "kind": "whatsapp_media_command",
        }),
    )
    .payload(payload)
    .build()
    .expect("WhatsApp media event envelope must be valid");
    event_store(state)?.append(&event).await?;
    let _ = state.event_bus.broadcast(event);
    Ok(())
}

pub(crate) async fn publish_whatsapp_sync_event(
    state: &AppState,
    event_type: &str,
    account_id: &str,
    subject_id: &str,
    payload: serde_json::Value,
) -> Result<(), ApiError> {
    let now = Utc::now();
    let scope = payload
        .get("scope")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown");
    let source_id = format!("{subject_id}:{scope}");
    let event = NewEventEnvelope::builder(
        whatsapp_event_id("sync", subject_id, now),
        event_type.to_owned(),
        now,
        json!({
            "channel": "whatsapp",
            "account_id": account_id,
            "actor_id": AUDIT_ACTOR_ID,
            "kind": "whatsapp_sync_requests",
            "source_id": source_id,
        }),
        json!({
            "id": subject_id,
            "entity_id": subject_id,
            "kind": "whatsapp_sync",
        }),
    )
    .payload(payload)
    .build()
    .expect("WhatsApp sync event envelope must be valid");
    event_store(state)?.append(&event).await?;
    let _ = state.event_bus.broadcast(event);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn publish_whatsapp_projection_event(
    state: &AppState,
    event_type: &str,
    subject_kind: &str,
    subject_id: &str,
    provider_chat_id: Option<&str>,
    provider_message_id: Option<&str>,
    occurred_at: DateTime<Utc>,
    payload: serde_json::Value,
) -> Result<(), ApiError> {
    let source_id = payload
        .get("raw_record_id")
        .and_then(Value::as_str)
        .unwrap_or(subject_id);
    let source_kind = if payload
        .get("raw_record_id")
        .and_then(Value::as_str)
        .is_some()
    {
        "communication_raw_records"
    } else {
        "whatsapp_projection_events"
    };
    let event = NewEventEnvelope::builder(
        whatsapp_event_id("projection", subject_id, occurred_at),
        event_type.to_owned(),
        occurred_at,
        json!({
            "channel": "whatsapp",
            "actor_id": AUDIT_ACTOR_ID,
            "kind": source_kind,
            "source_id": source_id,
        }),
        json!({
            "id": subject_id,
            "entity_id": subject_id,
            "kind": subject_kind,
            "provider_chat_id": provider_chat_id,
            "provider_message_id": provider_message_id,
        }),
    )
    .payload(sanitize_event_payload(payload))
    .build()
    .expect("WhatsApp projection event envelope must be valid");
    event_store(state)?.append(&event).await?;
    let _ = state.event_bus.broadcast(event);
    Ok(())
}

pub(super) async fn publish_whatsapp_runtime_status_event(
    state: &AppState,
    status: &WhatsAppRuntimeStatus,
    source: &str,
) -> Result<(), ApiError> {
    let now = Utc::now();
    let source_id = format!(
        "{}:{}:{}:{}",
        status.account_id,
        source,
        status.status,
        status.updated_at.timestamp_micros()
    );
    let event = NewEventEnvelope::builder(
        whatsapp_event_id("runtime", &status.account_id, now),
        whatsapp_event_types::RUNTIME_STATUS_CHANGED.to_owned(),
        now,
        json!({
            "channel": "whatsapp",
            "account_id": status.account_id,
            "actor_id": AUDIT_ACTOR_ID,
            "kind": "whatsapp_runtime_status",
            "source_id": source_id,
        }),
        json!({
            "id": status.account_id,
            "entity_id": status.account_id,
            "kind": "whatsapp_runtime",
        }),
    )
    .payload(sanitize_event_payload(json!({
        "account_id": status.account_id,
        "provider_kind": status.provider_kind,
        "provider_shape": status.provider_shape,
        "runtime_kind": status.runtime_kind,
        "status": status.status,
        "fixture_runtime": status.fixture_runtime,
        "live_runtime_available": status.live_runtime_available,
        "live_send_available": status.live_send_available,
        "qr_pairing_available": status.qr_pairing_available,
        "pair_code_available": status.pair_code_available,
        "media_download_available": status.media_download_available,
        "media_upload_available": status.media_upload_available,
        "session_restore_available": status.session_restore_available,
        "runtime_blockers": status.runtime_blockers,
        "last_error": status.last_error,
        "source": source,
    })))
    .build()
    .expect("WhatsApp runtime status event envelope must be valid");
    event_store(state)?.append(&event).await?;
    let _ = state.event_bus.broadcast(event);
    Ok(())
}

pub(super) async fn publish_whatsapp_session_link_state_event(
    state: &AppState,
    account_id: &str,
    provider_shape: &str,
    runtime_kind: &str,
    link_state: &str,
    source: &str,
    observed_at: chrono::DateTime<chrono::Utc>,
) -> Result<(), ApiError> {
    let now = Utc::now();
    let source_id = format!(
        "{}:{}:{}:{}",
        account_id,
        source,
        link_state,
        observed_at.timestamp_micros()
    );
    let event = NewEventEnvelope::builder(
        whatsapp_event_id("session", account_id, now),
        whatsapp_event_types::SESSION_LINK_STATE_CHANGED.to_owned(),
        now,
        json!({
            "channel": "whatsapp",
            "account_id": account_id,
            "actor_id": AUDIT_ACTOR_ID,
            "kind": "whatsapp_session_link_state",
            "source_id": source_id,
        }),
        json!({
            "id": account_id,
            "entity_id": account_id,
            "kind": "whatsapp_session",
        }),
    )
    .payload(sanitize_event_payload(json!({
        "account_id": account_id,
        "provider_shape": provider_shape,
        "runtime_kind": runtime_kind,
        "link_state": link_state,
        "source": source,
    })))
    .build()
    .expect("WhatsApp session lifecycle event envelope must be valid");
    event_store(state)?.append(&event).await?;
    let _ = state.event_bus.broadcast(event);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn publish_whatsapp_runtime_event(
    state: &AppState,
    account_id: &str,
    provider_event_id: &str,
    runtime_event_kind: &str,
    runtime_status: Option<&str>,
    lifecycle_state: Option<&str>,
    severity: Option<&str>,
    metadata_keys: Vec<String>,
    observed_at: chrono::DateTime<chrono::Utc>,
) -> Result<(), ApiError> {
    let now = Utc::now();
    let source_id = format!(
        "{}:{}:{}:{}",
        account_id,
        provider_event_id,
        runtime_event_kind,
        observed_at.timestamp_micros()
    );
    let event = NewEventEnvelope::builder(
        whatsapp_event_id("runtime_event", provider_event_id, now),
        whatsapp_event_types::RUNTIME_EVENT.to_owned(),
        now,
        json!({
            "channel": "whatsapp",
            "account_id": account_id,
            "actor_id": AUDIT_ACTOR_ID,
            "kind": "whatsapp_runtime_events",
            "source_id": source_id,
        }),
        json!({
            "id": provider_event_id,
            "entity_id": account_id,
            "kind": "whatsapp_runtime_event",
        }),
    )
    .payload(sanitize_event_payload(json!({
        "account_id": account_id,
        "provider_event_id": provider_event_id,
        "runtime_event_kind": runtime_event_kind,
        "runtime_status": runtime_status,
        "lifecycle_state": lifecycle_state,
        "severity": severity,
        "metadata_keys": metadata_keys,
        "observed_at": observed_at,
    })))
    .build()
    .expect("WhatsApp runtime event envelope must be valid");
    event_store(state)?.append(&event).await?;
    let _ = state.event_bus.broadcast(event);
    Ok(())
}

pub(super) async fn capture_whatsapp_runtime_lifecycle_signal(
    state: &AppState,
    status: &WhatsAppRuntimeStatus,
    source: &str,
) -> Result<(), ApiError> {
    let provider_event_id = format!(
        "{}:{}:{}",
        status.account_id,
        source,
        status.updated_at.timestamp_micros()
    );
    let metadata = json!({
        "source": source,
        "provider_kind": status.provider_kind,
        "provider_shape": status.provider_shape,
        "runtime_kind": status.runtime_kind,
        "fixture_runtime": status.fixture_runtime,
        "live_runtime_available": status.live_runtime_available,
        "live_send_available": status.live_send_available,
        "qr_pairing_available": status.qr_pairing_available,
        "pair_code_available": status.pair_code_available,
        "media_download_available": status.media_download_available,
        "media_upload_available": status.media_upload_available,
        "session_restore_available": status.session_restore_available,
        "runtime_blockers": status.runtime_blockers,
        "last_error": status.last_error,
    });
    let _ = whatsapp_fixture_ingest_service(state)?
        .capture_runtime_lifecycle_event(
            &status.account_id,
            &provider_event_id,
            source,
            Some(&status.status),
            Some(&status.status),
            Some(
                if status.status == "available" || status.status == "linked" {
                    "info"
                } else if status.status == "degraded" {
                    "warning"
                } else {
                    "blocked"
                },
            ),
            metadata,
            source,
            status.updated_at,
        )
        .await?;
    Ok(())
}

pub(crate) async fn capture_whatsapp_sync_runtime_signal(
    state: &AppState,
    account_id: &str,
    subject_id: &str,
    scope: &str,
    phase: &str,
    metadata: Value,
) -> Result<(), ApiError> {
    let observed_at = Utc::now();
    let provider_event_id = format!(
        "{}:{}:{}:{}",
        account_id,
        scope,
        phase,
        observed_at.timestamp_micros()
    );
    let runtime_status = match phase {
        "started" | "progress" => Some("syncing"),
        "completed" => Some("synced"),
        "failed" => Some("failed"),
        _ => None,
    };
    let severity = match phase {
        "failed" => Some("warning"),
        _ => Some("info"),
    };
    let _ = whatsapp_fixture_ingest_service(state)?
        .capture_runtime_lifecycle_event(
            account_id,
            &provider_event_id,
            &format!("sync.{scope}.{phase}"),
            runtime_status,
            runtime_status,
            severity,
            merged_whatsapp_runtime_event_metadata(
                metadata,
                json!({
                    "subject_id": subject_id,
                    "phase": phase,
                }),
            ),
            &format!("sync_{scope}_{phase}"),
            observed_at,
        )
        .await?;
    Ok(())
}

pub(crate) async fn capture_whatsapp_status_publish_runtime_signal(
    state: &AppState,
    account_id: &str,
    command_id: &str,
    phase: &str,
    metadata: Value,
) -> Result<(), ApiError> {
    let observed_at = Utc::now();
    let provider_event_id = format!(
        "{}:status.publish:{}:{}",
        command_id,
        phase,
        observed_at.timestamp_micros()
    );
    let runtime_status = match phase {
        "failed" => Some("degraded"),
        _ => None,
    };
    let severity = match phase {
        "failed" => Some("warning"),
        _ => Some("info"),
    };
    let _ = whatsapp_fixture_ingest_service(state)?
        .capture_runtime_lifecycle_event(
            account_id,
            &provider_event_id,
            &format!("status.publish.{phase}"),
            runtime_status,
            Some(phase),
            severity,
            metadata,
            &format!("status_publish_{phase}"),
            observed_at,
        )
        .await?;
    Ok(())
}

fn merged_whatsapp_runtime_event_metadata(current: Value, patch: Value) -> Value {
    let mut current_map = current.as_object().cloned().unwrap_or_default();
    if let Some(patch_map) = patch.as_object() {
        current_map.extend(patch_map.clone());
    }
    Value::Object(current_map)
}

async fn authorized_session_lifecycle_source(
    state: &AppState,
    account_id: &str,
) -> Result<&'static str, ApiError> {
    let status = whatsapp_provider_runtime_service(state)?
        .runtime_status(
            &whatsapp_secret_reference_store(state)?,
            &state.vault,
            account_id,
        )
        .await?;
    Ok(if status.session_restore_available {
        "session_rotated"
    } else {
        "session_authorized"
    })
}

fn build_whatsapp_command_event(response: &WhatsAppProviderCommandResponse) -> NewEventEnvelope {
    let now = Utc::now();
    let source_id = format!(
        "{}:{}:{}:{}",
        response.command_id,
        response.command_kind,
        response.status,
        response.updated_at.timestamp_micros()
    );
    NewEventEnvelope::builder(
        whatsapp_event_id("command_response", &response.command_id, now),
        whatsapp_event_types::COMMAND_STATUS_CHANGED.to_owned(),
        now,
        json!({
            "channel": "whatsapp",
            "account_id": response.account_id,
            "actor_id": AUDIT_ACTOR_ID,
            "kind": "whatsapp_provider_commands",
            "source_id": source_id,
        }),
        json!({
            "id": response.command_id,
            "entity_id": response.command_id,
            "kind": "whatsapp_provider_command",
        }),
    )
    .payload(json!({
        "account_id": response.account_id,
        "command_id": response.command_id,
        "idempotency_key": response.idempotency_key,
        "command_kind": response.command_kind,
        "action": response.command_kind,
        "provider_chat_id": response.provider_chat_id,
        "provider_message_id": response.provider_message_id,
        "status": response.status,
        "durable_status": response.durable_status,
        "delivery_state": response.delivery_state,
        "runtime_kind": response.runtime_kind,
        "provider_shape": response.provider_shape,
        "session_restore_available": response.session_restore_available,
        "runtime_blockers": response.runtime_blockers,
        "rendered_preview_hash": response.rendered_preview_hash,
    }))
    .build()
    .expect("WhatsApp command event envelope must be valid")
}

fn build_whatsapp_command_record_event(
    command: &WhatsAppProviderCommand,
    source: &str,
) -> NewEventEnvelope {
    let now = Utc::now();
    let source_id = format!(
        "{}:{}:{}:{}:{}",
        command.command_id,
        command.command_kind,
        command.status,
        source,
        command.updated_at.timestamp_micros()
    );
    NewEventEnvelope::builder(
        whatsapp_event_id("command_record", &command.command_id, now),
        whatsapp_event_types::COMMAND_STATUS_CHANGED.to_owned(),
        now,
        json!({
            "channel": "whatsapp",
            "account_id": command.account_id,
            "actor_id": AUDIT_ACTOR_ID,
            "kind": "whatsapp_provider_commands",
            "source_id": source_id,
        }),
        json!({
            "id": command.command_id,
            "entity_id": command.command_id,
            "kind": "whatsapp_provider_command",
        }),
    )
    .payload(json!({
        "account_id": command.account_id,
        "command_id": command.command_id,
        "idempotency_key": command.idempotency_key,
        "command_kind": command.command_kind,
        "action": command.command_kind,
        "provider_chat_id": command.provider_chat_id,
        "provider_message_id": command.provider_message_id,
        "capability_state": command.capability_state,
        "action_class": command.action_class,
        "confirmation_decision": command.confirmation_decision,
        "status": command.status,
        "retry_count": command.retry_count,
        "max_retries": command.max_retries,
        "last_error": command.last_error,
        "result_payload": command.result_payload,
        "audit_metadata": command.audit_metadata,
        "provider_state": command.provider_state,
        "reconciliation_status": command.reconciliation_status,
        "next_attempt_at": command.next_attempt_at,
        "last_attempt_at": command.last_attempt_at,
        "provider_observed_at": command.provider_observed_at,
        "reconciled_at": command.reconciled_at,
        "dead_lettered_at": command.dead_lettered_at,
        "completed_at": command.completed_at,
        "source": source,
    }))
    .build()
    .expect("WhatsApp command record event envelope must be valid")
}

fn whatsapp_event_id(scope: &str, subject: &str, now: chrono::DateTime<chrono::Utc>) -> String {
    let seq = WHATSAPP_EVENT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!(
        "evt_whatsapp_{}_{}_{}_{}",
        scope,
        subject.replace(|c: char| !c.is_ascii_alphanumeric(), "_"),
        now.timestamp_nanos_opt().unwrap_or_default(),
        seq
    )
}

fn whatsapp_runtime_bridge_media_event_type(
    media_direction: &str,
    lifecycle_phase: &str,
) -> Result<&'static str, ApiError> {
    match (media_direction, lifecycle_phase) {
        ("upload", "requested") => Ok(whatsapp_event_types::MEDIA_UPLOAD_REQUESTED),
        ("upload", "started") => Ok("whatsapp.media.upload.started"),
        ("upload", "progress") => Ok("whatsapp.media.upload.progress"),
        ("upload", "completed") => Ok("whatsapp.media.upload.completed"),
        ("upload", "failed") => Ok(whatsapp_event_types::MEDIA_UPLOAD_FAILED),
        ("download", "requested") => Ok(whatsapp_event_types::MEDIA_DOWNLOAD_REQUESTED),
        ("download", "started") => Ok("whatsapp.media.download.started"),
        ("download", "progress") => Ok("whatsapp.media.download.progress"),
        ("download", "completed") => Ok("whatsapp.media.download.completed"),
        ("download", "failed") => Ok(whatsapp_event_types::MEDIA_DOWNLOAD_FAILED),
        _ => Err(WhatsappWebError::InvalidRequest(format!(
            "unsupported runtime bridge media lifecycle `{media_direction}.{lifecycle_phase}`"
        ))
        .into()),
    }
}

pub(crate) async fn current_whatsapp_runtime_kind(
    state: &AppState,
    account_id: &str,
) -> Result<String, ApiError> {
    let status = whatsapp_provider_runtime_service(state)?
        .runtime_status(
            &whatsapp_secret_reference_store(state)?,
            &state.vault,
            account_id,
        )
        .await?;
    Ok(status.runtime_kind)
}

pub(crate) async fn ensure_whatsapp_sync_supported(
    state: &AppState,
    account_id: &str,
    operation: &'static str,
) -> Result<(), ApiError> {
    let status = whatsapp_provider_runtime_service(state)?
        .runtime_status(
            &whatsapp_secret_reference_store(state)?,
            &state.vault,
            account_id,
        )
        .await?;
    let _ = operation;
    Ok(())
}

async fn list_whatsapp_sync_members(
    state: &AppState,
    account_id: &str,
    provider_chat_id: &str,
    limit: i64,
) -> Result<Vec<WhatsAppMembersSyncItem>, ApiError> {
    let pool = state
        .database
        .pool()
        .expect("database pool configured")
        .clone();
    let rows = sqlx::query(
        r#"
        SELECT
            participant.participant_id,
            conversation.conversation_id,
            conversation.account_id,
            conversation.provider_conversation_id,
            participant.display_name,
            participant.role,
            participant.address,
            participant.metadata AS participant_metadata,
            identity.provider_identity_id,
            identity.identity_kind,
            identity.metadata AS identity_metadata
        FROM communication_conversation_participants participant
        JOIN communication_conversations conversation
          ON conversation.conversation_id = participant.conversation_id
        LEFT JOIN communication_identities identity
          ON identity.identity_id = participant.identity_id
        WHERE conversation.account_id = $1
          AND conversation.provider_conversation_id = $2
          AND conversation.channel_kind = 'whatsapp_web'
        ORDER BY participant.created_at ASC, participant.participant_id ASC
        LIMIT $3
        "#,
    )
    .bind(account_id)
    .bind(provider_chat_id)
    .bind(limit)
    .fetch_all(&pool)
    .await
    .map_err(WhatsappWebError::from)?;

    rows.into_iter()
        .map(|row| {
            let participant_metadata: Value = row
                .try_get("participant_metadata")
                .map_err(WhatsappWebError::from)?;
            let identity_metadata: Option<Value> = row
                .try_get("identity_metadata")
                .map_err(WhatsappWebError::from)?;
            let provider_identity_id: Option<String> = row
                .try_get("provider_identity_id")
                .map_err(WhatsappWebError::from)?;
            let provider_member_id = provider_identity_id
                .clone()
                .unwrap_or_else(|| row.try_get("participant_id").unwrap_or_default());
            Ok(WhatsAppMembersSyncItem {
                participant_id: row
                    .try_get("participant_id")
                    .map_err(WhatsappWebError::from)?,
                conversation_id: row
                    .try_get("conversation_id")
                    .map_err(WhatsappWebError::from)?,
                account_id: row.try_get("account_id").map_err(WhatsappWebError::from)?,
                provider_chat_id: row
                    .try_get("provider_conversation_id")
                    .map_err(WhatsappWebError::from)?,
                provider_member_id,
                provider_identity_id,
                sender_display_name: row
                    .try_get("display_name")
                    .map_err(WhatsappWebError::from)?,
                role: row.try_get("role").map_err(WhatsappWebError::from)?,
                status: Some("active".to_owned()),
                identity_kind: row
                    .try_get("identity_kind")
                    .map_err(WhatsappWebError::from)?,
                address: row.try_get("address").map_err(WhatsappWebError::from)?,
                is_admin: participant_metadata
                    .get("is_admin")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                is_owner: participant_metadata
                    .get("is_owner")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                participant_metadata,
                identity_metadata: identity_metadata.unwrap_or_else(|| json!({})),
            })
        })
        .collect::<Result<Vec<_>, WhatsappWebError>>()
        .map_err(Into::into)
}

async fn list_whatsapp_sync_presence(
    state: &AppState,
    account_id: &str,
    provider_chat_id: Option<&str>,
    limit: i64,
) -> Result<Vec<WhatsAppPresenceSyncItem>, ApiError> {
    let pool = state
        .database
        .pool()
        .expect("database pool configured")
        .clone();
    let rows = sqlx::query(
        r#"
        SELECT
            identity.identity_id,
            identity.account_id,
            channel.channel_kind,
            identity.provider_identity_id,
            identity.identity_kind,
            identity.display_name,
            identity.address,
            identity.metadata
        FROM communication_identities identity
        JOIN communication_channels channel
          ON channel.channel_id = identity.channel_id
        WHERE identity.account_id = $1
          AND channel.channel_kind = 'whatsapp_web'
          AND identity.metadata ? 'presence_state'
          AND ($2::text IS NULL OR identity.metadata->>'presence_provider_chat_id' = $2)
        ORDER BY COALESCE(identity.metadata->>'presence_observed_at', '') DESC, identity.identity_id ASC
        LIMIT $3
        "#,
    )
    .bind(account_id)
    .bind(provider_chat_id)
    .bind(limit)
    .fetch_all(&pool)
    .await
    .map_err(WhatsappWebError::from)?;

    rows.into_iter()
        .map(|row| {
            let identity_metadata: Value =
                row.try_get("metadata").map_err(WhatsappWebError::from)?;
            Ok(WhatsAppPresenceSyncItem {
                identity_id: row.try_get("identity_id").map_err(WhatsappWebError::from)?,
                account_id: row.try_get("account_id").map_err(WhatsappWebError::from)?,
                channel_kind: row
                    .try_get("channel_kind")
                    .map_err(WhatsappWebError::from)?,
                provider_chat_id: identity_metadata
                    .get("presence_provider_chat_id")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                provider_identity_id: row
                    .try_get("provider_identity_id")
                    .map_err(WhatsappWebError::from)?,
                identity_kind: row
                    .try_get("identity_kind")
                    .map_err(WhatsappWebError::from)?,
                display_name: row
                    .try_get("display_name")
                    .map_err(WhatsappWebError::from)?,
                address: row.try_get("address").map_err(WhatsappWebError::from)?,
                presence_state: identity_metadata
                    .get("presence_state")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
                    .to_owned(),
                last_seen_at: identity_metadata
                    .get("last_seen_at")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                observed_at: identity_metadata
                    .get("presence_observed_at")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                identity_metadata,
            })
        })
        .collect::<Result<Vec<_>, WhatsappWebError>>()
        .map_err(Into::into)
}

async fn list_whatsapp_sync_calls(
    state: &AppState,
    account_id: &str,
    provider_chat_id: Option<&str>,
    limit: i64,
) -> Result<Vec<WhatsAppCallsSyncItem>, ApiError> {
    let pool = state
        .database
        .pool()
        .expect("database pool configured")
        .clone();
    let rows = sqlx::query(
        r#"
        SELECT
            call_id,
            account_id,
            provider_call_id,
            provider_chat_id,
            direction,
            call_state,
            started_at,
            ended_at,
            metadata
        FROM telegram_calls
        WHERE account_id = $1
          AND metadata->>'provider' = 'whatsapp_web'
          AND ($2::text IS NULL OR provider_chat_id = $2)
        ORDER BY COALESCE(started_at, created_at) DESC, call_id ASC
        LIMIT $3
        "#,
    )
    .bind(account_id)
    .bind(provider_chat_id)
    .bind(limit)
    .fetch_all(&pool)
    .await
    .map_err(WhatsappWebError::from)?;

    rows.into_iter()
        .map(|row| {
            let metadata: Value = row.try_get("metadata").map_err(WhatsappWebError::from)?;
            Ok(WhatsAppCallsSyncItem {
                call_id: row.try_get("call_id").map_err(WhatsappWebError::from)?,
                account_id: row.try_get("account_id").map_err(WhatsappWebError::from)?,
                provider_call_id: row
                    .try_get("provider_call_id")
                    .map_err(WhatsappWebError::from)?,
                provider_chat_id: row
                    .try_get("provider_chat_id")
                    .map_err(WhatsappWebError::from)?,
                direction: row.try_get("direction").map_err(WhatsappWebError::from)?,
                call_state: row.try_get("call_state").map_err(WhatsappWebError::from)?,
                started_at: row
                    .try_get::<Option<chrono::DateTime<Utc>>, _>("started_at")
                    .map_err(WhatsappWebError::from)?
                    .map(|value| value.to_rfc3339()),
                ended_at: row
                    .try_get::<Option<chrono::DateTime<Utc>>, _>("ended_at")
                    .map_err(WhatsappWebError::from)?
                    .map(|value| value.to_rfc3339()),
                observed_at: metadata
                    .get("observed_at")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                metadata,
            })
        })
        .collect::<Result<Vec<_>, WhatsappWebError>>()
        .map_err(Into::into)
}

async fn list_whatsapp_sync_contacts(
    state: &AppState,
    account_id: &str,
    limit: i64,
) -> Result<Vec<WhatsAppContactsSyncItem>, ApiError> {
    let pool = state
        .database
        .pool()
        .expect("database pool configured")
        .clone();
    let rows = sqlx::query(
        r#"
        SELECT
            identity.identity_id,
            identity.account_id,
            channel.channel_kind,
            identity.provider_identity_id,
            identity.identity_kind,
            identity.display_name,
            identity.address,
            identity.metadata AS identity_metadata,
            whatsapp_trace.metadata AS whatsapp_trace_metadata,
            phone_trace.metadata AS phone_trace_metadata
        FROM communication_identities identity
        JOIN communication_channels channel
          ON channel.channel_id = identity.channel_id
        LEFT JOIN persona_identities whatsapp_trace
          ON whatsapp_trace.source = 'communication_projection'
         AND whatsapp_trace.status = 'active'
         AND whatsapp_trace.identity_type = 'whatsapp'
         AND whatsapp_trace.identity_value = identity.provider_identity_id
        LEFT JOIN persona_identities phone_trace
          ON phone_trace.source = 'communication_projection'
         AND phone_trace.status = 'active'
         AND phone_trace.identity_type = 'phone'
         AND phone_trace.identity_value = identity.address
        WHERE identity.account_id = $1
          AND channel.channel_kind = 'whatsapp_web'
        ORDER BY identity.updated_at DESC, identity.identity_id ASC
        LIMIT $2
        "#,
    )
    .bind(account_id)
    .bind(limit)
    .fetch_all(&pool)
    .await
    .map_err(WhatsappWebError::from)?;

    rows.into_iter()
        .map(|row| {
            let identity_metadata: Value = row
                .try_get("identity_metadata")
                .map_err(WhatsappWebError::from)?;
            let display_name_history = identity_metadata
                .get("display_name_history")
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            Ok(WhatsAppContactsSyncItem {
                identity_id: row.try_get("identity_id").map_err(WhatsappWebError::from)?,
                account_id: row.try_get("account_id").map_err(WhatsappWebError::from)?,
                channel_kind: row
                    .try_get("channel_kind")
                    .map_err(WhatsappWebError::from)?,
                provider_identity_id: row
                    .try_get("provider_identity_id")
                    .map_err(WhatsappWebError::from)?,
                identity_kind: row
                    .try_get("identity_kind")
                    .map_err(WhatsappWebError::from)?,
                display_name: row
                    .try_get("display_name")
                    .map_err(WhatsappWebError::from)?,
                address: row.try_get("address").map_err(WhatsappWebError::from)?,
                push_name: identity_metadata
                    .get("push_name")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                business_profile: identity_metadata
                    .get("business_profile")
                    .cloned()
                    .unwrap_or_else(|| json!({})),
                profile_photo_ref: identity_metadata
                    .get("profile_photo_ref")
                    .cloned()
                    .unwrap_or_else(|| json!({})),
                display_name_history,
                identity_metadata,
                whatsapp_trace_metadata: row
                    .try_get::<Option<Value>, _>("whatsapp_trace_metadata")
                    .map_err(WhatsappWebError::from)?
                    .unwrap_or_else(|| json!({})),
                phone_trace_metadata: row
                    .try_get::<Option<Value>, _>("phone_trace_metadata")
                    .map_err(WhatsappWebError::from)?
                    .unwrap_or_else(|| json!({})),
            })
        })
        .collect::<Result<Vec<_>, WhatsappWebError>>()
        .map_err(Into::into)
}

async fn list_whatsapp_sync_media(
    state: &AppState,
    account_id: &str,
    provider_chat_id: Option<&str>,
    content_type: Option<&str>,
    limit: i64,
) -> Result<Vec<WhatsAppMediaSyncItem>, ApiError> {
    let pool = state
        .database
        .pool()
        .expect("database pool configured")
        .clone();
    let rows = sqlx::query(
        r#"
        SELECT
            a.attachment_id,
            a.message_id,
            a.raw_record_id,
            m.account_id,
            m.channel_kind,
            COALESCE(c.provider_conversation_id, m.conversation_id) AS provider_conversation_id,
            m.provider_record_id,
            a.provider_attachment_id,
            a.filename,
            a.content_type,
            a.size_bytes,
            a.sha256,
            a.scan_status,
            b.storage_kind,
            b.storage_path,
            m.subject,
            m.sender,
            m.sender_display_name,
            m.occurred_at,
            a.created_at
        FROM communication_attachments a
        JOIN communication_messages m ON m.message_id = a.message_id
        JOIN communication_mail_blobs b ON b.blob_id = a.blob_id
        LEFT JOIN communication_conversations c
          ON c.conversation_id = m.conversation_id
          OR c.provider_conversation_id = m.conversation_id
        WHERE m.account_id = $1
          AND m.local_state = 'active'
          AND m.channel_kind = 'whatsapp_web'
          AND ($2::text IS NULL OR COALESCE(c.provider_conversation_id, m.conversation_id) = $2)
          AND ($3::text IS NULL OR a.content_type ILIKE $3 || '%')
        ORDER BY COALESCE(m.occurred_at, m.projected_at) DESC, a.created_at DESC, a.attachment_id ASC
        LIMIT $4
        "#,
    )
    .bind(account_id)
    .bind(provider_chat_id)
    .bind(content_type)
    .bind(limit)
    .fetch_all(&pool)
    .await
    .map_err(WhatsappWebError::from)?;

    rows.into_iter()
        .map(|row| {
            let occurred_at = row
                .try_get::<Option<chrono::DateTime<Utc>>, _>("occurred_at")
                .map_err(WhatsappWebError::from)?
                .map(|value| value.to_rfc3339());
            let created_at = row
                .try_get::<chrono::DateTime<Utc>, _>("created_at")
                .map_err(WhatsappWebError::from)?
                .to_rfc3339();
            Ok(WhatsAppMediaSyncItem {
                attachment_id: row
                    .try_get("attachment_id")
                    .map_err(WhatsappWebError::from)?,
                message_id: row.try_get("message_id").map_err(WhatsappWebError::from)?,
                raw_record_id: row
                    .try_get("raw_record_id")
                    .map_err(WhatsappWebError::from)?,
                account_id: row.try_get("account_id").map_err(WhatsappWebError::from)?,
                channel_kind: row
                    .try_get("channel_kind")
                    .map_err(WhatsappWebError::from)?,
                provider_chat_id: row
                    .try_get("provider_conversation_id")
                    .map_err(WhatsappWebError::from)?,
                provider_message_id: row
                    .try_get("provider_record_id")
                    .map_err(WhatsappWebError::from)?,
                provider_attachment_id: row
                    .try_get("provider_attachment_id")
                    .map_err(WhatsappWebError::from)?,
                filename: row.try_get("filename").map_err(WhatsappWebError::from)?,
                content_type: row
                    .try_get("content_type")
                    .map_err(WhatsappWebError::from)?,
                size_bytes: row.try_get("size_bytes").map_err(WhatsappWebError::from)?,
                sha256: row.try_get("sha256").map_err(WhatsappWebError::from)?,
                scan_status: row.try_get("scan_status").map_err(WhatsappWebError::from)?,
                storage_kind: row
                    .try_get("storage_kind")
                    .map_err(WhatsappWebError::from)?,
                storage_path: row
                    .try_get("storage_path")
                    .map_err(WhatsappWebError::from)?,
                message_subject: row.try_get("subject").map_err(WhatsappWebError::from)?,
                sender: row.try_get("sender").map_err(WhatsappWebError::from)?,
                sender_display_name: row
                    .try_get("sender_display_name")
                    .map_err(WhatsappWebError::from)?,
                occurred_at,
                created_at,
            })
        })
        .collect::<Result<Vec<_>, WhatsappWebError>>()
        .map_err(Into::into)
}

pub(crate) fn required_string(field: &'static str, value: &str) -> Result<String, ApiError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(WhatsappWebError::InvalidRequest(format!("{field} must not be empty")).into());
    }
    Ok(value.to_owned())
}

pub(crate) fn optional_string(
    field: &'static str,
    value: Option<String>,
) -> Result<Option<String>, ApiError> {
    value
        .map(|value| required_string(field, &value))
        .transpose()
}

fn parse_command_kinds(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}
