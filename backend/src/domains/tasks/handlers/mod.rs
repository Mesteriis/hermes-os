// ADR-0073: task handlers are grouped for the first handlers.rs extraction;
// split CRUD, candidates, planning and automation handlers in follow-up work.
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
pub(crate) struct TaskRecordsResponse {
    items: Vec<crate::domains::tasks::api::Task>,
}
#[derive(Deserialize)]
pub(crate) struct TaskListQueryParams {
    status: Option<String>,
    project_id: Option<String>,
    source_type: Option<String>,
    limit: Option<i64>,
}

pub(crate) async fn get_tasks(
    State(state): State<AppState>,
    Query(q): Query<TaskListQueryParams>,
) -> Result<Json<TaskRecordsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = TaskStore::new(pool)
        .list(&TaskListQuery {
            status: q.status,
            project_id: q.project_id,
            source_type: q.source_type,
            limit: q.limit,
        })
        .await?;
    Ok(Json(TaskRecordsResponse { items }))
}

pub(crate) async fn post_task(
    State(state): State<AppState>,
    Json(req): Json<NewTask>,
) -> Result<Json<crate::domains::tasks::api::Task>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let task = TaskStore::new(pool).create(&req).await?;
    Ok(Json(task))
}

pub(crate) async fn get_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<crate::domains::tasks::api::Task>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    TaskStore::new(pool)
        .get(&task_id)
        .await?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

pub(crate) async fn put_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(update): Json<TaskUpdate>,
) -> Result<Json<crate::domains::tasks::api::Task>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let task = TaskStore::new(pool).update(&task_id, &update).await?;
    Ok(Json(task))
}

#[derive(Deserialize)]
pub(crate) struct TaskStatusRequest {
    status: String,
}
pub(crate) async fn post_task_status(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(req): Json<TaskStatusRequest>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    TaskStore::new(pool)
        .set_status(&task_id, &req.status)
        .await?;
    Ok(Json(json!({"status": req.status})))
}

pub(crate) async fn post_task_archive(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    TaskStore::new(pool).archive(&task_id).await?;
    Ok(Json(json!({"archived": true})))
}

// ── Task Context Pack ─────────────────────────────────────────────────────

pub(crate) async fn get_task_context_pack(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let pack = TaskContextPackStore::new(pool)
        .get(&task_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&pack).unwrap_or_default()))
}

#[derive(Deserialize)]
pub(crate) struct UpsertContextPackRequest {
    summary: Option<String>,
    open_questions: Option<Value>,
    blockers: Option<Value>,
    risks: Option<Value>,
    suggested_next_action: Option<String>,
}
pub(crate) async fn post_task_context_pack(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(req): Json<UpsertContextPackRequest>,
) -> Result<Json<crate::domains::tasks::core::TaskContextPack>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let pack = TaskContextPackStore::new(pool)
        .upsert(
            &task_id,
            req.summary.as_deref(),
            req.open_questions.unwrap_or(json!([])),
            req.blockers.unwrap_or(json!([])),
            req.risks.unwrap_or(json!([])),
            req.suggested_next_action.as_deref(),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(pack))
}

// ── Task Evidence ─────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct TaskEvidenceResponse {
    items: Vec<crate::domains::tasks::core::TaskEvidence>,
}
pub(crate) async fn get_task_evidence(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskEvidenceResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = TaskEvidenceStore::new(pool)
        .list(&task_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(TaskEvidenceResponse { items }))
}
#[derive(Deserialize)]
pub(crate) struct NewEvidenceRequest {
    source_type: String,
    source_id: String,
    quote: Option<String>,
    confidence: Option<f64>,
}
pub(crate) async fn post_task_evidence(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(req): Json<NewEvidenceRequest>,
) -> Result<Json<crate::domains::tasks::core::TaskEvidence>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let ev = TaskEvidenceStore::new(pool)
        .add(
            &task_id,
            &req.source_type,
            &req.source_id,
            req.quote.as_deref(),
            req.confidence,
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(ev))
}

// ── Task Relations ────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct TaskRelationsResponse {
    items: Vec<crate::domains::tasks::core::TaskRelation>,
}
pub(crate) async fn get_task_relations(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskRelationsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = TaskRelationStore::new(pool)
        .list(&task_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(TaskRelationsResponse { items }))
}
#[derive(Deserialize)]
pub(crate) struct NewRelationReq {
    entity_type: String,
    entity_id: String,
    relation_type: String,
}
pub(crate) async fn post_task_relation(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(req): Json<NewRelationReq>,
) -> Result<Json<crate::domains::tasks::core::TaskRelation>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let rel = TaskRelationStore::new(pool)
        .link(
            &task_id,
            &req.entity_type,
            &req.entity_id,
            &req.relation_type,
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(rel))
}

// ── Task Checklist ────────────────────────────────────────────────────────

