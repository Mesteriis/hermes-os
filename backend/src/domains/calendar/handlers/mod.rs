// ADR-0073: calendar handlers stay grouped for the first handlers.rs extraction;
// split by accounts, events and intelligence once hard-v1 routing is stable.
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

#[derive(Serialize)]
pub(crate) struct CalendarAccountsResponse {
    items: Vec<crate::domains::calendar::events::CalendarAccount>,
}

#[derive(Deserialize)]
pub(crate) struct CalendarAccountQuery {
    provider: Option<String>,
}

pub(crate) async fn get_calendar_accounts(
    State(state): State<AppState>,
    Query(query): Query<CalendarAccountQuery>,
) -> Result<Json<CalendarAccountsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = CalendarAccountStore::new(pool)
        .list(query.provider.as_deref())
        .await?;
    Ok(Json(CalendarAccountsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewCalendarAccountRequest {
    provider: String,
    account_name: String,
    email: Option<String>,
}

pub(crate) async fn post_calendar_account(
    State(state): State<AppState>,
    Json(req): Json<NewCalendarAccountRequest>,
) -> Result<Json<crate::domains::calendar::events::CalendarAccount>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let acct = CalendarAccountStore::new(pool)
        .create(&req.provider, &req.account_name, req.email.as_deref())
        .await?;
    Ok(Json(acct))
}

pub(crate) async fn get_calendar_account(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<crate::domains::calendar::events::CalendarAccount>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CalendarAccountStore::new(pool)
        .get(&account_id)
        .await?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

pub(crate) async fn put_calendar_account(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
    Json(update): Json<CalendarAccountUpdate>,
) -> Result<Json<crate::domains::calendar::events::CalendarAccount>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let acct = CalendarAccountStore::new(pool)
        .update(&account_id, &update)
        .await?;
    Ok(Json(acct))
}

pub(crate) async fn delete_calendar_account(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CalendarAccountStore::new(pool).delete(&account_id).await?;
    Ok(Json(json!({"deleted": true})))
}

// ── Calendar Sources ───────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct CalendarSourcesResponse {
    items: Vec<crate::domains::calendar::events::CalendarSource>,
}

pub(crate) async fn get_calendar_sources(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<CalendarSourcesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = CalendarSourceStore::new(pool)
        .list_by_account(&account_id)
        .await?;
    Ok(Json(CalendarSourcesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewCalendarSourceRequest {
    name: String,
    provider_calendar_id: Option<String>,
    color: Option<String>,
    timezone: Option<String>,
}

pub(crate) async fn post_calendar_source(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
    Json(req): Json<NewCalendarSourceRequest>,
) -> Result<Json<crate::domains::calendar::events::CalendarSource>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let src = CalendarSourceStore::new(pool)
        .create(
            &account_id,
            &req.name,
            req.provider_calendar_id.as_deref(),
            req.color.as_deref(),
            req.timezone.as_deref(),
        )
        .await?;
    Ok(Json(src))
}

// ── Calendar Events ────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct CalendarEventsResponse {
    items: Vec<crate::domains::calendar::events::CalendarEvent>,
}

#[derive(Deserialize)]
pub(crate) struct CalendarEventQuery {
    account_id: Option<String>,
    source_id: Option<String>,
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
    status: Option<String>,
    event_type: Option<String>,
    limit: Option<i64>,
}

pub(crate) async fn get_calendar_events(
    State(state): State<AppState>,
    Query(query): Query<CalendarEventQuery>,
) -> Result<Json<CalendarEventsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let list_query = CalendarEventListQuery {
        account_id: query.account_id,
        source_id: query.source_id,
        from: query.from,
        to: query.to,
        status: query.status,
        event_type: query.event_type,
        limit: query.limit,
    };
    let items = CalendarEventStore::new(pool).list(&list_query).await?;
    Ok(Json(CalendarEventsResponse { items }))
}

pub(crate) async fn post_calendar_event(
    State(state): State<AppState>,
    Json(req): Json<NewCalendarEvent>,
) -> Result<Json<crate::domains::calendar::events::CalendarEvent>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let event = CalendarEventStore::new(pool).create(&req).await?;
    Ok(Json(event))
}

pub(crate) async fn get_calendar_event(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<crate::domains::calendar::events::CalendarEvent>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CalendarEventStore::new(pool)
        .get(&event_id)
        .await?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

pub(crate) async fn put_calendar_event(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(update): Json<CalendarEventUpdate>,
) -> Result<Json<crate::domains::calendar::events::CalendarEvent>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let event = CalendarEventStore::new(pool)
        .update(&event_id, &update)
        .await?;
    Ok(Json(event))
}

