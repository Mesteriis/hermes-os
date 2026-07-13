use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::Sha256;

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
    provider_account_or_not_found, sync_provider_account_signal_connection,
};
use crate::app::{ApiError, AppState};
use crate::integrations::zoom::client::errors::ZoomError;
use crate::integrations::zoom::client::models::{
    ZoomAccountListResponse, ZoomAccountSetupRequest, ZoomAccountSetupResponse,
    ZoomAuditEventResponse, ZoomAuthorizationResult, ZoomLiveAccountSetupRequest,
    ZoomMeetingIngestResult, ZoomMeetingObservationRequest, ZoomOAuthCompleteRequest,
    ZoomOAuthStartRequest, ZoomOAuthStartResponse, ZoomRecordingImportAuditResponse,
    ZoomRecordingImportRemoveRequest, ZoomRecordingImportRemoveResponse, ZoomRecordingIngestResult,
    ZoomRecordingMediaDownloadRequest, ZoomRecordingObservationRequest, ZoomRecordingRef,
    ZoomRecordingSyncRequest, ZoomRecordingSyncResult, ZoomRetentionCleanupRequest,
    ZoomRetentionCleanupResponse, ZoomRuntimeRemoveRequest, ZoomRuntimeRemoveResponse,
    ZoomRuntimeStartRequest, ZoomRuntimeStatus, ZoomRuntimeStopRequest,
    ZoomServerToServerAuthorizeRequest, ZoomTokenMaintenanceRequest, ZoomTokenMaintenanceResult,
    ZoomTokenRefreshRequest, ZoomTokenRefreshResult, ZoomTranscriptFileImportRequest,
    ZoomTranscriptFileImportResult, ZoomTranscriptIngestResult, ZoomTranscriptObservationRequest,
    ZoomWebhookSubscriptionReconcileRequest, ZoomWebhookSubscriptionReconcileResult,
    ZoomWebhookSubscriptionRemoveRequest, ZoomWebhookSubscriptionRemoveResult,
    ZoomWebhookSubscriptionStatusRequest, ZoomWebhookSubscriptionStatusResult,
};
use crate::vault::{HostVaultError, VaultMode};
use hermes_communications_api::accounts::ProviderAccountSecretPurpose;
use hermes_communications_postgres::provider_store::CommunicationProviderSecretBindingStore;

const ZOOM_SIGNATURE_HEADER: &str = "x-zm-signature";
const ZOOM_TIMESTAMP_HEADER: &str = "x-zm-request-timestamp";
const ZOOM_WEBHOOK_SIGNATURE_TOLERANCE_SECONDS: i64 = 300;
const ZOOM_REMOTE_TRANSCRIPT_DOWNLOAD_ENABLED_SETTING_KEY: &str =
    "privacy.zoom_remote_transcript_download_enabled";
const ZOOM_REMOTE_RECORDING_DOWNLOAD_ENABLED_SETTING_KEY: &str =
    "privacy.zoom_remote_recording_download_enabled";
const ZOOM_REMOTE_RECORDING_DOWNLOAD_NOT_ENABLED: &str =
    "zoom_remote_recording_download_not_enabled";
const ZOOM_REMOTE_TRANSCRIPT_DOWNLOAD_NOT_ENABLED: &str =
    "zoom_remote_transcript_download_not_enabled";
type HmacSha256 = Hmac<Sha256>;

#[derive(Deserialize)]
pub(crate) struct ZoomAccountsQuery {
    #[serde(default)]
    pub(crate) include_removed: bool,
}

#[derive(Deserialize)]
pub(crate) struct ZoomRuntimeStatusQuery {
    pub(crate) account_id: String,
}

#[derive(Deserialize)]
pub(crate) struct ZoomRecordingImportsQuery {
    #[serde(default = "default_zoom_recording_imports_limit")]
    pub(crate) limit: i64,
}

#[derive(Deserialize)]
pub(crate) struct ZoomAuditEventsQuery {
    #[serde(default = "default_zoom_audit_events_limit")]
    pub(crate) limit: i64,
}

