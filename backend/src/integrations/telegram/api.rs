use std::io;

use axum::extract::{Path, Query, RawQuery, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, header};
use axum::response::Html;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing_subscriber::EnvFilter;
use url::form_urlencoded;

use crate::ai::core::{
    AI_EMBEDDING_DIMENSION, AiAgentListResponse, AiAgentRun, AiAnswerRequest, AiError,
    AiMeetingPrepRequest, AiService, AiStatusResponse, AiTaskCandidateRefreshRequest, v3_agents,
};
use crate::domains::mail::core::{
    CommunicationIngestionError, CommunicationIngestionStore, EmailProviderKind, ProviderAccount,
};
use crate::domains::persons::analytics::{AnalyticsError, PersonAnalyticsService};
use crate::domains::persons::enrichment_engine::{EnrichmentEngineError, EnrichmentResultStore};
use crate::domains::persons::expertise::{PersonExpertiseError, PersonExpertiseStore};
use crate::domains::persons::export::{ExportError, ExportFormat, PersonExportService};
use crate::domains::persons::investigator::{InvestigatorError, PersonInvestigator};
use crate::engines::automation::{
    AutomationError, AutomationPolicy, AutomationStore, AutomationTemplate, NewAutomationPolicy,
    NewAutomationTemplate, TelegramSendDryRunRequest, TelegramSendDryRunResponse,
};
use crate::platform::audit::{ApiAuditError, ApiAuditLog, ApiAuditRecord, NewApiAuditRecord};
use crate::platform::calls::{
    CallDirection, CallError, CallIntelligenceStore, CallState, CallTranscript,
    FixtureSpeechToTextProvider, NewCallTranscript, NewTelegramCall, SpeechToTextProvider,
    TelegramCall, TranscriptStatus,
};
use crate::platform::capabilities::{CapabilityActionClass, CapabilityDecision};
use crate::platform::config::AppConfig;

use crate::domains::persons::health::{PersonHealthError, PersonHealthStore};

use crate::domains::persons::trust::{PersonPromiseStore, PersonRiskStore, PersonTrustError};

use crate::domains::persons::memory::{
    NewRelationshipEvent, PersonFactStore, PersonMemoryCardStore, PersonMemoryError,
    PersonPreferenceStore, RelationshipEventStore,
};

use crate::domains::persons::core::{
    NewPersonPersona, PersonCoreError, PersonIdentity, PersonPersona, PersonPersonaStore,
    PersonRole, PersonRoleStore, PersonsIdentityStore,
};
use crate::domains::persons::identity::{
    PersonIdentityCandidate, PersonIdentityDetail, PersonIdentityError,
    PersonIdentityReviewCommand, PersonIdentityReviewState, PersonIdentityStore,
};