pub(crate) async fn delete_calendar_event(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CalendarEventStore::new(pool).delete(&event_id).await?;
    Ok(Json(json!({"deleted": true})))
}

// ── Calendar Event Reschedule / Cancel ─────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct RescheduleRequest {
    start_at: DateTime<Utc>,
    end_at: DateTime<Utc>,
}

pub(crate) async fn post_calendar_event_reschedule(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<RescheduleRequest>,
) -> Result<Json<crate::domains::calendar::events::CalendarEvent>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let event = CalendarEventStore::new(pool)
        .reschedule(&event_id, req.start_at, req.end_at)
        .await?;
    Ok(Json(event))
}

pub(crate) async fn post_calendar_event_cancel(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CalendarEventStore::new(pool)
        .set_status(&event_id, "cancelled")
        .await?;
    Ok(Json(json!({"cancelled": true})))
}

// ── Event Participants ─────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct EventParticipantsResponse {
    items: Vec<crate::domains::calendar::core::EventParticipant>,
}

pub(crate) async fn get_event_participants(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<EventParticipantsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = EventParticipantStore::new(pool)
        .list(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(EventParticipantsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewParticipantRequest {
    email: String,
    display_name: Option<String>,
    role: Option<String>,
    person_id: Option<String>,
    organization_id: Option<String>,
}

pub(crate) async fn post_event_participant(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<NewParticipantRequest>,
) -> Result<Json<crate::domains::calendar::core::EventParticipant>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let p = EventParticipantStore::new(pool)
        .add(
            &event_id,
            &req.email,
            req.display_name.as_deref(),
            req.role.as_deref(),
            req.person_id.as_deref(),
            req.organization_id.as_deref(),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(p))
}

// ── Event Relations ────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct EventRelationsResponse {
    items: Vec<crate::domains::calendar::core::EventRelation>,
}

pub(crate) async fn get_event_relations(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<EventRelationsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = EventRelationStore::new(pool)
        .list(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(EventRelationsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewRelationRequest {
    entity_type: String,
    entity_id: String,
    relation_type: String,
}

pub(crate) async fn post_event_relation(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<NewRelationRequest>,
) -> Result<Json<crate::domains::calendar::core::EventRelation>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let rel = EventRelationStore::new(pool)
        .link(
            &event_id,
            &req.entity_type,
            &req.entity_id,
            &req.relation_type,
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(rel))
}

// ── Event Context Pack ─────────────────────────────────────────────────────

pub(crate) async fn get_event_context_pack(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let pack = EventContextPackStore::new(pool)
        .get(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&pack).unwrap_or_default()))
}

pub(crate) async fn post_event_context_pack(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<ContextPackInput>,
) -> Result<Json<crate::domains::calendar::core::EventContextPack>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let pack = EventContextPackStore::new(pool)
        .upsert(&event_id, &req)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(pack))
}

// ── Event Agenda ───────────────────────────────────────────────────────────

pub(crate) async fn get_event_agenda(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let agenda = EventAgendaStore::new(pool)
        .get(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&agenda).unwrap_or_default()))
}

#[derive(Deserialize)]
pub(crate) struct SetAgendaRequest {
    items: Value,
    source: Option<String>,
}

pub(crate) async fn post_event_agenda(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<SetAgendaRequest>,
) -> Result<Json<crate::domains::calendar::core::EventAgenda>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let agenda = EventAgendaStore::new(pool)
        .set(
            &event_id,
            req.items,
            req.source.as_deref().unwrap_or("manual"),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(agenda))
}

// ── Event Checklist ────────────────────────────────────────────────────────

pub(crate) async fn get_event_checklist(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let cl = EventChecklistStore::new(pool)
        .get(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&cl).unwrap_or_default()))
}

#[derive(Deserialize)]
pub(crate) struct SetChecklistRequest {
    items: Value,
    source: Option<String>,
}

pub(crate) async fn post_event_checklist(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<SetChecklistRequest>,
) -> Result<Json<crate::domains::calendar::core::EventChecklist>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let cl = EventChecklistStore::new(pool)
        .set(
            &event_id,
            req.items,
            req.source.as_deref().unwrap_or("manual"),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(cl))
}

// ── Event Intelligence ─────────────────────────────────────────────────────

