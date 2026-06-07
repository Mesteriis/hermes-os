// ADR-0073: person handlers are grouped for the first handlers.rs extraction;
// split profile, identity and intelligence handlers in a follow-up pass.
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
pub(crate) struct PersonListResponse {
    items: Vec<crate::domains::persons::enrichment::EnrichedPerson>,
}

#[derive(Deserialize)]
pub(crate) struct PersonListQuery {
    favorites_only: Option<bool>,
    limit: Option<i64>,
}

pub(crate) async fn get_persons(
    State(state): State<AppState>,
    Query(query): Query<PersonListQuery>,
) -> Result<Json<PersonListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::persons::enrichment::PersonEnrichmentStore::new(pool);
    let items = store
        .list_enriched(
            query.favorites_only.unwrap_or(false),
            query.limit.unwrap_or(50),
        )
        .await?;
    Ok(Json(PersonListResponse { items }))
}

pub(crate) async fn get_person(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<crate::domains::persons::enrichment::EnrichedPerson>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::persons::enrichment::PersonEnrichmentStore::new(pool);
    match store.get_enriched(&person_id).await? {
        Some(person) => Ok(Json(person)),
        None => Err(ApiError::PersonIdentityNotFound),
    }
}

pub(crate) async fn post_person_fingerprint(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let msg_store = crate::domains::mail::messages::MessageProjectionStore::new(pool.clone());
    // Build person messages from this person's email history
    let messages = msg_store.recent_messages(50).await?;
    let person_msgs: Vec<crate::domains::persons::intelligence::PersonMessage> = messages
        .into_iter()
        .filter(|m| {
            m.message.sender.contains(&person_id)
                || m.message.recipients.iter().any(|r| r.contains(&person_id))
        })
        .map(|m| crate::domains::persons::intelligence::PersonMessage {
            subject: m.message.subject,
            body_text: m.message.body_text,
            occurred_at: m.message.occurred_at,
        })
        .collect();
    let fp =
        crate::domains::persons::intelligence::PersonIntelligenceService::heuristic_fingerprint(
            &person_msgs,
        );
    let store = crate::domains::persons::enrichment::PersonEnrichmentStore::new(pool);
    store.enrich_person(&person_id, &fp).await?;
    Ok(Json(
        serde_json::json!({"enriched": true, "fingerprint": fp}),
    ))
}

pub(crate) async fn post_person_favorite(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::persons::enrichment::PersonEnrichmentStore::new(pool);
    let fav = store.toggle_favorite(&person_id).await?;
    Ok(Json(serde_json::json!({"is_favorite": fav})))
}

#[derive(Deserialize)]
pub(crate) struct PersonNotesRequest {
    notes: String,
}
pub(crate) async fn put_person_notes(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Json(req): Json<PersonNotesRequest>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::persons::enrichment::PersonEnrichmentStore::new(pool);
    store.set_notes(&person_id, &req.notes).await?;
    Ok(Json(serde_json::json!({"saved": true})))
}

#[derive(Deserialize)]
pub(crate) struct PersonSearchQuery {
    q: String,
    limit: Option<i64>,
}
pub(crate) async fn get_person_search(
    State(state): State<AppState>,
    Query(query): Query<PersonSearchQuery>,
) -> Result<Json<PersonListResponse>, ApiError> {
    if query.q.trim().is_empty() {
        return Err(ApiError::InvalidCommunicationQuery("search query required"));
    }
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::persons::enrichment::PersonEnrichmentStore::new(pool);
    let items = store
        .search_persons(&query.q, query.limit.unwrap_or(20))
        .await?;
    Ok(Json(PersonListResponse { items }))
}

#[derive(Serialize)]
pub(crate) struct PersonIdentitiesResponse {
    items: Vec<PersonIdentity>,
}