use crate::domains::calendar::brain::{CalendarBrainError, CalendarBrainService};
use crate::domains::calendar::core::{
    CalendarCoreError, ContextPackInput, EventAgendaStore, EventChecklistStore,
    EventContextPackStore, EventParticipantStore, EventRelationStore,
};
use crate::domains::calendar::events::{
    CalendarAccountStore, CalendarAccountUpdate, CalendarError, CalendarEventListQuery,
    CalendarEventStore, CalendarEventUpdate, CalendarSourceStore, NewCalendarEvent,
};
use crate::domains::calendar::health::{CalendarHealthError, CalendarWatchtowerService};
use crate::domains::calendar::intelligence::CalendarIntelligenceService;
use crate::domains::calendar::meetings::{
    EventRecordingStore, EventTranscriptStore, MeetingNoteStore, MeetingOutcomeStore, MeetingsError,
};
use crate::domains::calendar::reminders::{CalendarReminderStore, ReminderError};
use crate::domains::calendar::rules::{CalendarRuleError, CalendarRuleStore, RuleUpdate};
use crate::domains::calendar::scheduling::{
    DeadlineStore, FocusBlockStore, SchedulingError, SmartSchedulingService,
};
use crate::domains::calendar::sync::{export_event_ics, export_event_md};
use crate::domains::documents::processing::{
    DocumentProcessingError, DocumentProcessingJob, DocumentProcessingRecord,
    DocumentProcessingRetryCommand, DocumentProcessingRetryCommandResult, DocumentProcessingStatus,
    DocumentProcessingStore,
};
use crate::domains::graph::core::{GraphNodeKind, node_id};
use crate::domains::mail::accounts::{
    EmailAccountSetupError, EmailAccountSetupService, GmailOAuthPendingGrant,
    GmailOAuthSetupRequest, ImapAccountSetupRequest,
};
use crate::domains::mail::messages::{
    MessageProjectionError, MessageProjectionStore, ProjectedMessage, ProjectedMessageSummary,
    WorkflowState,
};
use crate::domains::mail::storage::{
    MailStorageError, MailStorageStore, StoredMailAttachmentWithBlob,
};
use crate::domains::organizations::api::{
    OrganizationError, OrganizationStore, OrganizationUpdate,
};
use crate::domains::projects::core::{ProjectListResponse, ProjectStore, ProjectStoreError};
use crate::domains::projects::link_reviews::{
    ProjectLinkReviewCommand, ProjectLinkReviewError, ProjectLinkReviewState,
    ProjectLinkReviewStore, ProjectLinkTargetKind,
};
use crate::domains::tasks::api::{NewTask, TaskError, TaskListQuery, TaskStore, TaskUpdate};
use crate::domains::tasks::brain::{TaskBrainError, TaskBrainService};
use crate::domains::tasks::candidates::{
    TaskCandidate, TaskCandidateError, TaskCandidateReviewCommand, TaskCandidateReviewState,
    TaskCandidateStore,
};
use crate::domains::tasks::core::{
    ExternalTaskIdentityStore, TaskChecklistStore, TaskContextPackStore, TaskCoreError,
    TaskEvidenceStore, TaskProviderStore, TaskRelationStore, TaskSubtaskStore,
};
use crate::domains::tasks::health::{TaskHealthError, TaskWatchtowerService};
use crate::domains::tasks::intelligence::TaskIntelligenceService;
use crate::domains::tasks::rules::{TaskRuleError, TaskRuleStore, TaskTemplateStore};
use crate::domains::tasks::sync::{export_task_json, export_task_md};
use crate::integrations::ollama::client::{OllamaClient, OllamaClientConfig};
use crate::integrations::telegram::client::{
    NewTelegramMessage, TelegramAccountSetupRequest, TelegramAccountSetupResponse, TelegramChat,
    TelegramError, TelegramLiveAccountSetupRequest, TelegramManualSendRequest,
    TelegramManualSendResponse, TelegramMessage, TelegramMessageIngestResult,
    TelegramQrLoginPasswordRequest, TelegramQrLoginStartRequest, TelegramQrLoginStatusResponse,
    TelegramStore,
};
use crate::integrations::telegram::runtime::{
    TelegramChatSyncRequest, TelegramChatSyncResponse, TelegramHistorySyncRequest,
    TelegramHistorySyncResponse, TelegramMediaDownloadContext, TelegramMediaDownloadRequest,
    TelegramMediaDownloadResponse, TelegramRuntimeStartRequest, TelegramRuntimeStatus,
};
use crate::integrations::telegram::tdjson;
use crate::integrations::whatsapp::client::{
    NewWhatsappWebMessage, WhatsappWebAccountSetupRequest, WhatsappWebAccountSetupResponse,
    WhatsappWebError, WhatsappWebMessage, WhatsappWebMessageIngestResult, WhatsappWebSession,
    WhatsappWebStore,
};
use crate::platform::events::{
    EventEnvelope, EventEnvelopeError, EventStore, EventStoreError, NewEventEnvelope,
};
use crate::platform::secrets::DatabaseEncryptedSecretVault;
use crate::platform::secrets::{SecretKind, SecretReferenceStore};
use crate::platform::settings::{
    AiRuntimeSettings, ApplicationSetting, ApplicationSettingsStore, SettingsError,
};
use crate::platform::storage::{
    Database, DatabaseReadiness, MigrationReadiness, ReadinessStatus, StorageError,
};
use crate::workflows::email_intelligence::{EmailIntelligenceError, EmailIntelligenceService};

