use hermes_communications_api::accounts::{CommunicationProviderKind, ProviderAccount};
use hermes_events_api::{EventEnvelope, EventEnvelopeError, NewEventEnvelope};
// ADR-0073: calendar handlers are split by documented Calendar domain responsibilities.
mod accounts;
mod analytics;
mod brain;
mod events;
mod health;
mod intelligence;
mod meetings;
mod reminders;
mod rules;
mod scheduling;
mod search;
mod sync;

pub(crate) use accounts::*;
pub(crate) use analytics::*;
pub(crate) use brain::*;
pub(crate) use events::*;
pub(crate) use health::*;
pub(crate) use intelligence::*;
pub(crate) use meetings::*;
pub(crate) use reminders::*;
pub(crate) use rules::*;
pub(crate) use scheduling::*;
pub(crate) use search::*;
pub(crate) use sync::*;

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
use crate::domains::personas::analytics::{AnalyticsError, PersonaAnalyticsService};
use crate::domains::personas::enrichment_engine::{EnrichmentEngineError, EnrichmentResultStore};
use crate::domains::personas::expertise::{PersonaExpertiseError, PersonaExpertiseStore};
use crate::domains::personas::export::{ExportError, ExportFormat, PersonaExportService};
use crate::domains::personas::investigator::{InvestigatorError, PersonaInvestigator};
use crate::engines::automation::{
    errors::AutomationError,
    models::{
        AutomationPolicy, AutomationTemplate, NewAutomationPolicy, NewAutomationTemplate,
        TelegramSendDryRunRequest, TelegramSendDryRunResponse,
    },
    store::AutomationStore,
};
use crate::platform::audit::{ApiAuditError, ApiAuditLog, ApiAuditRecord, NewApiAuditRecord};
use crate::platform::calls::{
    CallDirection, CallError, CallIntelligenceStore, CallState, CallTranscript,
    FixtureSpeechToTextProvider, NewCallTranscript, NewTelegramCall, SpeechToTextProvider,
    TelegramCall, TranscriptStatus,
};
use crate::platform::capabilities::{CapabilityActionClass, CapabilityDecision};
use hermes_communications_postgres::errors::CommunicationIngestionError;
use hermes_communications_postgres::store::CommunicationIngestionStore;

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

use crate::domains::calendar::brain::{CalendarBrainError, CalendarBrainService};
use crate::domains::calendar::command_service::{
    CalendarCommandService, CalendarCommandServiceError,
};
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
use crate::integrations::ollama::client::OllamaClient;
use crate::integrations::ollama::client::config::OllamaClientConfig;
use crate::platform::secrets::DatabaseEncryptedSecretVault;
use crate::platform::secrets::{SecretKind, SecretReferenceStore};
use crate::platform::settings::{
    AiRuntimeSettings, ApplicationSetting, ApplicationSettingsStore, SettingsError,
};
use crate::platform::storage::{
    Database, DatabaseReadiness, MigrationReadiness, ReadinessStatus, StorageError,
};
use crate::workflows::email_intelligence::errors::EmailIntelligenceError;
use crate::workflows::email_intelligence::service::EmailIntelligenceService;
use hermes_events_postgres::errors::EventStoreError;
use hermes_events_postgres::store::EventStore;

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
use crate::app::{ApiError, AppState};
use crate::application::calendar_meeting_outcomes::CalendarMeetingOutcomeApplicationService;