pub(crate) async fn get_person_identities(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<PersonIdentitiesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = PersonsIdentityStore::new(pool);
    let items = store
        .list_by_person(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonIdentitiesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewPersonIdentityRequest {
    identity_type: String,
    identity_value: String,
    source: Option<String>,
}

pub(crate) async fn post_person_identity(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Json(req): Json<NewPersonIdentityRequest>,
) -> Result<Json<PersonIdentity>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = PersonsIdentityStore::new(pool);
    let identity = store
        .upsert(
            &person_id,
            &req.identity_type,
            &req.identity_value,
            req.source.as_deref().unwrap_or("manual"),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(identity))
}

pub(crate) async fn delete_person_identity(
    State(state): State<AppState>,
    Path((_person_id, identity_id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = PersonsIdentityStore::new(pool);
    let deleted = store.delete(&identity_id).await.map_err(ApiError::from)?;
    Ok(Json(json!({"deleted": deleted})))
}

// ── Person Roles ────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonRolesResponse {
    items: Vec<PersonRole>,
}

pub(crate) async fn get_person_roles(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<PersonRolesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = PersonRoleStore::new(pool);
    let items = store
        .list_by_person(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonRolesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewPersonRoleRequest {
    role: String,
}

pub(crate) async fn post_person_role(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Json(req): Json<NewPersonRoleRequest>,
) -> Result<Json<PersonRole>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = PersonRoleStore::new(pool);
    let role = store
        .assign(&person_id, &req.role, None)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(role))
}

pub(crate) async fn delete_person_role(
    State(state): State<AppState>,
    Path((person_id, role)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = PersonRoleStore::new(pool);
    let deleted = store
        .remove(&person_id, &role)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(json!({"deleted": deleted})))
}

// ── Person Personas ─────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonPersonasResponse {
    items: Vec<PersonPersona>,
}

pub(crate) async fn get_person_personas(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<PersonPersonasResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = PersonPersonaStore::new(pool);
    let items = store
        .list_by_person(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonPersonasResponse { items }))
}

pub(crate) async fn post_person_persona(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Json(req): Json<NewPersonPersona>,
) -> Result<Json<PersonPersona>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = PersonPersonaStore::new(pool);
    let persona = store
        .upsert(&NewPersonPersona { person_id, ..req })
        .await
        .map_err(ApiError::from)?;
    Ok(Json(persona))
}