pub(crate) async fn get_task_checklist(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let cl = TaskChecklistStore::new(pool)
        .get(&task_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&cl).unwrap_or_default()))
}
#[derive(Deserialize)]
pub(crate) struct SetChecklistReq {
    items: Value,
    source: Option<String>,
}
pub(crate) async fn post_task_checklist(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(req): Json<SetChecklistReq>,
) -> Result<Json<crate::domains::tasks::core::TaskChecklist>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let cl = TaskChecklistStore::new(pool)
        .set(
            &task_id,
            req.items,
            req.source.as_deref().unwrap_or("manual"),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(cl))
}

// ── Task Subtasks ─────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct TaskSubtasksResponse {
    items: Vec<crate::domains::tasks::core::TaskSubtask>,
}
pub(crate) async fn get_task_subtasks(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskSubtasksResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = TaskSubtaskStore::new(pool)
        .list(&task_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(TaskSubtasksResponse { items }))
}
#[derive(Deserialize)]
pub(crate) struct NewSubtaskReq {
    child_task_id: String,
    sort_order: Option<i32>,
}
pub(crate) async fn post_task_subtask(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(req): Json<NewSubtaskReq>,
) -> Result<Json<crate::domains::tasks::core::TaskSubtask>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let sub = TaskSubtaskStore::new(pool)
        .add(&task_id, &req.child_task_id, req.sort_order.unwrap_or(0))
        .await
        .map_err(ApiError::from)?;
    Ok(Json(sub))
}

// ── Task Intelligence ─────────────────────────────────────────────────────

pub(crate) async fn post_task_analyze(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let task = TaskStore::new(pool.clone())
        .get(&task_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let has_ctx = TaskContextPackStore::new(pool.clone())
        .get(&task_id)
        .await
        .map(|c| c.is_some())
        .unwrap_or(false);
    let _has_relations = TaskRelationStore::new(pool.clone())
        .list(&task_id)
        .await
        .map(|r| !r.is_empty())
        .unwrap_or(false);
    let is_legal = task.area.as_deref() == Some("legal") || task.area.as_deref() == Some("tax");
    let is_tax = task.area.as_deref() == Some("tax");
    let has_contact = task.linked_person_id.is_some();
    let has_org = task.linked_organization_id.is_some();
    let priority = TaskIntelligenceService::calculate_priority(
        task.due_at,
        has_contact,
        has_org,
        task.project_id.is_some(),
        is_legal,
        is_tax,
        false,
    );
    let risk = TaskIntelligenceService::calculate_risk(
        task.due_at
            .map(|d| (d - Utc::now()).num_hours() < 24)
            .unwrap_or(false),
        false,
        false,
        false,
        is_legal,
        &task.title,
    );
    let readiness = TaskIntelligenceService::calculate_readiness(
        task.description.is_some(),
        has_ctx,
        false,
        task.due_at.is_some(),
        true,
        has_contact,
    );
    let missing = TaskIntelligenceService::detect_missing_context(
        task.description.is_some(),
        has_ctx,
        task.due_at.is_some(),
        has_contact,
        task.project_id.is_some(),
    );
    let next_action = TaskIntelligenceService::suggest_next_action(
        &task.hermes_status,
        false,
        false,
        task.waiting_reason.as_deref(),
    );
    let update = TaskUpdate {
        priority_score: Some(priority),
        risk_score: Some(risk),
        readiness_score: Some(readiness),
        ..Default::default()
    };
    TaskStore::new(pool).update(&task_id, &update).await?;
    Ok(Json(
        json!({"priority": priority, "risk": risk, "readiness": readiness, "missing_context": missing, "next_action": next_action}),
    ))
}

// ── Task Export ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct TaskExportQuery {
    format: Option<String>,
}
pub(crate) async fn get_task_export(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Query(q): Query<TaskExportQuery>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let task = TaskStore::new(pool)
        .get(&task_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    match q.format.as_deref().unwrap_or("json") {
        "md" => Ok(Json(
            json!({"format":"markdown","content": export_task_md(&task.title, task.description.as_deref(), &task.hermes_status, task.why.as_deref(), task.outcome.as_deref())}),
        )),
        _ => Ok(Json(export_task_json(
            &task.title,
            task.description.as_deref(),
            &task.hermes_status,
            task.priority_score,
            task.due_at.map(|d| d.to_rfc3339()).as_deref(),
        ))),
    }
}

// ── Task External ─────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct ExtIdentitiesResponse {
    items: Vec<crate::domains::tasks::core::ExternalTaskIdentity>,
}
pub(crate) async fn get_task_external(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<ExtIdentitiesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = ExternalTaskIdentityStore::new(pool)
        .list(&task_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(ExtIdentitiesResponse { items }))
}