pub(crate) async fn post_event_classify(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let event = CalendarEventStore::new(pool.clone())
        .get(&event_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let participants = EventParticipantStore::new(pool.clone())
        .list(&event_id)
        .await
        .unwrap_or_default();
    let event_type = CalendarIntelligenceService::classify_event(
        &event.title,
        participants.len(),
        (event.end_at - event.start_at).num_minutes(),
    );
    let update = CalendarEventUpdate {
        event_type: Some(event_type.clone()),
        ..Default::default()
    };
    CalendarEventStore::new(pool)
        .update(&event_id, &update)
        .await?;
    Ok(Json(json!({"event_type": event_type})))
}

pub(crate) async fn post_event_analyze(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let event = CalendarEventStore::new(pool.clone())
        .get(&event_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let parts = EventParticipantStore::new(pool.clone())
        .list(&event_id)
        .await
        .unwrap_or_default();
    let has_agenda = EventAgendaStore::new(pool.clone())
        .get(&event_id)
        .await
        .map(|a| a.is_some())
        .unwrap_or(false);
    let has_checklist = EventChecklistStore::new(pool.clone())
        .get(&event_id)
        .await
        .map(|c| c.is_some())
        .unwrap_or(false);
    let has_relations = EventRelationStore::new(pool.clone())
        .list(&event_id)
        .await
        .map(|r| !r.is_empty())
        .unwrap_or(false);

    let importance = CalendarIntelligenceService::calculate_importance(
        &event.title,
        parts.len(),
        has_relations,
        false,
    );
    let readiness = CalendarIntelligenceService::calculate_readiness(
        has_agenda,
        false,
        has_relations,
        has_checklist,
        !parts.is_empty(),
    );
    let risks = CalendarIntelligenceService::detect_risks(
        has_agenda,
        false,
        !parts.is_empty(),
        has_relations,
        event.start_at < Utc::now() + chrono::Duration::hours(24),
    );

    let update = CalendarEventUpdate {
        importance_score: Some(importance),
        readiness_score: Some(readiness),
        ..Default::default()
    };
    CalendarEventStore::new(pool.clone())
        .update(&event_id, &update)
        .await?;

    Ok(Json(
        json!({"importance": importance, "readiness": readiness, "risks": risks}),
    ))
}

pub(crate) async fn get_event_risks(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let event = CalendarEventStore::new(pool.clone())
        .get(&event_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let parts = EventParticipantStore::new(pool.clone())
        .list(&event_id)
        .await
        .unwrap_or_default();
    let has_agenda = EventAgendaStore::new(pool.clone())
        .get(&event_id)
        .await
        .map(|a| a.is_some())
        .unwrap_or(false);
    let has_relations = EventRelationStore::new(pool.clone())
        .list(&event_id)
        .await
        .map(|r| !r.is_empty())
        .unwrap_or(false);
    let is_soon = event.start_at < Utc::now() + chrono::Duration::hours(24);
    let risks = CalendarIntelligenceService::detect_risks(
        has_agenda,
        false,
        !parts.is_empty(),
        has_relations,
        is_soon,
    );
    Ok(Json(json!({"risks": risks})))
}

// ── Meeting Notes ──────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct MeetingNotesResponse {
    items: Vec<crate::domains::calendar::meetings::MeetingNote>,
}

pub(crate) async fn get_meeting_notes(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<MeetingNotesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = MeetingNoteStore::new(pool)
        .list(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(MeetingNotesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewNoteRequest {
    content: String,
    format: Option<String>,
    source: Option<String>,
}

pub(crate) async fn post_meeting_note(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<NewNoteRequest>,
) -> Result<Json<crate::domains::calendar::meetings::MeetingNote>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let note = MeetingNoteStore::new(pool)
        .create(
            &event_id,
            &req.content,
            req.format.as_deref(),
            req.source.as_deref(),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(note))
}

// ── Meeting Outcomes ───────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct MeetingOutcomesResponse {
    items: Vec<crate::domains::calendar::meetings::MeetingOutcome>,
}

pub(crate) async fn get_meeting_outcomes(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<MeetingOutcomesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = MeetingOutcomeStore::new(pool)
        .list(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(MeetingOutcomesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewOutcomeRequest {
    outcome_type: String,
    title: String,
    description: Option<String>,
    owner_person_id: Option<String>,
    due_date: Option<DateTime<Utc>>,
}

pub(crate) async fn post_meeting_outcome(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<NewOutcomeRequest>,
) -> Result<Json<crate::domains::calendar::meetings::MeetingOutcome>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let outcome = MeetingOutcomeStore::new(pool)
        .add(
            &event_id,
            &req.outcome_type,
            &req.title,
            req.description.as_deref(),
            req.owner_person_id.as_deref(),
            req.due_date,
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(outcome))
}

pub(crate) async fn post_event_follow_up(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CalendarEventStore::new(pool.clone())
        .set_status(&event_id, "needs_follow_up")
        .await?;
    Ok(Json(json!({"follow_up_created": true})))
}

pub(crate) async fn get_event_follow_up_status(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let status = MeetingOutcomeStore::new(pool)
        .follow_up_status(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(status))
}

// ── Event Recordings ───────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct EventRecordingsResponse {
    items: Vec<crate::domains::calendar::meetings::EventRecording>,
}

pub(crate) async fn get_event_recordings(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<EventRecordingsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = EventRecordingStore::new(pool)
        .list(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(EventRecordingsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewRecordingRequest {
    file_path: Option<String>,
    source: Option<String>,
    duration_seconds: Option<i32>,
}

pub(crate) async fn post_event_recording(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<NewRecordingRequest>,
) -> Result<Json<crate::domains::calendar::meetings::EventRecording>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let rec = EventRecordingStore::new(pool)
        .add(
            &event_id,
            req.file_path.as_deref(),
            req.source.as_deref(),
            req.duration_seconds,
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(rec))
}

pub(crate) async fn get_event_transcript(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let t = EventTranscriptStore::new(pool)
        .get(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&t).unwrap_or_default()))
}

// ── Calendar Brain ─────────────────────────────────────────────────────────

pub(crate) async fn get_event_brief(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let brief = CalendarBrainService::meeting_brief(&pool, &event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(brief))
}

pub(crate) async fn post_generate_agenda(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let agenda = CalendarBrainService::generate_agenda(&pool, &event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(agenda))
}

// ── Deadlines ──────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct DeadlinesResponse {
    items: Vec<crate::domains::calendar::scheduling::DeadlineEvent>,
}

#[derive(Deserialize)]
pub(crate) struct DeadlineQuery {
    status: Option<String>,
    limit: Option<i64>,
}

pub(crate) async fn get_deadlines(
    State(state): State<AppState>,
    Query(query): Query<DeadlineQuery>,
) -> Result<Json<DeadlinesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = DeadlineStore::new(pool)
        .list(query.status.as_deref(), query.limit.unwrap_or(50))
        .await
        .map_err(ApiError::from)?;
    Ok(Json(DeadlinesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewDeadlineRequest {
    title: String,
    due_at: DateTime<Utc>,
    severity: Option<String>,
    source_entity_type: Option<String>,
    source_entity_id: Option<String>,
}

pub(crate) async fn post_deadline(
    State(state): State<AppState>,
    Json(req): Json<NewDeadlineRequest>,
) -> Result<Json<crate::domains::calendar::scheduling::DeadlineEvent>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let d = DeadlineStore::new(pool)
        .create(
            &req.title,
            req.due_at,
            req.severity.as_deref(),
            req.source_entity_type.as_deref(),
            req.source_entity_id.as_deref(),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(d))
}

// ── Focus Blocks ───────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct FocusBlocksResponse {
    items: Vec<crate::domains::calendar::scheduling::FocusBlock>,
}

#[derive(Deserialize)]
pub(crate) struct FocusBlockQuery {
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
    limit: Option<i64>,
}

pub(crate) async fn get_focus_blocks(
    State(state): State<AppState>,
    Query(query): Query<FocusBlockQuery>,
) -> Result<Json<FocusBlocksResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = FocusBlockStore::new(pool)
        .list(query.from, query.to, query.limit.unwrap_or(50))
        .await
        .map_err(ApiError::from)?;
    Ok(Json(FocusBlocksResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewFocusBlockRequest {
    title: String,
    start_at: DateTime<Utc>,
    end_at: DateTime<Utc>,
    purpose: Option<String>,
    linked_project_id: Option<String>,
    protection_level: Option<String>,
}

pub(crate) async fn post_focus_block(
    State(state): State<AppState>,
    Json(req): Json<NewFocusBlockRequest>,
) -> Result<Json<crate::domains::calendar::scheduling::FocusBlock>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let fb = FocusBlockStore::new(pool)
        .create(
            &req.title,
            req.start_at,
            req.end_at,
            req.purpose.as_deref(),
            req.linked_project_id.as_deref(),
            req.protection_level.as_deref(),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(fb))
}

// ── Smart Schedule ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct SmartScheduleRequest {
    duration_minutes: Option<i64>,
    lookahead_hours: Option<i64>,
}

pub(crate) async fn post_smart_schedule(
    State(state): State<AppState>,
    Json(req): Json<SmartScheduleRequest>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let events = CalendarEventStore::new(pool)
        .list(&CalendarEventListQuery {
            limit: Some(200),
            ..Default::default()
        })
        .await?;
    let pairs: Vec<(DateTime<Utc>, DateTime<Utc>)> =
        events.iter().map(|e| (e.start_at, e.end_at)).collect();
    let slots = SmartSchedulingService::find_slots(
        &pairs,
        req.duration_minutes.unwrap_or(30),
        req.lookahead_hours.unwrap_or(48),
    );
    Ok(Json(json!({"slots": slots})))
}

// ── Calendar Watchtower ────────────────────────────────────────────────────

pub(crate) async fn get_calendar_watchtower(
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let preparation = CalendarWatchtowerService::events_needing_preparation(&pool)
        .await
        .map_err(ApiError::from)?;
    let no_outcomes = CalendarWatchtowerService::events_without_outcomes(&pool)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(
        json!({"preparation": preparation, "without_outcomes": no_outcomes}),
    ))
}

pub(crate) async fn get_calendar_health(
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let load = CalendarWatchtowerService::meeting_load_analysis(&pool)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(load))
}

// ── Weekly Brief ───────────────────────────────────────────────────────────

pub(crate) async fn get_weekly_brief(
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let brief = CalendarWatchtowerService::weekly_brief(&pool)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(brief))
}