pub(crate) async fn delete_person_persona(
    State(state): State<AppState>,
    Path((_person_id, persona_id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = PersonPersonaStore::new(pool);
    let deleted = store.delete(&persona_id).await.map_err(ApiError::from)?;
    Ok(Json(json!({"deleted": deleted})))
}

// ── Person Facts ────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonFactsResponse {
    items: Vec<crate::domains::persons::memory::PersonFact>,
}

pub(crate) async fn get_person_facts(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<PersonFactsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = PersonFactStore::new(pool)
        .list(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonFactsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewPersonFactRequest {
    fact_type: String,
    value: String,
    source: Option<String>,
    confidence: Option<f64>,
}

pub(crate) async fn post_person_fact(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Json(req): Json<NewPersonFactRequest>,
) -> Result<Json<crate::domains::persons::memory::PersonFact>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let fact = PersonFactStore::new(pool)
        .upsert(
            &person_id,
            &req.fact_type,
            &req.value,
            req.source.as_deref().unwrap_or("manual"),
            req.confidence.unwrap_or(1.0),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(fact))
}

// ── Person Memory Cards ─────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonMemoryCardsResponse {
    items: Vec<crate::domains::persons::memory::PersonMemoryCard>,
}

pub(crate) async fn get_person_memory_cards(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<PersonMemoryCardsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = PersonMemoryCardStore::new(pool)
        .list(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonMemoryCardsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewPersonMemoryCardRequest {
    title: String,
    description: String,
    source: Option<String>,
    importance: Option<i16>,
}

pub(crate) async fn post_person_memory_card(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Json(req): Json<NewPersonMemoryCardRequest>,
) -> Result<Json<crate::domains::persons::memory::PersonMemoryCard>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let card = PersonMemoryCardStore::new(pool)
        .upsert(
            &person_id,
            &req.title,
            &req.description,
            req.source.as_deref().unwrap_or("manual"),
            req.importance.unwrap_or(5),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(card))
}

// ── Person Preferences ──────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonPreferencesResponse {
    items: Vec<crate::domains::persons::memory::PersonPreference>,
}

pub(crate) async fn get_person_preferences(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<PersonPreferencesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = PersonPreferenceStore::new(pool)
        .list(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonPreferencesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewPersonPreferenceRequest {
    preference_type: String,
    value: String,
    source: Option<String>,
}

pub(crate) async fn post_person_preference(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Json(req): Json<NewPersonPreferenceRequest>,
) -> Result<Json<crate::domains::persons::memory::PersonPreference>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let pref = PersonPreferenceStore::new(pool)
        .upsert(
            &person_id,
            &req.preference_type,
            &req.value,
            req.source.as_deref().unwrap_or("manual"),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(pref))
}

// ── Relationship Timeline ───────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct RelationshipTimelineResponse {
    items: Vec<crate::domains::persons::memory::RelationshipEvent>,
}

pub(crate) async fn get_person_timeline(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Query(query): Query<TimelineQuery>,
) -> Result<Json<RelationshipTimelineResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = RelationshipEventStore::new(pool)
        .timeline(&person_id, query.limit.unwrap_or(50))
        .await
        .map_err(ApiError::from)?;
    Ok(Json(RelationshipTimelineResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct TimelineQuery {
    limit: Option<i64>,
}

pub(crate) async fn post_relationship_event(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Json(req): Json<NewRelationshipEvent>,
) -> Result<Json<crate::domains::persons::memory::RelationshipEvent>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let event = RelationshipEventStore::new(pool)
        .add(&NewRelationshipEvent { person_id, ..req })
        .await
        .map_err(ApiError::from)?;
    Ok(Json(event))
}

impl From<EnrichmentEngineError> for ApiError {
    fn from(error: EnrichmentEngineError) -> Self {
        tracing::error!(error = %error, "enrichment engine operation failed");
        ApiError::InvalidCommunicationQuery("enrichment engine operation failed")
    }
}

impl From<PersonExpertiseError> for ApiError {
    fn from(error: PersonExpertiseError) -> Self {
        tracing::error!(error = %error, "expertise operation failed");
        ApiError::InvalidCommunicationQuery("expertise operation failed")
    }
}

impl From<PersonTrustError> for ApiError {
    fn from(error: PersonTrustError) -> Self {
        tracing::error!(error = %error, "trust operation failed");
        ApiError::InvalidCommunicationQuery("trust operation failed")
    }
}

// ── Person Enrichment ──────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct EnrichmentResultsResponse {
    items: Vec<crate::domains::persons::enrichment_engine::EnrichmentResult>,
}

pub(crate) async fn get_person_enrichment(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<EnrichmentResultsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = EnrichmentResultStore::new(pool)
        .list(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(EnrichmentResultsResponse { items }))
}

pub(crate) async fn post_person_enrichment_apply(
    State(state): State<AppState>,
    Path((_person_id, result_id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    EnrichmentResultStore::new(pool)
        .apply(&result_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(json!({"applied": true})))
}

pub(crate) async fn post_person_enrichment_reject(
    State(state): State<AppState>,
    Path((_person_id, result_id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    EnrichmentResultStore::new(pool)
        .reject(&result_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(json!({"rejected": true})))
}

// ── Person Expertise ───────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonExpertiseResponse {
    items: Vec<crate::domains::persons::expertise::PersonExpertise>,
}

pub(crate) async fn get_person_expertise(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<PersonExpertiseResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = PersonExpertiseStore::new(pool)
        .list(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonExpertiseResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct ExpertiseSearchQuery {
    skill: String,
    limit: Option<i64>,
}

pub(crate) async fn get_person_expertise_search(
    State(state): State<AppState>,
    Query(query): Query<ExpertiseSearchQuery>,
) -> Result<Json<PersonExpertiseResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = PersonExpertiseStore::new(pool)
        .search_by_skill(&query.skill, query.limit.unwrap_or(20))
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonExpertiseResponse { items }))
}

// ── Person Promises ────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonPromisesResponse {
    items: Vec<crate::domains::persons::trust::PersonPromise>,
}

pub(crate) async fn get_person_promises(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<PersonPromisesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = PersonPromiseStore::new(pool)
        .list(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonPromisesResponse { items }))
}

// ── Person Risks ────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonRisksResponse {
    items: Vec<crate::domains::persons::trust::PersonRisk>,
}

pub(crate) async fn get_person_risks(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<PersonRisksResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = PersonRiskStore::new(pool)
        .list(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonRisksResponse { items }))
}

impl From<PersonHealthError> for ApiError {
    fn from(error: PersonHealthError) -> Self {
        tracing::error!(error = %error, "health operation failed");
        ApiError::InvalidCommunicationQuery("health operation failed")
    }
}

// ── Person Health ──────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonHealthResponse {
    items: Vec<crate::domains::persons::health::PersonHealth>,
}

pub(crate) async fn get_person_health(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<crate::domains::persons::health::PersonHealth>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    PersonHealthStore::new(pool)
        .get(&person_id)
        .await
        .map_err(ApiError::from)?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

pub(crate) async fn get_persons_health(
    State(state): State<AppState>,
) -> Result<Json<PersonHealthResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = PersonHealthStore::new(pool)
        .list_health()
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonHealthResponse { items }))
}

pub(crate) async fn get_persons_watchlist(
    State(state): State<AppState>,
) -> Result<Json<PersonHealthResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = PersonHealthStore::new(pool)
        .list_watchlist()
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonHealthResponse { items }))
}

pub(crate) async fn post_person_watchlist_toggle(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let on = PersonHealthStore::new(pool)
        .toggle_watchlist(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(json!({"watchlist": on})))
}