#[derive(Deserialize)]
pub(crate) struct ZoomWebhookSubscriptionStatusQuery {
    pub(crate) account_id: String,
    pub(crate) api_base_url: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct ZoomWebhookQuery {
    pub(crate) account_id: String,
}

#[derive(Clone, Debug)]
struct ZoomWebhookTranscriptDownload {
    request: ZoomTranscriptFileImportRequest,
    download_url: String,
    download_token: Option<String>,
}

#[derive(Clone, Debug)]
struct ZoomWebhookRecordingMediaDownload {
    request: ZoomRecordingMediaDownloadRequest,
    download_token: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ZoomCapabilitiesResponse {
    pub(crate) version: &'static str,
    pub(crate) runtime_mode: &'static str,
    pub(crate) capabilities: Vec<ZoomCapabilityStatus>,
    pub(crate) planned_features: Vec<&'static str>,
    pub(crate) unsupported_features: Vec<&'static str>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ZoomCapabilityStatus {
    pub(crate) capability: &'static str,
    pub(crate) category: &'static str,
    pub(crate) status: &'static str,
    pub(crate) action_class: &'static str,
    pub(crate) confirmation_required: bool,
    pub(crate) reason: &'static str,
}

pub(crate) async fn get_zoom_capabilities() -> Result<Json<ZoomCapabilitiesResponse>, ApiError> {
    Ok(Json(ZoomCapabilitiesResponse {
        version: "1.0",
        runtime_mode: "fixture_plus_authorized_live_workers",
        capabilities: vec![
            ZoomCapabilityStatus {
                capability: "accounts.fixture",
                category: "accounts",
                status: "available",
                action_class: "local_write",
                confirmation_required: false,
                reason: "Fixture Zoom accounts can be registered for local validation.",
            },
            ZoomCapabilityStatus {
                capability: "accounts.live_blocked",
                category: "accounts",
                status: "degraded",
                action_class: "local_write",
                confirmation_required: true,
                reason: "Live Zoom account metadata and secret references can be registered, but provider execution is blocked.",
            },
            ZoomCapabilityStatus {
                capability: "auth.oauth_user",
                category: "authorization",
                status: "available",
                action_class: "external_provider_write",
                confirmation_required: true,
                reason: "Zoom OAuth user grants can be exchanged and stored through host-vault secret references.",
            },
            ZoomCapabilityStatus {
                capability: "auth.server_to_server",
                category: "authorization",
                status: "available",
                action_class: "external_provider_write",
                confirmation_required: true,
                reason: "Zoom Server-to-Server OAuth account credentials can be exchanged and stored through host-vault secret references.",
            },
            ZoomCapabilityStatus {
                capability: "auth.token_refresh",
                category: "authorization",
                status: "available",
                action_class: "external_provider_write",
                confirmation_required: true,
                reason: "Authorized Zoom OAuth and Server-to-Server credentials can be renewed through host-vault token bundle updates.",
            },
            ZoomCapabilityStatus {
                capability: "auth.token_maintenance",
                category: "authorization",
                status: "available",
                action_class: "external_provider_write",
                confirmation_required: true,
                reason: "Authorized Zoom accounts can be scanned and expiring token bundles renewed through the same host-vault refresh boundary.",
            },
            ZoomCapabilityStatus {
                capability: "auth.token_rotation_policy",
                category: "authorization",
                status: "available",
                action_class: "read",
                confirmation_required: false,
                reason: "Runtime status exposes the Zoom token rotation policy, refresh due state and failure blocker without exposing raw token material.",
            },
            ZoomCapabilityStatus {
                capability: "token_maintenance.scheduler",
                category: "runtime",
                status: "available",
                action_class: "local_automation",
                confirmation_required: false,
                reason: "The local backend scheduler can invoke token maintenance behind Signal Hub and HostVault gates.",
            },
            ZoomCapabilityStatus {
                capability: "runtime.status",
                category: "runtime",
                status: "available",
                action_class: "read",
                confirmation_required: false,
                reason: "Runtime/account lifecycle state is exposed without reading provider secrets.",
            },
            ZoomCapabilityStatus {
                capability: "bridge.meetings",
                category: "ingest",
                status: "available",
                action_class: "local_write",
                confirmation_required: false,
                reason: "Meeting observations are stored as provider call evidence and emitted as Zoom events.",
            },
            ZoomCapabilityStatus {
                capability: "bridge.recordings",
                category: "ingest",
                status: "available",
                action_class: "local_write",
                confirmation_required: false,
                reason: "Recording observations are event-sourced and sanitized before dispatch.",
            },
            ZoomCapabilityStatus {
                capability: "bridge.transcripts",
                category: "ingest",
                status: "available",
                action_class: "local_write",
                confirmation_required: false,
                reason: "Transcript observations are linked to provider call evidence for AI/consistency workflows.",
            },
            ZoomCapabilityStatus {
                capability: "bridge.transcript_files",
                category: "ingest",
                status: "available",
                action_class: "local_write",
                confirmation_required: false,
                reason: "Zoom transcript files can be imported from VTT, SRT or plain text into provider call transcript evidence.",
            },
            ZoomCapabilityStatus {
                capability: "provider_sync.recordings",
                category: "ingest",
                status: "available",
                action_class: "external_provider_read",
                confirmation_required: true,
                reason: "Authorized Zoom accounts can manually synchronize cloud recording metadata; provider-side recording media and transcript-like file downloads require owner-visible privacy opt-in settings.",
            },
            ZoomCapabilityStatus {
                capability: "recording_imports.remove",
                category: "retention",
                status: "available",
                action_class: "local_delete",
                confirmation_required: true,
                reason: "Imported Zoom recording blobs can be explicitly removed per account through the local retention control surface, with follow-up audit events.",
            },
            ZoomCapabilityStatus {
                capability: "retention.cleanup",
                category: "retention",
                status: "available",
                action_class: "local_delete",
                confirmation_required: true,
                reason: "Expired Zoom recording imports and transcript evidence can be pruned through the owner-visible retention control surface using stamped expiry intent.",
            },
            ZoomCapabilityStatus {
                capability: "retention.cleanup.scheduler",
                category: "runtime",
                status: "available",
                action_class: "local_automation",
                confirmation_required: false,
                reason: "The local backend scheduler can periodically prune expired Zoom recording imports and transcript evidence through the same retention boundary.",
            },
            ZoomCapabilityStatus {
                capability: "provider_sync.recordings.scheduler",
                category: "runtime",
                status: "available",
                action_class: "local_automation",
                confirmation_required: false,
                reason: "Started authorized Zoom runtimes can periodically synchronize recent cloud recording metadata through the same HostVault-backed provider sync boundary.",
            },
            ZoomCapabilityStatus {
                capability: "webhooks.verified",
                category: "ingest",
                status: "available",
                action_class: "local_write",
                confirmation_required: true,
                reason: "Account-scoped Zoom webhooks are accepted only after URL validation or HMAC signature verification.",
            },
            ZoomCapabilityStatus {
                capability: "webhooks.subscription_management",
                category: "ingest",
                status: "available",
                action_class: "external_provider_write",
                confirmation_required: true,
                reason: "Authorized Zoom accounts can reconcile managed event subscriptions through the live provider API using app-owned access tokens.",
            },
            ZoomCapabilityStatus {
                capability: "webhooks.edge_proxy",
                category: "ingest",
                status: "available",
                action_class: "external_ingress",
                confirmation_required: true,
                reason: "The hermes-zoom-edge-proxy binary forwards raw public Zoom webhooks into the protected verified runtime bridge.",
            },
            ZoomCapabilityStatus {
                capability: "calendar_event_matching",
                category: "workflow",
                status: "available",
                action_class: "local_automation",
                confirmation_required: false,
                reason: "Zoom meeting observations can be projected into existing Calendar event relations through the downstream workflow boundary.",
            },
            ZoomCapabilityStatus {
                capability: "meeting_participant_identity_resolution",
                category: "workflow",
                status: "available",
                action_class: "local_automation",
                confirmation_required: false,
                reason: "Zoom meeting participants can feed conservative identity-candidate generation into the existing Review workflow.",
            },
        ],
        planned_features: vec![],
        unsupported_features: vec![
            "hidden_recording",
            "joining_meetings_as_a_bot_without_explicit_setup",
            "auto_dialing",
            "training_models_on_zoom_content_by_default",
        ],
    }))
}

pub(crate) async fn post_zoom_fixture_account(
    State(state): State<AppState>,
    Json(request): Json<ZoomAccountSetupRequest>,
) -> Result<Json<ZoomAccountSetupResponse>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let response = zoom_provider_runtime_service(&state)?
        .setup_fixture_account(&request)
        .await?;
    sync_zoom_signal_connection(&state, &response.account.account_id).await?;
    Ok(Json(response))
}

pub(crate) async fn post_zoom_account(
    State(state): State<AppState>,
    Json(request): Json<ZoomLiveAccountSetupRequest>,
) -> Result<Json<ZoomAccountSetupResponse>, ApiError> {
    let response = zoom_provider_runtime_service(&state)?
        .setup_live_blocked_account(&request)
        .await?;
    sync_zoom_signal_connection(&state, &response.account.account_id).await?;
    Ok(Json(response))
}

pub(crate) async fn post_zoom_oauth_start(
    State(state): State<AppState>,
    Json(request): Json<ZoomOAuthStartRequest>,
) -> Result<Json<ZoomOAuthStartResponse>, ApiError> {
    require_zoom_unlocked_host_vault(&state)?;
    let pending = zoom_provider_runtime_service(&state)?
        .start_oauth(&request)
        .await?;
    let response = pending.response();
    let mut pending_map = state
        .account_setup
        .pending_zoom_oauth
        .lock()
        .map_err(|_| ApiError::AccountSetupState)?;
    pending_map.insert(pending.setup_id.clone(), pending);
    Ok(Json(response))
}

pub(crate) async fn post_zoom_oauth_complete(
    State(state): State<AppState>,
    Json(request): Json<ZoomOAuthCompleteRequest>,
) -> Result<Json<ZoomAuthorizationResult>, ApiError> {
    require_zoom_unlocked_host_vault(&state)?;
    request.validate()?;
    let pending = {
        let mut pending_map = state
            .account_setup
            .pending_zoom_oauth
            .lock()
            .map_err(|_| ApiError::AccountSetupState)?;
        pending_map
            .remove(&request.setup_id)
            .ok_or(ApiError::AccountSetupPendingGrantNotFound)?
    };
    if pending.state != request.state {
        return Err(ApiError::AccountSetupStateMismatch);
    }
    let secret_store = zoom_secret_reference_store(&state)?;
    let result = zoom_provider_runtime_service(&state)?
        .complete_oauth(
            &secret_store,
            &state.vault,
            pending,
            &request.authorization_code,
            request.external_account_id.as_deref(),
        )
        .await?;
    let account = provider_account_or_not_found(&state, &result.account_id).await?;
    sync_provider_account_signal_connection(&state, &account, Some(&result.token_secret_ref))
        .await?;
    Ok(Json(result))
}

pub(crate) async fn post_zoom_server_to_server_authorize(
    State(state): State<AppState>,
    Json(request): Json<ZoomServerToServerAuthorizeRequest>,
) -> Result<Json<ZoomAuthorizationResult>, ApiError> {
    require_zoom_unlocked_host_vault(&state)?;
    let secret_store = zoom_secret_reference_store(&state)?;
    let result = zoom_provider_runtime_service(&state)?
        .authorize_server_to_server(&secret_store, &state.vault, &request)
        .await?;
    let account = provider_account_or_not_found(&state, &result.account_id).await?;
    sync_provider_account_signal_connection(&state, &account, Some(&result.token_secret_ref))
        .await?;
    Ok(Json(result))
}

pub(crate) async fn post_zoom_oauth_refresh(
    State(state): State<AppState>,
    Json(request): Json<ZoomTokenRefreshRequest>,
) -> Result<Json<ZoomTokenRefreshResult>, ApiError> {
    require_zoom_unlocked_host_vault(&state)?;
    let secret_store = zoom_secret_reference_store(&state)?;
    let result = zoom_provider_runtime_service(&state)?
        .refresh_token(&secret_store, &state.vault, &request)
        .await?;
    let account = provider_account_or_not_found(&state, &result.account_id).await?;
    sync_provider_account_signal_connection(&state, &account, Some(&result.token_secret_ref))
        .await?;
    Ok(Json(result))
}

pub(crate) async fn post_zoom_oauth_maintenance(
    State(state): State<AppState>,
    Json(request): Json<ZoomTokenMaintenanceRequest>,
) -> Result<Json<ZoomTokenMaintenanceResult>, ApiError> {
    require_zoom_unlocked_host_vault(&state)?;
    let secret_store = zoom_secret_reference_store(&state)?;
    let result = zoom_provider_runtime_service(&state)?
        .maintain_tokens(&secret_store, &state.vault, &request)
        .await?;
    for item in &result.items {
        if item.status == "failed" {
            continue;
        }
        if let Ok(account) = provider_account_or_not_found(&state, &item.account_id).await {
            sync_provider_account_signal_connection(&state, &account, None).await?;
        }
    }
    Ok(Json(result))
}

pub(crate) async fn post_zoom_provider_sync_recordings(
    State(state): State<AppState>,
    Json(request): Json<ZoomRecordingSyncRequest>,
) -> Result<Json<ZoomRecordingSyncResult>, ApiError> {
    require_zoom_unlocked_host_vault(&state)?;
    let secret_store = zoom_secret_reference_store(&state)?;
    let allow_remote_recording_downloads = zoom_remote_recording_download_enabled(&state).await?;
    let allow_remote_transcript_downloads = zoom_remote_transcript_download_enabled(&state).await?;
    let result = zoom_provider_runtime_service(&state)?
        .sync_recordings(
            &secret_store,
            &state.vault,
            &request,
            allow_remote_recording_downloads,
            allow_remote_transcript_downloads,
        )
        .await?;
    Ok(Json(result))
}

pub(crate) async fn get_zoom_webhook_subscription_status(
    State(state): State<AppState>,
    Query(query): Query<ZoomWebhookSubscriptionStatusQuery>,
) -> Result<Json<ZoomWebhookSubscriptionStatusResult>, ApiError> {
    require_zoom_unlocked_host_vault(&state)?;
    let secret_store = zoom_secret_reference_store(&state)?;
    Ok(Json(
        zoom_provider_runtime_service(&state)?
            .webhook_subscription_status(
                &secret_store,
                &state.vault,
                &ZoomWebhookSubscriptionStatusRequest {
                    account_id: query.account_id,
                    api_base_url: query.api_base_url,
                },
            )
            .await?,
    ))
}

pub(crate) async fn post_zoom_webhook_subscription_reconcile(
    State(state): State<AppState>,
    Json(request): Json<ZoomWebhookSubscriptionReconcileRequest>,
) -> Result<Json<ZoomWebhookSubscriptionReconcileResult>, ApiError> {
    require_zoom_unlocked_host_vault(&state)?;
    let secret_store = zoom_secret_reference_store(&state)?;
    Ok(Json(
        zoom_provider_runtime_service(&state)?
            .reconcile_webhook_subscription(&secret_store, &state.vault, &request)
            .await?,
    ))
}

pub(crate) async fn post_zoom_webhook_subscription_remove(
    State(state): State<AppState>,
    Json(request): Json<ZoomWebhookSubscriptionRemoveRequest>,
) -> Result<Json<ZoomWebhookSubscriptionRemoveResult>, ApiError> {
    require_zoom_unlocked_host_vault(&state)?;
    let secret_store = zoom_secret_reference_store(&state)?;
    Ok(Json(
        zoom_provider_runtime_service(&state)?
            .remove_webhook_subscription(&secret_store, &state.vault, &request)
            .await?,
    ))
}

pub(crate) async fn get_zoom_accounts(
    State(state): State<AppState>,
    Query(query): Query<ZoomAccountsQuery>,
) -> Result<Json<ZoomAccountListResponse>, ApiError> {
    Ok(Json(
        zoom_provider_runtime_service(&state)?
            .list_accounts(query.include_removed)
            .await?,
    ))
}

pub(crate) async fn get_zoom_runtime_status(
    State(state): State<AppState>,
    Query(query): Query<ZoomRuntimeStatusQuery>,
) -> Result<Json<ZoomRuntimeStatus>, ApiError> {
    Ok(Json(
        zoom_provider_runtime_service(&state)?
            .runtime_status(&query.account_id)
            .await?,
    ))
}

pub(crate) async fn get_zoom_account_runtime_status(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<ZoomRuntimeStatus>, ApiError> {
    Ok(Json(
        zoom_provider_runtime_service(&state)?
            .runtime_status(&account_id)
            .await?,
    ))
}

pub(crate) async fn get_zoom_recording_imports(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
    Query(query): Query<ZoomRecordingImportsQuery>,
) -> Result<Json<ZoomRecordingImportAuditResponse>, ApiError> {
    Ok(Json(
        zoom_provider_runtime_service(&state)?
            .list_recording_imports(&account_id, query.limit)
            .await?,
    ))
}

pub(crate) async fn post_zoom_recording_import_remove(
    State(state): State<AppState>,
    Path((account_id, attachment_id)): Path<(String, String)>,
    Json(request): Json<ZoomRecordingImportRemoveRequest>,
) -> Result<Json<ZoomRecordingImportRemoveResponse>, ApiError> {
    Ok(Json(
        zoom_provider_runtime_service(&state)?
            .remove_recording_import(&account_id, &attachment_id, &request)
            .await?,
    ))
}

pub(crate) async fn get_zoom_audit_events(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
    Query(query): Query<ZoomAuditEventsQuery>,
) -> Result<Json<ZoomAuditEventResponse>, ApiError> {
    Ok(Json(
        zoom_provider_runtime_service(&state)?
            .list_audit_events(&account_id, query.limit)
            .await?,
    ))
}

pub(crate) async fn post_zoom_retention_cleanup(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
    Json(request): Json<ZoomRetentionCleanupRequest>,
) -> Result<Json<ZoomRetentionCleanupResponse>, ApiError> {
    Ok(Json(
        zoom_provider_runtime_service(&state)?
            .cleanup_retention(&account_id, &request)
            .await?,
    ))
}

pub(crate) async fn post_zoom_runtime_start(
    State(state): State<AppState>,
    Json(request): Json<ZoomRuntimeStartRequest>,
) -> Result<Json<ZoomRuntimeStatus>, ApiError> {
    let status = zoom_provider_runtime_service(&state)?
        .start_runtime(&request)
        .await?;
    sync_zoom_signal_connection(&state, &status.account_id).await?;
    Ok(Json(status))
}

pub(crate) async fn post_zoom_runtime_stop(
    State(state): State<AppState>,
    Json(request): Json<ZoomRuntimeStopRequest>,
) -> Result<Json<ZoomRuntimeStatus>, ApiError> {
    let status = zoom_provider_runtime_service(&state)?
        .stop_runtime(&request)
        .await?;
    sync_zoom_signal_connection(&state, &status.account_id).await?;
    Ok(Json(status))
}

pub(crate) async fn post_zoom_runtime_remove(
    State(state): State<AppState>,
    Json(request): Json<ZoomRuntimeRemoveRequest>,
) -> Result<Json<ZoomRuntimeRemoveResponse>, ApiError> {
    let response = zoom_provider_runtime_service(&state)?
        .remove_runtime(&request)
        .await?;
    sync_zoom_signal_connection(&state, &response.account_id).await?;
    Ok(Json(response))
}

pub(crate) async fn post_zoom_runtime_bridge_meeting(
    State(state): State<AppState>,
    Json(request): Json<ZoomMeetingObservationRequest>,
) -> Result<Json<ZoomMeetingIngestResult>, ApiError> {
    Ok(Json(
        zoom_provider_runtime_service(&state)?
            .observe_meeting(&request)
            .await?,
    ))
}

pub(crate) async fn post_zoom_runtime_bridge_recording(
    State(state): State<AppState>,
    Json(request): Json<ZoomRecordingObservationRequest>,
) -> Result<Json<ZoomRecordingIngestResult>, ApiError> {
    Ok(Json(
        zoom_provider_runtime_service(&state)?
            .observe_recording(&request)
            .await?,
    ))
}

pub(crate) async fn post_zoom_runtime_bridge_transcript(
    State(state): State<AppState>,
    Json(request): Json<ZoomTranscriptObservationRequest>,
) -> Result<Json<ZoomTranscriptIngestResult>, ApiError> {
    Ok(Json(
        zoom_provider_runtime_service(&state)?
            .observe_transcript(&request)
            .await?,
    ))
}

pub(crate) async fn post_zoom_runtime_bridge_transcript_file(
    State(state): State<AppState>,
    Json(request): Json<ZoomTranscriptFileImportRequest>,
) -> Result<Json<ZoomTranscriptFileImportResult>, ApiError> {
    Ok(Json(
        zoom_provider_runtime_service(&state)?
            .import_transcript_file(&request)
            .await?,
    ))
}

pub(crate) async fn post_zoom_runtime_bridge_webhook(
    State(state): State<AppState>,
    Query(query): Query<ZoomWebhookQuery>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, ApiError> {
    let account_id = validate_zoom_webhook_account_id(&query.account_id)?;
    let webhook_secret = read_zoom_webhook_secret(&state, &account_id).await?;
    let envelope = zoom_webhook_envelope(&body)?;
    let event = zoom_webhook_event(&envelope)?;

    if event == "endpoint.url_validation" {
        let plain_token = zoom_webhook_plain_token(&envelope)?;
        let encrypted_token = zoom_webhook_validation_token(&webhook_secret, &plain_token)?;
        return Ok(Json(json!({
            "plainToken": plain_token,
            "encryptedToken": encrypted_token,
        })));
    }

    verify_zoom_webhook_signature(&webhook_secret, &headers, &body)?;

    let service = zoom_provider_runtime_service(&state)?;
    if event.starts_with("meeting.") {
        let request = zoom_meeting_observation_from_webhook(&account_id, event, &envelope)?;
        let result = service.observe_meeting(&request).await?;
        return Ok(Json(json!({
            "account_id": account_id,
            "event": event,
            "status": "recorded",
            "meeting": result,
        })));
    }

    if event.starts_with("recording.") {
        let recording_requests =
            zoom_recording_observations_from_webhook(&account_id, event, &envelope)?;
        let recording_downloads =
            zoom_recording_media_downloads_from_recording_webhook(&account_id, event, &envelope)?;
        let transcript_downloads =
            zoom_transcript_downloads_from_recording_webhook(&account_id, event, &envelope)?;
        let allow_remote_recording_downloads =
            zoom_remote_recording_download_enabled(&state).await?;
        let allow_remote_transcript_downloads =
            zoom_remote_transcript_download_enabled(&state).await?;
        let mut recordings = Vec::with_capacity(recording_requests.len());
        for request in recording_requests {
            recordings.push(service.observe_recording(&request).await?);
        }
        let mut recording_imports = Vec::with_capacity(recording_downloads.len());
        for download in recording_downloads {
            if !allow_remote_recording_downloads {
                recording_imports.push(json!({
                    "status": "blocked",
                    "recording_id": download.request.recording.recording_id,
                    "error": ZOOM_REMOTE_RECORDING_DOWNLOAD_NOT_ENABLED,
                }));
                continue;
            }
            match service
                .import_recording_media_download(
                    &download.request,
                    download.download_token.as_deref(),
                )
                .await
            {
                Ok(result) => recording_imports.push(json!({
                    "status": "recorded",
                    "recording": result,
                })),
                Err(error) => recording_imports.push(json!({
                    "status": "failed",
                    "recording_id": download.request.recording.recording_id,
                    "error": error.to_string(),
                })),
            }
        }
        let mut transcript_imports = Vec::with_capacity(transcript_downloads.len());
        for download in transcript_downloads {
            if !allow_remote_transcript_downloads {
                transcript_imports.push(json!({
                    "status": "blocked",
                    "transcript_id": download.request.transcript_id,
                    "error": ZOOM_REMOTE_TRANSCRIPT_DOWNLOAD_NOT_ENABLED,
                }));
                continue;
            }
            match service
                .import_transcript_file_download(
                    &download.request,
                    &download.download_url,
                    download.download_token.as_deref(),
                )
                .await
            {
                Ok(result) => transcript_imports.push(json!({
                    "status": "recorded",
                    "transcript": result,
                })),
                Err(error) => transcript_imports.push(json!({
                    "status": "failed",
                    "transcript_id": download.request.transcript_id,
                    "error": error.to_string(),
                })),
            }
        }
        return Ok(Json(json!({
            "account_id": account_id,
            "event": event,
            "status": if recordings.is_empty() { "ignored" } else { "recorded" },
            "recordings": recordings,
            "recording_imports": recording_imports,
            "transcript_imports": transcript_imports,
        })));
    }

    Ok(Json(json!({
        "account_id": account_id,
        "event": event,
        "status": "ignored",
        "reason": "unsupported_zoom_webhook_event",
    })))
}

async fn sync_zoom_signal_connection(state: &AppState, account_id: &str) -> Result<(), ApiError> {
    let account = provider_account_or_not_found(state, account_id).await?;
    sync_provider_account_signal_connection(state, &account, None).await
}

async fn zoom_remote_transcript_download_enabled(state: &AppState) -> Result<bool, ApiError> {
    Ok(settings_store(state)?
        .setting(ZOOM_REMOTE_TRANSCRIPT_DOWNLOAD_ENABLED_SETTING_KEY)
        .await?
        .and_then(|setting| setting.value.as_bool())
        .unwrap_or(false))
}

async fn zoom_remote_recording_download_enabled(state: &AppState) -> Result<bool, ApiError> {
    Ok(settings_store(state)?
        .setting(ZOOM_REMOTE_RECORDING_DOWNLOAD_ENABLED_SETTING_KEY)
        .await?
        .and_then(|setting| setting.value.as_bool())
        .unwrap_or(false))
}

const fn default_zoom_recording_imports_limit() -> i64 {
    20
}

const fn default_zoom_audit_events_limit() -> i64 {
    25
}

fn validate_zoom_webhook_account_id(account_id: &str) -> Result<String, ZoomError> {
    let trimmed = account_id.trim();
    if trimmed.is_empty() {
        return Err(ZoomError::InvalidRequest(
            "account_id query parameter is required for Zoom webhook ingestion".to_owned(),
        ));
    }
    Ok(trimmed.to_owned())
}

fn require_zoom_unlocked_host_vault(state: &AppState) -> Result<(), ApiError> {
    match state.vault.status()?.state {
        VaultMode::Unlocked => Ok(()),
        VaultMode::Locked => Err(ApiError::HostVault(HostVaultError::Locked)),
        VaultMode::Uninitialized => Err(ApiError::HostVault(HostVaultError::Uninitialized)),
    }
}

async fn read_zoom_webhook_secret(state: &AppState, account_id: &str) -> Result<String, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let binding = CommunicationProviderSecretBindingStore::new(pool)
        .get_for_account(account_id, ProviderAccountSecretPurpose::ZoomWebhookSecret)
        .await
        .map_err(|error| ZoomError::InvalidRequest(error.to_string()))?
        .ok_or_else(|| {
            ZoomError::InvalidRequest(format!(
                "Zoom webhook secret is not configured for account `{account_id}`"
            ))
        })?;
    state
        .vault
        .read_secret(&binding.secret_ref)
        .map_err(ApiError::HostVault)
}

fn zoom_webhook_envelope(body: &[u8]) -> Result<Value, ZoomError> {
    serde_json::from_slice::<Value>(body).map_err(|error| {
        ZoomError::InvalidRequest(format!("Zoom webhook body must be valid JSON: {error}"))
    })
}

fn zoom_webhook_event(envelope: &Value) -> Result<&str, ZoomError> {
    envelope
        .get("event")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ZoomError::InvalidRequest("Zoom webhook event is required".to_owned()))
}

fn zoom_webhook_plain_token(envelope: &Value) -> Result<String, ZoomError> {
    envelope
        .get("payload")
        .and_then(|payload| payload.get("plainToken"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            ZoomError::InvalidRequest(
                "Zoom endpoint.url_validation payload.plainToken is required".to_owned(),
            )
        })
}

fn verify_zoom_webhook_signature(
    webhook_secret: &str,
    headers: &HeaderMap,
    body: &[u8],
) -> Result<(), ZoomError> {
    let timestamp = headers
        .get(ZOOM_TIMESTAMP_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            ZoomError::InvalidRequest(
                "x-zm-request-timestamp is required for Zoom webhook ingestion".to_owned(),
            )
        })?;
    let timestamp_seconds = timestamp.parse::<i64>().map_err(|_| {
        ZoomError::InvalidRequest(
            "x-zm-request-timestamp must be a Unix timestamp in seconds".to_owned(),
        )
    })?;
    let now_seconds = Utc::now().timestamp();
    if (now_seconds - timestamp_seconds).abs() > ZOOM_WEBHOOK_SIGNATURE_TOLERANCE_SECONDS {
        return Err(ZoomError::InvalidRequest(
            "Zoom webhook timestamp is outside the allowed replay window".to_owned(),
        ));
    }