// ── Task Providers ────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct TaskProvidersResponse {
    items: Vec<crate::domains::tasks::core::TaskProviderAccount>,
}
pub(crate) async fn get_task_providers(
    State(state): State<AppState>,
) -> Result<Json<TaskProvidersResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = TaskProviderStore::new(pool)
        .list()
        .await
        .map_err(ApiError::from)?;
    Ok(Json(TaskProvidersResponse { items }))
}
#[derive(Deserialize)]
pub(crate) struct NewTaskProviderReq {
    provider: String,
    account_name: String,
}
pub(crate) async fn post_task_provider(
    State(state): State<AppState>,
    Json(req): Json<NewTaskProviderReq>,
) -> Result<Json<crate::domains::tasks::core::TaskProviderAccount>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let prov = TaskProviderStore::new(pool)
        .create(&req.provider, &req.account_name)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(prov))
}

// ── Task Brain ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct TaskBrainQueryParams {
    q: String,
}
pub(crate) async fn post_task_brain(
    State(state): State<AppState>,
    Json(req): Json<TaskBrainQueryParams>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let answer = TaskBrainService::explain_task(&pool, &req.q)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(answer))
}

#[derive(Deserialize)]
pub(crate) struct TaskSearchQueryParams {
    q: String,
}
pub(crate) async fn get_task_search(
    State(state): State<AppState>,
    Query(q): Query<TaskSearchQueryParams>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let results = TaskBrainService::search_tasks(&pool, &q.q).await?;
    Ok(Json(results))
}

pub(crate) async fn get_task_daily_brief(
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let brief = TaskBrainService::daily_brief(&pool).await?;
    Ok(Json(brief))
}

// ── Task Rules ────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct TaskRulesResponse {
    items: Vec<crate::domains::tasks::rules::TaskRule>,
}
pub(crate) async fn get_task_rules(
    State(state): State<AppState>,
) -> Result<Json<TaskRulesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = TaskRuleStore::new(pool)
        .list()
        .await
        .map_err(ApiError::from)?;
    Ok(Json(TaskRulesResponse { items }))
}
#[derive(Deserialize)]
pub(crate) struct NewTaskRuleReq {
    name: String,
    description: Option<String>,
    dsl: Value,
    approval_mode: Option<String>,
}
pub(crate) async fn post_task_rule(
    State(state): State<AppState>,
    Json(req): Json<NewTaskRuleReq>,
) -> Result<Json<crate::domains::tasks::rules::TaskRule>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let rule = TaskRuleStore::new(pool)
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
pub(crate) async fn delete_task_rule(
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    TaskRuleStore::new(pool)
        .delete(&rule_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(json!({"deleted": true})))
}

// ── Task Templates ────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct TaskTemplatesResponse {
    items: Vec<crate::domains::tasks::rules::TaskTemplate>,
}
pub(crate) async fn get_task_templates(
    State(state): State<AppState>,
) -> Result<Json<TaskTemplatesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = TaskTemplateStore::new(pool)
        .list()
        .await
        .map_err(ApiError::from)?;
    Ok(Json(TaskTemplatesResponse { items }))
}

// ── Task Watchtower ───────────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct WatchtowerQuery {
    days: Option<i64>,
}
pub(crate) async fn get_task_watchtower(
    State(state): State<AppState>,
    Query(q): Query<WatchtowerQuery>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let days = q.days.unwrap_or(14);
    let overdue = TaskWatchtowerService::overdue(&pool)
        .await
        .map_err(ApiError::from)?;
    let stale = TaskWatchtowerService::stale_tasks(&pool, days)
        .await
        .map_err(ApiError::from)?;
    let no_ctx = TaskWatchtowerService::without_context(&pool)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(
        json!({"overdue": overdue, "stale": stale, "without_context": no_ctx}),
    ))
}

pub(crate) async fn get_task_health(
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let wl = TaskWatchtowerService::workload(&pool)
        .await
        .map_err(ApiError::from)?;
    let ct = TaskWatchtowerService::cycle_time(&pool)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(json!({"workload": wl, "cycle_time": ct})))
}

pub(crate) async fn get_task_analytics(
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let _pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        json!({"analytics": "available via /tasks/health and /tasks/watchtower"}),
    ))
}

pub(crate) async fn get_task_candidates(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<TaskCandidateListResponse>, ApiError> {
    let query = parse_task_candidates_query(raw_query.as_deref())?;
    let items = task_candidate_store(&state)?
        .list_candidates(query.limit)
        .await?;

    Ok(Json(TaskCandidateListResponse { items }))
}

pub(crate) async fn put_task_candidate_review(
    State(state): State<AppState>,
    Path(task_candidate_id): Path<String>,
    Json(request): Json<TaskCandidateReviewApiRequest>,
) -> Result<Json<TaskCandidateReviewApiResponse>, ApiError> {
    let actor_id = "hermes-frontend".to_string();
    let command = request.into_command(task_candidate_id, actor_id)?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::task_candidate_review_set(
            &command.actor_id,
            &command.task_candidate_id,
        ))
        .await?;

    let result = task_candidate_store(&state)?
        .set_review_state(&command)
        .await?;

    Ok(Json(result.into()))
}