// ── Calendar Analytics ─────────────────────────────────────────────────────

pub(crate) async fn get_calendar_analytics(
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let load = CalendarWatchtowerService::meeting_load_analysis(&pool)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(load))
}

// ── Calendar Brain ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct CalendarBrainQueryParams {
    q: String,
}

pub(crate) async fn post_calendar_brain(
    State(state): State<AppState>,
    Json(req): Json<CalendarBrainQueryParams>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let answer = CalendarBrainService::answer(&pool, &req.q)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(answer))
}

// ── Calendar Search ────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct CalendarSearchQuery {
    q: String,
}

pub(crate) async fn get_calendar_search(
    State(state): State<AppState>,
    Query(query): Query<CalendarSearchQuery>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let results = CalendarBrainService::search_events(&pool, &query.q)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(results))
}

// ── Calendar Rules ─────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct CalendarRulesResponse {
    items: Vec<crate::domains::calendar::rules::CalendarRule>,
}

pub(crate) async fn get_calendar_rules(
    State(state): State<AppState>,
) -> Result<Json<CalendarRulesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = CalendarRuleStore::new(pool)
        .list()
        .await
        .map_err(ApiError::from)?;
    Ok(Json(CalendarRulesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewRuleRequest {
    name: String,
    description: Option<String>,
    dsl: Value,
    approval_mode: Option<String>,
}

pub(crate) async fn post_calendar_rule(
    State(state): State<AppState>,
    Json(req): Json<NewRuleRequest>,
) -> Result<Json<crate::domains::calendar::rules::CalendarRule>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let rule = CalendarRuleStore::new(pool)
        .create(
            &req.name,
            req.description.as_deref(),
            req.dsl,
            req.approval_mode.as_deref(),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(rule))
}

pub(crate) async fn put_calendar_rule(
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
    Json(update): Json<RuleUpdate>,
) -> Result<Json<crate::domains::calendar::rules::CalendarRule>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let rule = CalendarRuleStore::new(pool)
        .update(&rule_id, &update)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(rule))
}

