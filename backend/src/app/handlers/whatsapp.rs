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
use crate::domains::communications::core::{
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

use crate::app::workflow_services::email_intelligence::{
    EmailIntelligenceError, EmailIntelligenceService,
};
use crate::app::workflow_services::provider_communication_projection::record_and_project_whatsapp_web_message;
use crate::app::workflow_services::review_inbox::{
    refresh_message_decisions_into_review, refresh_message_task_candidates_into_review,
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
use crate::domains::communications::messages::{
    MessageProjectionError, MessageProjectionStore, ProjectedMessage, ProjectedMessageSummary,
    WorkflowState,
};
use crate::domains::communications::storage::{
    CommunicationStorageError, CommunicationStorageStore, StoredCommunicationAttachmentWithBlob,
};
use crate::domains::documents::processing::{
    DocumentProcessingError, DocumentProcessingJob, DocumentProcessingRecord,
    DocumentProcessingRetryCommand, DocumentProcessingRetryCommandResult, DocumentProcessingStatus,
    DocumentProcessingStore,
};
use crate::domains::graph::core::{GraphNodeKind, node_id};
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
use crate::integrations::mail::accounts::{
    EmailAccountSetupError, EmailAccountSetupService, GmailOAuthPendingGrant,
    GmailOAuthSetupRequest, ImapAccountSetupRequest,
};
use crate::integrations::ollama::client::{OllamaClient, OllamaClientConfig};
use crate::integrations::telegram::client::{
    NewTelegramMessage, TelegramAccountSetupRequest, TelegramAccountSetupResponse, TelegramChat,
    TelegramError, TelegramMessage, TelegramMessageIngestResult, TelegramStore,
};
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

use crate::app::api_support::*;
use crate::app::{ApiError, AppState};

pub(crate) async fn get_whatsapp_capabilities(
    State(_state): State<AppState>,
) -> Result<Json<WhatsappCapabilitiesResponse>, ApiError> {
    Ok(Json(WhatsappCapabilitiesResponse::current()))
}

pub(crate) async fn post_whatsapp_fixture_account(
    State(state): State<AppState>,
    Json(request): Json<WhatsappWebAccountSetupRequest>,
) -> Result<Json<WhatsappWebAccountSetupResponse>, ApiError> {
    Ok(Json(
        whatsapp_web_store(&state)?
            .setup_fixture_account(&request)
            .await?,
    ))
}

pub(crate) async fn get_whatsapp_sessions(
    State(state): State<AppState>,
    Query(query): Query<WhatsappWebListQuery>,
) -> Result<Json<WhatsappWebSessionListResponse>, ApiError> {
    let items = whatsapp_web_store(&state)?
        .list_sessions(query.account_id.as_deref(), query.limit.unwrap_or(50))
        .await?;

    Ok(Json(WhatsappWebSessionListResponse { items }))
}

pub(crate) async fn post_whatsapp_fixture_message(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebMessage>,
) -> Result<Json<WhatsappWebMessageIngestResult>, ApiError> {
    let observed = whatsapp_web_store(&state)?
        .ingest_fixture_message(&request)
        .await?;
    let Some(pool) = state.database.pool().cloned() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let projected = record_and_project_whatsapp_web_message(pool.clone(), observed.raw).await?;
    let message_ids = vec![projected.message_id.clone()];
    refresh_message_decisions_into_review(&pool, &message_ids).await?;
    refresh_message_task_candidates_into_review(&pool, &message_ids).await?;
    Ok(Json(WhatsappWebMessageIngestResult {
        raw_record_id: projected.raw_record_id,
        message_id: projected.message_id,
    }))
}

pub(crate) async fn get_whatsapp_messages(
    State(state): State<AppState>,
    Query(query): Query<WhatsappWebListQuery>,
) -> Result<Json<WhatsappWebMessageListResponse>, ApiError> {
    let items = whatsapp_web_store(&state)?
        .recent_messages(
            query.account_id.as_deref(),
            query.provider_chat_id.as_deref(),
            query.limit.unwrap_or(50),
        )
        .await?;

    Ok(Json(WhatsappWebMessageListResponse { items }))
}
