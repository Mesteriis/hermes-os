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
use crate::workflows::email_intelligence::{EmailIntelligenceError, EmailIntelligenceService};

use crate::app::{ApiError, AppState};
use crate::domains::api_support::*;

pub(crate) async fn get_ai_status(
    State(state): State<AppState>,
) -> Result<Json<AiStatusResponse>, ApiError> {
    let runtime_settings = ai_runtime_settings(&state).await?;
    let ollama = ollama_client(&runtime_settings)?;
    let version = ollama.version().await;
    let tags = ollama.tags().await;
    let chat_model = runtime_settings.chat_model;
    let embedding_model = runtime_settings.embedding_model;
    let chat_model_available = tags
        .as_ref()
        .map(|models| models.iter().any(|model| model == &chat_model))
        .unwrap_or(false);
    let embedding_model_available = tags
        .as_ref()
        .map(|models| models.iter().any(|model| model == &embedding_model))
        .unwrap_or(false);

    Ok(Json(AiStatusResponse {
        runtime: "ollama".to_owned(),
        status: if version.is_ok() && chat_model_available && embedding_model_available {
            "ok"
        } else {
            "unavailable"
        }
        .to_owned(),
        version: version.ok(),
        chat_model,
        embedding_model,
        embedding_dimension: AI_EMBEDDING_DIMENSION,
        chat_model_available,
        embedding_model_available,
    }))
}

pub(crate) async fn get_ai_agents(
    State(state): State<AppState>,
) -> Result<Json<AiAgentListResponse>, ApiError> {
    let runtime_settings = ai_runtime_settings(&state).await?;

    Ok(Json(AiAgentListResponse {
        items: v3_agents(&runtime_settings.chat_model),
    }))
}

pub(crate) async fn get_ai_runs(
    State(state): State<AppState>,
    Query(query): Query<AiRunsQuery>,
) -> Result<Json<AiRunListResponse>, ApiError> {
    let limit = query.limit.unwrap_or(25).clamp(1, 100);
    let runs = ai_run_store(&state)?.list_runs(limit).await?;

    Ok(Json(AiRunListResponse { items: runs }))
}

pub(crate) async fn get_ai_run(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<AiAgentRun>, ApiError> {
    let Some(run) = ai_run_store(&state)?.get_run(&run_id).await? else {
        return Err(ApiError::AiRunNotFound);
    };

    Ok(Json(run))
}

pub(crate) async fn post_ai_answer(
    State(state): State<AppState>,
    Json(request): Json<AiAnswerRequest>,
) -> Result<Json<crate::ai::core::AiAnswerResponse>, ApiError> {
    let actor_id = "hermes-frontend".to_string();
    let service = ai_service(&state).await?;
    let response = service.answer(request, &actor_id).await?;

    Ok(Json(response))
}

pub(crate) async fn post_ai_task_candidates_refresh(
    State(state): State<AppState>,
    Json(request): Json<AiTaskCandidateRefreshRequest>,
) -> Result<Json<crate::ai::core::AiTaskCandidateRefreshResponse>, ApiError> {
    let actor_id = "hermes-frontend".to_string();
    let service = ai_service(&state).await?;
    let response = service.refresh_task_candidates(request, &actor_id).await?;

    Ok(Json(response))
}

pub(crate) async fn post_ai_meeting_prep(
    State(state): State<AppState>,
    Json(request): Json<AiMeetingPrepRequest>,
) -> Result<Json<crate::ai::core::AiMeetingPrepResponse>, ApiError> {
    let actor_id = "hermes-frontend".to_string();
    let service = ai_service(&state).await?;
    let response = service.meeting_prep(request, &actor_id).await?;

    Ok(Json(response))
}