pub(crate) async fn delete_calendar_rule(
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CalendarRuleStore::new(pool)
        .delete(&rule_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(json!({"deleted": true})))
}

// ── Calendar Import/Export ─────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct CalendarImportRequest {
    ics_data: Option<String>,
    events: Option<Value>,
}

pub(crate) async fn post_calendar_import(
    State(state): State<AppState>,
    Json(req): Json<CalendarImportRequest>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let ics_data_received = req
        .ics_data
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty());
    let mut imported = 0;
    if let Some(events) = req.events {
        if let Some(arr) = events.as_array() {
            for evt in arr {
                let title = evt
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Imported Event");
                let start = evt
                    .get("start_at")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<DateTime<Utc>>().ok())
                    .unwrap_or(Utc::now());
                let end = evt
                    .get("end_at")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<DateTime<Utc>>().ok())
                    .unwrap_or(start);
                let _ = CalendarEventStore::new(pool.clone())
                    .create(&NewCalendarEvent {
                        title: title.to_string(),
                        start_at: start,
                        end_at: end,
                        ..Default::default()
                    })
                    .await;
                imported += 1;
            }
        }
    }
    Ok(Json(
        json!({"imported": imported, "ics_data_received": ics_data_received}),
    ))
}

pub(crate) async fn post_calendar_sync(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CalendarAccountStore::new(pool.clone())
        .update(
            &account_id,
            &CalendarAccountUpdate {
                sync_status: Some("syncing".into()),
                ..Default::default()
            },
        )
        .await?;
    Ok(Json(
        json!({"sync_triggered": true, "note": "Provider sync is deferred to future implementation"}),
    ))
}