impl From<InvestigatorError> for ApiError {
    fn from(error: InvestigatorError) -> Self {
        match error {
            InvestigatorError::PersonNotFound => ApiError::PersonIdentityNotFound,
            _ => {
                tracing::error!(error = %error, "investigator operation failed");
                ApiError::InvalidCommunicationQuery("investigator operation failed")
            }
        }
    }
}

impl From<AnalyticsError> for ApiError {
    fn from(error: AnalyticsError) -> Self {
        tracing::error!(error = %error, "analytics operation failed");
        ApiError::InvalidCommunicationQuery("analytics operation failed")
    }
}

impl From<ExportError> for ApiError {
    fn from(error: ExportError) -> Self {
        tracing::error!(error = %error, "export operation failed");
        ApiError::InvalidCommunicationQuery("export operation failed")
    }
}

// ── Person Investigator ────────────────────────────────────────────────────

pub(crate) async fn post_person_investigate(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let dossier = PersonInvestigator::new(pool)
        .assemble_dossier(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&dossier).unwrap_or_default()))
}

pub(crate) async fn get_person_dossier(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let dossier = PersonInvestigator::new(pool)
        .assemble_dossier(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&dossier).unwrap_or_default()))
}

pub(crate) async fn get_person_meeting_prep(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let prep = PersonInvestigator::new(pool)
        .meeting_prep(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&prep).unwrap_or_default()))
}

// ── Person Analytics ────────────────────────────────────────────────────────

pub(crate) async fn get_person_analytics(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let analytics = PersonAnalyticsService::new(pool)
        .compute(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&analytics).unwrap_or_default()))
}

// ── Person Export ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct PersonDownloadQuery {
    format: Option<String>,
}

pub(crate) async fn get_person_export_handler(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Query(query): Query<PersonDownloadQuery>,
) -> Result<(HeaderMap, String), ApiError> {
    let format = query
        .format
        .as_deref()
        .and_then(ExportFormat::parse)
        .unwrap_or(ExportFormat::Json);
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let content = PersonExportService::new(pool)
        .export(&person_id, format.clone())
        .await
        .map_err(ApiError::from)?;
    let mut headers_map = HeaderMap::new();
    headers_map.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(format.content_type())
            .unwrap_or(HeaderValue::from_static("application/json")),
    );
    headers_map.insert(
        HeaderName::from_static("content-disposition"),
        HeaderValue::from_str(&format!(
            "attachment; filename=person_{}.{}",
            person_id,
            format.extension()
        ))
        .unwrap(),
    );
    Ok((headers_map, content))
}

// ── Person Snapshots & History Diff ─────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonSnapshotsResponse {
    items: Vec<crate::domains::persons::memory::PersonSnapshot>,
}

pub(crate) async fn get_person_snapshots(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<PersonSnapshotsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::domains::persons::memory::PersonSnapshotStore::new(pool)
        .list(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonSnapshotsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct HistoryDiffQuery {
    from: String,
    to: String,
}

pub(crate) async fn get_person_history_diff(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Query(query): Query<HistoryDiffQuery>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let from_date = DateTime::parse_from_rfc3339(&query.from)
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid from date"))?
        .with_timezone(&Utc);
    let to_date = DateTime::parse_from_rfc3339(&query.to)
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid to date"))?
        .with_timezone(&Utc);
    let diff = crate::domains::persons::memory::PersonSnapshotStore::new(pool)
        .history_diff(&person_id, from_date, to_date)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&diff).unwrap_or_default()))
}

pub(crate) async fn get_identity_candidates(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<PersonIdentityCandidateListResponse>, ApiError> {
    let query = parse_person_identity_candidates_query(raw_query.as_deref())?;
    let items = person_identity_store(&state)?
        .list_candidates(query.limit)
        .await?;

    Ok(Json(PersonIdentityCandidateListResponse { items }))
}

pub(crate) async fn put_identity_candidate_review(
    State(state): State<AppState>,
    Path(identity_candidate_id): Path<String>,
    Json(request): Json<PersonIdentityReviewApiRequest>,
) -> Result<Json<PersonIdentityReviewApiResponse>, ApiError> {
    let actor_id = "hermes-frontend".to_string();
    let command = request.into_command(identity_candidate_id, actor_id)?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::person_identity_review_set(
            &command.actor_id,
            &command.identity_candidate_id,
        ))
        .await?;

    let result = person_identity_store(&state)?
        .set_review_state(&command)
        .await?;

    Ok(Json(result.into()))
}

pub(crate) async fn get_person_identity(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<PersonIdentityDetail>, ApiError> {
    let _ = validate_non_empty_person_identity_field("person_id", &person_id)?;

    let detail = person_identity_store(&state)?
        .person_identity(&person_id)
        .await?;
    Ok(Json(detail))
}