    let signature = headers
        .get(ZOOM_SIGNATURE_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            ZoomError::InvalidRequest(
                "x-zm-signature is required for Zoom webhook ingestion".to_owned(),
            )
        })?;
    let signature = signature.strip_prefix("v0=").ok_or_else(|| {
        ZoomError::InvalidRequest("x-zm-signature must use v0=<hex-digest>".to_owned())
    })?;
    let signature = decode_sha256_hex(signature).ok_or_else(|| {
        ZoomError::InvalidRequest("x-zm-signature must contain a 32-byte hex digest".to_owned())
    })?;

    let mut mac = HmacSha256::new_from_slice(webhook_secret.as_bytes()).map_err(|_| {
        ZoomError::InvalidRequest("Zoom webhook secret is invalid for HMAC".to_owned())
    })?;
    mac.update(b"v0:");
    mac.update(timestamp.as_bytes());
    mac.update(b":");
    mac.update(body);
    mac.verify_slice(&signature).map_err(|_| {
        ZoomError::InvalidRequest("Zoom webhook signature verification failed".to_owned())
    })
}

fn zoom_webhook_validation_token(
    webhook_secret: &str,
    plain_token: &str,
) -> Result<String, ZoomError> {
    let mut mac = HmacSha256::new_from_slice(webhook_secret.as_bytes()).map_err(|_| {
        ZoomError::InvalidRequest("Zoom webhook secret is invalid for HMAC".to_owned())
    })?;
    mac.update(plain_token.as_bytes());
    Ok(bytes_to_lower_hex(&mac.finalize().into_bytes()))
}

