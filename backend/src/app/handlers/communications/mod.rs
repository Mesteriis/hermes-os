// ADR-0073: mail handlers are grouped by bounded context for the first
// handlers.rs extraction; split by communications, accounts and workflow next.
mod account_management;
mod account_setup;
mod account_support;
mod communication_messages;
mod communication_queries;
mod finance_analytics;
mod legal_export;
mod message_actions;
mod message_ai_state;
mod remote_images;
mod sending;
mod templates_status;
mod workflow_actions;
mod workflow_state;
pub(crate) use account_management::*;
pub(crate) use account_setup::*;
use account_support::*;
pub(crate) use communication_messages::*;
pub(crate) use communication_queries::*;
pub(crate) use finance_analytics::*;
pub(crate) use legal_export::*;
pub(crate) use message_actions::*;
pub(crate) use message_ai_state::*;
pub(crate) use remote_images::get_v1_communication_message_remote_image;
pub(crate) use sending::*;
pub(crate) use templates_status::*;
pub(crate) use workflow_actions::{
    WorkflowActionInput, WorkflowActionKind, WorkflowActionProvenance, WorkflowActionRequest,
    WorkflowActionResponse, WorkflowActionSource, WorkflowActionStatus, WorkflowActionTarget,
    WorkflowActionTargetKind, execute_workflow_action, post_v1_workflow_action,
};
pub(crate) use workflow_state::*;

use std::collections::HashMap;
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
    ProviderAccountSecretPurpose, ProviderCredentialReader,
};
use crate::domains::personas::analytics::{AnalyticsError, PersonaAnalyticsService};
use crate::domains::personas::enrichment_engine::{EnrichmentEngineError, EnrichmentResultStore};
use crate::domains::personas::expertise::{PersonaExpertiseError, PersonaExpertiseStore};
use crate::domains::personas::export::{ExportError, ExportFormat, PersonaExportService};
use crate::domains::personas::investigator::{InvestigatorError, PersonaInvestigator};
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

use crate::domains::personas::health::{PersonaHealthError, PersonaHealthStore};

use crate::domains::personas::trust::{PersonaPromiseStore, PersonaRiskStore, PersonaTrustError};

use crate::domains::personas::memory::{
    NewRelationshipEvent, PersonaFactStore, PersonaMemoryCardStore, PersonaMemoryError,
    PersonaPreferenceStore, RelationshipEventStore,
};

use crate::domains::personas::core::{
    NewPersonaInteractionContext, PersonaCoreError, PersonaIdentity, PersonaIdentityStore,
    PersonaInteractionContext, PersonaInteractionContextStore, PersonaRole, PersonaRoleStore,
};
use crate::domains::personas::identity::{
    PersonaIdentityCandidate, PersonaIdentityDetail, PersonaIdentityError,
    PersonaIdentityReviewCommand, PersonaIdentityReviewState, PersonaIdentityReviewStore,
};

use crate::application::address_book_sync::{
    AddressBookSyncError, AddressBookSyncRunResponse, AddressBookSyncService,
    AddressBookSyncTrigger,
};
use crate::application::email_intelligence::{
    EmailIntelligenceError, EmailIntelligenceService, EmailSummaryContract,
};
use crate::application::mail_background_sync::{
    DEFAULT_MAIL_SYNC_BLOB_ROOT, MailBackgroundSyncService, MailSyncError, MailSyncRunResponse,
    MailSyncSettings, MailSyncSettingsUpdate, MailSyncStatus, MailSyncStore, MailSyncTrigger,
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
use crate::domains::communications::core::CommunicationProviderAccountStore;
use crate::domains::communications::messages::{
    LocalMessageState, MessageProjectionError, MessageProjectionStore, ProjectedMessage,
    ProjectedMessageSummary, WorkflowState, parse_raw_email_message_from_blob,
};
use crate::domains::communications::storage::{
    CommunicationStorageError, CommunicationStorageStore, LocalCommunicationBlobStore,
    StoredCommunicationAttachmentWithBlob,
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
use crate::platform::events::{
    EventEnvelope, EventEnvelopeError, EventStore, EventStoreError, NewEventEnvelope,
};
use crate::platform::secrets::DatabaseEncryptedSecretVault;
use crate::platform::secrets::{SecretKind, SecretReferenceStore, SecretStoreKind};
use crate::platform::settings::{
    AiRuntimeSettings, ApplicationSetting, ApplicationSettingsStore, SettingsError,
};
use crate::platform::storage::{
    Database, DatabaseReadiness, MigrationReadiness, ReadinessStatus, StorageError,
};
use crate::vault::{EntropyEvent, HostVaultError, VaultMode};

use crate::app::api_support::*;
use crate::app::{ApiError, AppState};
