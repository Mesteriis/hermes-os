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

use crate::ai::control_center::AiControlCenterError;
use crate::ai::core::{
    AI_EMBEDDING_DIMENSION, AiAgentListResponse, AiAgentRun, AiAnswerRequest, AiError,
    AiMeetingPrepRequest, AiService, AiStatusResponse, AiTaskCandidateRefreshRequest, v3_agents,
};
use crate::domains::mail::core::{
    CommunicationIngestionError, CommunicationIngestionStore, EmailProviderKind, ProviderAccount,
};
use crate::domains::persons::analytics::{AnalyticsError, PersonAnalyticsService};
use crate::domains::persons::api::PersonProjectionError;
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
use crate::domains::decisions::DecisionStoreError;
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
use crate::domains::obligations::ObligationStoreError;
use crate::domains::organizations::api::{
    OrganizationError, OrganizationStore, OrganizationUpdate,
};
use crate::domains::projects::core::{ProjectListResponse, ProjectStore, ProjectStoreError};
use crate::domains::projects::link_reviews::{
    ProjectLinkReviewCommand, ProjectLinkReviewError, ProjectLinkReviewState,
    ProjectLinkReviewStore, ProjectLinkTargetKind,
};
use crate::domains::relationships::RelationshipStoreError;
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
use crate::engines::consistency::ConsistencyError;
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
use crate::vault::HostVaultError;
use crate::workflows::email_intelligence::{EmailIntelligenceError, EmailIntelligenceService};

use axum::response::IntoResponse;

#[derive(Serialize)]
struct ErrorResponse {
    error: &'static str,
    message: String,
}