use crate::app::{ApiError, AppState};
use crate::domains::api_support::*;

pub(crate) async fn get_telegram_capabilities(
    State(state): State<AppState>,
) -> Result<Json<TelegramCapabilitiesResponse>, ApiError> {
    Ok(Json(TelegramCapabilitiesResponse::current(&state.config)))
}

pub(crate) async fn post_telegram_fixture_account(
    State(state): State<AppState>,
    Json(request): Json<TelegramAccountSetupRequest>,
) -> Result<Json<TelegramAccountSetupResponse>, ApiError> {
    Ok(Json(
        telegram_store(&state)?
            .setup_fixture_account(&request)
            .await?,
    ))
}

pub(crate) async fn post_telegram_account(
    State(state): State<AppState>,
    Json(request): Json<TelegramLiveAccountSetupRequest>,
) -> Result<Json<TelegramAccountSetupResponse>, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let secret_store = SecretReferenceStore::new(pool.clone());
    let request = request
        .with_inferred_qr_authorization()
        .with_app_credentials(
            state.config.telegram_api_id(),
            telegram_api_hash_from_config(&state.config),
        );

    Ok(Json(
        telegram_store(&state)?
            .setup_live_blocked_account(
                &secret_store,
                &crate::integrations::telegram::client::TelegramSecretVault::host(
                    state.vault.clone(),
                ),
                &request,
            )
            .await?,
    ))
}

#[derive(Deserialize)]
pub(crate) struct TelegramRuntimeStatusQuery {
    pub(crate) account_id: String,
}

pub(crate) async fn get_telegram_runtime_status(
    State(state): State<AppState>,
    Query(query): Query<TelegramRuntimeStatusQuery>,
) -> Result<Json<TelegramRuntimeStatus>, ApiError> {
    Ok(Json(
        state
            .telegram_runtime
            .status_for_account(
                &communication_ingestion_store(&state)?,
                &state.config,
                &query.account_id,
            )
            .await?,
    ))
}

pub(crate) async fn post_telegram_runtime_start(
    State(state): State<AppState>,
    Json(request): Json<TelegramRuntimeStartRequest>,
) -> Result<Json<TelegramRuntimeStatus>, ApiError> {
    let secret_store = telegram_secret_store(&state)?;
    Ok(Json(
        state
            .telegram_runtime
            .start_account(
                &communication_ingestion_store(&state)?,
                &secret_store,
                &state.vault,
                &state.config,
                &request,
            )
            .await?,
    ))
}

pub(crate) async fn post_telegram_qr_login_start(
    State(state): State<AppState>,
    Json(request): Json<TelegramQrLoginStartRequest>,
) -> Result<Json<TelegramQrLoginStatusResponse>, ApiError> {
    let request = request.with_app_credentials(
        state.config.telegram_api_id(),
        telegram_api_hash_from_config(&state.config),
    );

    Ok(Json(
        tdjson::start_qr_login(
            state.config.clone(),
            state.account_setup.pending_telegram_qr_login.clone(),
            request,
        )
        .await?,
    ))
}

pub(crate) async fn get_telegram_qr_login_status(
    State(state): State<AppState>,
    Path(setup_id): Path<String>,
) -> Result<Json<TelegramQrLoginStatusResponse>, ApiError> {
    let pending = state
        .account_setup
        .pending_telegram_qr_login
        .lock()
        .map_err(|_| ApiError::AccountSetupState)?;
    let session = pending
        .get(setup_id.trim())
        .map(|session| session.response.clone())
        .ok_or(ApiError::Telegram(TelegramError::QrLoginNotFound))?;

    Ok(Json(session))
}