fn zoom_meeting_observation_from_webhook(
    account_id: &str,
    event: &str,
    envelope: &Value,
) -> Result<ZoomMeetingObservationRequest, ZoomError> {
    let object = zoom_webhook_object(envelope)?;
    let meeting_id = required_value_string(object, &["id", "meeting_id"], "payload.object.id")?;
    Ok(ZoomMeetingObservationRequest {
        observation_id: Some(zoom_webhook_observation_id(event, envelope, &meeting_id)),
        account_id: account_id.to_owned(),
        meeting_id,
        meeting_uuid: value_string(object, &["uuid", "meeting_uuid"]),
        topic: value_string(object, &["topic"]),
        host_email: value_string(object, &["host_email"]),
        join_url: value_string(object, &["join_url"]),
        started_at: value_datetime(object, &["start_time", "started_at"]),
        ended_at: value_datetime(object, &["end_time", "ended_at"]),
        duration_seconds: value_i64(object, &["duration_seconds"]),
        participants: Vec::new(),
        recording_refs: Vec::new(),
        transcript_ref: value_string(object, &["transcript_ref"]),
        metadata: json!({
            "webhook_event": event,
            "webhook_event_ts": envelope.get("event_ts"),
            "webhook_payload": envelope.get("payload"),
        }),
        causation_id: value_string(envelope, &["event_id"]),
        correlation_id: Some(format!("zoom-webhook:{account_id}")),
    })
}