#[derive(Deserialize)]
pub(crate) struct EventExportQuery {
    format: Option<String>,
}

pub(crate) async fn get_event_export(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Query(query): Query<EventExportQuery>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let event = CalendarEventStore::new(pool)
        .get(&event_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let fmt = query.format.as_deref().unwrap_or("json");
    match fmt {
        "ics" => {
            let ics = export_event_ics(
                &event.title,
                event.description.as_deref(),
                event.location.as_deref(),
                &event.start_at.format("%Y%m%dT%H%M%S").to_string(),
                &event.end_at.format("%Y%m%dT%H%M%S").to_string(),
                event.timezone.as_deref(),
            );
            Ok(Json(json!({"format": "ics", "content": ics})))
        }
        "md" => {
            let md = export_event_md(
                &event.title,
                event.description.as_deref(),
                event.location.as_deref(),
                &event.start_at.to_rfc3339(),
                &event.end_at.to_rfc3339(),
                &[],
            );
            Ok(Json(json!({"format": "markdown", "content": md})))
        }
        _ => Ok(Json(serde_json::to_value(&event).unwrap_or_default())),
    }
}

// ── Event Reminders ────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct EventRemindersResponse {
    items: Vec<crate::domains::calendar::reminders::CalendarReminder>,
}

pub(crate) async fn get_event_reminders(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<EventRemindersResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = CalendarReminderStore::new(pool)
        .list(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(EventRemindersResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewReminderRequest {
    reminder_type: String,
    minutes_before: Option<i32>,
    message: Option<String>,
}

pub(crate) async fn post_event_reminder(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<NewReminderRequest>,
) -> Result<Json<crate::domains::calendar::reminders::CalendarReminder>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let r = CalendarReminderStore::new(pool)
        .create(
            &event_id,
            &req.reminder_type,
            req.minutes_before,
            req.message.as_deref(),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(r))
}

#[derive(Deserialize)]
pub(crate) struct ToggleReminderRequest {
    active: bool,
}

pub(crate) async fn post_event_reminder_toggle(
    State(state): State<AppState>,
    Path((_event_id, reminder_id)): Path<(String, String)>,
    Json(req): Json<ToggleReminderRequest>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CalendarReminderStore::new(pool)
        .set_active(&reminder_id, req.active)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(json!({"toggled": true, "active": req.active})))
}

// ── Analytics: Time Distribution ───────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct AnalyticsRangeQuery {
    from: Option<String>,
    to: Option<String>,
}

pub(crate) async fn get_time_distribution(
    State(state): State<AppState>,
    Query(query): Query<AnalyticsRangeQuery>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let now = Utc::now();
    let from: DateTime<Utc> = query
        .from
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(now - chrono::Duration::days(7));
    let to: DateTime<Utc> = query
        .to
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(now);
    let dist = CalendarWatchtowerService::time_distribution(&pool, from, to)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(dist))
}

pub(crate) async fn get_focus_balance(
    State(state): State<AppState>,
    Query(query): Query<AnalyticsRangeQuery>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let now = Utc::now();
    let from: DateTime<Utc> = query
        .from
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(now - chrono::Duration::days(7));
    let to: DateTime<Utc> = query
        .to
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(now);
    let balance = CalendarWatchtowerService::focus_balance(&pool, from, to)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(balance))
}

pub(crate) async fn get_back_to_back(
    State(state): State<AppState>,
    Query(query): Query<AnalyticsRangeQuery>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let now = Utc::now();
    let from: DateTime<Utc> = query
        .from
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(now - chrono::Duration::days(7));
    let to: DateTime<Utc> = query
        .to
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(now);
    let b2b = CalendarWatchtowerService::back_to_back_meetings(&pool, from, to)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(b2b))
}