pub(crate) async fn delete_telegram_qr_login(
    State(state): State<AppState>,
    Path(setup_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let setup_id = setup_id.trim().to_owned();
    tdjson::cancel_qr_login(
        state.account_setup.pending_telegram_qr_login.clone(),
        &setup_id,
    )?;

    Ok(Json(json!({
        "setup_id": setup_id,
        "cancelled": true
    })))
}

pub(crate) async fn post_telegram_qr_login_password(
    State(state): State<AppState>,
    Path(setup_id): Path<String>,
    Json(request): Json<TelegramQrLoginPasswordRequest>,
) -> Result<Json<TelegramQrLoginStatusResponse>, ApiError> {
    Ok(Json(tdjson::submit_qr_login_password(
        state.account_setup.pending_telegram_qr_login.clone(),
        &setup_id,
        request,
    )?))
}

fn telegram_api_hash_from_config(config: &AppConfig) -> Option<String> {
    config
        .telegram_api_hash()
        .map(|secret| secret.expose_for_runtime().to_owned())
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

pub(crate) async fn post_telegram_fixture_message(
    State(state): State<AppState>,
    Json(request): Json<NewTelegramMessage>,
) -> Result<Json<TelegramMessageIngestResult>, ApiError> {
    Ok(Json(
        telegram_store(&state)?
            .ingest_fixture_message(&request)
            .await?,
    ))
}

pub(crate) async fn post_telegram_manual_send(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<TelegramManualSendRequest>,
) -> Result<Json<TelegramManualSendResponse>, ApiError> {
    let secret_store = telegram_secret_store(&state)?;
    let response = state
        .telegram_runtime
        .send_manual_message(
            &communication_ingestion_store(&state)?,
            &telegram_store(&state)?,
            &secret_store,
            &state.vault,
            &state.config,
            &request,
        )
        .await?;
    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_message_send(
            actor_id_from_headers(&headers),
            &response.message_id,
            &response.account_id,
            &response.provider_chat_id,
            &response.rendered_preview_hash,
        ))
        .await?;

    Ok(Json(response))
}

pub(crate) async fn post_telegram_media_download(
    State(state): State<AppState>,
    Json(request): Json<TelegramMediaDownloadRequest>,
) -> Result<Json<TelegramMediaDownloadResponse>, ApiError> {
    let secret_store = telegram_secret_store(&state)?;
    let communication_store = communication_ingestion_store(&state)?;
    let telegram_store = telegram_store(&state)?;
    let mail_store = mail_storage_store(&state)?;
    Ok(Json(
        state
            .telegram_runtime
            .download_media(
                TelegramMediaDownloadContext {
                    communication_store: &communication_store,
                    telegram_store: &telegram_store,
                    mail_store: &mail_store,
                    secret_store: &secret_store,
                    secret_resolver: &state.vault,
                    config: &state.config,
                },
                &request,
            )
            .await?,
    ))
}

pub(crate) async fn get_telegram_messages(
    State(state): State<AppState>,
    Query(query): Query<TelegramListQuery>,
) -> Result<Json<TelegramMessageListResponse>, ApiError> {
    let items = telegram_store(&state)?
        .recent_messages(
            query.account_id.as_deref(),
            query.provider_chat_id.as_deref(),
            query.limit.unwrap_or(50),
        )
        .await?;

    Ok(Json(TelegramMessageListResponse { items }))
}

fn actor_id_from_headers(headers: &HeaderMap) -> String {
    headers
        .get("x-hermes-actor-id")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("hermes-frontend")
        .to_owned()
}

fn telegram_secret_store(state: &AppState) -> Result<SecretReferenceStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    Ok(SecretReferenceStore::new(pool.clone()))
}