fn zoom_recording_observations_from_webhook(
    account_id: &str,
    event: &str,
    envelope: &Value,
) -> Result<Vec<ZoomRecordingObservationRequest>, ZoomError> {
    let object = zoom_webhook_object(envelope)?;
    let meeting_id = required_value_string(object, &["id", "meeting_id"], "payload.object.id")?;
    let Some(files) = object.get("recording_files").and_then(Value::as_array) else {
        return Ok(Vec::new());
    };
    let mut requests = Vec::new();
    for (index, file) in files.iter().enumerate() {
        let recording_id = value_string(file, &["id", "file_id", "recording_id"])
            .unwrap_or_else(|| format!("{}:recording-file:{index}", meeting_id.trim()));
        let recording = ZoomRecordingRef {
            recording_id,
            recording_type: value_string(file, &["recording_type", "file_type"]),
            download_ref: value_string(file, &["download_url", "download_ref"]),
            file_extension: value_string(file, &["file_extension", "file_type"]),
            file_size_bytes: value_i64(file, &["file_size", "file_size_bytes"]),
            recorded_at: value_datetime(file, &["recording_start", "start_time", "recorded_at"]),
            metadata: json!({
                "webhook_event": event,
                "webhook_file": file,
            }),
        };
        requests.push(ZoomRecordingObservationRequest {
            observation_id: Some(zoom_webhook_observation_id(
                event,
                envelope,
                &format!("{}:{}", meeting_id.trim(), recording.recording_id.trim()),
            )),
            account_id: account_id.to_owned(),
            meeting_id: meeting_id.clone(),
            recording,
            metadata: json!({
                "webhook_event": event,
                "webhook_event_ts": envelope.get("event_ts"),
                "webhook_payload": envelope.get("payload"),
            }),
            causation_id: value_string(envelope, &["event_id"]),
            correlation_id: Some(format!("zoom-webhook:{account_id}")),
        });
    }
    Ok(requests)
}

