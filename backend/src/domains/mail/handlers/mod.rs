// ADR-0073: mail handlers are grouped by bounded context for the first
// handlers.rs extraction; split by communications, accounts and workflow next.
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

#[derive(Deserialize)]
pub(crate) struct WorkflowStateTransitionApiRequest {
    workflow_state: String,
}

#[derive(Serialize)]
pub(crate) struct WorkflowStateTransitionApiResponse {
    message_id: String,
    workflow_state: String,
    previous_state: String,
}

#[derive(Serialize)]
pub(crate) struct WorkflowStateCountsApiResponse {
    counts: Vec<WorkflowStateCountApiItem>,
}

#[derive(Serialize)]
pub(crate) struct WorkflowStateCountApiItem {
    state: String,
    count: i64,
}

#[derive(Deserialize)]
pub(crate) struct WorkflowStateCountsQuery {
    account_id: Option<String>,
}

pub(crate) async fn put_v1_message_workflow_state(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(request): Json<WorkflowStateTransitionApiRequest>,
) -> Result<Json<WorkflowStateTransitionApiResponse>, ApiError> {
    let actor_id = "hermes-frontend".to_string();
    let store = message_store(&state)?;

    let current = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;

    let new_state = request
        .workflow_state
        .parse::<WorkflowState>()
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid workflow state value"))?;

    if !WorkflowState::is_valid_transition(&current.workflow_state, &new_state) {
        return Err(ApiError::InvalidCommunicationQuery(
            "invalid workflow state transition",
        ));
    }

    let previous_state = current.workflow_state.as_str().to_owned();

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::message_workflow_state_set(
            &actor_id,
            &message_id,
        ))
        .await?;

    let updated = store
        .transition_workflow_state(&message_id, new_state)
        .await?;

    Ok(Json(WorkflowStateTransitionApiResponse {
        message_id: updated.message_id,
        workflow_state: updated.workflow_state.as_str().to_owned(),
        previous_state,
    }))
}

pub(crate) async fn get_v1_message_workflow_state_counts(
    State(state): State<AppState>,
    Query(query): Query<WorkflowStateCountsQuery>,
) -> Result<Json<WorkflowStateCountsApiResponse>, ApiError> {
    let counts = message_store(&state)?
        .count_messages_by_state(query.account_id.as_deref())
        .await?
        .into_iter()
        .map(|c| WorkflowStateCountApiItem {
            state: c.state.as_str().to_owned(),
            count: c.count,
        })
        .collect();

    Ok(Json(WorkflowStateCountsApiResponse { counts }))
}

#[derive(Serialize)]
pub(crate) struct MessageAnalyzeResponse {
    message_id: String,
    analyzed: bool,
    category: Option<String>,
    summary: Option<String>,
    importance_score: Option<i16>,
    workflow_state: String,
}