pub enum ApiError {
    DatabaseNotConfigured,
    InvalidEnvelope(EventEnvelopeError),
    Audit(ApiAuditError),
    Store(EventStoreError),
    Graph(crate::domains::graph::core::GraphStoreError),
    InvalidGraphQuery(&'static str),
    InvalidPersonaQuery(&'static str),
    Projects(ProjectStoreError),
    InvalidProjectQuery(&'static str),
    InvalidProjectLinkReview(&'static str),
    InvalidTaskCandidateQuery(&'static str),
    InvalidTaskCandidateReview(&'static str),
    InvalidObligationQuery(&'static str),
    InvalidObligationReview(&'static str),
    InvalidDecisionQuery(&'static str),
    InvalidDecisionReview(&'static str),
    InvalidRelationshipQuery(&'static str),
    InvalidRelationshipReview(&'static str),
    InvalidContradictionQuery(&'static str),
    InvalidContradictionReview(&'static str),
    InvalidPersonIdentityReview(&'static str),
    InvalidDocumentProcessingQuery(&'static str),
    Settings(SettingsError),
    SettingNotFound,
    DocumentProcessing(DocumentProcessingError),
    TaskCandidateNotFound,
    TaskCandidate(TaskCandidateError),
    ObligationNotFound,
    Obligation(ObligationStoreError),
    DecisionNotFound,
    Decision(DecisionStoreError),
    RelationshipNotFound,
    Relationship(RelationshipStoreError),
    ContradictionObservationNotFound,
    Consistency(ConsistencyError),
    AiRunNotFound,
    Ai(AiError),
    AiControlCenter(AiControlCenterError),
    Telegram(TelegramError),
    WhatsappWeb(WhatsappWebError),
    Automation(AutomationError),
    Call(CallError),
    ProjectLinkTargetNotFound,
    ProjectLinkReview(ProjectLinkReviewError),
    PersonIdentityNotFound,
    PersonProjection(PersonProjectionError),
    PersonIdentity(PersonIdentityError),
    Messages(MessageProjectionError),
    CommunicationIngestion(CommunicationIngestionError),
    MailStorage(MailStorageError),
    InvalidCommunicationQuery(&'static str),
    ProviderWriteConfirmationRequired,
    CommunicationMessageNotFound,
    SecretVaultNotConfigured,
    HostVault(HostVaultError),
    AccountSetup(EmailAccountSetupError),
    AccountSetupState,
    AccountSetupPendingGrantNotFound,
    AccountSetupStateMismatch,
    GraphNotFound,
    ProjectNotFound,
    NotFound,
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error, message, authenticate) = match self {
            Self::DatabaseNotConfigured => (
                StatusCode::SERVICE_UNAVAILABLE,
                "database_not_configured",
                "DATABASE_URL is not configured".to_owned(),
                false,
            ),
            Self::SecretVaultNotConfigured => (
                StatusCode::SERVICE_UNAVAILABLE,
                "secret_vault_not_configured",
                "host vault must be initialized and unlocked for account setup".to_owned(),
                false,
            ),
            Self::HostVault(error) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "host_vault_error",
                error.to_string(),
                false,
            ),
            Self::InvalidEnvelope(error) => (
                StatusCode::BAD_REQUEST,
                "invalid_event_envelope",
                error.to_string(),
                false,
            ),
            Self::Audit(error) => {
                tracing::error!(error = %error, "event API audit operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "api_audit_error",
                    "API audit operation failed".to_owned(),
                    false,
                )
            }
            Self::Store(error) if error.is_unique_violation() => (
                StatusCode::CONFLICT,
                "event_conflict",
                "event already exists or violates idempotency constraints".to_owned(),
                false,
            ),
            Self::Store(error) => {
                tracing::error!(error = %error, "event API store operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "event_store_error",
                    "event store operation failed".to_owned(),
                    false,
                )
            }
            Self::Graph(error) => {
                tracing::error!(error = %error, "graph store operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "graph_store_error",
                    "graph store operation failed".to_owned(),
                    false,
                )
            }
            Self::InvalidGraphQuery(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_graph_query",
                message.to_owned(),
                false,
            ),
            Self::InvalidPersonaQuery(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_persona_query",
                message.to_owned(),
                false,
            ),
            Self::Projects(error) => {
                tracing::error!(error = %error, "project API store operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "project_store_error",
                    "project store operation failed".to_owned(),
                    false,
                )
            }
            Self::InvalidProjectQuery(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_project_query",
                message.to_owned(),
                false,
            ),
            Self::InvalidProjectLinkReview(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_project_link_review",
                message.to_owned(),
                false,
            ),
            Self::InvalidTaskCandidateQuery(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_task_candidate_query",
                message.to_owned(),
                false,
            ),
            Self::InvalidTaskCandidateReview(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_task_candidate_review",
                message.to_owned(),
                false,
            ),
            Self::InvalidObligationQuery(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_obligation_query",
                message.to_owned(),
                false,
            ),
            Self::InvalidObligationReview(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_obligation_review",
                message.to_owned(),
                false,
            ),
            Self::InvalidDecisionQuery(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_decision_query",
                message.to_owned(),
                false,
            ),
            Self::InvalidDecisionReview(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_decision_review",
                message.to_owned(),
                false,
            ),
            Self::InvalidRelationshipQuery(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_relationship_query",
                message.to_owned(),
                false,
            ),
            Self::InvalidRelationshipReview(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_relationship_review",
                message.to_owned(),
                false,
            ),
            Self::InvalidContradictionQuery(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_contradiction_query",
                message.to_owned(),
                false,
            ),
            Self::InvalidContradictionReview(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_contradiction_review",
                message.to_owned(),
                false,
            ),
            Self::InvalidPersonIdentityReview(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_person_identity_review",
                message.to_owned(),
                false,
            ),
            Self::InvalidDocumentProcessingQuery(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_document_processing_query",
                message.to_owned(),
                false,
            ),
            Self::SettingNotFound => (
                StatusCode::NOT_FOUND,
                "setting_not_found",
                "application setting was not found".to_owned(),
                false,
            ),
            Self::Settings(error) if error.is_invalid_request() => (
                StatusCode::BAD_REQUEST,
                "invalid_application_setting",
                error.to_string(),
                false,
            ),
            Self::Settings(error) => {
                tracing::error!(error = %error, "application settings operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "application_settings_error",
                    "application settings operation failed".to_owned(),
                    false,
                )
            }
            Self::DocumentProcessing(error) => {
                let (status, message) = match error {
                    DocumentProcessingError::InvalidLimit => (
                        StatusCode::BAD_REQUEST,
                        "document processing limit must be between 1 and 100",
                    ),
                    DocumentProcessingError::EmptyField(_)
                    | DocumentProcessingError::InvalidStep(_)
                    | DocumentProcessingError::InvalidStatus(_)
                    | DocumentProcessingError::InvalidArtifactKind(_) => (
                        StatusCode::BAD_REQUEST,
                        "invalid document processing request payload",
                    ),
                    DocumentProcessingError::DocumentNotFound
                    | DocumentProcessingError::JobNotFound => (
                        StatusCode::NOT_FOUND,
                        "document processing job was not found",
                    ),
                    DocumentProcessingError::RetryRequiresFailedJob => (
                        StatusCode::BAD_REQUEST,
                        "document processing retry requires a failed job",
                    ),
                    DocumentProcessingError::RetryCommandConflict => (
                        StatusCode::CONFLICT,
                        "document processing retry command conflicts with existing event",
                    ),
                    DocumentProcessingError::EventStore(error) if error.is_unique_violation() => (
                        StatusCode::CONFLICT,
                        "document processing retry command conflicts with existing event",
                    ),
                    _ => {
                        tracing::error!(error = %error, "document processing store operation failed");
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "document processing store operation failed",
                        )
                    }
                };

                (
                    status,
                    "document_processing_store_error",
                    message.to_owned(),
                    false,
                )
            }
            Self::TaskCandidateNotFound => (
                StatusCode::NOT_FOUND,
                "task_candidate_not_found",
                "task candidate was not found".to_owned(),
                false,
            ),
            Self::TaskCandidate(error) => {
                tracing::error!(error = %error, "task candidate store operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "task_candidate_store_error",
                    "task candidate store operation failed".to_owned(),
                    false,
                )
            }
            Self::ObligationNotFound => (
                StatusCode::NOT_FOUND,
                "obligation_not_found",
                "obligation was not found".to_owned(),
                false,
            ),
            Self::Obligation(error) => {
                tracing::error!(error = %error, "obligation store operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "obligation_store_error",
                    "obligation store operation failed".to_owned(),
                    false,
                )
            }
            Self::DecisionNotFound => (
                StatusCode::NOT_FOUND,
                "decision_not_found",
                "decision was not found".to_owned(),
                false,
            ),
            Self::Decision(error) => {
                tracing::error!(error = %error, "decision store operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "decision_store_error",
                    "decision store operation failed".to_owned(),
                    false,
                )
            }
            Self::RelationshipNotFound => (
                StatusCode::NOT_FOUND,
                "relationship_not_found",
                "relationship was not found".to_owned(),
                false,
            ),
            Self::Relationship(error) => {
                tracing::error!(error = %error, "relationship store operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "relationship_store_error",
                    "relationship store operation failed".to_owned(),
                    false,
                )
            }
            Self::ContradictionObservationNotFound => (
                StatusCode::NOT_FOUND,
                "contradiction_observation_not_found",
                "contradiction observation was not found".to_owned(),
                false,
            ),
            Self::Consistency(error) => {
                tracing::error!(error = %error, "consistency engine operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "consistency_engine_error",
                    "consistency engine operation failed".to_owned(),
                    false,
                )
            }
            Self::AiRunNotFound => (
                StatusCode::NOT_FOUND,
                "ai_run_not_found",
                "AI run was not found".to_owned(),
                false,
            ),
            Self::Ai(error) => match error {
                AiError::InvalidRequest(message) => (
                    StatusCode::BAD_REQUEST,
                    "invalid_ai_request",
                    message.to_owned(),
                    false,
                ),
                AiError::UnknownAgent(agent_id) => (
                    StatusCode::BAD_REQUEST,
                    "unknown_ai_agent",
                    format!("unknown AI agent `{agent_id}`"),
                    false,
                ),
                AiError::RunNotFound => (
                    StatusCode::NOT_FOUND,
                    "ai_run_not_found",
                    "AI run was not found".to_owned(),
                    false,
                ),
                AiError::Runtime(error) => (
                    StatusCode::BAD_GATEWAY,
                    "ai_runtime_error",
                    error.to_string(),
                    false,
                ),
                AiError::InvalidEmbeddingDimension { .. } => (
                    StatusCode::BAD_GATEWAY,
                    "invalid_embedding_dimension",
                    "embedding provider returned an unexpected vector dimension".to_owned(),
                    false,
                ),
                AiError::Json(error) => (
                    StatusCode::BAD_GATEWAY,
                    "ai_provider_json_error",
                    error.to_string(),
                    false,
                ),
                AiError::InvalidSourceKind(value) => {
                    tracing::error!(source_kind = %value, "AI runtime saw invalid semantic source kind");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "ai_runtime_error",
                        "AI runtime operation failed".to_owned(),
                        false,
                    )
                }
                AiError::EventEnvelope(error) => {
                    tracing::error!(error = %error, "AI runtime operation failed");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "ai_runtime_error",
                        "AI runtime operation failed".to_owned(),
                        false,
                    )
                }
                AiError::EventStore(error) => {
                    tracing::error!(error = %error, "AI event store operation failed");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "ai_runtime_error",
                        "AI runtime operation failed".to_owned(),
                        false,
                    )
                }
                AiError::PersonProjection(error) => {
                    tracing::error!(error = %error, "AI persona attribution operation failed");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "ai_runtime_error",
                        "AI runtime operation failed".to_owned(),
                        false,
                    )
                }
                AiError::Sqlx(error) => {
                    tracing::error!(error = %error, "AI database operation failed");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "ai_runtime_error",
                        "AI runtime operation failed".to_owned(),
                        false,
                    )
                }
            },
            Self::AiControlCenter(error) => match error {
                AiControlCenterError::ProviderNotFound => (
                    StatusCode::NOT_FOUND,
                    "ai_provider_not_found",
                    "AI provider was not found".to_owned(),
                    false,
                ),
                AiControlCenterError::ModelNotFound => (
                    StatusCode::NOT_FOUND,
                    "ai_model_not_found",
                    "AI model was not found".to_owned(),
                    false,
                ),
                AiControlCenterError::PromptNotFound => (
                    StatusCode::NOT_FOUND,
                    "ai_prompt_not_found",
                    "AI prompt was not found".to_owned(),
                    false,
                ),
                AiControlCenterError::PromptVersionNotFound => (
                    StatusCode::NOT_FOUND,
                    "ai_prompt_version_not_found",
                    "AI prompt version was not found".to_owned(),
                    false,
                ),
                AiControlCenterError::InvalidRequest(_)
                | AiControlCenterError::EmptyField { .. }
                | AiControlCenterError::SecretLikePayload => (
                    StatusCode::BAD_REQUEST,
                    "invalid_ai_control_center_request",
                    error.to_string(),
                    false,
                ),
                AiControlCenterError::HostVault(error) => (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "host_vault_error",
                    error.to_string(),
                    false,
                ),
                AiControlCenterError::SecretReference(error) => {
                    tracing::error!(error = %error, "AI control center secret reference operation failed");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "ai_secret_reference_error",
                        "AI provider secret reference operation failed".to_owned(),
                        false,
                    )
                }
                AiControlCenterError::Sqlx(error) => {
                    tracing::error!(error = %error, "AI control center store operation failed");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "ai_control_center_error",
                        "AI control center operation failed".to_owned(),
                        false,
                    )
                }
            },
            Self::Telegram(error) => match error {
                TelegramError::InvalidRequest(message) => (
                    StatusCode::BAD_REQUEST,
                    "invalid_telegram_request",
                    message,
                    false,
                ),
                TelegramError::TdlibRuntimeUnavailable(error) => {
                    tracing::warn!(error = %error, "Telegram TDLib runtime is unavailable");
                    (
                        StatusCode::SERVICE_UNAVAILABLE,
                        "telegram_tdlib_runtime_unavailable",
                        "Telegram TDLib runtime is not configured on this host".to_owned(),
                        false,
                    )
                }
                TelegramError::TdlibRuntime(error) => {
                    tracing::warn!(error = %error, "Telegram TDLib runtime operation failed");
                    (
                        StatusCode::BAD_GATEWAY,
                        "telegram_tdlib_runtime_error",
                        "Telegram TDLib runtime operation failed".to_owned(),
                        false,
                    )
                }
                TelegramError::QrGeneration(error) => {
                    tracing::warn!(error = %error, "Telegram QR generation failed");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "telegram_qr_generation_error",
                        "Telegram QR generation failed".to_owned(),
                        false,
                    )
                }
                TelegramError::QrLoginNotFound => (
                    StatusCode::NOT_FOUND,
                    "telegram_qr_login_not_found",
                    "Telegram QR login setup was not found".to_owned(),
                    false,
                ),
                TelegramError::Communication(error) => {
                    tracing::error!(error = %error, "Telegram communication store operation failed");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "telegram_store_error",
                        "Telegram store operation failed".to_owned(),
                        false,
                    )
                }
                TelegramError::SecretReference(error) => {
                    tracing::error!(error = %error, "Telegram secret reference operation failed");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "telegram_secret_reference_error",
                        "Telegram secret reference operation failed".to_owned(),
                        false,
                    )
                }
                TelegramError::DatabaseVault(error) => {
                    tracing::error!(error = %error, "Telegram database vault operation failed");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "telegram_secret_vault_error",
                        "Telegram secret vault operation failed".to_owned(),
                        false,
                    )
                }
                TelegramError::HostVault(error) => {
                    tracing::warn!(error = %error, "Telegram host vault operation failed");
                    (
                        StatusCode::SERVICE_UNAVAILABLE,
                        "telegram_host_vault_error",
                        "Telegram host vault operation failed".to_owned(),
                        false,
                    )
                }
                TelegramError::MessageProjection(error) => {
                    tracing::error!(error = %error, "Telegram message projection failed");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "telegram_projection_error",
                        "Telegram message projection failed".to_owned(),
                        false,
                    )
                }
                TelegramError::Decision(error) => {
                    tracing::error!(error = %error, "Telegram decision candidate refresh failed");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "telegram_decision_refresh_error",
                        "Telegram decision candidate refresh failed".to_owned(),
                        false,
                    )
                }
                TelegramError::TaskCandidate(error) => {
                    tracing::error!(error = %error, "Telegram task candidate refresh failed");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "telegram_task_candidate_refresh_error",
                        "Telegram task candidate refresh failed".to_owned(),
                        false,
                    )
                }
                TelegramError::Sqlx(error) => {
                    tracing::error!(error = %error, "Telegram database operation failed");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "telegram_store_error",
                        "Telegram store operation failed".to_owned(),
                        false,
                    )
                }
            },
            Self::WhatsappWeb(error) => match error {
                WhatsappWebError::InvalidRequest(message) => (
                    StatusCode::BAD_REQUEST,
                    "invalid_whatsapp_web_request",
                    message,
                    false,
                ),
                WhatsappWebError::Communication(error) => {
                    tracing::error!(error = %error, "WhatsApp Web communication store operation failed");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "whatsapp_web_store_error",
                        "WhatsApp Web store operation failed".to_owned(),
                        false,
                    )
                }
                WhatsappWebError::MessageProjection(error) => {
                    tracing::error!(error = %error, "WhatsApp Web message projection failed");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "whatsapp_web_projection_error",
                        "WhatsApp Web message projection failed".to_owned(),
                        false,
                    )
                }
                WhatsappWebError::Decision(error) => {
                    tracing::error!(error = %error, "WhatsApp Web decision candidate refresh failed");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "whatsapp_web_decision_refresh_error",
                        "WhatsApp Web decision candidate refresh failed".to_owned(),
                        false,
                    )
                }
                WhatsappWebError::TaskCandidate(error) => {
                    tracing::error!(error = %error, "WhatsApp Web task candidate refresh failed");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "whatsapp_web_task_candidate_refresh_error",
                        "WhatsApp Web task candidate refresh failed".to_owned(),
                        false,
                    )
                }
                WhatsappWebError::Sqlx(error) => {
                    tracing::error!(error = %error, "WhatsApp Web database operation failed");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "whatsapp_web_store_error",
                        "WhatsApp Web store operation failed".to_owned(),
                        false,
                    )
                }
            },
            Self::Automation(error) => match error {
                AutomationError::InvalidRequest(message) => (
                    StatusCode::BAD_REQUEST,
                    "invalid_automation_request",
                    message,
                    false,
                ),
                AutomationError::PolicyNotFound => (
                    StatusCode::NOT_FOUND,
                    "automation_policy_not_found",
                    "automation policy was not found".to_owned(),
                    false,
                ),
                AutomationError::PolicyDisabled
                | AutomationError::ChatNotAllowed
                | AutomationError::MissingTemplateVariable(_)
                | AutomationError::UndeclaredTemplateVariable(_) => (
                    StatusCode::FORBIDDEN,
                    "automation_policy_denied",
                    error.to_string(),
                    false,
                ),
                AutomationError::EventEnvelope(error) => (
                    StatusCode::BAD_REQUEST,
                    "invalid_automation_event",
                    error.to_string(),
                    false,
                ),
                AutomationError::EventStore(error) => {
                    tracing::error!(error = %error, "automation event store operation failed");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "automation_store_error",
                        "automation operation failed".to_owned(),
                        false,
                    )
                }
                AutomationError::Sqlx(error) => {
                    tracing::error!(error = %error, "automation database operation failed");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "automation_store_error",
                        "automation operation failed".to_owned(),
                        false,
                    )
                }
            },
            Self::Call(error) => match error {
                CallError::InvalidRequest(message) => (
                    StatusCode::BAD_REQUEST,
                    "invalid_call_request",
                    message,
                    false,
                ),
                CallError::Sqlx(error) => {
                    tracing::error!(error = %error, "call intelligence database operation failed");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "call_store_error",
                        "call intelligence operation failed".to_owned(),
                        false,
                    )
                }
            },
            Self::ProjectLinkTargetNotFound => (
                StatusCode::NOT_FOUND,
                "project_link_target_not_found",
                "project link target was not found".to_owned(),
                false,
            ),
            Self::ProjectLinkReview(error) => {
                tracing::error!(error = %error, "project link review store operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "project_link_review_store_error",
                    "project link review store operation failed".to_owned(),
                    false,
                )
            }
            Self::PersonIdentityNotFound => (
                StatusCode::NOT_FOUND,
                "person_identity_candidate_not_found",
                "person identity candidate was not found".to_owned(),
                false,
            ),
            Self::PersonProjection(error) => match error {
                PersonProjectionError::PersonNotFound(_) => (
                    StatusCode::NOT_FOUND,
                    "person_not_found",
                    "person was not found".to_owned(),
                    false,
                ),
                PersonProjectionError::EmptyEmailAddress
                | PersonProjectionError::InvalidEmailAddress(_)
                | PersonProjectionError::EmptyAiAgentId
                | PersonProjectionError::InvalidAiAgentId(_)
                | PersonProjectionError::EmptyDisplayName
                | PersonProjectionError::InvalidPersonaType(_) => (
                    StatusCode::BAD_REQUEST,
                    "invalid_person_projection",
                    error.to_string(),
                    false,
                ),
                PersonProjectionError::Graph(error) => {
                    tracing::error!(error = %error, "person graph projection operation failed");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "person_projection_error",
                        "person projection operation failed".to_owned(),
                        false,
                    )
                }
                PersonProjectionError::Sqlx(error) => {
                    tracing::error!(error = %error, "person projection operation failed");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "person_projection_error",
                        "person projection operation failed".to_owned(),
                        false,
                    )
                }
            },
            Self::PersonIdentity(error) => {
                tracing::error!(
                    error = %error,
                    "person identity store operation failed"
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "person_identity_store_error",
                    "person identity store operation failed".to_owned(),
                    false,
                )
            }
            Self::Messages(error) => {
                tracing::error!(error = %error, "communication message API store operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "communication_message_store_error",
                    "communication message store operation failed".to_owned(),
                    false,
                )
            }
            Self::CommunicationIngestion(error) => {
                tracing::error!(error = %error, "communication account API store operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "communication_account_store_error",
                    "communication account store operation failed".to_owned(),
                    false,
                )
            }
            Self::MailStorage(error) => {
                tracing::error!(error = %error, "communication attachment API store operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "communication_attachment_store_error",
                    "communication attachment store operation failed".to_owned(),
                    false,
                )
            }
            Self::InvalidCommunicationQuery(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_communication_query",
                message.to_owned(),
                false,
            ),
            Self::ProviderWriteConfirmationRequired => (
                StatusCode::BAD_REQUEST,
                "provider_write_confirmation_required",
                "explicit provider write confirmation is required".to_owned(),
                false,
            ),
            Self::CommunicationMessageNotFound => (
                StatusCode::NOT_FOUND,
                "communication_message_not_found",
                "communication message was not found".to_owned(),
                false,
            ),
            Self::AccountSetup(error) => {
                let status = if matches!(
                    error,
                    EmailAccountSetupError::InvalidRequest { .. }
                        | EmailAccountSetupError::MissingProviderField { .. }
                ) {
                    StatusCode::BAD_REQUEST
                } else {
                    tracing::error!(error = %error, "account setup failed");
                    StatusCode::INTERNAL_SERVER_ERROR
                };
                (
                    status,
                    "account_setup_error",
                    if status == StatusCode::BAD_REQUEST {
                        error.to_string()
                    } else {
                        "account setup failed".to_owned()
                    },
                    false,
                )
            }
            Self::AccountSetupState => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "account_setup_state_error",
                "account setup state is unavailable".to_owned(),
                false,
            ),
            Self::AccountSetupPendingGrantNotFound => (
                StatusCode::NOT_FOUND,
                "account_setup_pending_grant_not_found",
                "pending Gmail OAuth setup was not found".to_owned(),
                false,
            ),
            Self::AccountSetupStateMismatch => (
                StatusCode::BAD_REQUEST,
                "account_setup_state_mismatch",
                "Gmail OAuth state does not match the pending setup".to_owned(),
                false,
            ),
            Self::GraphNotFound => (
                StatusCode::NOT_FOUND,
                "graph_node_not_found",
                "graph node was not found".to_owned(),
                false,
            ),
            Self::ProjectNotFound => (
                StatusCode::NOT_FOUND,
                "project_not_found",
                "project was not found".to_owned(),
                false,
            ),
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                "event_not_found",
                "event was not found".to_owned(),
                false,
            ),
        };

        let mut response = (status, Json(ErrorResponse { error, message })).into_response();
        if authenticate {
            response
                .headers_mut()
                .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        }
        response
    }
}