fn zoom_transcript_downloads_from_recording_webhook(
    account_id: &str,
    event: &str,
    envelope: &Value,
) -> Result<Vec<ZoomWebhookTranscriptDownload>, ZoomError> {
    let object = zoom_webhook_object(envelope)?;
    let meeting_id = required_value_string(object, &["id", "meeting_id"], "payload.object.id")?;
    let meeting_uuid = value_string(object, &["uuid"]);
    let Some(files) = object.get("recording_files").and_then(Value::as_array) else {
        return Ok(Vec::new());
    };

    let mut requests = Vec::new();
    for (index, file) in files.iter().enumerate() {
        if !is_zoom_transcript_recording_file(file) {
            continue;
        }
        let Some(download_url) = value_string(file, &["download_url", "download_ref"]) else {
            continue;
        };
        let recording_id = value_string(file, &["id", "file_id", "recording_id"])
            .unwrap_or_else(|| format!("{}:transcript-file:{index}", meeting_id.trim()));
        let file_name = value_string(file, &["file_name"]).or_else(|| {
            transcript_file_name(
                &recording_id,
                value_string(file, &["file_extension", "file_type"]).as_deref(),
            )
        });
        let content_type = transcript_content_type(
            value_string(file, &["file_extension", "file_type"]).as_deref(),
            file_name.as_deref(),
        );
        requests.push(ZoomWebhookTranscriptDownload {
            request: ZoomTranscriptFileImportRequest {
                observation_id: Some(zoom_webhook_observation_id(
                    event,
                    envelope,
                    &format!("{}:{}:transcript", meeting_id.trim(), recording_id.trim()),
                )),
                transcript_id: format!(
                    "zoom-transcript-download:{}:{}",
                    meeting_id.trim(),
                    recording_id.trim()
                ),
                account_id: account_id.to_owned(),
                meeting_id: meeting_id.clone(),
                meeting_uuid: meeting_uuid.clone(),
                source_recording_ref: Some(recording_id),
                language_code: value_string(file, &["file_language", "language"]),
                file_name,
                content_type,
                file_text: String::new(),
                metadata: json!({
                    "webhook_transcript_download": {
                        "event": event,
                        "source": "recording_webhook",
                        "file_type": value_string(file, &["file_type"]),
                        "file_extension": value_string(file, &["file_extension"]),
                    }
                }),
                causation_id: value_string(envelope, &["event_id"]),
                correlation_id: Some(format!("zoom-webhook:{account_id}")),
            },
            download_url,
            download_token: value_string(file, &["download_token"]),
        });
    }

    Ok(requests)
}

