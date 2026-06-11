// ADR-0073: shared DTO conversion helpers are grouped here for the first
// handlers.rs split; follow-up work should move helpers closer to each domain.
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
use crate::platform::config::{AiRuntimeProvider, AppConfig};

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
use crate::integrations::ai_runtime::{AiRuntimeClient, AiRuntimeError};
use crate::integrations::ollama::client::{OllamaClient, OllamaClientConfig};
use crate::integrations::omniroute::client::{
    OmniRouteClient, OmniRouteClientConfig, OmniRouteError,
};
use crate::integrations::telegram::client::{
    NewTelegramMessage, TelegramAccountSetupRequest, TelegramAccountSetupResponse, TelegramChat,
    TelegramError, TelegramMessage, TelegramMessageIngestResult, TelegramStore,
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
use crate::vault::VaultStatus;
use crate::workflows::email_intelligence::{EmailIntelligenceError, EmailIntelligenceService};

use crate::app::{ApiError, AppState};

pub(crate) fn event_store(state: &AppState) -> Result<EventStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(EventStore::new(pool.clone()))
}

pub(crate) fn graph_store(
    state: &AppState,
) -> Result<crate::domains::graph::core::GraphStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(crate::domains::graph::core::GraphStore::new(pool.clone()))
}

pub(crate) fn message_store(state: &AppState) -> Result<MessageProjectionStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(MessageProjectionStore::new(pool.clone()))
}

pub(crate) fn mail_storage_store(state: &AppState) -> Result<MailStorageStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(MailStorageStore::new(pool.clone()))
}

pub(crate) fn communication_ingestion_store(
    state: &AppState,
) -> Result<CommunicationIngestionStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(CommunicationIngestionStore::new(pool.clone()))
}

pub(crate) fn project_store(state: &AppState) -> Result<ProjectStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ProjectStore::new(pool.clone()))
}

pub(crate) fn project_link_review_store(
    state: &AppState,
) -> Result<ProjectLinkReviewStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ProjectLinkReviewStore::new(pool.clone()))
}

pub(crate) fn task_candidate_store(state: &AppState) -> Result<TaskCandidateStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(TaskCandidateStore::new(pool.clone()))
}

pub(crate) fn ai_run_store(state: &AppState) -> Result<crate::ai::core::AiRunStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(crate::ai::core::AiRunStore::new(pool.clone()))
}

pub(crate) async fn ai_service(state: &AppState) -> Result<AiService, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let runtime_settings = ai_runtime_settings(state).await?;
    let runtime = ai_runtime_client(state, &runtime_settings)?;

    Ok(AiService::new(
        pool.clone(),
        runtime,
        &runtime_settings.chat_model,
        &runtime_settings.embedding_model,
    ))
}

pub(crate) fn telegram_store(state: &AppState) -> Result<TelegramStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(TelegramStore::new(pool.clone()))
}

pub(crate) fn whatsapp_web_store(state: &AppState) -> Result<WhatsappWebStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(WhatsappWebStore::new(pool.clone()))
}

pub(crate) fn automation_store(state: &AppState) -> Result<AutomationStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(AutomationStore::new(pool.clone()))
}

pub(crate) fn call_intelligence_store(state: &AppState) -> Result<CallIntelligenceStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(CallIntelligenceStore::new(pool.clone()))
}

pub(crate) fn settings_store(state: &AppState) -> Result<ApplicationSettingsStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ApplicationSettingsStore::new(pool.clone()))
}

pub(crate) async fn ai_runtime_settings(state: &AppState) -> Result<AiRuntimeSettings, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Ok(AiRuntimeSettings::from_config(&state.config));
    };

    Ok(ApplicationSettingsStore::new(pool.clone())
        .ai_runtime_settings(&state.config)
        .await?)
}

pub(crate) fn ai_runtime_client(
    state: &AppState,
    settings: &AiRuntimeSettings,
) -> Result<AiRuntimeClient, ApiError> {
    match settings.provider {
        AiRuntimeProvider::Ollama => Ok(AiRuntimeClient::Ollama(OllamaClient::new(
            OllamaClientConfig::new(
                &settings.base_url,
                &settings.chat_model,
                &settings.embedding_model,
            )
            .with_timeout_seconds(settings.timeout_seconds),
        )?)),
        AiRuntimeProvider::OmniRoute => {
            let api_key = state.config.omniroute_api_key().cloned().ok_or_else(|| {
                ApiError::Ai(AiError::Runtime(AiRuntimeError::OmniRoute(
                    OmniRouteError::MissingApiKey,
                )))
            })?;
            Ok(AiRuntimeClient::OmniRoute(OmniRouteClient::new(
                OmniRouteClientConfig::new(
                    &settings.base_url,
                    &settings.chat_model,
                    &settings.embedding_model,
                    api_key,
                )
                .with_timeout_seconds(settings.timeout_seconds),
            )?))
        }
    }
}

pub(crate) fn document_processing_store(
    state: &AppState,
) -> Result<DocumentProcessingStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(DocumentProcessingStore::new(pool.clone()))
}

pub(crate) fn person_identity_store(state: &AppState) -> Result<PersonIdentityStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(PersonIdentityStore::new(pool.clone()))
}

pub(crate) async fn email_multilingual_service(
    state: &AppState,
) -> Result<crate::domains::mail::multilingual::MultilingualService, ApiError> {
    let settings = ai_runtime_settings(state).await?;
    Ok(
        crate::domains::mail::multilingual::MultilingualService::new(
            ai_runtime_client(state, &settings).ok(),
        ),
    )
}