impl From<EventEnvelopeError> for ApiError {
    fn from(error: EventEnvelopeError) -> Self {
        Self::InvalidEnvelope(error)
    }
}

impl From<EventStoreError> for ApiError {
    fn from(error: EventStoreError) -> Self {
        Self::Store(error)
    }
}

impl From<crate::domains::graph::core::GraphStoreError> for ApiError {
    fn from(error: crate::domains::graph::core::GraphStoreError) -> Self {
        Self::Graph(error)
    }
}

impl From<ProjectLinkReviewError> for ApiError {
    fn from(error: ProjectLinkReviewError) -> Self {
        match error {
            ProjectLinkReviewError::ProjectNotFound | ProjectLinkReviewError::TargetNotFound => {
                Self::ProjectLinkTargetNotFound
            }
            _ => Self::ProjectLinkReview(error),
        }
    }
}

impl From<TaskCandidateError> for ApiError {
    fn from(error: TaskCandidateError) -> Self {
        match error {
            TaskCandidateError::TaskCandidateNotFound => Self::TaskCandidateNotFound,
            _ => Self::TaskCandidate(error),
        }
    }
}

impl From<ObligationStoreError> for ApiError {
    fn from(error: ObligationStoreError) -> Self {
        match error {
            ObligationStoreError::ObligationNotFound => Self::ObligationNotFound,
            ObligationStoreError::UnknownEntityKind(_) => Self::InvalidObligationQuery(
                "entity_kind must be persona, organization, project, communication, document, task, event, decision, obligation, or knowledge",
            ),
            ObligationStoreError::UnknownReviewState(_) => Self::InvalidObligationReview(
                "review_state must be suggested, user_confirmed, or user_rejected",
            ),
            _ => Self::Obligation(error),
        }
    }
}