fn zoom_recording_media_downloads_from_recording_webhook(
    account_id: &str,
    event: &str,
    envelope: &Value,
) -> Result<Vec<ZoomWebhookRecordingMediaDownload>, ZoomError> {
    let object = zoom_webhook_object(envelope)?;
    let meeting_id = required_value_string(object, &["id", "meeting_id"], "payload.object.id")?;
    let meeting_uuid = value_string(object, &["uuid"]);
    let Some(files) = object.get("recording_files").and_then(Value::as_array) else {
        return Ok(Vec::new());
    };

    let mut requests = Vec::new();
    for (index, file) in files.iter().enumerate() {
        if is_zoom_transcript_recording_file(file) {
            continue;
        }
        let Some(download_url) = value_string(file, &["download_url", "download_ref"]) else {
            continue;
        };
        let recording_id = value_string(file, &["id", "file_id", "recording_id"])
            .unwrap_or_else(|| format!("{}:recording-file:{index}", meeting_id.trim()));
        let file_extension = value_string(file, &["file_extension", "file_type"]);
        let file_name = value_string(file, &["file_name"])
            .or_else(|| recording_media_file_name(&recording_id, file_extension.as_deref()));
        requests.push(ZoomWebhookRecordingMediaDownload {
            request: ZoomRecordingMediaDownloadRequest {
                observation_id: Some(zoom_webhook_observation_id(
                    event,
                    envelope,
                    &format!(
                        "{}:{}:recording-download",
                        meeting_id.trim(),
                        recording_id.trim()
                    ),
                )),
                account_id: account_id.to_owned(),
                meeting_id: meeting_id.clone(),
                meeting_uuid: meeting_uuid.clone(),
                recording: ZoomRecordingRef {
                    recording_id,
                    recording_type: value_string(file, &["recording_type", "file_type"]),
                    download_ref: Some(download_url.clone()),
                    file_extension: file_extension.clone(),
                    file_size_bytes: value_i64(file, &["file_size", "file_size_bytes"]),
                    recorded_at: value_datetime(
                        file,
                        &["recording_start", "start_time", "recorded_at"],
                    ),
                    metadata: json!({
                        "webhook_event": event,
                        "webhook_file": file,
                        "file_type": value_string(file, &["file_type"]),
                    }),
                },
                file_name,
                content_type: None,
                download_url,
                metadata: json!({
                    "source": "zoom_recording_webhook",
                    "webhook_event": event,
                    "webhook_event_ts": envelope.get("event_ts"),
                    "webhook_payload": envelope.get("payload"),
                }),
                causation_id: value_string(envelope, &["event_id"]),
                correlation_id: Some(format!("zoom-webhook:{account_id}")),
            },
            download_token: value_string(file, &["download_token"]),
        });
    }

    Ok(requests)
}