pub(crate) async fn email_ai_reply_service(
    state: &AppState,
) -> Result<crate::domains::mail::ai_reply::AiReplyService, ApiError> {
    let settings = ai_runtime_settings(state).await?;
    Ok(crate::domains::mail::ai_reply::AiReplyService::new(
        ai_runtime_client(state, &settings).ok(),
    ))
}

pub(crate) fn api_audit_log(state: &AppState) -> Result<ApiAuditLog, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ApiAuditLog::new(pool.clone()))
}

pub(crate) fn account_setup_service(
    state: &AppState,
) -> Result<EmailAccountSetupService, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(EmailAccountSetupService::new_with_host_vault(
        CommunicationIngestionStore::new(pool.clone()),
        SecretReferenceStore::new(pool.clone()),
        state.vault.clone(),
    ))
}

pub(crate) fn database_encrypted_vault(
    config: &AppConfig,
    pool: sqlx::postgres::PgPool,
) -> Option<DatabaseEncryptedSecretVault> {
    Some(DatabaseEncryptedSecretVault::new(
        pool,
        config.secret_vault_key()?.clone(),
    ))
}

#[derive(Serialize)]
pub(crate) struct ApplicationSettingsResponse {
    pub(crate) items: Vec<ApplicationSetting>,
}

#[derive(Serialize)]
pub(crate) struct ApplicationAccountsResponse {
    pub(crate) items: Vec<ProviderAccount>,
}

#[derive(Deserialize)]
pub(crate) struct ApplicationSettingUpdateRequest {
    pub(crate) value: Value,
}

#[derive(Deserialize)]
pub(crate) struct AppendEventRequest {
    pub(crate) event_id: String,
    pub(crate) event_type: String,
    #[serde(default = "default_schema_version")]
    pub(crate) schema_version: i32,
    pub(crate) occurred_at: DateTime<Utc>,
    pub(crate) source: Value,
    pub(crate) actor: Option<Value>,
    pub(crate) subject: Value,
    #[serde(default = "empty_json_object")]
    pub(crate) payload: Value,
    #[serde(default = "empty_json_object")]
    pub(crate) provenance: Value,
    pub(crate) causation_id: Option<String>,
    pub(crate) correlation_id: Option<String>,
}

impl AppendEventRequest {
    pub(crate) fn into_new_event(self) -> Result<NewEventEnvelope, EventEnvelopeError> {
        let mut builder = NewEventEnvelope::builder(
            self.event_id,
            self.event_type,
            self.occurred_at,
            self.source,
            self.subject,
        )
        .schema_version(self.schema_version)
        .payload(self.payload)
        .provenance(self.provenance);

        if let Some(actor) = self.actor {
            builder = builder.actor(actor);
        }

        if let Some(causation_id) = self.causation_id {
            builder = builder.causation_id(causation_id);
        }

        if let Some(correlation_id) = self.correlation_id {
            builder = builder.correlation_id(correlation_id);
        }

        builder.build()
    }
}

#[derive(Serialize)]
pub(crate) struct AppendEventResponse {
    pub(crate) event_id: String,
    pub(crate) position: i64,
}

#[derive(Deserialize)]
pub(crate) struct AuditEventsQuery {
    pub(crate) target_id: Option<String>,
    pub(crate) actor_id: Option<String>,
    pub(crate) after_audit_id: Option<i64>,
    pub(crate) limit: Option<u32>,
}

#[derive(Serialize)]
pub(crate) struct AuditEventsResponse {
    pub(crate) items: Vec<ApiAuditRecord>,
}

#[derive(Serialize)]
pub(crate) struct V1StatusResponse {
    pub(crate) version: &'static str,
    pub(crate) surfaces: V1Surfaces,
    pub(crate) vault_status: VaultStatus,
}

#[derive(Serialize)]
pub(crate) struct V1Surfaces {
    pub(crate) messages: bool,
    pub(crate) persons: bool,
    pub(crate) search: bool,
    pub(crate) documents: bool,
    pub(crate) account_setup: bool,
}

#[derive(Serialize)]
pub(crate) struct CommunicationMessagesResponse {
    pub(crate) items: Vec<CommunicationMessageSummaryResponse>,
}

#[derive(Serialize)]
pub(crate) struct CommunicationMessageSummaryResponse {
    pub(crate) message_id: String,
    pub(crate) raw_record_id: String,
    pub(crate) account_id: String,
    pub(crate) provider_record_id: String,
    pub(crate) subject: String,
    pub(crate) sender: String,
    pub(crate) recipients: Vec<String>,
    pub(crate) body_text_preview: String,
    pub(crate) occurred_at: Option<DateTime<Utc>>,
    pub(crate) projected_at: DateTime<Utc>,
    pub(crate) channel_kind: String,
    pub(crate) conversation_id: Option<String>,
    pub(crate) sender_display_name: Option<String>,
    pub(crate) delivery_state: String,
    pub(crate) message_metadata: Value,
    pub(crate) attachment_count: i64,
    pub(crate) local_state: String,
    pub(crate) local_state_changed_at: Option<DateTime<Utc>>,
}