impl From<DecisionStoreError> for ApiError {
    fn from(error: DecisionStoreError) -> Self {
        match error {
            DecisionStoreError::DecisionNotFound => Self::DecisionNotFound,
            DecisionStoreError::UnknownEntityKind(_) => Self::InvalidDecisionQuery(
                "entity_kind must be persona, organization, project, communication, document, task, event, decision, obligation, or knowledge",
            ),
            DecisionStoreError::UnknownReviewState(_) => Self::InvalidDecisionReview(
                "review_state must be suggested, user_confirmed, or user_rejected",
            ),
            _ => Self::Decision(error),
        }
    }
}

impl From<RelationshipStoreError> for ApiError {
    fn from(error: RelationshipStoreError) -> Self {
        match error {
            RelationshipStoreError::RelationshipNotFound => Self::RelationshipNotFound,
            RelationshipStoreError::UnknownEntityKind(_) => Self::InvalidRelationshipQuery(
                "entity_kind must be persona, organization, project, communication, document, task, event, decision, obligation, or knowledge",
            ),
            RelationshipStoreError::UnknownReviewState(_) => Self::InvalidRelationshipReview(
                "review_state must be suggested, system_accepted, user_confirmed, or user_rejected",
            ),
            _ => Self::Relationship(error),
        }
    }
}