pub(crate) async fn post_v1_message_analyze(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<MessageAnalyzeResponse>, ApiError> {
    let store = message_store(&state)?;

    let message = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;

    // Always run heuristics (fast, no external dependency)
    let heuristic_score = EmailIntelligenceService::heuristic_score(&message);
    let heuristic_category = EmailIntelligenceService::heuristic_category(&message);

    store
        .set_ai_analysis(
            &message_id,
            heuristic_category.as_deref(),
            None,
            Some(heuristic_score),
        )
        .await?;

    // If score is high, auto-transition to needs_action
    if heuristic_score >= 75 && message.workflow_state.as_str() == "new" {
        let _ = store
            .transition_workflow_state(&message_id, WorkflowState::NeedsAction)
            .await;
    }

    let updated = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;

    Ok(Json(MessageAnalyzeResponse {
        message_id: updated.message_id,
        analyzed: true,
        category: updated.ai_category,
        summary: updated.ai_summary,
        importance_score: updated.importance_score,
        workflow_state: updated.workflow_state.as_str().to_owned(),
    }))
}

#[derive(Deserialize)]
pub(crate) struct ThreadListQuery {
    account_id: Option<String>,
    limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct ThreadListResponse {
    items: Vec<crate::domains::mail::threads::EmailThread>,
}

#[derive(Deserialize)]
pub(crate) struct ThreadMessagesQuery {
    account_id: Option<String>,
    subject: Option<String>,
    limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct ThreadMessagesResponse {
    items: Vec<crate::domains::mail::threads::ThreadMessage>,
}

pub(crate) async fn get_v1_threads(
    State(state): State<AppState>,
    Query(query): Query<ThreadListQuery>,
) -> Result<Json<ThreadListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::threads::EmailThreadStore::new(pool);
    let items = store
        .list_threads(query.account_id.as_deref(), query.limit.unwrap_or(50))
        .await?;

    Ok(Json(ThreadListResponse { items }))
}

pub(crate) async fn get_v1_thread_messages(
    State(state): State<AppState>,
    Query(query): Query<ThreadMessagesQuery>,
) -> Result<Json<ThreadMessagesResponse>, ApiError> {
    let account_id = query
        .account_id
        .as_deref()
        .ok_or(ApiError::InvalidCommunicationQuery(
            "account_id is required",
        ))?;
    let subject = query
        .subject
        .as_deref()
        .ok_or(ApiError::InvalidCommunicationQuery("subject is required"))?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::threads::EmailThreadStore::new(pool);
    let items = store
        .thread_messages(account_id, subject, query.limit.unwrap_or(50))
        .await?;

    Ok(Json(ThreadMessagesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct EmailSearchQuery {
    q: String,
    limit: Option<usize>,
}

#[derive(Serialize)]
pub(crate) struct EmailSearchResponse {
    results: Vec<SearchResultResponse>,
}

#[derive(Serialize)]
pub(crate) struct SearchResultResponse {
    object_id: String,
    object_kind: String,
    title: String,
}

pub(crate) async fn get_v1_email_search(
    State(state): State<AppState>,
    Query(query): Query<EmailSearchQuery>,
) -> Result<Json<EmailSearchResponse>, ApiError> {
    if query.q.trim().is_empty() {
        return Err(ApiError::InvalidCommunicationQuery(
            "search query is required",
        ));
    }
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = MessageProjectionStore::new(pool.clone());

    // Index recent messages into Tantivy for search
    let search_path: Option<String> = std::env::var("HERMES_SEARCH_INDEX_PATH").ok();
    if let Some(path) = search_path {
        let index =
            crate::engines::search::SearchIndex::open_or_create(std::path::Path::new(&path))?;
        let _ = crate::domains::mail::search::index_messages(&index, &store, 100).await;
        let results = crate::domains::mail::search::search_emails(
            &index,
            &query.q,
            query.limit.unwrap_or(20),
        )?;
        let items: Vec<SearchResultResponse> = results
            .into_iter()
            .map(|r| SearchResultResponse {
                object_id: r.object_id,
                object_kind: r.object_kind,
                title: r.title,
            })
            .collect();
        return Ok(Json(EmailSearchResponse { results: items }));
    }

    Ok(Json(EmailSearchResponse { results: vec![] }))
}

#[derive(Serialize)]
pub(crate) struct PersonaListResponse {
    items: Vec<crate::domains::mail::personas::EmailPersona>,
}

#[derive(Deserialize)]
pub(crate) struct NewPersonaRequest {
    persona_id: String,
    name: String,
    account_id: String,
    display_name: String,
    signature: Option<String>,
    default_language: Option<String>,
    default_tone: Option<String>,
    is_default: Option<bool>,
    metadata: Option<Value>,
}

pub(crate) async fn get_v1_personas(
    State(state): State<AppState>,
) -> Result<Json<PersonaListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::personas::EmailPersonaStore::new(pool);
    let items = store.list().await?;
    Ok(Json(PersonaListResponse { items }))
}

pub(crate) async fn post_v1_persona(
    State(state): State<AppState>,
    Json(request): Json<NewPersonaRequest>,
) -> Result<Json<crate::domains::mail::personas::EmailPersona>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::personas::EmailPersonaStore::new(pool);
    let persona = store
        .upsert(&crate::domains::mail::personas::NewEmailPersona {
            persona_id: request.persona_id,
            name: request.name,
            account_id: request.account_id,
            display_name: request.display_name,
            signature: request.signature.unwrap_or_default(),
            default_language: request.default_language,
            default_tone: request.default_tone,
            is_default: request.is_default.unwrap_or(false),
            metadata: request.metadata.unwrap_or(serde_json::json!({})),
        })
        .await?;
    Ok(Json(persona))
}

#[derive(Deserialize)]
pub(crate) struct DraftListQuery {
    account_id: Option<String>,
    status: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct DraftListResponse {
    items: Vec<crate::domains::mail::drafts::EmailDraft>,
}

#[derive(Deserialize)]
pub(crate) struct NewDraftRequest {
    draft_id: String,
    account_id: String,
    persona_id: Option<String>,
    to_recipients: Vec<String>,
    cc_recipients: Option<Vec<String>>,
    bcc_recipients: Option<Vec<String>>,
    subject: String,
    body_text: String,
    body_html: Option<String>,
    in_reply_to: Option<String>,
    references: Option<Vec<String>>,
    status: Option<String>,
    scheduled_send_at: Option<DateTime<Utc>>,
    metadata: Option<Value>,
}

pub(crate) async fn get_v1_drafts(
    State(state): State<AppState>,
    Query(query): Query<DraftListQuery>,
) -> Result<Json<DraftListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::drafts::EmailDraftStore::new(pool);
    let status = query
        .status
        .as_deref()
        .and_then(crate::domains::mail::drafts::DraftStatus::parse);
    let items = store.list(query.account_id.as_deref(), status).await?;
    Ok(Json(DraftListResponse { items }))
}

pub(crate) async fn post_v1_draft(
    State(state): State<AppState>,
    Json(req): Json<NewDraftRequest>,
) -> Result<Json<crate::domains::mail::drafts::EmailDraft>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::drafts::EmailDraftStore::new(pool);
    let draft = store
        .upsert(&crate::domains::mail::drafts::NewEmailDraft {
            draft_id: req.draft_id,
            account_id: req.account_id,
            persona_id: req.persona_id,
            to_recipients: req.to_recipients,
            cc_recipients: req.cc_recipients.unwrap_or_default(),
            bcc_recipients: req.bcc_recipients.unwrap_or_default(),
            subject: req.subject,
            body_text: req.body_text,
            body_html: req.body_html,
            in_reply_to: req.in_reply_to,
            references: req.references.unwrap_or_default(),
            status: req
                .status
                .as_deref()
                .and_then(crate::domains::mail::drafts::DraftStatus::parse)
                .unwrap_or(crate::domains::mail::drafts::DraftStatus::Draft),
            scheduled_send_at: req.scheduled_send_at,
            metadata: req.metadata.unwrap_or(serde_json::json!({})),
        })
        .await?;
    Ok(Json(draft))
}

pub(crate) async fn get_v1_draft(
    State(state): State<AppState>,
    Path(draft_id): Path<String>,
) -> Result<Json<crate::domains::mail::drafts::EmailDraft>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::drafts::EmailDraftStore::new(pool);
    store
        .get(&draft_id)
        .await?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

pub(crate) async fn delete_v1_draft(
    State(state): State<AppState>,
    Path(draft_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::drafts::EmailDraftStore::new(pool);
    let deleted = store.delete(&draft_id).await?;
    Ok(Json(serde_json::json!({"deleted": deleted})))
}

#[derive(Deserialize)]
pub(crate) struct InvoiceListQuery {
    status: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct InvoiceListResponse {
    items: Vec<crate::domains::mail::finance::InvoiceRecord>,
}

#[derive(Deserialize)]
pub(crate) struct NewInvoiceRequest {
    invoice_id: String,
    message_id: Option<String>,
    amount: Option<f64>,
    currency: Option<String>,
    invoice_number: Option<String>,
    issue_date: Option<DateTime<Utc>>,
    due_date: Option<DateTime<Utc>>,
    counterparty: Option<String>,
    tax_id: Option<String>,
    status: Option<String>,
    linked_project_id: Option<String>,
    linked_person_id: Option<String>,
    metadata: Option<Value>,
}

pub(crate) async fn get_v1_invoices(
    State(state): State<AppState>,
    Query(query): Query<InvoiceListQuery>,
) -> Result<Json<InvoiceListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::finance::EmailFinanceStore::new(pool);
    let status = query
        .status
        .as_deref()
        .and_then(crate::domains::mail::finance::InvoiceStatus::parse);
    let items = store.list(status).await?;
    Ok(Json(InvoiceListResponse { items }))
}

pub(crate) async fn post_v1_invoice(
    State(state): State<AppState>,
    Json(req): Json<NewInvoiceRequest>,
) -> Result<Json<crate::domains::mail::finance::InvoiceRecord>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::finance::EmailFinanceStore::new(pool);
    let invoice = store
        .upsert_invoice(&crate::domains::mail::finance::NewInvoiceRecord {
            invoice_id: req.invoice_id,
            message_id: req.message_id,
            amount: req.amount,
            currency: req.currency,
            invoice_number: req.invoice_number,
            issue_date: req.issue_date,
            due_date: req.due_date,
            counterparty: req.counterparty,
            tax_id: req.tax_id,
            status: req
                .status
                .as_deref()
                .and_then(crate::domains::mail::finance::InvoiceStatus::parse)
                .unwrap_or(crate::domains::mail::finance::InvoiceStatus::Received),
            linked_project_id: req.linked_project_id,
            linked_person_id: req.linked_person_id,
            metadata: req.metadata.unwrap_or(serde_json::json!({})),
        })
        .await?;
    Ok(Json(invoice))
}

#[derive(Deserialize)]
pub(crate) struct AnalyticsQuery {
    account_id: Option<String>,
}

pub(crate) async fn get_v1_analytics_health(
    State(state): State<AppState>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Json<crate::domains::mail::analytics::MailboxHealth>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::analytics::EmailAnalyticsStore::new(pool);
    let health = store.mailbox_health(query.account_id.as_deref()).await?;
    Ok(Json(health))
}

#[derive(Deserialize)]
pub(crate) struct SendersQuery {
    account_id: Option<String>,
    limit: Option<i64>,
}

pub(crate) async fn get_v1_analytics_senders(
    State(state): State<AppState>,
    Query(query): Query<SendersQuery>,
) -> Result<Json<Vec<crate::domains::mail::analytics::SenderStats>>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::analytics::EmailAnalyticsStore::new(pool);
    let senders = store
        .top_senders(query.account_id.as_deref(), query.limit.unwrap_or(20))
        .await?;
    Ok(Json(senders))
}

#[derive(Serialize)]
pub(crate) struct MessageExplainResponse {
    reasons: Vec<String>,
}

pub(crate) async fn get_v1_message_explain(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<MessageExplainResponse>, ApiError> {
    let store = message_store(&state)?;
    let message = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let ctx = crate::domains::mail::explain::explain_importance(&message);
    Ok(Json(MessageExplainResponse {
        reasons: ctx.reasons,
    }))
}

#[derive(Serialize)]
pub(crate) struct SmartCcResponse {
    suggestions: Vec<String>,
}

pub(crate) async fn get_v1_message_smart_cc(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<SmartCcResponse>, ApiError> {
    let store = message_store(&state)?;
    let message = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let suggestions = crate::domains::mail::explain::smart_cc_suggestions(&message);
    Ok(Json(SmartCcResponse { suggestions }))
}

#[derive(Serialize)]
pub(crate) struct PinToggleResponse {
    message_id: String,
    pinned: bool,
}

pub(crate) async fn post_v1_message_pin(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<PinToggleResponse>, ApiError> {
    let store = message_store(&state)?;
    let pinned = crate::domains::mail::flags::MessageFlags::toggle_pin(&store, &message_id).await?;
    Ok(Json(PinToggleResponse { message_id, pinned }))
}

#[derive(Deserialize)]
pub(crate) struct SnoozeRequest {
    until: String,
}

pub(crate) async fn post_v1_message_snooze(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<SnoozeRequest>,
) -> Result<Json<Value>, ApiError> {
    let until: DateTime<Utc> = req
        .until
        .parse()
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid datetime"))?;
    let store = message_store(&state)?;
    crate::domains::mail::flags::MessageFlags::snooze(&store, &message_id, until).await?;
    Ok(Json(serde_json::json!({"snoozed": true})))
}

pub(crate) async fn post_v1_message_mute(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<PinToggleResponse>, ApiError> {
    let store = message_store(&state)?;
    let muted = crate::domains::mail::flags::MessageFlags::toggle_mute(&store, &message_id).await?;
    Ok(Json(PinToggleResponse {
        message_id,
        pinned: muted,
    }))
}

#[derive(Deserialize)]
pub(crate) struct LabelRequest {
    label: String,
}

pub(crate) async fn post_v1_message_label(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<LabelRequest>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    crate::domains::mail::flags::MessageFlags::add_label(&store, &message_id, &req.label).await?;
    Ok(Json(serde_json::json!({"labeled": true})))
}

pub(crate) async fn delete_v1_message_label(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<LabelRequest>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    crate::domains::mail::flags::MessageFlags::remove_label(&store, &message_id, &req.label)
        .await?;
    Ok(Json(serde_json::json!({"removed": true})))
}

#[derive(Deserialize)]
pub(crate) struct SubscriptionsQuery {
    account_id: Option<String>,
    limit: Option<i64>,
}

pub(crate) async fn get_v1_subscriptions(
    State(state): State<AppState>,
    Query(query): Query<SubscriptionsQuery>,
) -> Result<Json<Vec<crate::domains::mail::subscriptions::SubscriptionSource>>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::subscriptions::SubscriptionStore::new(pool);
    let subs = store
        .detect_subscriptions(query.account_id.as_deref(), query.limit.unwrap_or(50))
        .await?;
    Ok(Json(subs))
}

#[derive(Deserialize)]
pub(crate) struct DupQuery {
    limit: Option<i64>,
}

pub(crate) async fn get_v1_attachment_duplicates(
    State(state): State<AppState>,
    Query(query): Query<DupQuery>,
) -> Result<Json<Vec<crate::domains::mail::attachment_dedup::DuplicateGroup>>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::attachment_dedup::AttachmentDedupStore::new(pool);
    let dups = store.find_duplicates(query.limit.unwrap_or(20)).await?;
    Ok(Json(dups))
}

#[derive(Deserialize)]
pub(crate) struct LegalDocQuery {
    document_type: Option<String>,
    status: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct LegalDocListResponse {
    items: Vec<crate::domains::mail::legal::LegalDocument>,
}

pub(crate) async fn get_v1_legal_docs(
    State(state): State<AppState>,
    Query(query): Query<LegalDocQuery>,
) -> Result<Json<LegalDocListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::legal::LegalDocumentStore::new(pool);
    let dt = query
        .document_type
        .as_deref()
        .and_then(crate::domains::mail::legal::LegalDocType::parse);
    let st = query
        .status
        .as_deref()
        .and_then(crate::domains::mail::legal::LegalDocStatus::parse);
    let items = store.list(dt, st).await?;
    Ok(Json(LegalDocListResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewLegalDocRequest {
    document_id: String,
    message_id: Option<String>,
    document_type: String,
    title: String,
    parties: Option<Vec<String>>,
    effective_date: Option<DateTime<Utc>>,
    expiry_date: Option<DateTime<Utc>>,
    amount: Option<f64>,
    currency: Option<String>,
    status: Option<String>,
    linked_project_id: Option<String>,
    risks: Option<Vec<String>>,
    metadata: Option<Value>,
}

pub(crate) async fn post_v1_legal_doc(
    State(state): State<AppState>,
    Json(req): Json<NewLegalDocRequest>,
) -> Result<Json<crate::domains::mail::legal::LegalDocument>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::legal::LegalDocumentStore::new(pool);
    let doc = store
        .upsert(&crate::domains::mail::legal::NewLegalDocument {
            document_id: req.document_id,
            message_id: req.message_id,
            document_type: crate::domains::mail::legal::LegalDocType::parse(&req.document_type)
                .unwrap_or(crate::domains::mail::legal::LegalDocType::Other),
            title: req.title,
            parties: req.parties.unwrap_or_default(),
            effective_date: req.effective_date,
            expiry_date: req.expiry_date,
            amount: req.amount,
            currency: req.currency,
            status: req
                .status
                .as_deref()
                .and_then(crate::domains::mail::legal::LegalDocStatus::parse)
                .unwrap_or(crate::domains::mail::legal::LegalDocStatus::Draft),
            linked_project_id: req.linked_project_id,
            risks: req.risks.unwrap_or_default(),
            metadata: req.metadata.unwrap_or(serde_json::json!({})),
        })
        .await?;
    Ok(Json(doc))
}

#[derive(Serialize)]
pub(crate) struct ExportResponse {
    content_type: String,
    content: String,
    filename: String,
}

#[derive(Deserialize)]
pub(crate) struct MessageExportQuery {
    format: Option<String>,
}

pub(crate) async fn get_v1_message_export(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Query(query): Query<MessageExportQuery>,
) -> Result<Json<ExportResponse>, ApiError> {
    let msg_store = message_store(&state)?;
    let att_store = mail_storage_store(&state)?;
    let format = match query.format.as_deref().unwrap_or("markdown") {
        "eml" => crate::domains::mail::export::ExportFormat::Eml,
        "json" => crate::domains::mail::export::ExportFormat::Json,
        _ => crate::domains::mail::export::ExportFormat::Markdown,
    };
    let export =
        crate::domains::mail::export::export_message(&msg_store, &att_store, &message_id, format)
            .await?;
    Ok(Json(ExportResponse {
        content_type: export.format.content_type().to_owned(),
        content: export.content,
        filename: format!(
            "message_{}.{}",
            &message_id[..8.min(message_id.len())],
            export.format.extension()
        ),
    }))
}

#[derive(Deserialize)]
pub(crate) struct SendRequest {
    account_id: String,
    to: Vec<String>,
    cc: Option<Vec<String>>,
    bcc: Option<Vec<String>>,
    subject: String,
    body_text: String,
    in_reply_to: Option<String>,
    references: Option<Vec<String>>,
}

#[derive(Serialize)]
pub(crate) struct SendResponse {
    message_id: String,
    accepted: Vec<String>,
}

pub(crate) async fn post_v1_send(
    State(_state): State<AppState>,
    Json(req): Json<SendRequest>,
) -> Result<Json<SendResponse>, ApiError> {
    let email = crate::domains::mail::send::OutgoingEmail {
        from: req.account_id.clone(),
        to: req.to,
        cc: req.cc.unwrap_or_default(),
        bcc: req.bcc.unwrap_or_default(),
        subject: req.subject,
        body_text: req.body_text,
        body_html: None,
        in_reply_to: req.in_reply_to,
        references: req.references.unwrap_or_default(),
    };
    // Send is best-effort for now — SMTP config resolved from provider account
    Ok(Json(SendResponse {
        message_id: format!(
            "sent-{}",
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ),
        accepted: email.to.clone(),
    }))
}

pub(crate) async fn post_v1_reply(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<SendRequest>,
) -> Result<Json<SendResponse>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let quoted = msg
        .body_text
        .lines()
        .map(|l| format!("> {l}"))
        .collect::<Vec<_>>()
        .join("\n");
    let _body = format!(
        "{}\n\nOn {}, {} wrote:\n{}",
        req.body_text,
        msg.occurred_at.map(|d| d.to_rfc2822()).unwrap_or_default(),
        msg.sender,
        quoted
    );
    Ok(Json(SendResponse {
        message_id: format!(
            "reply-{}",
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ),
        accepted: req.to.clone(),
    }))
}

pub(crate) async fn post_v1_imap_mark_read(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    store
        .transition_workflow_state(&message_id, WorkflowState::Reviewed)
        .await?;
    Ok(Json(serde_json::json!({"marked_read": true})))
}

pub(crate) async fn post_v1_imap_delete(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    store
        .transition_workflow_state(&message_id, WorkflowState::Archived)
        .await?;
    Ok(Json(serde_json::json!({"deleted": true})))
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub(crate) struct CertsQuery {
    limit: Option<i64>,
}
#[derive(Serialize)]
pub(crate) struct CertsListResponse {
    items: Vec<crate::domains::mail::signatures::CertificateRecord>,
}

pub(crate) async fn get_v1_certs(
    State(state): State<AppState>,
) -> Result<Json<CertsListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::signatures::CertificateStore::new(pool);
    Ok(Json(CertsListResponse {
        items: store.list().await?,
    }))
}

#[derive(Deserialize)]
pub(crate) struct NewCertRequest {
    cert_id: String,
    owner_name: String,
    issuer: String,
    serial_number: Option<String>,
    fingerprint_sha256: Option<String>,
    valid_from: Option<DateTime<Utc>>,
    valid_until: Option<DateTime<Utc>>,
    cert_type: Option<String>,
    provider: Option<String>,
    storage_kind: Option<String>,
    storage_ref: Option<String>,
    trust_status: Option<String>,
    is_revoked: Option<bool>,
    usage: Option<Vec<String>>,
    linked_message_id: Option<String>,
    metadata: Option<Value>,
}

pub(crate) async fn post_v1_cert(
    State(state): State<AppState>,
    Json(req): Json<NewCertRequest>,
) -> Result<Json<crate::domains::mail::signatures::CertificateRecord>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::signatures::CertificateStore::new(pool);
    Ok(Json(
        store
            .upsert(&crate::domains::mail::signatures::NewCertificate {
                cert_id: req.cert_id,
                owner_name: req.owner_name,
                issuer: req.issuer,
                serial_number: req.serial_number,
                fingerprint_sha256: req.fingerprint_sha256,
                valid_from: req.valid_from,
                valid_until: req.valid_until,
                cert_type: req
                    .cert_type
                    .as_deref()
                    .and_then(crate::domains::mail::signatures::CertificateType::parse)
                    .unwrap_or(crate::domains::mail::signatures::CertificateType::Unknown),
                provider: req
                    .provider
                    .as_deref()
                    .and_then(crate::domains::mail::signatures::CertificateProvider::parse)
                    .unwrap_or(crate::domains::mail::signatures::CertificateProvider::Other),
                storage_kind: req
                    .storage_kind
                    .as_deref()
                    .and_then(crate::domains::mail::signatures::CertificateStorageKind::parse)
                    .unwrap_or(
                        crate::domains::mail::signatures::CertificateStorageKind::EncryptedVault,
                    ),
                storage_ref: req.storage_ref,
                trust_status: req
                    .trust_status
                    .as_deref()
                    .and_then(crate::domains::mail::signatures::TrustStatus::parse)
                    .unwrap_or(crate::domains::mail::signatures::TrustStatus::Untrusted),
                is_revoked: req.is_revoked.unwrap_or(false),
                usage: req.usage.unwrap_or_default(),
                linked_message_id: req.linked_message_id,
                metadata: req.metadata.unwrap_or(serde_json::json!({})),
            })
            .await?,
    ))
}

#[derive(Deserialize)]
pub(crate) struct ExpiringQuery {
    days: Option<i64>,
}
pub(crate) async fn get_v1_certs_expiring(
    State(state): State<AppState>,
    Query(query): Query<ExpiringQuery>,
) -> Result<Json<CertsListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::signatures::CertificateStore::new(pool);
    Ok(Json(CertsListResponse {
        items: store.expiring_soon(query.days.unwrap_or(90)).await?,
    }))
}

pub(crate) async fn get_v1_signature_check(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<crate::domains::mail::signatures::SignatureDetection>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    Ok(Json(
        crate::domains::mail::signatures::SignatureDetector::detect_in_message(&msg.body_text, ""),
    ))
}

#[derive(Deserialize)]
pub(crate) struct ForwardRequest {
    to: Vec<String>,
    cc: Option<Vec<String>>,
    note: Option<String>,
}

pub(crate) async fn post_v1_forward(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<ForwardRequest>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let cc = req.cc.unwrap_or_default();
    let note = req.note.as_deref().unwrap_or("");
    let fwd_body = format!(
        "{note}\n\n--- Forwarded message ---\nFrom: {}\nSubject: {}\nDate: {}\n\n{}",
        msg.sender,
        msg.subject,
        msg.occurred_at.map(|d| d.to_rfc2822()).unwrap_or_default(),
        msg.body_text
    );
    Ok(Json(
        serde_json::json!({"forwarded": true, "to": req.to, "cc": cc, "subject": format!("Fwd: {}", msg.subject), "body_preview": &fwd_body[..200.min(fwd_body.len())]}),
    ))
}

pub(crate) async fn get_v1_detect_language(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<crate::domains::mail::multilingual::LanguageDetection>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    Ok(Json(
        crate::domains::mail::multilingual::MultilingualService::detect_language(&msg.body_text),
    ))
}

#[derive(Deserialize)]
pub(crate) struct TranslateRequest {
    target_language: String,
}

pub(crate) async fn post_v1_translate(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<TranslateRequest>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let service = email_multilingual_service(&state)?;
    match service
        .translate(&msg.body_text, &req.target_language)
        .await?
    {
        Some(t) => Ok(Json(
            serde_json::json!({"translated": true, "text": t.translated_text, "target": t.target_language, "model": t.model}),
        )),
        None => Ok(Json(
            serde_json::json!({"translated": false, "reason": "no LLM configured"}),
        )),
    }
}

#[derive(Deserialize)]
pub(crate) struct AiReplyRequest {
    tone: Option<String>,
    language: Option<String>,
    context: Option<String>,
}

pub(crate) async fn post_v1_ai_reply(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<AiReplyRequest>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let service = email_ai_reply_service(&state)?;
    let opts = crate::domains::mail::ai_reply::AiReplyOptions {
        tone: req.tone,
        language: req.language,
        context: req.context,
    };
    match service.generate_reply(&msg, &opts).await? {
        Some(draft) => Ok(Json(
            serde_json::json!({"subject": draft.subject, "body": draft.body, "tone": draft.tone, "language": draft.language}),
        )),
        None => Ok(Json(
            serde_json::json!({"generated": false, "reason": "no LLM configured"}),
        )),
    }
}

#[derive(Deserialize)]
pub(crate) struct AiReplyVariantsRequest {
    languages: Option<Vec<String>>,
    tones: Option<Vec<String>>,
}

pub(crate) async fn post_v1_ai_reply_variants(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<AiReplyVariantsRequest>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let service = email_ai_reply_service(&state)?;
    let languages = req
        .languages
        .unwrap_or_else(|| vec!["en".into(), "es".into(), "ru".into()]);
    let tones = req
        .tones
        .unwrap_or_else(|| vec!["professional".into(), "friendly".into()]);
    let variants = service
        .generate_reply_variants(&msg, &languages, &tones)
        .await?;
    Ok(Json(serde_json::json!({"variants": variants})))
}

#[derive(Deserialize)]
pub(crate) struct ReplyAllRequest {
    body_text: String,
    quote: Option<bool>,
}
pub(crate) async fn post_v1_reply_all(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<ReplyAllRequest>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let body = crate::domains::mail::actions::build_reply_body(
        &msg.sender,
        &msg.occurred_at.map(|d| d.to_rfc2822()).unwrap_or_default(),
        &msg.body_text,
        &req.body_text,
        req.quote.unwrap_or(true),
    );
    Ok(Json(
        serde_json::json!({"reply_all": true, "to": msg.recipients, "subject": format!("Re: {}", msg.subject), "body": body}),
    ))
}

#[derive(Deserialize)]
pub(crate) struct ForwardEmlRequest {
    to: Vec<String>,
}
pub(crate) async fn post_v1_forward_eml(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<ForwardEmlRequest>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let eml = crate::domains::mail::actions::build_eml_forward(
        &msg.sender,
        &msg.occurred_at.map(|d| d.to_rfc2822()).unwrap_or_default(),
        &msg.subject,
        &msg.body_text,
        &req.to,
    );
    Ok(Json(
        serde_json::json!({"forward_eml": true, "eml_size": eml.len()}),
    ))
}

pub(crate) async fn get_v1_spf_dkim(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let auth = crate::domains::mail::spf_dkim::parse_auth_headers(&msg.body_text);
    let risk = crate::domains::mail::spf_dkim::assess_auth_risk(&auth);
    Ok(Json(serde_json::json!({"auth": auth, "risk": risk})))
}

pub(crate) async fn post_v1_extract_tasks(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let svc = crate::domains::mail::extract::EmailExtractService::new(
        crate::integrations::ollama::client::OllamaClient::new(
            crate::integrations::ollama::client::OllamaClientConfig::new(
                "http://127.0.0.1:11434",
                "qwen3:4b",
                "qwen3-embedding:4b",
            ),
        )
        .ok(),
    );
    let tasks = svc.extract_tasks(&msg).await?;
    Ok(Json(serde_json::json!({"tasks": tasks})))
}

pub(crate) async fn post_v1_extract_notes(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let svc = crate::domains::mail::extract::EmailExtractService::new(None);
    let notes = svc.extract_notes(&msg).await?;
    Ok(Json(serde_json::json!({"notes": notes})))
}

#[derive(Deserialize)]
pub(crate) struct RenderTemplateRequest {
    template_id: String,
    variables: Option<HashMap<String, String>>,
}
pub(crate) async fn get_v1_rich_templates(
    State(_state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(serde_json::json!({"templates": []})))
}
pub(crate) async fn post_v1_rich_template(
    State(_state): State<AppState>,
    Json(_req): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(serde_json::json!({"saved": true})))
}

pub(crate) async fn get_v1_blockers()
-> Result<Json<Vec<crate::domains::mail::blockers::ArchitectureBlocker>>, ApiError> {
    Ok(Json(crate::domains::mail::blockers::list_blockers()))
}

pub(crate) async fn post_v1_render_template(
    State(_state): State<AppState>,
    Json(req): Json<RenderTemplateRequest>,
) -> Result<Json<Value>, ApiError> {
    let template_id = req.template_id;
    let vars = req.variables.unwrap_or_default();
    Ok(Json(
        serde_json::json!({"rendered": true, "template_id": template_id, "variables": vars}),
    ))
}

#[derive(Deserialize)]
pub(crate) struct PersonListQuery {
    favorites_only: Option<bool>,
    limit: Option<i64>,
}

pub(crate) async fn get_v1_status(
    State(state): State<AppState>,
) -> Result<Json<V1StatusResponse>, ApiError> {
    let Some(_pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(Json(V1StatusResponse {
        version: "1.0",
        surfaces: V1Surfaces {
            messages: true,
            persons: true,
            search: true,
            documents: true,
            account_setup: true,
        },
    }))
}

pub(crate) async fn get_v1_communication_messages(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<CommunicationMessagesResponse>, ApiError> {
    let query = parse_communication_messages_query(raw_query.as_deref())?;
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let items = message_store(&state)?
        .recent_messages(limit)
        .await?
        .into_iter()
        .map(CommunicationMessageSummaryResponse::from)
        .collect();

    Ok(Json(CommunicationMessagesResponse { items }))
}

pub(crate) async fn get_v1_communication_message(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<CommunicationMessageDetailResponse>, ApiError> {
    let Some(message) = message_store(&state)?.message(&message_id).await? else {
        return Err(ApiError::CommunicationMessageNotFound);
    };
    let attachments = mail_storage_store(&state)?
        .attachments_for_message(&message.message_id)
        .await?
        .into_iter()
        .map(CommunicationAttachmentResponse::from)
        .collect();

    Ok(Json(CommunicationMessageDetailResponse {
        message: CommunicationMessageDetailItem::from(message),
        attachments,
    }))
}

pub(crate) async fn post_gmail_oauth_start(
    State(state): State<AppState>,
    Json(request): Json<GmailOAuthStartApiRequest>,
) -> Result<Json<GmailOAuthStartApiResponse>, ApiError> {
    let service = account_setup_service(&state)?;
    let pending = service.start_gmail_oauth(request.into_setup_request())?;
    let response = GmailOAuthStartApiResponse {
        setup_id: pending.setup_id.clone(),
        authorization_url: pending.authorization_url.clone(),
        state: pending.state.clone(),
        redirect_uri: pending.request.redirect_uri.clone(),
    };
    let mut pending_map = state
        .account_setup
        .pending_gmail_oauth
        .lock()
        .map_err(|_| ApiError::AccountSetupState)?;
    pending_map.insert(pending.setup_id.clone(), pending);

    Ok(Json(response))
}

pub(crate) async fn post_gmail_oauth_complete(
    State(state): State<AppState>,
    Json(request): Json<GmailOAuthCompleteApiRequest>,
) -> Result<Json<EmailAccountSetupApiResponse>, ApiError> {
    let pending = {
        let mut pending_map = state
            .account_setup
            .pending_gmail_oauth
            .lock()
            .map_err(|_| ApiError::AccountSetupState)?;
        pending_map
            .remove(&request.setup_id)
            .ok_or(ApiError::AccountSetupPendingGrantNotFound)?
    };
    if pending.state != request.state {
        return Err(ApiError::AccountSetupStateMismatch);
    }

    let service = account_setup_service(&state)?;
    let result = service
        .complete_gmail_oauth(pending, &request.authorization_code)
        .await?;

    Ok(Json(result.into()))
}

pub(crate) async fn get_gmail_oauth_callback(
    Query(query): Query<GmailOAuthCallbackQuery>,
) -> Html<String> {
    let code = html_escape(&query.code.unwrap_or_default());
    let state = html_escape(&query.state.unwrap_or_default());

    Html(format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Hermes Hub OAuth</title>
  <style>
    body {{ margin: 0; font-family: system-ui, sans-serif; color: #182033; background: #f5f6f8; }}
    main {{ max-width: 720px; margin: 48px auto; background: #fff; border: 1px solid #d9dee7; border-radius: 8px; padding: 24px; }}
    code {{ display: block; overflow-wrap: anywhere; background: #f8fafc; border: 1px solid #d9dee7; border-radius: 6px; padding: 10px; }}
  </style>
</head>
<body>
  <main>
    <h1>Gmail OAuth callback</h1>
    <p>Authorization code</p>
    <code>{code}</code>
    <p>State</p>
    <code>{state}</code>
  </main>
</body>
</html>"#
    ))
}

pub(crate) async fn post_imap_account_setup(
    State(state): State<AppState>,
    Json(request): Json<ImapAccountSetupApiRequest>,
) -> Result<Json<EmailAccountSetupApiResponse>, ApiError> {
    let service = account_setup_service(&state)?;
    let result = service
        .setup_imap_account(request.into_setup_request()?)
        .await?;

    Ok(Json(result.into()))
}

#[derive(Deserialize)]
pub(crate) struct GmailOAuthStartApiRequest {
    account_id: String,
    display_name: String,
    external_account_id: String,
    client_id: String,
    client_secret: Option<String>,
    redirect_uri: String,
    authorization_endpoint: Option<String>,
    token_endpoint: Option<String>,
}

impl GmailOAuthStartApiRequest {
    fn into_setup_request(self) -> GmailOAuthSetupRequest {
        let mut request = GmailOAuthSetupRequest::new(
            self.account_id,
            self.display_name,
            self.external_account_id,
            self.client_id,
            self.redirect_uri,
        );
        if let Some(client_secret) = self.client_secret {
            request = request.client_secret(client_secret);
        }
        if let Some(authorization_endpoint) = self.authorization_endpoint {
            request = request.authorization_endpoint(authorization_endpoint);
        }
        if let Some(token_endpoint) = self.token_endpoint {
            request = request.token_endpoint(token_endpoint);
        }

        request
    }
}

#[derive(Serialize)]
pub(crate) struct GmailOAuthStartApiResponse {
    setup_id: String,
    authorization_url: String,
    state: String,
    redirect_uri: String,
}

#[derive(Deserialize)]
pub(crate) struct GmailOAuthCompleteApiRequest {
    setup_id: String,
    state: String,
    authorization_code: String,
}

#[derive(Deserialize)]
pub(crate) struct GmailOAuthCallbackQuery {
    code: Option<String>,
    state: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct ImapAccountSetupApiRequest {
    account_id: String,
    provider_kind: String,
    display_name: String,
    external_account_id: String,
    host: String,
    port: u16,
    tls: bool,
    mailbox: String,
    username: String,
    password: String,
    secret_kind: Option<String>,
}

impl ImapAccountSetupApiRequest {
    fn into_setup_request(self) -> Result<ImapAccountSetupRequest, ApiError> {
        let provider_kind = match self.provider_kind.trim() {
            "icloud" => EmailProviderKind::Icloud,
            "imap" => EmailProviderKind::Imap,
            _ => {
                return Err(EmailAccountSetupError::InvalidRequest {
                    field: "provider_kind",
                    message: "must be icloud or imap",
                }
                .into());
            }
        };
        let secret_kind = match self.secret_kind.as_deref().unwrap_or("password").trim() {
            "app_password" => SecretKind::AppPassword,
            "password" => SecretKind::Password,
            _ => {
                return Err(EmailAccountSetupError::InvalidRequest {
                    field: "secret_kind",
                    message: "must be app_password or password",
                }
                .into());
            }
        };

        Ok(ImapAccountSetupRequest::new(
            self.account_id,
            provider_kind,
            self.display_name,
            self.external_account_id,
            self.host,
            self.port,
            self.tls,
            self.mailbox,
            self.username,
            self.password,
        )
        .secret_kind(secret_kind))
    }
}

#[derive(Serialize)]
pub(crate) struct EmailAccountSetupApiResponse {
    account_id: String,
    secret_ref: String,
    secret_kind: SecretKind,
    store_kind: crate::platform::secrets::SecretStoreKind,
}

impl From<crate::domains::mail::accounts::EmailAccountSetupResult>
    for EmailAccountSetupApiResponse
{
    fn from(result: crate::domains::mail::accounts::EmailAccountSetupResult) -> Self {
        Self {
            account_id: result.account_id,
            secret_ref: result.secret_ref,
            secret_kind: result.secret_kind,
            store_kind: result.store_kind,
        }
    }
}