fn zoom_webhook_object(envelope: &Value) -> Result<&Value, ZoomError> {
    envelope
        .get("payload")
        .and_then(|payload| payload.get("object"))
        .filter(|value| value.is_object())
        .ok_or_else(|| {
            ZoomError::InvalidRequest("Zoom webhook payload.object is required".to_owned())
        })
}

fn zoom_webhook_observation_id(event: &str, envelope: &Value, subject_id: &str) -> String {
    if let Some(event_id) = value_string(envelope, &["event_id"]) {
        return event_id;
    }
    format!(
        "zoom-webhook:{}:{}:{}",
        event,
        envelope
            .get("event_ts")
            .and_then(Value::as_i64)
            .unwrap_or_default(),
        subject_id.trim()
    )
}

fn is_zoom_transcript_recording_file(file: &Value) -> bool {
    let file_type = value_string(file, &["file_type"])
        .unwrap_or_default()
        .to_ascii_lowercase();
    let recording_type = value_string(file, &["recording_type"])
        .unwrap_or_default()
        .to_ascii_lowercase();
    let file_extension = value_string(file, &["file_extension"])
        .or_else(|| {
            value_string(file, &["file_name"]).and_then(|name| {
                name.rsplit_once('.')
                    .map(|(_, extension)| extension.trim().to_owned())
            })
        })
        .unwrap_or_default()
        .to_ascii_lowercase();

    file_type.contains("transcript")
        || recording_type.contains("transcript")
        || matches!(file_extension.as_str(), "vtt" | "srt" | "txt")
        || file_type == "cc"
        || file_type == "vtt"
        || file_type == "srt"
}

fn transcript_file_name(recording_id: &str, extension_hint: Option<&str>) -> Option<String> {
    let extension = extension_hint
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.trim_matches('.').to_ascii_lowercase())?;
    Some(format!("{recording_id}.{extension}"))
}

fn recording_media_file_name(recording_id: &str, extension_hint: Option<&str>) -> Option<String> {
    let extension = extension_hint
        .map(str::trim)
        .filter(|value| !value.is_empty())?
        .to_ascii_lowercase();
    Some(format!("{recording_id}.{extension}"))
}

fn transcript_content_type(
    extension_hint: Option<&str>,
    file_name: Option<&str>,
) -> Option<String> {
    let extension = extension_hint
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.trim_matches('.').to_ascii_lowercase())
        .or_else(|| {
            file_name.and_then(|value| {
                value
                    .rsplit_once('.')
                    .map(|(_, extension)| extension.trim().to_ascii_lowercase())
            })
        })?;
    Some(
        match extension.as_str() {
            "vtt" => "text/vtt",
            "srt" => "application/x-subrip",
            "txt" => "text/plain",
            _ => return None,
        }
        .to_owned(),
    )
}

fn required_value_string(
    object: &Value,
    keys: &[&str],
    field: &'static str,
) -> Result<String, ZoomError> {
    value_string(object, keys)
        .ok_or_else(|| ZoomError::InvalidRequest(format!("Zoom webhook {field} must not be empty")))
}

fn value_string(object: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| {
            object.get(*key).and_then(|value| match value {
                Value::String(raw) => Some(raw.trim().to_owned()),
                Value::Number(number) => Some(number.to_string()),
                _ => None,
            })
        })
        .filter(|value| !value.trim().is_empty())
}

fn value_i64(object: &Value, keys: &[&str]) -> Option<i64> {
    keys.iter().find_map(|key| {
        object.get(*key).and_then(|value| match value {
            Value::Number(number) => number.as_i64(),
            Value::String(raw) => raw.trim().parse::<i64>().ok(),
            _ => None,
        })
    })
}

fn value_datetime(object: &Value, keys: &[&str]) -> Option<DateTime<Utc>> {
    keys.iter().find_map(|key| {
        object
            .get(*key)
            .and_then(Value::as_str)
            .and_then(|raw| DateTime::parse_from_rfc3339(raw.trim()).ok())
            .map(|value| value.with_timezone(&Utc))
    })
}

fn decode_sha256_hex(value: &str) -> Option<[u8; 32]> {
    let value = value.trim();
    if value.len() != 64 {
        return None;
    }
    let mut output = [0_u8; 32];
    for (index, chunk) in value.as_bytes().chunks_exact(2).enumerate() {
        let high = decode_hex_nibble(chunk[0])?;
        let low = decode_hex_nibble(chunk[1])?;
        output[index] = (high << 4) | low;
    }
    Some(output)
}

fn decode_hex_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn bytes_to_lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}