impl From<ConsistencyError> for ApiError {
    fn from(error: ConsistencyError) -> Self {
        match error {
            ConsistencyError::ObservationNotFound(_) => Self::ContradictionObservationNotFound,
            ConsistencyError::UnknownReviewState(_) => Self::InvalidContradictionReview(
                "review_state must be suggested, user_confirmed, or user_rejected",
            ),
            _ => Self::Consistency(error),
        }
    }
}

impl From<AiError> for ApiError {
    fn from(error: AiError) -> Self {
        match error {
            AiError::RunNotFound => Self::AiRunNotFound,
            _ => Self::Ai(error),
        }
    }
}

impl From<AiControlCenterError> for ApiError {
    fn from(error: AiControlCenterError) -> Self {
        Self::AiControlCenter(error)
    }
}

impl From<TelegramError> for ApiError {
    fn from(error: TelegramError) -> Self {
        Self::Telegram(error)
    }
}

impl From<WhatsappWebError> for ApiError {
    fn from(error: WhatsappWebError) -> Self {
        Self::WhatsappWeb(error)
    }
}

impl From<AutomationError> for ApiError {
    fn from(error: AutomationError) -> Self {
        Self::Automation(error)
    }
}

impl From<CallError> for ApiError {
    fn from(error: CallError) -> Self {
        Self::Call(error)
    }
}

