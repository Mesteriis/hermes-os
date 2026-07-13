use hermes_communications_api::accounts::{CommunicationProviderKind, ProviderAccount};
use hermes_events_api::{EventEnvelope, EventEnvelopeError, NewEventEnvelope};
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
    errors::EmailAccountSetupError,
    models::{GmailOAuthPendingGrant, GmailOAuthSetupRequest, ImapAccountSetupRequest},
    service::EmailAccountSetupService,
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

pub(crate) async fn get_graph_summary(
    State(state): State<AppState>,
) -> Result<Json<crate::domains::graph::core::GraphSummary>, ApiError> {
    Ok(Json(graph_store(&state)?.summary().await?))
}

pub(crate) async fn get_graph_nodes(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<Vec<crate::domains::graph::core::GraphNode>>, ApiError> {
    let query = parse_graph_nodes_query(raw_query.as_deref())?;
    let limit = query.limit.unwrap_or(20).clamp(1, 50);
    Ok(Json(
        graph_store(&state)?.list_nodes_for_picker(limit).await?,
    ))
}

pub(crate) async fn get_graph_neighborhood(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<crate::domains::graph::core::GraphNeighborhood>, ApiError> {
    let query = parse_graph_neighborhood_query(raw_query.as_deref())?;
    if query.depth.unwrap_or(1) != 1 {
        return Err(ApiError::InvalidGraphQuery("depth supports only 1"));
    }
    let Some(node_id) = query
        .node_id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty())
    else {
        return Err(ApiError::GraphNotFound);
    };
    let Some(neighborhood) = graph_store(&state)?.neighborhood(node_id).await? else {
        return Err(ApiError::GraphNotFound);
    };
    Ok(Json(neighborhood))
}

pub(crate) async fn get_graph_search(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<Vec<crate::domains::graph::core::GraphNode>>, ApiError> {
    let query = parse_graph_search_query(raw_query.as_deref())?;
    let search = query.q.as_deref().unwrap_or_default().trim();
    if search.is_empty() {
        return Err(ApiError::InvalidGraphQuery("q must not be empty"));
    }
    let limit = query.limit.unwrap_or(20).clamp(1, 50);
    Ok(Json(
        graph_store(&state)?.search_nodes(search, limit).await?,
    ))
}