impl From<ProjectedMessageSummary> for CommunicationMessageSummaryResponse {
    fn from(summary: ProjectedMessageSummary) -> Self {
        Self {
            message_id: summary.message.message_id,
            raw_record_id: summary.message.raw_record_id,
            account_id: summary.message.account_id,
            provider_record_id: summary.message.provider_record_id,
            subject: summary.message.subject,
            sender: summary.message.sender,
            recipients: summary.message.recipients,
            body_text_preview: text_preview(&summary.message.body_text, 240),
            occurred_at: summary.message.occurred_at,
            projected_at: summary.message.projected_at,
            channel_kind: summary.message.channel_kind,
            conversation_id: summary.message.conversation_id,
            sender_display_name: summary.message.sender_display_name,
            delivery_state: summary.message.delivery_state,
            message_metadata: summary.message.message_metadata,
            attachment_count: summary.attachment_count,
            local_state: summary.message.local_state.as_str().to_owned(),
            local_state_changed_at: summary.message.local_state_changed_at,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct CommunicationMessageDetailResponse {
    pub(crate) message: CommunicationMessageDetailItem,
    pub(crate) attachments: Vec<CommunicationAttachmentResponse>,
}

#[derive(Serialize)]
pub(crate) struct CommunicationMessageDetailItem {
    pub(crate) message_id: String,
    pub(crate) raw_record_id: String,
    pub(crate) account_id: String,
    pub(crate) provider_record_id: String,
    pub(crate) subject: String,
    pub(crate) sender: String,
    pub(crate) recipients: Vec<String>,
    pub(crate) body_text: String,
    pub(crate) body_html: Option<String>,
    pub(crate) occurred_at: Option<DateTime<Utc>>,
    pub(crate) projected_at: DateTime<Utc>,
    pub(crate) channel_kind: String,
    pub(crate) conversation_id: Option<String>,
    pub(crate) sender_display_name: Option<String>,
    pub(crate) delivery_state: String,
    pub(crate) message_metadata: Value,
    pub(crate) local_state: String,
    pub(crate) local_state_changed_at: Option<DateTime<Utc>>,
    pub(crate) local_state_reason: Option<String>,
}

impl CommunicationMessageDetailItem {
    pub(crate) fn from_message(message: ProjectedMessage, body_html: Option<String>) -> Self {
        let message_metadata = message.message_metadata.clone();
        Self::from_message_with_metadata(message, body_html, message_metadata)
    }

    pub(crate) fn from_message_with_metadata(
        message: ProjectedMessage,
        body_html: Option<String>,
        message_metadata: Value,
    ) -> Self {
        Self {
            message_id: message.message_id,
            raw_record_id: message.raw_record_id,
            account_id: message.account_id,
            provider_record_id: message.provider_record_id,
            subject: message.subject,
            sender: message.sender,
            recipients: message.recipients,
            body_text: message.body_text,
            body_html,
            occurred_at: message.occurred_at,
            projected_at: message.projected_at,
            channel_kind: message.channel_kind,
            conversation_id: message.conversation_id,
            sender_display_name: message.sender_display_name,
            delivery_state: message.delivery_state,
            message_metadata,
            local_state: message.local_state.as_str().to_owned(),
            local_state_changed_at: message.local_state_changed_at,
            local_state_reason: message.local_state_reason,
        }
    }
}

impl From<ProjectedMessage> for CommunicationMessageDetailItem {
    fn from(message: ProjectedMessage) -> Self {
        Self::from_message(message, None)
    }
}

#[derive(Serialize)]
pub(crate) struct CommunicationAttachmentResponse {
    pub(crate) attachment_id: String,
    pub(crate) message_id: String,
    pub(crate) raw_record_id: String,
    pub(crate) blob_id: String,
    pub(crate) provider_attachment_id: String,
    pub(crate) filename: Option<String>,
    pub(crate) content_type: String,
    pub(crate) size_bytes: i64,
    pub(crate) sha256: String,
    pub(crate) disposition: &'static str,
    pub(crate) scan_status: &'static str,
    pub(crate) scan_engine: Option<String>,
    pub(crate) scan_checked_at: Option<DateTime<Utc>>,
    pub(crate) scan_summary: Option<String>,
    pub(crate) scan_metadata: Value,
    pub(crate) storage_kind: String,
    pub(crate) storage_path: String,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
}

impl From<StoredMailAttachmentWithBlob> for CommunicationAttachmentResponse {
    fn from(record: StoredMailAttachmentWithBlob) -> Self {
        let attachment = record.attachment;
        Self {
            attachment_id: attachment.attachment_id,
            message_id: attachment.message_id,
            raw_record_id: attachment.raw_record_id,
            blob_id: attachment.blob_id,
            provider_attachment_id: attachment.provider_attachment_id,
            filename: attachment.filename,
            content_type: attachment.content_type,
            size_bytes: attachment.size_bytes,
            sha256: attachment.sha256,
            disposition: attachment.disposition.as_str(),
            scan_status: attachment.scan_status.as_str(),
            scan_engine: attachment.scan_engine,
            scan_checked_at: attachment.scan_checked_at,
            scan_summary: attachment.scan_summary,
            scan_metadata: attachment.scan_metadata,
            storage_kind: record.storage_kind,
            storage_path: record.storage_path,
            created_at: attachment.created_at,
            updated_at: attachment.updated_at,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct ProjectLinkCandidate {
    pub(crate) project_id: String,
    pub(crate) target_kind: String,
    pub(crate) target_id: String,
    pub(crate) graph_node_id: String,
    pub(crate) title: String,
    pub(crate) subtitle: String,
    pub(crate) source_label: String,
    pub(crate) occurred_at: DateTime<Utc>,
    pub(crate) review_state: String,
    pub(crate) evidence_excerpt: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct ProjectLinkCandidateListResponse {
    pub(crate) items: Vec<ProjectLinkCandidate>,
}

#[derive(Serialize)]
pub(crate) struct TaskCandidateListResponse {
    pub(crate) items: Vec<TaskCandidate>,
}

#[derive(Deserialize)]
pub(crate) struct AiRunsQuery {
    pub(crate) limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct AiRunListResponse {
    pub(crate) items: Vec<AiAgentRun>,
}

#[derive(Serialize)]
pub(crate) struct TelegramCapabilitiesResponse {
    pub(crate) version: &'static str,
    pub(crate) runtime_mode: &'static str,
    pub(crate) telegram_app_credentials_configured: bool,
    pub(crate) tdjson_runtime_available: bool,
    pub(crate) qr_login_ready: bool,
    pub(crate) capabilities: Vec<TelegramCapabilityStatus>,
    pub(crate) unsupported_features: Vec<&'static str>,
}

impl TelegramCapabilitiesResponse {
    pub(crate) fn current(config: &AppConfig) -> Self {
        let telegram_app_credentials_configured =
            config.telegram_api_id().is_some() && config.telegram_api_hash().is_some();
        let tdjson_runtime_available = tdjson::runtime_available(config.tdjson_path());
        let qr_login_ready = telegram_app_credentials_configured && tdjson_runtime_available;

        Self {
            version: "1.0",
            runtime_mode: if qr_login_ready {
                "tdlib_qr"
            } else {
                "fixture"
            },
            telegram_app_credentials_configured,
            tdjson_runtime_available,
            qr_login_ready,
            capabilities: vec![
                TelegramCapabilityStatus::available(
                    "telegram_fixture_runtime",
                    "Fixture Telegram accounts, chats and message projection are available for CI and local smoke validation.",
                    true,
                ),
                if qr_login_ready {
                    TelegramCapabilityStatus::available(
                        "tdlib_live_runtime",
                        "TDLib QR login runtime is configured for local development.",
                        true,
                    )
                } else {
                    TelegramCapabilityStatus::blocked(
                        "tdlib_live_runtime",
                        "Live TDLib sessions require a loadable native TDLib JSON runtime and Telegram app credentials.",
                        false,
                    )
                },
                TelegramCapabilityStatus::blocked(
                    "telegram_bot_live_runtime",
                    "Live bot sends require the Bot API runtime adapter and account-scoped bot token resolution.",
                    false,
                ),
                TelegramCapabilityStatus::available(
                    "automation_dry_run",
                    "Policy/template validation and audited dry-run records are available.",
                    true,
                ),
                TelegramCapabilityStatus::blocked(
                    "automation_live_send",
                    "Live automated sends remain blocked until a live Telegram runtime passes the same policy evaluator and audit contract.",
                    false,
                ),
                TelegramCapabilityStatus::available(
                    "call_state_and_transcript_storage",
                    "1:1 call metadata and transcript artifact storage are available through fixture APIs.",
                    true,
                ),
                TelegramCapabilityStatus::blocked(
                    "desktop_audio_capture",
                    "Desktop audio capture requires a visible recording runtime boundary and explicit platform permissions.",
                    false,
                ),
                TelegramCapabilityStatus::available(
                    "fixture_speech_to_text",
                    "Fixture speech-to-text provider is available for deterministic tests.",
                    true,
                ),
                TelegramCapabilityStatus::blocked(
                    "whisper_rs_speech_to_text",
                    "Real local transcription requires the whisper-rs provider adapter and local model configuration.",
                    false,
                ),
            ],
            unsupported_features: vec![
                "video_calls",
                "group_calls",
                "screen_sharing",
                "hidden_recording",
                "telegram_data_fine_tuning",
                "third_party_plugin_execution",
            ],
        }
    }
}

#[derive(Serialize)]
pub(crate) struct TelegramCapabilityStatus {
    pub(crate) capability: &'static str,
    pub(crate) status: &'static str,
    pub(crate) closure_gate: bool,
    pub(crate) reason: &'static str,
}

impl TelegramCapabilityStatus {
    fn available(capability: &'static str, reason: &'static str, closure_gate: bool) -> Self {
        Self {
            capability,
            status: "available",
            closure_gate,
            reason,
        }
    }

    fn blocked(capability: &'static str, reason: &'static str, closure_gate: bool) -> Self {
        Self {
            capability,
            status: "blocked",
            closure_gate,
            reason,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct WhatsappCapabilitiesResponse {
    pub(crate) version: &'static str,
    pub(crate) runtime_mode: &'static str,
    pub(crate) capabilities: Vec<WhatsappCapabilityStatus>,
    pub(crate) unsupported_features: Vec<&'static str>,
}

impl WhatsappCapabilitiesResponse {
    pub(crate) fn current() -> Self {
        Self {
            version: "1.0",
            runtime_mode: "fixture",
            capabilities: vec![
                WhatsappCapabilityStatus::available(
                    "whatsapp_web_fixture_runtime",
                    "Fixture WhatsApp Web accounts, session metadata and message projection are available for CI and local smoke validation.",
                    true,
                ),
                WhatsappCapabilityStatus::available(
                    "whatsapp_web_manual_session_state",
                    "Manual companion session metadata is stored without session secrets or pairing material in PostgreSQL.",
                    true,
                ),
                WhatsappCapabilityStatus::available(
                    "whatsapp_web_fixture_ingestion",
                    "Fixture WhatsApp Web messages preserve append-only raw provenance and project into canonical communication messages.",
                    true,
                ),
                WhatsappCapabilityStatus::blocked(
                    "whatsapp_web_live_runtime",
                    "Live WhatsApp Web requires a user-visible desktop runtime, explicit session lifecycle and smoke validation.",
                    false,
                ),
                WhatsappCapabilityStatus::blocked(
                    "whatsapp_web_live_send",
                    "Live outbound sends require a WhatsApp-specific policy, audit and visible runtime contract.",
                    false,
                ),
            ],
            unsupported_features: vec![
                "hidden_web_scraping",
                "reverse_engineered_protocol_runtime",
                "bulk_messaging",
                "auto_messaging",
                "auto_dialing",
                "whatsapp_data_fine_tuning",
                "whatsapp_business_cloud_as_personal_provider",
            ],
        }
    }
}

#[derive(Serialize)]
pub(crate) struct WhatsappCapabilityStatus {
    pub(crate) capability: &'static str,
    pub(crate) status: &'static str,
    pub(crate) closure_gate: bool,
    pub(crate) reason: &'static str,
}

impl WhatsappCapabilityStatus {
    fn available(capability: &'static str, reason: &'static str, closure_gate: bool) -> Self {
        Self {
            capability,
            status: "available",
            closure_gate,
            reason,
        }
    }

    fn blocked(capability: &'static str, reason: &'static str, closure_gate: bool) -> Self {
        Self {
            capability,
            status: "blocked",
            closure_gate,
            reason,
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct TelegramListQuery {
    pub(crate) account_id: Option<String>,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct TelegramChatListResponse {
    pub(crate) items: Vec<TelegramChat>,
}

#[derive(Serialize)]
pub(crate) struct TelegramMessageListResponse {
    pub(crate) items: Vec<TelegramMessage>,
}

#[derive(Deserialize)]
pub(crate) struct WhatsappWebListQuery {
    pub(crate) account_id: Option<String>,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct WhatsappWebSessionListResponse {
    pub(crate) items: Vec<WhatsappWebSession>,
}

#[derive(Serialize)]
pub(crate) struct WhatsappWebMessageListResponse {
    pub(crate) items: Vec<WhatsappWebMessage>,
}

#[derive(Deserialize)]
pub(crate) struct PolicyTemplateApiRequest {
    pub(crate) template_id: String,
    pub(crate) name: String,
    pub(crate) body_template: String,
    #[serde(default)]
    pub(crate) required_variables: Vec<String>,
}

impl PolicyTemplateApiRequest {
    pub(crate) fn into_template(self) -> NewAutomationTemplate {
        NewAutomationTemplate {
            template_id: self.template_id,
            name: self.name,
            body_template: self.body_template,
            required_variables: self.required_variables,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct PolicyTemplateListResponse {
    pub(crate) items: Vec<AutomationTemplate>,
}

#[derive(Deserialize)]
pub(crate) struct PolicyApiRequest {
    pub(crate) policy_id: String,
    pub(crate) template_id: String,
    pub(crate) name: String,
    pub(crate) enabled: bool,
    pub(crate) account_id: String,
    pub(crate) allowed_chat_ids: Vec<String>,
    pub(crate) trigger_kind: String,
    pub(crate) max_sends_per_hour: i32,
    #[serde(default = "empty_json_object")]
    pub(crate) quiet_hours: Value,
    pub(crate) expires_at: Option<DateTime<Utc>>,
    #[serde(default = "empty_json_object")]
    pub(crate) conditions: Value,
}

impl PolicyApiRequest {
    pub(crate) fn into_policy(self) -> NewAutomationPolicy {
        NewAutomationPolicy {
            policy_id: self.policy_id,
            template_id: self.template_id,
            name: self.name,
            enabled: self.enabled,
            account_id: self.account_id,
            allowed_chat_ids: self.allowed_chat_ids,
            trigger_kind: self.trigger_kind,
            max_sends_per_hour: self.max_sends_per_hour,
            quiet_hours: self.quiet_hours,
            expires_at: self.expires_at,
            conditions: self.conditions,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct PolicyListResponse {
    pub(crate) items: Vec<AutomationPolicy>,
}

#[derive(Deserialize)]
pub(crate) struct CallApiRequest {
    pub(crate) call_id: String,
    pub(crate) account_id: String,
    pub(crate) provider_call_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) direction: CallDirection,
    pub(crate) call_state: CallState,
    pub(crate) started_at: Option<DateTime<Utc>>,
    pub(crate) ended_at: Option<DateTime<Utc>>,
    pub(crate) transcription_policy_id: Option<String>,
    #[serde(default = "empty_json_object")]
    pub(crate) metadata: Value,
}

impl CallApiRequest {
    pub(crate) fn into_call(self) -> NewTelegramCall {
        NewTelegramCall {
            call_id: self.call_id,
            account_id: self.account_id,
            provider_call_id: self.provider_call_id,
            provider_chat_id: self.provider_chat_id,
            direction: self.direction,
            call_state: self.call_state,
            started_at: self.started_at,
            ended_at: self.ended_at,
            transcription_policy_id: self.transcription_policy_id,
            metadata: self.metadata,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct CallListResponse {
    pub(crate) items: Vec<TelegramCall>,
}

#[derive(Deserialize)]
pub(crate) struct CallTranscriptFixtureApiRequest {
    pub(crate) transcript_id: String,
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) source_audio_ref: String,
    pub(crate) language_code: Option<String>,
    #[serde(default)]
    pub(crate) always_on_policy: bool,
}

#[derive(Serialize)]
pub(crate) struct CallTranscriptResponse {
    pub(crate) transcript: Option<CallTranscript>,
}

#[derive(Serialize)]
pub(crate) struct PersonIdentityCandidateListResponse {
    pub(crate) items: Vec<PersonIdentityCandidate>,
}

#[derive(Deserialize)]
pub(crate) struct PersonIdentityReviewApiRequest {
    pub(crate) command_id: String,
    pub(crate) review_state: String,
}

impl PersonIdentityReviewApiRequest {
    pub(crate) fn into_command(
        self,
        identity_candidate_id: String,
        actor_id: String,
    ) -> Result<PersonIdentityReviewCommand, ApiError> {
        let command_id = validate_non_empty_person_identity_field("command_id", &self.command_id)?;
        let identity_candidate_id = validate_non_empty_person_identity_field(
            "identity_candidate_id",
            &identity_candidate_id,
        )?;
        let review_state = parse_person_identity_review_state(&self.review_state)?;

        Ok(PersonIdentityReviewCommand {
            command_id,
            identity_candidate_id,
            review_state,
            actor_id,
        })
    }
}

#[derive(Serialize)]
pub(crate) struct PersonIdentityReviewApiResponse {
    pub(crate) identity_candidate_id: String,
    pub(crate) review_state: String,
    pub(crate) event_id: String,
}

impl From<crate::domains::persons::identity::PersonIdentityReviewCommandResult>
    for PersonIdentityReviewApiResponse
{
    fn from(result: crate::domains::persons::identity::PersonIdentityReviewCommandResult) -> Self {
        Self {
            identity_candidate_id: result.identity_candidate_id,
            review_state: result.review_state.as_str().to_owned(),
            event_id: result.event_id,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct DocumentProcessingJobsResponse {
    pub(crate) items: Vec<DocumentProcessingJob>,
}

#[derive(Deserialize)]
pub(crate) struct DocumentProcessingRetryApiRequest {
    pub(crate) command_id: String,
}

impl DocumentProcessingRetryApiRequest {
    pub(crate) fn into_command(
        self,
        job_id: String,
        actor_id: String,
    ) -> Result<DocumentProcessingRetryCommand, ApiError> {
        let command_id =
            validate_non_empty_document_processing_field("command_id", &self.command_id)?;
        let job_id = validate_non_empty_document_processing_field("job_id", &job_id)?;

        Ok(DocumentProcessingRetryCommand {
            command_id,
            job_id,
            actor_id,
        })
    }
}

#[derive(Serialize)]
pub(crate) struct DocumentProcessingRetryApiResponse {
    pub(crate) job_id: String,
    pub(crate) status: DocumentProcessingStatus,
    pub(crate) event_id: String,
}

impl From<DocumentProcessingRetryCommandResult> for DocumentProcessingRetryApiResponse {
    fn from(result: DocumentProcessingRetryCommandResult) -> Self {
        Self {
            job_id: result.job_id,
            status: result.status,
            event_id: result.event_id,
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct TaskCandidateReviewApiRequest {
    pub(crate) command_id: String,
    pub(crate) review_state: String,
}

impl TaskCandidateReviewApiRequest {
    pub(crate) fn into_command(
        self,
        task_candidate_id: String,
        actor_id: String,
    ) -> Result<TaskCandidateReviewCommand, ApiError> {
        let command_id = validate_non_empty_task_candidate_field("command_id", &self.command_id)?;
        let task_candidate_id =
            validate_non_empty_task_candidate_field("task_candidate_id", &task_candidate_id)?;
        let review_state = parse_task_candidate_review_state(&self.review_state)?;

        Ok(TaskCandidateReviewCommand {
            command_id,
            task_candidate_id,
            review_state,
            actor_id,
        })
    }
}

#[derive(Serialize)]
pub(crate) struct TaskCandidateReviewApiResponse {
    pub(crate) task_candidate_id: String,
    pub(crate) review_state: String,
    pub(crate) event_id: String,
}

impl From<crate::domains::tasks::candidates::TaskCandidateReviewCommandResult>
    for TaskCandidateReviewApiResponse
{
    fn from(result: crate::domains::tasks::candidates::TaskCandidateReviewCommandResult) -> Self {
        Self {
            task_candidate_id: result.task_candidate_id,
            review_state: result.review_state.as_str().to_owned(),
            event_id: result.event_id,
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct ProjectLinkReviewApiRequest {
    pub(crate) command_id: String,
    pub(crate) target_kind: String,
    pub(crate) target_id: String,
    pub(crate) review_state: String,
}

impl ProjectLinkReviewApiRequest {
    pub(crate) fn into_command(
        self,
        project_id: String,
        actor_id: String,
    ) -> Result<ProjectLinkReviewCommand, ApiError> {
        let command_id = validate_non_empty_project_link_field("command_id", &self.command_id)?;
        let project_id = validate_non_empty_project_link_field("project_id", &project_id)?;
        let target_id = validate_non_empty_project_link_field("target_id", &self.target_id)?;
        let target_kind = parse_project_link_target_kind(&self.target_kind)?;
        let review_state = parse_project_link_review_state(&self.review_state)?;

        Ok(ProjectLinkReviewCommand {
            command_id,
            project_id,
            target_kind,
            target_id,
            review_state,
            actor_id,
        })
    }
}

#[derive(Serialize)]
pub(crate) struct ProjectLinkReviewApiResponse {
    pub(crate) project_id: String,
    pub(crate) target_kind: String,
    pub(crate) target_id: String,
    pub(crate) review_state: String,
    pub(crate) event_id: String,
}

impl From<crate::domains::projects::link_reviews::ProjectLinkReviewCommandResult>
    for ProjectLinkReviewApiResponse
{
    fn from(
        result: crate::domains::projects::link_reviews::ProjectLinkReviewCommandResult,
    ) -> Self {
        Self {
            project_id: result.project_id,
            target_kind: result.target_kind.as_str().to_owned(),
            target_id: result.target_id,
            review_state: result.review_state.as_str().to_owned(),
            event_id: result.event_id,
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct ProjectLinkCandidatesQuery {
    pub(crate) limit: Option<usize>,
}

pub(crate) struct TaskCandidatesQuery {
    pub(crate) limit: Option<i64>,
}

pub(crate) struct DocumentProcessingJobsQuery {
    pub(crate) limit: Option<i64>,
}

pub(crate) struct CommunicationMessagesQuery {
    pub(crate) account_id: Option<String>,
    pub(crate) workflow_state: Option<String>,
    pub(crate) channel_kind: Option<String>,
    pub(crate) q: Option<String>,
    pub(crate) local_state: Option<String>,
    pub(crate) limit: Option<i64>,
}

pub(crate) struct GraphNeighborhoodQuery {
    pub(crate) node_id: Option<String>,
    pub(crate) depth: Option<u8>,
}

pub(crate) struct GraphNodesQuery {
    pub(crate) limit: Option<i64>,
}

pub(crate) struct GraphSearchQuery {
    pub(crate) q: Option<String>,
    pub(crate) limit: Option<i64>,
}

pub(crate) struct ProjectsQuery {
    pub(crate) limit: Option<i64>,
}

pub(crate) fn parse_communication_messages_query(
    raw_query: Option<&str>,
) -> Result<CommunicationMessagesQuery, ApiError> {
    let mut query = CommunicationMessagesQuery {
        account_id: None,
        workflow_state: None,
        channel_kind: None,
        q: None,
        local_state: None,
        limit: None,
    };

    if let Some(raw_query) = raw_query {
        for (key, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            match key.as_ref() {
                "account_id" => query.account_id = non_empty_query_value(value.as_ref()),
                "workflow_state" => query.workflow_state = non_empty_query_value(value.as_ref()),
                "channel_kind" => query.channel_kind = non_empty_query_value(value.as_ref()),
                "q" => query.q = non_empty_query_value(value.as_ref()),
                "local_state" => query.local_state = non_empty_query_value(value.as_ref()),
                "limit" => {
                    query.limit = Some(value.parse::<i64>().map_err(|_| {
                        ApiError::InvalidCommunicationQuery("limit must be an integer")
                    })?);
                }
                _ => {}
            }
        }
    }

    Ok(query)
}

fn non_empty_query_value(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

pub(crate) fn parse_graph_neighborhood_query(
    raw_query: Option<&str>,
) -> Result<GraphNeighborhoodQuery, ApiError> {
    let mut query = GraphNeighborhoodQuery {
        node_id: None,
        depth: None,
    };

    if let Some(raw_query) = raw_query {
        for (key, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            match key.as_ref() {
                "node_id" => query.node_id = Some(value.into_owned()),
                "depth" => {
                    query.depth = Some(
                        value
                            .parse::<u8>()
                            .map_err(|_| ApiError::InvalidGraphQuery("depth supports only 1"))?,
                    );
                }
                _ => {}
            }
        }
    }

    Ok(query)
}

pub(crate) fn parse_graph_nodes_query(
    raw_query: Option<&str>,
) -> Result<GraphNodesQuery, ApiError> {
    let mut query = GraphNodesQuery { limit: None };

    if let Some(raw_query) = raw_query {
        for (key, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            if key.as_ref() == "limit" {
                query.limit = Some(
                    value
                        .parse::<i64>()
                        .map_err(|_| ApiError::InvalidGraphQuery("limit must be an integer"))?,
                );
            }
        }
    }

    Ok(query)
}

pub(crate) fn parse_graph_search_query(
    raw_query: Option<&str>,
) -> Result<GraphSearchQuery, ApiError> {
    let mut query = GraphSearchQuery {
        q: None,
        limit: None,
    };

    if let Some(raw_query) = raw_query {
        for (key, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            match key.as_ref() {
                "q" => query.q = Some(value.into_owned()),
                "limit" => {
                    query.limit =
                        Some(value.parse::<i64>().map_err(|_| {
                            ApiError::InvalidGraphQuery("limit must be an integer")
                        })?);
                }
                _ => {}
            }
        }
    }

    Ok(query)
}

pub(crate) fn parse_projects_query(raw_query: Option<&str>) -> Result<ProjectsQuery, ApiError> {
    let mut query = ProjectsQuery { limit: None };

    if let Some(raw_query) = raw_query {
        for (key, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            if key.as_ref() == "limit" {
                query.limit = Some(
                    value
                        .parse::<i64>()
                        .map_err(|_| ApiError::InvalidProjectQuery("limit must be an integer"))?,
                );
            }
        }
    }

    Ok(query)
}

pub(crate) fn parse_project_link_candidates_query(
    raw_query: Option<&str>,
) -> Result<ProjectLinkCandidatesQuery, ApiError> {
    let mut query = ProjectLinkCandidatesQuery { limit: None };

    if let Some(raw_query) = raw_query {
        for (key, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            if key.as_ref() == "limit" {
                query.limit = Some(
                    value
                        .parse::<usize>()
                        .map_err(|_| {
                            ApiError::InvalidProjectLinkReview("limit must be an integer")
                        })?
                        .clamp(1, 100),
                );
            }
        }
    }

    Ok(query)
}

pub(crate) fn parse_task_candidates_query(
    raw_query: Option<&str>,
) -> Result<TaskCandidatesQuery, ApiError> {
    let mut query = TaskCandidatesQuery { limit: None };

    if let Some(raw_query) = raw_query {
        for (key, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            if key.as_ref() == "limit" {
                query.limit = Some(
                    value
                        .parse::<i64>()
                        .map_err(|_| {
                            ApiError::InvalidTaskCandidateQuery("limit must be an integer")
                        })?
                        .clamp(1, 100),
                );
            }
        }
    }

    Ok(query)
}

pub(crate) fn parse_document_processing_jobs_query(
    raw_query: Option<&str>,
) -> Result<DocumentProcessingJobsQuery, ApiError> {
    let mut query = DocumentProcessingJobsQuery { limit: None };

    if let Some(raw_query) = raw_query {
        for (key, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            if key.as_ref() == "limit" {
                query.limit = Some(
                    value
                        .parse::<i64>()
                        .map_err(|_| {
                            ApiError::InvalidDocumentProcessingQuery("limit must be an integer")
                        })?
                        .clamp(1, 100),
                );
            }
        }
    }

    Ok(query)
}

pub(crate) struct PersonIdentityCandidatesQuery {
    pub(crate) limit: Option<i64>,
}

pub(crate) fn parse_person_identity_candidates_query(
    raw_query: Option<&str>,
) -> Result<PersonIdentityCandidatesQuery, ApiError> {
    let mut query = PersonIdentityCandidatesQuery { limit: None };

    if let Some(raw_query) = raw_query {
        for (key, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            if key.as_ref() == "limit" {
                query.limit = Some(
                    value
                        .parse::<i64>()
                        .map_err(|_| {
                            ApiError::InvalidPersonIdentityReview("limit must be an integer")
                        })?
                        .clamp(1, 100),
                );
            }
        }
    }

    Ok(query)
}

pub(crate) fn parse_person_identity_review_state(
    value: &str,
) -> Result<PersonIdentityReviewState, ApiError> {
    match value.trim() {
        "suggested" => Ok(PersonIdentityReviewState::Suggested),
        "user_confirmed" => Ok(PersonIdentityReviewState::UserConfirmed),
        "user_rejected" => Ok(PersonIdentityReviewState::UserRejected),
        _ => Err(ApiError::InvalidPersonIdentityReview(
            "review_state must be suggested, user_confirmed, or user_rejected",
        )),
    }
}

pub(crate) fn parse_project_link_target_kind(
    value: &str,
) -> Result<ProjectLinkTargetKind, ApiError> {
    match value.trim() {
        "message" => Ok(ProjectLinkTargetKind::Message),
        "document" => Ok(ProjectLinkTargetKind::Document),
        _ => Err(ApiError::InvalidProjectLinkReview(
            "target_kind must be message or document",
        )),
    }
}

pub(crate) fn parse_project_link_review_state(
    value: &str,
) -> Result<ProjectLinkReviewState, ApiError> {
    match value.trim() {
        "suggested" => Ok(ProjectLinkReviewState::Suggested),
        "user_confirmed" => Ok(ProjectLinkReviewState::UserConfirmed),
        "user_rejected" => Ok(ProjectLinkReviewState::UserRejected),
        _ => Err(ApiError::InvalidProjectLinkReview(
            "review_state must be suggested, user_confirmed, or user_rejected",
        )),
    }
}

pub(crate) fn parse_task_candidate_review_state(
    value: &str,
) -> Result<TaskCandidateReviewState, ApiError> {
    match value.trim() {
        "suggested" => Ok(TaskCandidateReviewState::Suggested),
        "user_confirmed" => Ok(TaskCandidateReviewState::UserConfirmed),
        "user_rejected" => Ok(TaskCandidateReviewState::UserRejected),
        _ => Err(ApiError::InvalidTaskCandidateReview(
            "review_state must be suggested, user_confirmed, or user_rejected",
        )),
    }
}

pub(crate) fn validate_non_empty_project_link_field(
    field: &'static str,
    value: &str,
) -> Result<String, ApiError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(ApiError::InvalidProjectLinkReview(field));
    }

    Ok(normalized.to_owned())
}

pub(crate) fn validate_non_empty_task_candidate_field(
    field: &'static str,
    value: &str,
) -> Result<String, ApiError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(ApiError::InvalidTaskCandidateReview(match field {
            "command_id" => "command_id must not be empty",
            "review_state" => "review_state must not be empty",
            "task_candidate_id" => "task_candidate_id must not be empty",
            _ => "field must not be empty",
        }));
    }

    Ok(normalized.to_owned())
}

pub(crate) fn validate_non_empty_document_id(value: &str) -> Result<String, ApiError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(ApiError::InvalidDocumentProcessingQuery(
            "document_id must not be empty",
        ));
    }

    Ok(normalized.to_owned())
}

pub(crate) fn validate_non_empty_document_processing_field(
    field: &'static str,
    value: &str,
) -> Result<String, ApiError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(ApiError::InvalidDocumentProcessingQuery(match field {
            "command_id" => "command_id must not be empty",
            "job_id" => "job_id must not be empty",
            _ => "field must not be empty",
        }));
    }

    Ok(normalized.to_owned())
}

pub(crate) fn validate_non_empty_person_identity_field(
    field: &'static str,
    value: &str,
) -> Result<String, ApiError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(ApiError::InvalidPersonIdentityReview(match field {
            "command_id" => "command_id must not be empty",
            "identity_candidate_id" => "identity_candidate_id must not be empty",
            "person_id" => "person_id must not be empty",
            _ => "field must not be empty",
        }));
    }

    Ok(normalized.to_owned())
}

pub(crate) fn text_preview(value: &str, max_chars: usize) -> String {
    let mut preview = value.chars().take(max_chars).collect::<String>();
    if value.chars().count() > max_chars {
        preview.push_str("...");
    }

    preview
}

pub(crate) fn default_schema_version() -> i32 {
    1
}

pub(crate) fn empty_json_object() -> Value {
    json!({})
}

pub(crate) fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