impl From<crate::integrations::ollama::client::OllamaError> for ApiError {
    fn from(error: crate::integrations::ollama::client::OllamaError) -> Self {
        Self::Ai(AiError::Runtime(
            crate::integrations::ai_runtime::AiRuntimeError::Ollama(error),
        ))
    }
}

impl From<crate::integrations::omniroute::client::OmniRouteError> for ApiError {
    fn from(error: crate::integrations::omniroute::client::OmniRouteError) -> Self {
        Self::Ai(AiError::Runtime(
            crate::integrations::ai_runtime::AiRuntimeError::OmniRoute(error),
        ))
    }
}

impl From<crate::integrations::ai_runtime::AiRuntimeError> for ApiError {
    fn from(error: crate::integrations::ai_runtime::AiRuntimeError) -> Self {
        Self::Ai(AiError::Runtime(error))
    }
}

impl From<PersonIdentityError> for ApiError {
    fn from(error: PersonIdentityError) -> Self {
        match error {
            PersonIdentityError::IdentityCandidateNotFound => Self::PersonIdentityNotFound,
            PersonIdentityError::InvalidLimit | PersonIdentityError::InvalidReviewState(_) => {
                Self::InvalidPersonIdentityReview(
                    "review_state or limit must be valid for person identity candidates",
                )
            }
            PersonIdentityError::InvalidPayload(_)
            | PersonIdentityError::MissingPayloadField(_)
            | PersonIdentityError::MissingActorId => {
                Self::InvalidPersonIdentityReview("invalid identity candidate review payload")
            }
            _ => Self::PersonIdentity(error),
        }
    }
}

impl From<PersonProjectionError> for ApiError {
    fn from(error: PersonProjectionError) -> Self {
        Self::PersonProjection(error)
    }
}

impl From<DocumentProcessingError> for ApiError {
    fn from(error: DocumentProcessingError) -> Self {
        Self::DocumentProcessing(error)
    }
}

impl From<SettingsError> for ApiError {
    fn from(error: SettingsError) -> Self {
        match error {
            SettingsError::SettingNotFound { .. } => Self::SettingNotFound,
            _ => Self::Settings(error),
        }
    }
}

impl From<CommunicationIngestionError> for ApiError {
    fn from(error: CommunicationIngestionError) -> Self {
        Self::CommunicationIngestion(error)
    }
}

impl From<ProjectStoreError> for ApiError {
    fn from(error: ProjectStoreError) -> Self {
        Self::Projects(error)
    }
}

impl From<MessageProjectionError> for ApiError {
    fn from(error: MessageProjectionError) -> Self {
        Self::Messages(error)
    }
}

impl From<MailStorageError> for ApiError {
    fn from(error: MailStorageError) -> Self {
        Self::MailStorage(error)
    }
}

impl From<ApiAuditError> for ApiError {
    fn from(error: ApiAuditError) -> Self {
        Self::Audit(error)
    }
}

impl From<crate::domains::mail::threads::EmailThreadError> for ApiError {
    fn from(error: crate::domains::mail::threads::EmailThreadError) -> Self {
        tracing::error!(error = %error, "email thread operation failed");
        ApiError::InvalidCommunicationQuery("email thread operation failed")
    }
}

impl From<EmailIntelligenceError> for ApiError {
    fn from(error: EmailIntelligenceError) -> Self {
        match error {
            EmailIntelligenceError::ParseError(_msg) => {
                ApiError::InvalidCommunicationQuery("failed to parse AI analysis result")
            }
            _ => {
                tracing::error!(error = %error, "email intelligence operation failed");
                ApiError::InvalidCommunicationQuery("email intelligence operation failed")
            }
        }
    }
}

impl From<crate::engines::search::SearchError> for ApiError {
    fn from(error: crate::engines::search::SearchError) -> Self {
        tracing::error!(error = %error, "search operation failed");
        ApiError::InvalidCommunicationQuery("search operation failed")
    }
}

impl From<crate::domains::mail::drafts::EmailDraftError> for ApiError {
    fn from(error: crate::domains::mail::drafts::EmailDraftError) -> Self {
        tracing::error!(error = %error, "email draft operation failed");
        ApiError::InvalidCommunicationQuery("email draft operation failed")
    }
}

impl From<crate::domains::mail::finance::EmailFinanceError> for ApiError {
    fn from(error: crate::domains::mail::finance::EmailFinanceError) -> Self {
        tracing::error!(error = %error, "email finance operation failed");
        ApiError::InvalidCommunicationQuery("email finance operation failed")
    }
}

impl From<crate::domains::mail::analytics::EmailAnalyticsError> for ApiError {
    fn from(error: crate::domains::mail::analytics::EmailAnalyticsError) -> Self {
        tracing::error!(error = %error, "email analytics operation failed");
        ApiError::InvalidCommunicationQuery("email analytics operation failed")
    }
}

impl From<crate::domains::mail::personas::EmailPersonaError> for ApiError {
    fn from(error: crate::domains::mail::personas::EmailPersonaError) -> Self {
        tracing::error!(error = %error, "email persona operation failed");
        ApiError::InvalidCommunicationQuery("email persona operation failed")
    }
}

impl From<crate::domains::mail::search::IndexEmailError> for ApiError {
    fn from(error: crate::domains::mail::search::IndexEmailError) -> Self {
        tracing::error!(error = %error, "email search operation failed");
        ApiError::InvalidCommunicationQuery("email search operation failed")
    }
}

impl From<crate::domains::mail::flags::MessageFlagsError> for ApiError {
    fn from(error: crate::domains::mail::flags::MessageFlagsError) -> Self {
        tracing::error!(error = %error, "message flags operation failed");
        ApiError::InvalidCommunicationQuery("message flags operation failed")
    }
}

impl From<crate::domains::mail::subscriptions::SubscriptionError> for ApiError {
    fn from(error: crate::domains::mail::subscriptions::SubscriptionError) -> Self {
        tracing::error!(error = %error, "subscriptions operation failed");
        ApiError::InvalidCommunicationQuery("subscriptions operation failed")
    }
}

impl From<crate::domains::mail::attachment_dedup::AttachmentDedupError> for ApiError {
    fn from(error: crate::domains::mail::attachment_dedup::AttachmentDedupError) -> Self {
        tracing::error!(error = %error, "attachment dedup operation failed");
        ApiError::InvalidCommunicationQuery("attachment dedup operation failed")
    }
}

impl From<crate::domains::mail::legal::LegalDocumentError> for ApiError {
    fn from(error: crate::domains::mail::legal::LegalDocumentError) -> Self {
        tracing::error!(error = %error, "legal document operation failed");
        ApiError::InvalidCommunicationQuery("legal document operation failed")
    }
}

impl From<crate::domains::mail::export::EmailExportError> for ApiError {
    fn from(error: crate::domains::mail::export::EmailExportError) -> Self {
        match error {
            crate::domains::mail::export::EmailExportError::NotFound => {
                ApiError::CommunicationMessageNotFound
            }
            _ => {
                tracing::error!(error = %error, "email export failed");
                ApiError::InvalidCommunicationQuery("email export failed")
            }
        }
    }
}

impl From<crate::domains::mail::send::EmailSendError> for ApiError {
    fn from(error: crate::domains::mail::send::EmailSendError) -> Self {
        tracing::error!(error = %error, "email send failed");
        ApiError::InvalidCommunicationQuery("email send failed")
    }
}
impl From<crate::domains::mail::imap_write::ImapWriteError> for ApiError {
    fn from(error: crate::domains::mail::imap_write::ImapWriteError) -> Self {
        tracing::error!(error = %error, "IMAP write operation failed");
        ApiError::InvalidCommunicationQuery("IMAP write operation failed")
    }
}
impl From<crate::domains::mail::signatures::CertificateError> for ApiError {
    fn from(error: crate::domains::mail::signatures::CertificateError) -> Self {
        tracing::error!(error = %error, "certificate operation failed");
        ApiError::InvalidCommunicationQuery("certificate operation failed")
    }
}
impl From<crate::domains::mail::multilingual::MultilingualError> for ApiError {
    fn from(error: crate::domains::mail::multilingual::MultilingualError) -> Self {
        tracing::error!(error = %error, "multilingual operation failed");
        ApiError::InvalidCommunicationQuery("multilingual operation failed")
    }
}
impl From<crate::domains::mail::ai_reply::AiReplyError> for ApiError {
    fn from(error: crate::domains::mail::ai_reply::AiReplyError) -> Self {
        tracing::error!(error = %error, "AI reply generation failed");
        ApiError::InvalidCommunicationQuery("AI reply generation failed")
    }
}
impl From<crate::domains::mail::extract::ExtractError> for ApiError {
    fn from(error: crate::domains::mail::extract::ExtractError) -> Self {
        tracing::error!(error = %error, "extract failed");
        ApiError::InvalidCommunicationQuery("extract failed")
    }
}
impl From<crate::domains::persons::enrichment::PersonEnrichmentError> for ApiError {
    fn from(error: crate::domains::persons::enrichment::PersonEnrichmentError) -> Self {
        match error {
            crate::domains::persons::enrichment::PersonEnrichmentError::NotFound => {
                ApiError::PersonIdentityNotFound
            }
            _ => {
                tracing::error!(error = %error, "person enrichment failed");
                ApiError::InvalidCommunicationQuery("person enrichment failed")
            }
        }
    }
}
impl From<PersonMemoryError> for ApiError {
    fn from(error: PersonMemoryError) -> Self {
        match error {
            PersonMemoryError::NotFound => ApiError::PersonIdentityNotFound,
            _ => {
                tracing::error!(error = %error, "person memory operation failed");
                ApiError::InvalidCommunicationQuery("person memory operation failed")
            }
        }
    }
}

impl From<PersonCoreError> for ApiError {
    fn from(error: PersonCoreError) -> Self {
        match error {
            PersonCoreError::IdentityNotFound | PersonCoreError::PersonaNotFound => {
                ApiError::PersonIdentityNotFound
            }
            _ => {
                tracing::error!(error = %error, "person core operation failed");
                ApiError::InvalidCommunicationQuery("person core operation failed")
            }
        }
    }
}

impl From<crate::domains::organizations::core::OrgCoreError> for ApiError {
    fn from(error: crate::domains::organizations::core::OrgCoreError) -> Self {
        tracing::error!(error = %error, "org core operation failed");
        ApiError::InvalidCommunicationQuery("org core operation failed")
    }
}
impl From<crate::domains::organizations::memory::OrgMemoryError> for ApiError {
    fn from(error: crate::domains::organizations::memory::OrgMemoryError) -> Self {
        tracing::error!(error = %error, "org memory operation failed");
        ApiError::InvalidCommunicationQuery("org memory operation failed")
    }
}
impl From<crate::domains::organizations::workflows::OrgWorkflowError> for ApiError {
    fn from(error: crate::domains::organizations::workflows::OrgWorkflowError) -> Self {
        tracing::error!(error = %error, "org workflow operation failed");
        ApiError::InvalidCommunicationQuery("org workflow operation failed")
    }
}
impl From<crate::domains::organizations::finance::OrgFinanceError> for ApiError {
    fn from(error: crate::domains::organizations::finance::OrgFinanceError) -> Self {
        tracing::error!(error = %error, "org finance operation failed");
        ApiError::InvalidCommunicationQuery("org finance operation failed")
    }
}
impl From<crate::domains::organizations::enrichment::OrgEnrichmentError> for ApiError {
    fn from(error: crate::domains::organizations::enrichment::OrgEnrichmentError) -> Self {
        tracing::error!(error = %error, "org enrichment operation failed");
        ApiError::InvalidCommunicationQuery("org enrichment operation failed")
    }
}
impl From<crate::domains::organizations::health::OrgHealthError> for ApiError {
    fn from(error: crate::domains::organizations::health::OrgHealthError) -> Self {
        tracing::error!(error = %error, "org health operation failed");
        ApiError::InvalidCommunicationQuery("org health operation failed")
    }
}
impl From<crate::domains::organizations::investigator::InvestigatorError> for ApiError {
    fn from(error: crate::domains::organizations::investigator::InvestigatorError) -> Self {
        match error {
            crate::domains::organizations::investigator::InvestigatorError::NotFound => {
                ApiError::NotFound
            }
            _ => {
                tracing::error!(error = %error, "investigator operation failed");
                ApiError::InvalidCommunicationQuery("investigator operation failed")
            }
        }
    }
}

impl From<CalendarCoreError> for ApiError {
    fn from(error: CalendarCoreError) -> Self {
        match error {
            CalendarCoreError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "calendar core operation failed");
                ApiError::InvalidCommunicationQuery("calendar core operation failed")
            }
        }
    }
}
impl From<MeetingsError> for ApiError {
    fn from(error: MeetingsError) -> Self {
        match error {
            MeetingsError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "meetings operation failed");
                ApiError::InvalidCommunicationQuery("meetings operation failed")
            }
        }
    }
}
impl From<SchedulingError> for ApiError {
    fn from(error: SchedulingError) -> Self {
        match error {
            SchedulingError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "scheduling operation failed");
                ApiError::InvalidCommunicationQuery("scheduling operation failed")
            }
        }
    }
}
impl From<CalendarHealthError> for ApiError {
    fn from(error: CalendarHealthError) -> Self {
        tracing::error!(error = %error, "calendar health operation failed");
        ApiError::InvalidCommunicationQuery("calendar health operation failed")
    }
}
impl From<CalendarBrainError> for ApiError {
    fn from(error: CalendarBrainError) -> Self {
        match error {
            CalendarBrainError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "calendar brain operation failed");
                ApiError::InvalidCommunicationQuery("calendar brain operation failed")
            }
        }
    }
}
impl From<ReminderError> for ApiError {
    fn from(error: ReminderError) -> Self {
        tracing::error!(error = %error, "reminder operation failed");
        ApiError::InvalidCommunicationQuery("reminder operation failed")
    }
}

impl From<CalendarRuleError> for ApiError {
    fn from(error: CalendarRuleError) -> Self {
        match error {
            CalendarRuleError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "calendar rule operation failed");
                ApiError::InvalidCommunicationQuery("calendar rule operation failed")
            }
        }
    }
}

impl From<TaskError> for ApiError {
    fn from(error: TaskError) -> Self {
        match error {
            TaskError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "task operation failed");
                ApiError::InvalidCommunicationQuery("task operation failed")
            }
        }
    }
}
impl From<TaskCoreError> for ApiError {
    fn from(error: TaskCoreError) -> Self {
        match error {
            TaskCoreError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "task core operation failed");
                ApiError::InvalidCommunicationQuery("task core operation failed")
            }
        }
    }
}
impl From<TaskHealthError> for ApiError {
    fn from(error: TaskHealthError) -> Self {
        tracing::error!(error = %error, "task health failed");
        ApiError::InvalidCommunicationQuery("task health failed")
    }
}
impl From<TaskRuleError> for ApiError {
    fn from(error: TaskRuleError) -> Self {
        match error {
            TaskRuleError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "task rule failed");
                ApiError::InvalidCommunicationQuery("task rule failed")
            }
        }
    }
}

impl From<TaskBrainError> for ApiError {
    fn from(error: TaskBrainError) -> Self {
        match error {
            TaskBrainError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "task brain failed");
                ApiError::InvalidCommunicationQuery("task brain failed")
            }
        }
    }
}
impl From<CalendarError> for ApiError {
    fn from(error: CalendarError) -> Self {
        match error {
            CalendarError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "calendar operation failed");
                ApiError::InvalidCommunicationQuery("calendar operation failed")
            }
        }
    }
}

impl From<OrganizationError> for ApiError {
    fn from(error: OrganizationError) -> Self {
        match error {
            OrganizationError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "organization operation failed");
                ApiError::InvalidCommunicationQuery("organization operation failed")
            }
        }
    }
}

impl From<EmailAccountSetupError> for ApiError {
    fn from(error: EmailAccountSetupError) -> Self {
        match error {
            EmailAccountSetupError::HostVault(error) => Self::HostVault(error),
            error => Self::AccountSetup(error),
        }
    }
}

impl From<HostVaultError> for ApiError {
    fn from(error: HostVaultError) -> Self {
        Self::HostVault(error)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error(transparent)]
    Io(#[from] io::Error),
}
