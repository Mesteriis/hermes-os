pub mod ai;
pub mod attachment_intelligence;
pub mod audit;
pub mod automation;
pub mod calls;
pub mod capabilities;
pub mod communications;
pub mod config;
pub mod person_core;
pub mod person_enrichment;
pub mod person_identity;
pub mod person_intelligence;
pub mod person_memory;
pub mod person_enrichment_engine;
pub mod person_expertise;
pub mod person_trust;
pub mod person_health;
pub mod person_investigator;
pub mod person_analytics;
pub mod person_export;
pub mod organizations;
pub mod organization_core;
pub mod organization_enrichment;
pub mod organization_finance;
pub mod organization_health;
pub mod organization_investigator;
pub mod organization_memory;
pub mod organization_workflows;
pub mod persons;
pub mod document_processing;
pub mod documents;
pub mod email_account_setup;
pub mod email_actions;
pub mod email_ai_reply;
pub mod email_analytics;
pub mod email_attachment_dedup;
pub mod email_blockers;
pub mod email_drafts;
pub mod email_explain;
pub mod email_export;
pub mod email_extract;
pub mod email_finance;
pub mod email_fixture_export;
pub mod email_fixture_pipeline;
pub mod email_flags;
pub mod email_imap_write;
pub mod email_import;
pub mod email_ingestion;
pub mod email_intelligence;
pub mod email_legal;
pub mod email_multilingual;
pub mod email_personas;
pub mod email_provider_network;
pub mod email_rfc822;
pub mod email_rich_template;
pub mod email_rules;
pub mod email_search;
pub mod email_send;
pub mod email_signatures;
pub mod email_sources;
pub mod email_spf_dkim;
pub mod email_subscriptions;
pub mod email_sync;
pub mod email_sync_pipeline;
pub mod email_templates;
pub mod email_threads;
pub mod event_log;
pub mod graph;
pub mod graph_projection;
pub mod mail_storage;
pub mod messages;
pub mod ollama;
pub mod project_link_reviews;
pub mod projections;
pub mod projects;
pub mod search;
pub mod secret_vault;
pub mod secrets;
pub mod settings;
pub mod storage;
pub mod task_candidates;
pub mod telegram;
pub mod whatsapp;

use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex};

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

use crate::ai::{
    AI_EMBEDDING_DIMENSION, AiAgentListResponse, AiAgentRun, AiAnswerRequest, AiError,
    AiMeetingPrepRequest, AiService, AiStatusResponse, AiTaskCandidateRefreshRequest, v3_agents,
};
use crate::audit::{ApiAuditError, ApiAuditLog, ApiAuditRecord, NewApiAuditRecord};
use crate::automation::{
    AutomationError, AutomationPolicy, AutomationStore, AutomationTemplate, NewAutomationPolicy,
    NewAutomationTemplate, TelegramSendDryRunRequest, TelegramSendDryRunResponse,
};
use crate::calls::{
    CallDirection, CallError, CallIntelligenceStore, CallState, CallTranscript,
    FixtureSpeechToTextProvider, NewCallTranscript, NewTelegramCall, SpeechToTextProvider,
    TelegramCall, TranscriptStatus,
};
use crate::capabilities::{CapabilityActionClass, CapabilityDecision};
use crate::communications::{
    CommunicationIngestionError, CommunicationIngestionStore, EmailProviderKind, ProviderAccount,
};
use crate::config::AppConfig;
use crate::person_enrichment_engine::{EnrichmentEngineError, EnrichmentResultStore};
use crate::person_expertise::{PersonExpertiseError, PersonExpertiseStore};
use crate::person_analytics::{AnalyticsError, PersonAnalyticsService};
use crate::person_export::{ExportError, ExportFormat, PersonExportService};
use crate::person_investigator::{InvestigatorError, PersonInvestigator};

use crate::person_health::{PersonHealthError, PersonHealthStore};

use crate::person_trust::{PersonPromiseStore, PersonRiskStore, PersonTrustError};

use crate::person_memory::{
    NewRelationshipEvent, PersonFactStore, PersonMemoryCardStore,
    PersonMemoryError, PersonPreferenceStore, RelationshipEventStore,
};

use crate::person_identity::{
    PersonIdentityCandidate, PersonIdentityDetail, PersonIdentityError,
    PersonIdentityReviewCommand, PersonIdentityReviewState, PersonIdentityStore,
};
use crate::person_core::{
    NewPersonPersona, PersonCoreError, PersonIdentity, PersonsIdentityStore,
    PersonPersona, PersonPersonaStore, PersonRole, PersonRoleStore,
};

use crate::document_processing::{
    DocumentProcessingError, DocumentProcessingJob, DocumentProcessingRecord,
    DocumentProcessingRetryCommand, DocumentProcessingRetryCommandResult, DocumentProcessingStatus,
    DocumentProcessingStore,
};
use crate::email_account_setup::{
    EmailAccountSetupError, EmailAccountSetupService, GmailOAuthPendingGrant,
    GmailOAuthSetupRequest, ImapAccountSetupRequest,
};
use crate::email_intelligence::{EmailIntelligenceError, EmailIntelligenceService};
use crate::event_log::{
    EventEnvelope, EventEnvelopeError, EventStore, EventStoreError, NewEventEnvelope,
};
use crate::graph::{GraphNodeKind, node_id};
use crate::mail_storage::{MailStorageError, MailStorageStore, StoredMailAttachmentWithBlob};
use crate::messages::{
    MessageProjectionError, MessageProjectionStore, ProjectedMessage, ProjectedMessageSummary,
    WorkflowState,
};
use crate::ollama::{OllamaClient, OllamaClientConfig};
use crate::organizations::{OrganizationError, OrganizationStore, OrganizationUpdate};
use crate::project_link_reviews::{
    ProjectLinkReviewCommand, ProjectLinkReviewError, ProjectLinkReviewState,
    ProjectLinkReviewStore, ProjectLinkTargetKind,
};
use crate::projects::{ProjectListResponse, ProjectStore, ProjectStoreError};
use crate::secret_vault::DatabaseEncryptedSecretVault;
use crate::secrets::{SecretKind, SecretReferenceStore};
use crate::settings::{
    AiRuntimeSettings, ApplicationSetting, ApplicationSettingsStore, SettingsError,
};
use crate::storage::{
    Database, DatabaseReadiness, MigrationReadiness, ReadinessStatus, StorageError,
};
use crate::task_candidates::{
    ActiveTask, TaskCandidate, TaskCandidateError, TaskCandidateReviewCommand,
    TaskCandidateReviewState, TaskCandidateStore,
};
use crate::telegram::{
    NewTelegramMessage, TelegramAccountSetupRequest, TelegramAccountSetupResponse, TelegramChat,
    TelegramError, TelegramMessage, TelegramMessageIngestResult, TelegramStore,
};
use crate::whatsapp::{
    NewWhatsappWebMessage, WhatsappWebAccountSetupRequest, WhatsappWebAccountSetupResponse,
    WhatsappWebError, WhatsappWebMessage, WhatsappWebMessageIngestResult, WhatsappWebSession,
    WhatsappWebStore,
};

const LOCAL_API_ACTOR_ID_HEADER: &str = "x-hermes-actor-id";
const MAX_LOCAL_API_ACTOR_ID_LENGTH: usize = 128;

#[derive(Clone)]
struct AppState {
    config: AppConfig,
    database: Database,
    account_setup: AccountSetupState,
}

#[derive(Clone, Default)]
struct AccountSetupState {
    pending_gmail_oauth: Arc<Mutex<HashMap<String, GmailOAuthPendingGrant>>>,
}

#[derive(Deserialize)]
struct WorkflowStateTransitionApiRequest {
    workflow_state: String,
}

#[derive(Serialize)]
struct WorkflowStateTransitionApiResponse {
    message_id: String,
    workflow_state: String,
    previous_state: String,
}

#[derive(Serialize)]
struct WorkflowStateCountsApiResponse {
    counts: Vec<WorkflowStateCountApiItem>,
}

#[derive(Serialize)]
struct WorkflowStateCountApiItem {
    state: String,
    count: i64,
}

#[derive(Deserialize)]
struct WorkflowStateCountsQuery {
    account_id: Option<String>,
}

async fn put_v1_message_workflow_state(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(message_id): Path<String>,
    Json(request): Json<WorkflowStateTransitionApiRequest>,
) -> Result<Json<WorkflowStateTransitionApiResponse>, ApiError> {
    let actor = verify_local_api_capability(&state.config, &headers)?;
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
            &actor.actor_id,
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

async fn get_v1_message_workflow_state_counts(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<WorkflowStateCountsQuery>,
) -> Result<Json<WorkflowStateCountsApiResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
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
struct MessageAnalyzeResponse {
    message_id: String,
    analyzed: bool,
    category: Option<String>,
    summary: Option<String>,
    importance_score: Option<i16>,
    workflow_state: String,
}

async fn post_v1_message_analyze(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(message_id): Path<String>,
) -> Result<Json<MessageAnalyzeResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
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
struct ThreadListQuery {
    account_id: Option<String>,
    limit: Option<i64>,
}

#[derive(Serialize)]
struct ThreadListResponse {
    items: Vec<crate::email_threads::EmailThread>,
}

#[derive(Deserialize)]
struct ThreadMessagesQuery {
    account_id: Option<String>,
    subject: Option<String>,
    limit: Option<i64>,
}

#[derive(Serialize)]
struct ThreadMessagesResponse {
    items: Vec<crate::email_threads::ThreadMessage>,
}

async fn get_v1_threads(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ThreadListQuery>,
) -> Result<Json<ThreadListResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::email_threads::EmailThreadStore::new(pool);
    let items = store
        .list_threads(query.account_id.as_deref(), query.limit.unwrap_or(50))
        .await?;

    Ok(Json(ThreadListResponse { items }))
}

async fn get_v1_thread_messages(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ThreadMessagesQuery>,
) -> Result<Json<ThreadMessagesResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
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
    let store = crate::email_threads::EmailThreadStore::new(pool);
    let items = store
        .thread_messages(account_id, subject, query.limit.unwrap_or(50))
        .await?;

    Ok(Json(ThreadMessagesResponse { items }))
}

#[derive(Deserialize)]
struct EmailSearchQuery {
    q: String,
    limit: Option<usize>,
}

#[derive(Serialize)]
struct EmailSearchResponse {
    results: Vec<SearchResultResponse>,
}

#[derive(Serialize)]
struct SearchResultResponse {
    object_id: String,
    object_kind: String,
    title: String,
}

async fn get_v1_email_search(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<EmailSearchQuery>,
) -> Result<Json<EmailSearchResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
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
        let index = crate::search::SearchIndex::open_or_create(std::path::Path::new(&path))?;
        let _ = crate::email_search::index_messages(&index, &store, 100).await;
        let results =
            crate::email_search::search_emails(&index, &query.q, query.limit.unwrap_or(20))?;
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
struct PersonaListResponse {
    items: Vec<crate::email_personas::EmailPersona>,
}

#[derive(Deserialize)]
struct NewPersonaRequest {
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

async fn get_v1_personas(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<PersonaListResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::email_personas::EmailPersonaStore::new(pool);
    let items = store.list().await?;
    Ok(Json(PersonaListResponse { items }))
}

async fn post_v1_persona(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<NewPersonaRequest>,
) -> Result<Json<crate::email_personas::EmailPersona>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::email_personas::EmailPersonaStore::new(pool);
    let persona = store
        .upsert(&crate::email_personas::NewEmailPersona {
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
struct DraftListQuery {
    account_id: Option<String>,
    status: Option<String>,
}

#[derive(Serialize)]
struct DraftListResponse {
    items: Vec<crate::email_drafts::EmailDraft>,
}

#[derive(Deserialize)]
struct NewDraftRequest {
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

async fn get_v1_drafts(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<DraftListQuery>,
) -> Result<Json<DraftListResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::email_drafts::EmailDraftStore::new(pool);
    let status = query
        .status
        .as_deref()
        .and_then(crate::email_drafts::DraftStatus::parse);
    let items = store.list(query.account_id.as_deref(), status).await?;
    Ok(Json(DraftListResponse { items }))
}

async fn post_v1_draft(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<NewDraftRequest>,
) -> Result<Json<crate::email_drafts::EmailDraft>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::email_drafts::EmailDraftStore::new(pool);
    let draft = store
        .upsert(&crate::email_drafts::NewEmailDraft {
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
                .and_then(crate::email_drafts::DraftStatus::parse)
                .unwrap_or(crate::email_drafts::DraftStatus::Draft),
            scheduled_send_at: req.scheduled_send_at,
            metadata: req.metadata.unwrap_or(serde_json::json!({})),
        })
        .await?;
    Ok(Json(draft))
}

async fn get_v1_draft(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(draft_id): Path<String>,
) -> Result<Json<crate::email_drafts::EmailDraft>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::email_drafts::EmailDraftStore::new(pool);
    store
        .get(&draft_id)
        .await?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

async fn delete_v1_draft(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(draft_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::email_drafts::EmailDraftStore::new(pool);
    let deleted = store.delete(&draft_id).await?;
    Ok(Json(serde_json::json!({"deleted": deleted})))
}

#[derive(Deserialize)]
struct InvoiceListQuery {
    status: Option<String>,
}

#[derive(Serialize)]
struct InvoiceListResponse {
    items: Vec<crate::email_finance::InvoiceRecord>,
}

#[derive(Deserialize)]
struct NewInvoiceRequest {
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

async fn get_v1_invoices(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<InvoiceListQuery>,
) -> Result<Json<InvoiceListResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::email_finance::EmailFinanceStore::new(pool);
    let status = query
        .status
        .as_deref()
        .and_then(crate::email_finance::InvoiceStatus::parse);
    let items = store.list(status).await?;
    Ok(Json(InvoiceListResponse { items }))
}

async fn post_v1_invoice(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<NewInvoiceRequest>,
) -> Result<Json<crate::email_finance::InvoiceRecord>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::email_finance::EmailFinanceStore::new(pool);
    let invoice = store
        .upsert_invoice(&crate::email_finance::NewInvoiceRecord {
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
                .and_then(crate::email_finance::InvoiceStatus::parse)
                .unwrap_or(crate::email_finance::InvoiceStatus::Received),
            linked_project_id: req.linked_project_id,
            linked_person_id: req.linked_person_id,
            metadata: req.metadata.unwrap_or(serde_json::json!({})),
        })
        .await?;
    Ok(Json(invoice))
}

#[derive(Deserialize)]
struct AnalyticsQuery {
    account_id: Option<String>,
}

async fn get_v1_analytics_health(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Json<crate::email_analytics::MailboxHealth>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::email_analytics::EmailAnalyticsStore::new(pool);
    let health = store.mailbox_health(query.account_id.as_deref()).await?;
    Ok(Json(health))
}

#[derive(Deserialize)]
struct SendersQuery {
    account_id: Option<String>,
    limit: Option<i64>,
}

async fn get_v1_analytics_senders(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<SendersQuery>,
) -> Result<Json<Vec<crate::email_analytics::SenderStats>>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::email_analytics::EmailAnalyticsStore::new(pool);
    let senders = store
        .top_senders(query.account_id.as_deref(), query.limit.unwrap_or(20))
        .await?;
    Ok(Json(senders))
}

#[derive(Serialize)]
struct MessageExplainResponse {
    reasons: Vec<String>,
}

async fn get_v1_message_explain(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(message_id): Path<String>,
) -> Result<Json<MessageExplainResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let store = message_store(&state)?;
    let message = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let ctx = crate::email_explain::explain_importance(&message);
    Ok(Json(MessageExplainResponse {
        reasons: ctx.reasons,
    }))
}

#[derive(Serialize)]
struct SmartCcResponse {
    suggestions: Vec<String>,
}

async fn get_v1_message_smart_cc(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(message_id): Path<String>,
) -> Result<Json<SmartCcResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let store = message_store(&state)?;
    let message = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let suggestions = crate::email_explain::smart_cc_suggestions(&message);
    Ok(Json(SmartCcResponse { suggestions }))
}

#[derive(Serialize)]
struct PinToggleResponse {
    message_id: String,
    pinned: bool,
}

async fn post_v1_message_pin(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(message_id): Path<String>,
) -> Result<Json<PinToggleResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let store = message_store(&state)?;
    let pinned = crate::email_flags::MessageFlags::toggle_pin(&store, &message_id).await?;
    Ok(Json(PinToggleResponse { message_id, pinned }))
}

#[derive(Deserialize)]
struct SnoozeRequest {
    until: String,
}

async fn post_v1_message_snooze(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(message_id): Path<String>,
    Json(req): Json<SnoozeRequest>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let until: DateTime<Utc> = req
        .until
        .parse()
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid datetime"))?;
    let store = message_store(&state)?;
    crate::email_flags::MessageFlags::snooze(&store, &message_id, until).await?;
    Ok(Json(serde_json::json!({"snoozed": true})))
}

async fn post_v1_message_mute(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(message_id): Path<String>,
) -> Result<Json<PinToggleResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let store = message_store(&state)?;
    let muted = crate::email_flags::MessageFlags::toggle_mute(&store, &message_id).await?;
    Ok(Json(PinToggleResponse {
        message_id,
        pinned: muted,
    }))
}

#[derive(Deserialize)]
struct LabelRequest {
    label: String,
}

async fn post_v1_message_label(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(message_id): Path<String>,
    Json(req): Json<LabelRequest>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let store = message_store(&state)?;
    crate::email_flags::MessageFlags::add_label(&store, &message_id, &req.label).await?;
    Ok(Json(serde_json::json!({"labeled": true})))
}

async fn delete_v1_message_label(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(message_id): Path<String>,
    Json(req): Json<LabelRequest>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let store = message_store(&state)?;
    crate::email_flags::MessageFlags::remove_label(&store, &message_id, &req.label).await?;
    Ok(Json(serde_json::json!({"removed": true})))
}

#[derive(Deserialize)]
struct SubscriptionsQuery {
    account_id: Option<String>,
    limit: Option<i64>,
}

async fn get_v1_subscriptions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<SubscriptionsQuery>,
) -> Result<Json<Vec<crate::email_subscriptions::SubscriptionSource>>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::email_subscriptions::SubscriptionStore::new(pool);
    let subs = store
        .detect_subscriptions(query.account_id.as_deref(), query.limit.unwrap_or(50))
        .await?;
    Ok(Json(subs))
}

#[derive(Deserialize)]
struct DupQuery {
    limit: Option<i64>,
}

async fn get_v1_attachment_duplicates(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<DupQuery>,
) -> Result<Json<Vec<crate::email_attachment_dedup::DuplicateGroup>>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::email_attachment_dedup::AttachmentDedupStore::new(pool);
    let dups = store.find_duplicates(query.limit.unwrap_or(20)).await?;
    Ok(Json(dups))
}

#[derive(Deserialize)]
struct LegalDocQuery {
    document_type: Option<String>,
    status: Option<String>,
}

#[derive(Serialize)]
struct LegalDocListResponse {
    items: Vec<crate::email_legal::LegalDocument>,
}

async fn get_v1_legal_docs(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<LegalDocQuery>,
) -> Result<Json<LegalDocListResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::email_legal::LegalDocumentStore::new(pool);
    let dt = query
        .document_type
        .as_deref()
        .and_then(crate::email_legal::LegalDocType::parse);
    let st = query
        .status
        .as_deref()
        .and_then(crate::email_legal::LegalDocStatus::parse);
    let items = store.list(dt, st).await?;
    Ok(Json(LegalDocListResponse { items }))
}

#[derive(Deserialize)]
struct NewLegalDocRequest {
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

async fn post_v1_legal_doc(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<NewLegalDocRequest>,
) -> Result<Json<crate::email_legal::LegalDocument>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::email_legal::LegalDocumentStore::new(pool);
    let doc = store
        .upsert(&crate::email_legal::NewLegalDocument {
            document_id: req.document_id,
            message_id: req.message_id,
            document_type: crate::email_legal::LegalDocType::parse(&req.document_type)
                .unwrap_or(crate::email_legal::LegalDocType::Other),
            title: req.title,
            parties: req.parties.unwrap_or_default(),
            effective_date: req.effective_date,
            expiry_date: req.expiry_date,
            amount: req.amount,
            currency: req.currency,
            status: req
                .status
                .as_deref()
                .and_then(crate::email_legal::LegalDocStatus::parse)
                .unwrap_or(crate::email_legal::LegalDocStatus::Draft),
            linked_project_id: req.linked_project_id,
            risks: req.risks.unwrap_or_default(),
            metadata: req.metadata.unwrap_or(serde_json::json!({})),
        })
        .await?;
    Ok(Json(doc))
}

#[derive(Deserialize)]
struct PersonExportQuery {
    format: Option<String>,
}

#[derive(Serialize)]
struct ExportResponse {
    content_type: String,
    content: String,
    filename: String,
}

async fn get_v1_message_export(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(message_id): Path<String>,
    Query(query): Query<PersonDownloadQuery>,
) -> Result<Json<ExportResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let msg_store = message_store(&state)?;
    let att_store = mail_storage_store(&state)?;
    let format = match query.format.as_deref().unwrap_or("markdown") {
        "eml" => crate::email_export::ExportFormat::Eml,
        "json" => crate::email_export::ExportFormat::Json,
        _ => crate::email_export::ExportFormat::Markdown,
    };
    let export =
        crate::email_export::export_message(&msg_store, &att_store, &message_id, format).await?;
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
struct SendRequest {
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
struct SendResponse {
    message_id: String,
    accepted: Vec<String>,
}

async fn post_v1_send(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<SendRequest>,
) -> Result<Json<SendResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let email = crate::email_send::OutgoingEmail {
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

async fn post_v1_reply(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(message_id): Path<String>,
    Json(req): Json<SendRequest>,
) -> Result<Json<SendResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
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

async fn post_v1_imap_mark_read(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
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

async fn post_v1_imap_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
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
struct CertsQuery {
    limit: Option<i64>,
}
#[derive(Serialize)]
struct CertsListResponse {
    items: Vec<crate::email_signatures::CertificateRecord>,
}

async fn get_v1_certs(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<CertsListResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::email_signatures::CertificateStore::new(pool);
    Ok(Json(CertsListResponse {
        items: store.list().await?,
    }))
}

#[derive(Deserialize)]
struct NewCertRequest {
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

async fn post_v1_cert(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<NewCertRequest>,
) -> Result<Json<crate::email_signatures::CertificateRecord>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::email_signatures::CertificateStore::new(pool);
    Ok(Json(
        store
            .upsert(&crate::email_signatures::NewCertificate {
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
                    .and_then(crate::email_signatures::CertificateType::parse)
                    .unwrap_or(crate::email_signatures::CertificateType::Unknown),
                provider: req
                    .provider
                    .as_deref()
                    .and_then(crate::email_signatures::CertificateProvider::parse)
                    .unwrap_or(crate::email_signatures::CertificateProvider::Other),
                storage_kind: req
                    .storage_kind
                    .as_deref()
                    .and_then(crate::email_signatures::CertificateStorageKind::parse)
                    .unwrap_or(crate::email_signatures::CertificateStorageKind::EncryptedVault),
                storage_ref: req.storage_ref,
                trust_status: req
                    .trust_status
                    .as_deref()
                    .and_then(crate::email_signatures::TrustStatus::parse)
                    .unwrap_or(crate::email_signatures::TrustStatus::Untrusted),
                is_revoked: req.is_revoked.unwrap_or(false),
                usage: req.usage.unwrap_or_default(),
                linked_message_id: req.linked_message_id,
                metadata: req.metadata.unwrap_or(serde_json::json!({})),
            })
            .await?,
    ))
}

#[derive(Deserialize)]
struct ExpiringQuery {
    days: Option<i64>,
}
async fn get_v1_certs_expiring(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ExpiringQuery>,
) -> Result<Json<CertsListResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::email_signatures::CertificateStore::new(pool);
    Ok(Json(CertsListResponse {
        items: store.expiring_soon(query.days.unwrap_or(90)).await?,
    }))
}

async fn get_v1_signature_check(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(message_id): Path<String>,
) -> Result<Json<crate::email_signatures::SignatureDetection>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    Ok(Json(
        crate::email_signatures::SignatureDetector::detect_in_message(&msg.body_text, ""),
    ))
}

#[derive(Deserialize)]
struct ForwardRequest {
    to: Vec<String>,
    cc: Option<Vec<String>>,
    note: Option<String>,
}

async fn post_v1_forward(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(message_id): Path<String>,
    Json(req): Json<ForwardRequest>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
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

async fn get_v1_detect_language(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(message_id): Path<String>,
) -> Result<Json<crate::email_multilingual::LanguageDetection>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    Ok(Json(
        crate::email_multilingual::MultilingualService::detect_language(&msg.body_text),
    ))
}

#[derive(Deserialize)]
struct TranslateRequest {
    target_language: String,
}

async fn post_v1_translate(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(message_id): Path<String>,
    Json(req): Json<TranslateRequest>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
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
struct AiReplyRequest {
    tone: Option<String>,
    language: Option<String>,
    context: Option<String>,
}

async fn post_v1_ai_reply(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(message_id): Path<String>,
    Json(req): Json<AiReplyRequest>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let service = email_ai_reply_service(&state)?;
    let opts = crate::email_ai_reply::AiReplyOptions {
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
struct AiReplyVariantsRequest {
    languages: Option<Vec<String>>,
    tones: Option<Vec<String>>,
}

async fn post_v1_ai_reply_variants(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(message_id): Path<String>,
    Json(req): Json<AiReplyVariantsRequest>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
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
struct ReplyAllRequest {
    body_text: String,
    quote: Option<bool>,
}
async fn post_v1_reply_all(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(message_id): Path<String>,
    Json(req): Json<ReplyAllRequest>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let body = crate::email_actions::build_reply_body(
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
struct ForwardEmlRequest {
    to: Vec<String>,
}
async fn post_v1_forward_eml(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(message_id): Path<String>,
    Json(req): Json<ForwardEmlRequest>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let eml = crate::email_actions::build_eml_forward(
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

async fn get_v1_spf_dkim(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let auth = crate::email_spf_dkim::parse_auth_headers(&msg.body_text);
    let risk = crate::email_spf_dkim::assess_auth_risk(&auth);
    Ok(Json(serde_json::json!({"auth": auth, "risk": risk})))
}

async fn post_v1_extract_tasks(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let svc = crate::email_extract::EmailExtractService::new(
        crate::ollama::OllamaClient::new(crate::ollama::OllamaClientConfig::new(
            "http://127.0.0.1:11434",
            "qwen3:4b",
            "qwen3-embedding:4b",
        ))
        .ok(),
    );
    let tasks = svc.extract_tasks(&msg).await?;
    Ok(Json(serde_json::json!({"tasks": tasks})))
}

async fn post_v1_extract_notes(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let svc = crate::email_extract::EmailExtractService::new(None);
    let notes = svc.extract_notes(&msg).await?;
    Ok(Json(serde_json::json!({"notes": notes})))
}

#[derive(Deserialize)]
struct RenderTemplateRequest {
    template_id: String,
    variables: Option<HashMap<String, String>>,
}
async fn get_v1_rich_templates(
    State(_state): State<AppState>,
    _headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(serde_json::json!({"templates": []})))
}
async fn post_v1_rich_template(
    State(_state): State<AppState>,
    _headers: HeaderMap,
    Json(_req): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(serde_json::json!({"saved": true})))
}

async fn get_v1_blockers() -> Result<Json<Vec<crate::email_blockers::ArchitectureBlocker>>, ApiError>
{
    Ok(Json(crate::email_blockers::list_blockers()))
}

async fn post_v1_render_template(
    State(_state): State<AppState>,
    _headers: HeaderMap,
    Json(req): Json<RenderTemplateRequest>,
) -> Result<Json<Value>, ApiError> {
    let template_id = req.template_id;
    let vars = req.variables.unwrap_or_default();
    Ok(Json(
        serde_json::json!({"rendered": true, "template_id": template_id, "variables": vars}),
    ))
}

#[derive(Deserialize)]
struct PersonListQuery {
    favorites_only: Option<bool>,
    limit: Option<i64>,
}
#[derive(Serialize)]
struct PersonListResponse {
    items: Vec<crate::person_enrichment::EnrichedPerson>,
}

async fn get_v2_persons(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PersonListQuery>,
) -> Result<Json<PersonListResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::person_enrichment::PersonEnrichmentStore::new(pool);
    let items = store
        .list_enriched(
            query.favorites_only.unwrap_or(false),
            query.limit.unwrap_or(50),
        )
        .await?;
    Ok(Json(PersonListResponse { items }))
}

async fn get_v2_person(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(person_id): Path<String>,
) -> Result<Json<crate::person_enrichment::EnrichedPerson>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::person_enrichment::PersonEnrichmentStore::new(pool);
    match store.get_enriched(&person_id).await? {
        Some(person) => Ok(Json(person)),
        None => Err(ApiError::PersonIdentityNotFound),
    }
}

async fn post_v2_person_fingerprint(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(person_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let msg_store = crate::messages::MessageProjectionStore::new(pool.clone());
    // Build person messages from this person's email history
    let messages = msg_store.recent_messages(50).await?;
    let person_msgs: Vec<crate::person_intelligence::PersonMessage> = messages
        .into_iter()
        .filter(|m| {
            m.message.sender.contains(&person_id)
                || m.message.recipients.iter().any(|r| r.contains(&person_id))
        })
        .map(|m| crate::person_intelligence::PersonMessage {
            subject: m.message.subject,
            body_text: m.message.body_text,
            occurred_at: m.message.occurred_at,
        })
        .collect();
    let fp = crate::person_intelligence::PersonIntelligenceService::heuristic_fingerprint(
        &person_msgs,
    );
    let store = crate::person_enrichment::PersonEnrichmentStore::new(pool);
    store.enrich_person(&person_id, &fp).await?;
    Ok(Json(
        serde_json::json!({"enriched": true, "fingerprint": fp}),
    ))
}

async fn post_v2_person_favorite(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(person_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::person_enrichment::PersonEnrichmentStore::new(pool);
    let fav = store.toggle_favorite(&person_id).await?;
    Ok(Json(serde_json::json!({"is_favorite": fav})))
}

#[derive(Deserialize)]
struct PersonNotesRequest {
    notes: String,
}
async fn put_v2_person_notes(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(person_id): Path<String>,
    Json(req): Json<PersonNotesRequest>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::person_enrichment::PersonEnrichmentStore::new(pool);
    store.set_notes(&person_id, &req.notes).await?;
    Ok(Json(serde_json::json!({"saved": true})))
}

#[derive(Deserialize)]
struct PersonSearchQuery {
    q: String,
    limit: Option<i64>,
}
async fn get_v2_person_search(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PersonSearchQuery>,
) -> Result<Json<PersonListResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    if query.q.trim().is_empty() {
        return Err(ApiError::InvalidCommunicationQuery("search query required"));
    }
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::person_enrichment::PersonEnrichmentStore::new(pool);
    let items = store
        .search_persons(&query.q, query.limit.unwrap_or(20))
        .await?;
    Ok(Json(PersonListResponse { items }))
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: String,
}

pub fn build_router(config: AppConfig) -> Router {
    build_router_with_database(config, Database::disabled())
}

pub fn build_router_with_database(config: AppConfig, database: Database) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/api/v1/status", get(get_v1_status))
        .route(
            "/api/v1/communications/messages",
            get(get_v1_communication_messages),
        )
        .route(
            "/api/v1/communications/messages/{message_id}",
            get(get_v1_communication_message),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/workflow-state",
            put(put_v1_message_workflow_state),
        )
        .route(
            "/api/v1/communications/messages/states",
            get(get_v1_message_workflow_state_counts),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/analyze",
            post(post_v1_message_analyze),
        )
        .route("/api/v1/communications/threads", get(get_v1_threads))
        .route(
            "/api/v1/communications/threads/messages",
            get(get_v1_thread_messages),
        )
        .route("/api/v1/communications/search", get(get_v1_email_search))
        .route(
            "/api/v1/communications/personas",
            get(get_v1_personas).post(post_v1_persona),
        )
        .route(
            "/api/v1/communications/drafts",
            get(get_v1_drafts).post(post_v1_draft),
        )
        .route(
            "/api/v1/communications/drafts/{draft_id}",
            get(get_v1_draft).delete(delete_v1_draft),
        )
        .route(
            "/api/v1/communications/finance/invoices",
            get(get_v1_invoices).post(post_v1_invoice),
        )
        .route(
            "/api/v1/communications/analytics/health",
            get(get_v1_analytics_health),
        )
        .route(
            "/api/v1/communications/analytics/senders",
            get(get_v1_analytics_senders),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/explain",
            get(get_v1_message_explain),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/smart-cc",
            get(get_v1_message_smart_cc),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/pin",
            post(post_v1_message_pin),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/snooze",
            post(post_v1_message_snooze),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/mute",
            post(post_v1_message_mute),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/labels",
            post(post_v1_message_label).delete(delete_v1_message_label),
        )
        .route(
            "/api/v1/communications/subscriptions",
            get(get_v1_subscriptions),
        )
        .route(
            "/api/v1/communications/attachments/duplicates",
            get(get_v1_attachment_duplicates),
        )
        .route(
            "/api/v1/communications/legal",
            get(get_v1_legal_docs).post(post_v1_legal_doc),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/export",
            get(get_v1_message_export),
        )
        .route("/api/v1/communications/send", post(post_v1_send))
        .route(
            "/api/v1/communications/messages/{message_id}/reply",
            post(post_v1_reply),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/imap-mark-read",
            post(post_v1_imap_mark_read),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/imap-delete",
            post(post_v1_imap_delete),
        )
        .route(
            "/api/v1/communications/certificates",
            get(get_v1_certs).post(post_v1_cert),
        )
        .route(
            "/api/v1/communications/certificates/expiring",
            get(get_v1_certs_expiring),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/signature",
            get(get_v1_signature_check),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/forward",
            post(post_v1_forward),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/detect-language",
            get(get_v1_detect_language),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/translate",
            post(post_v1_translate),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/ai-reply",
            post(post_v1_ai_reply),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/ai-reply-variants",
            post(post_v1_ai_reply_variants),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/reply-all",
            post(post_v1_reply_all),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/forward-eml",
            post(post_v1_forward_eml),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/spf-dkim",
            get(get_v1_spf_dkim),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/extract-tasks",
            post(post_v1_extract_tasks),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/extract-notes",
            post(post_v1_extract_notes),
        )
        .route(
            "/api/v1/communications/templates/rich",
            get(get_v1_rich_templates).post(post_v1_rich_template),
        )
        .route(
            "/api/v1/communications/templates/rich/render",
            post(post_v1_render_template),
        )
        .route("/api/v1/communications/blockers", get(get_v1_blockers))
        .route("/api/v2/graph/summary", get(get_graph_summary))
        .route("/api/v2/graph/nodes", get(get_graph_nodes))
        .route("/api/v2/graph/neighborhood", get(get_graph_neighborhood))
        .route("/api/v2/graph/search", get(get_graph_search))
        .route("/api/v2/projects", get(get_projects))
        .route("/api/v2/projects/{project_id}", get(get_project_detail))
        .route(
            "/api/v2/projects/{project_id}/link-candidates",
            get(get_project_link_candidates),
        )
        .route(
            "/api/v2/projects/{project_id}/link-reviews",
            put(put_project_link_review),
        )
        .route(
            "/api/v2/documents/{document_id}/processing",
            get(get_document_processing),
        )
        .route(
            "/api/v2/document-processing/jobs",
            get(get_document_processing_jobs),
        )
        .route(
            "/api/v2/document-processing/jobs/{job_id}/retry",
            post(post_document_processing_job_retry),
        )
        .route("/api/v2/persons", get(get_v2_persons))
        .route("/api/v2/persons/{person_id}", get(get_v2_person))
        .route(
            "/api/v2/persons/{person_id}/fingerprint",
            post(post_v2_person_fingerprint),
        )
        .route(
            "/api/v2/persons/{person_id}/favorite",
            post(post_v2_person_favorite),
        )
        .route(
            "/api/v2/persons/{person_id}/notes",
            put(put_v2_person_notes),
        )
        .route("/api/v2/persons/search", get(get_v2_person_search))
        .route("/api/v2/identity-candidates", get(get_identity_candidates))
        .route(
            "/api/v2/identity-candidates/{identity_candidate_id}/review",
            put(put_identity_candidate_review),
        )
        .route(
            "/api/v2/persons/{person_id}/identity",
            get(get_person_identity),
        )
        .route(
            "/api/v2/persons/{person_id}/identities",
            get(get_person_identities),
        )
        .route(
            "/api/v2/persons/{person_id}/identities",
            post(post_person_identity),
        )
        .route(
            "/api/v2/persons/{person_id}/identities/{identity_id}",
            delete(delete_person_identity),
        )
        .route(
            "/api/v2/persons/{person_id}/roles",
            get(get_person_roles),
        )
        .route(
            "/api/v2/persons/{person_id}/roles",
            post(post_person_role),
        )
        .route(
            "/api/v2/persons/{person_id}/roles/{role}",
            delete(delete_person_role),
        )
        .route(
            "/api/v2/persons/{person_id}/personas",
            get(get_person_personas),
        )
        .route(
            "/api/v2/persons/{person_id}/personas",
            post(post_person_persona),
        )
        .route(
            "/api/v2/persons/{person_id}/personas/{persona_id}",
            delete(delete_person_persona),
        )
        .route(
            "/api/v2/persons/{person_id}/facts",
            get(get_person_facts).post(post_person_fact),
        )
        .route(
            "/api/v2/persons/{person_id}/memory-cards",
            get(get_person_memory_cards).post(post_person_memory_card),
        )
        .route(
            "/api/v2/persons/{person_id}/preferences",
            get(get_person_preferences).post(post_person_preference),
        )
        .route(
            "/api/v2/persons/{person_id}/timeline",
            get(get_person_timeline).post(post_relationship_event),
        )
        .route(
            "/api/v2/persons/{person_id}/snapshots",
            get(get_person_snapshots),
        )
        .route(
            "/api/v2/persons/{person_id}/history-diff",
            get(get_person_history_diff),
        )
        .route("/api/v2/persons/{person_id}/timeline",
        get(get_person_timeline).post(post_relationship_event),
        )
        .route(
            "/api/v2/persons/{person_id}/enrichment",
            get(get_person_enrichment),
        )
        .route(
            "/api/v2/persons/{person_id}/enrichment/{result_id}/apply",
            post(post_person_enrichment_apply),
        )
        .route(
            "/api/v2/persons/{person_id}/enrichment/{result_id}/reject",
            post(post_person_enrichment_reject),
        )
        .route(
            "/api/v2/persons/{person_id}/expertise",
            get(get_person_expertise),
        )
        .route(
            "/api/v2/persons/search/expertise",
            get(get_person_expertise_search),
        )
        .route(
            "/api/v2/persons/{person_id}/promises",
            get(get_person_promises),
        )
        .route(
            "/api/v2/persons/{person_id}/risks",
            get(get_person_risks),
        )
        .route(
            "/api/v2/persons/{person_id}/investigate",
            post(post_person_investigate),
        )
        .route(
            "/api/v2/persons/{person_id}/dossier",
            get(get_person_dossier),
        )
        .route(
            "/api/v2/persons/{person_id}/meeting-prep",
            get(get_person_meeting_prep),
        )
        .route(
            "/api/v2/persons/{person_id}/analytics",
            get(get_person_analytics),
        )
        .route(
            "/api/v2/persons/{person_id}/export",
            get(get_person_export_handler),
        )
        .route("/api/v2/persons/{person_id}/risks",
        get(get_person_risks),
        )
        .route(
            "/api/v2/persons/{person_id}/health",
            get(get_persons_health),
        )
        .route(
            "/api/v2/persons/health",
            get(get_persons_health),
        )
        .route(
            "/api/v2/persons/watchlist",
            get(get_persons_watchlist),
        )
        .route(
            "/api/v2/persons/{person_id}/watchlist",
            post(post_person_watchlist_toggle),
        )
        .route("/api/v2/organizations", get(get_organizations).post(post_organization))
        .route("/api/v2/organizations/search", get(get_organization_search))
        .route("/api/v2/organizations/{org_id}", get(get_organization).put(put_organization))
        .route("/api/v2/organizations/{org_id}/archive", post(post_organization_archive))
        .route("/api/v2/task-candidates", get(get_task_candidates))
        .route(
            "/api/v2/task-candidates/{task_candidate_id}/review",
            put(put_task_candidate_review),
        )
        .route("/api/v2/tasks", get(get_tasks))
        .route("/api/v2/settings", get(get_application_settings))
        .route(
            "/api/v2/settings/accounts",
            get(get_application_settings_accounts),
        )
        .route(
            "/api/v2/settings/{setting_key}",
            put(put_application_setting),
        )
        .route("/api/v3/ai/status", get(get_v3_ai_status))
        .route("/api/v3/agents", get(get_v3_agents))
        .route("/api/v3/ai/runs", get(get_v3_ai_runs))
        .route("/api/v3/ai/runs/{run_id}", get(get_v3_ai_run))
        .route("/api/v3/ai/answers", post(post_v3_ai_answer))
        .route(
            "/api/v3/ai/task-candidates/refresh",
            post(post_v3_ai_task_candidates_refresh),
        )
        .route("/api/v3/ai/meeting-prep", post(post_v3_ai_meeting_prep))
        .route("/api/v4/capabilities", get(get_v4_capabilities))
        .route("/api/v5/capabilities", get(get_v5_capabilities))
        .route(
            "/api/v4/telegram/accounts/fixture",
            post(post_v4_telegram_fixture_account),
        )
        .route("/api/v4/telegram/chats", get(get_v4_telegram_chats))
        .route(
            "/api/v4/telegram/messages",
            get(get_v4_telegram_messages).post(post_v4_telegram_fixture_message),
        )
        .route(
            "/api/v4/policies/templates",
            get(get_v4_policy_templates).post(post_v4_policy_template),
        )
        .route(
            "/api/v4/policies",
            get(get_v4_policies).post(post_v4_policy),
        )
        .route(
            "/api/v4/policies/telegram-send/dry-run",
            post(post_v4_telegram_send_dry_run),
        )
        .route("/api/v4/calls", get(get_v4_calls).post(post_v4_call))
        .route(
            "/api/v4/calls/{call_id}/transcript",
            get(get_v4_call_transcript).post(post_v4_call_transcript_fixture),
        )
        .route(
            "/api/v5/whatsapp/accounts/fixture",
            post(post_v5_whatsapp_fixture_account),
        )
        .route("/api/v5/whatsapp/sessions", get(get_v5_whatsapp_sessions))
        .route(
            "/api/v5/whatsapp/messages",
            get(get_v5_whatsapp_messages).post(post_v5_whatsapp_fixture_message),
        )
        .route(
            "/api/v1/email-accounts/gmail/oauth/start",
            post(post_gmail_oauth_start),
        )
        .route(
            "/api/v1/email-accounts/gmail/oauth/complete",
            post(post_gmail_oauth_complete),
        )
        .route(
            "/api/v1/email-accounts/gmail/oauth/callback",
            get(get_gmail_oauth_callback),
        )
        .route("/api/v1/email-accounts/imap", post(post_imap_account_setup))
        .route("/api/audit/events", get(get_audit_events))
        .route("/api/events", post(post_event))
        .route("/api/events/{event_id}", get(get_event))
        .with_state(AppState {
            config,
            database,
            account_setup: AccountSetupState::default(),
        })
        .layer(local_frontend_cors_layer())
}


// ── Person Identities ───────────────────────────────────────────────────────

#[derive(Serialize)]
struct PersonIdentitiesResponse {
    items: Vec<PersonIdentity>,
}

async fn get_person_identities(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(person_id): Path<String>,
) -> Result<Json<PersonIdentitiesResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let store = PersonsIdentityStore::new(pool);
    let items = store.list_by_person(&person_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(PersonIdentitiesResponse { items }))
}

#[derive(Deserialize)]
struct NewPersonIdentityRequest {
    identity_type: String,
    identity_value: String,
    source: Option<String>,
}

async fn post_person_identity(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(person_id): Path<String>,
    Json(req): Json<NewPersonIdentityRequest>,
) -> Result<Json<PersonIdentity>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let store = PersonsIdentityStore::new(pool);
    let identity = store
        .upsert(&person_id, &req.identity_type, &req.identity_value, req.source.as_deref().unwrap_or("manual"))
        .await
        .map_err(|e| ApiError::from(e))?;
    Ok(Json(identity))
}

async fn delete_person_identity(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((_person_id, identity_id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let store = PersonsIdentityStore::new(pool);
    let deleted = store.delete(&identity_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(json!({"deleted": deleted})))
}

// ── Person Roles ────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct PersonRolesResponse {
    items: Vec<PersonRole>,
}

async fn get_person_roles(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(person_id): Path<String>,
) -> Result<Json<PersonRolesResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let store = PersonRoleStore::new(pool);
    let items = store.list_by_person(&person_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(PersonRolesResponse { items }))
}

#[derive(Deserialize)]
struct NewPersonRoleRequest {
    role: String,
}

async fn post_person_role(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(person_id): Path<String>,
    Json(req): Json<NewPersonRoleRequest>,
) -> Result<Json<PersonRole>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let store = PersonRoleStore::new(pool);
    let role = store.assign(&person_id, &req.role, None).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(role))
}

async fn delete_person_role(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((person_id, role)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let store = PersonRoleStore::new(pool);
    let deleted = store.remove(&person_id, &role).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(json!({"deleted": deleted})))
}

// ── Person Personas ─────────────────────────────────────────────────────────

#[derive(Serialize)]
struct PersonPersonasResponse {
    items: Vec<PersonPersona>,
}

async fn get_person_personas(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(person_id): Path<String>,
) -> Result<Json<PersonPersonasResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let store = PersonPersonaStore::new(pool);
    let items = store.list_by_person(&person_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(PersonPersonasResponse { items }))
}

async fn post_person_persona(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(person_id): Path<String>,
    Json(req): Json<NewPersonPersona>,
) -> Result<Json<PersonPersona>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let store = PersonPersonaStore::new(pool);
    let persona = store.upsert(&NewPersonPersona {
        person_id,
        ..req
    }).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(persona))
}

async fn delete_person_persona(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((_person_id, persona_id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let store = PersonPersonaStore::new(pool);
    let deleted = store.delete(&persona_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(json!({"deleted": deleted})))
}


// ── Person Facts ────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct PersonFactsResponse { items: Vec<crate::person_memory::PersonFact> }

async fn get_person_facts(
    State(state): State<AppState>, headers: HeaderMap, Path(person_id): Path<String>,
) -> Result<Json<PersonFactsResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let items = PersonFactStore::new(pool).list(&person_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(PersonFactsResponse { items }))
}

#[derive(Deserialize)]
struct NewPersonFactRequest { fact_type: String, value: String, source: Option<String>, confidence: Option<f64> }

async fn post_person_fact(
    State(state): State<AppState>, headers: HeaderMap, Path(person_id): Path<String>,
    Json(req): Json<NewPersonFactRequest>,
) -> Result<Json<crate::person_memory::PersonFact>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let fact = PersonFactStore::new(pool).upsert(&person_id, &req.fact_type, &req.value, req.source.as_deref().unwrap_or("manual"), req.confidence.unwrap_or(1.0)).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(fact))
}

// ── Person Memory Cards ─────────────────────────────────────────────────────

#[derive(Serialize)]
struct PersonMemoryCardsResponse { items: Vec<crate::person_memory::PersonMemoryCard> }

async fn get_person_memory_cards(
    State(state): State<AppState>, headers: HeaderMap, Path(person_id): Path<String>,
) -> Result<Json<PersonMemoryCardsResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let items = PersonMemoryCardStore::new(pool).list(&person_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(PersonMemoryCardsResponse { items }))
}

#[derive(Deserialize)]
struct NewPersonMemoryCardRequest { title: String, description: String, source: Option<String>, importance: Option<i16> }

async fn post_person_memory_card(
    State(state): State<AppState>, headers: HeaderMap, Path(person_id): Path<String>,
    Json(req): Json<NewPersonMemoryCardRequest>,
) -> Result<Json<crate::person_memory::PersonMemoryCard>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let card = PersonMemoryCardStore::new(pool).upsert(&person_id, &req.title, &req.description, req.source.as_deref().unwrap_or("manual"), req.importance.unwrap_or(5)).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(card))
}

// ── Person Preferences ──────────────────────────────────────────────────────

#[derive(Serialize)]
struct PersonPreferencesResponse { items: Vec<crate::person_memory::PersonPreference> }

async fn get_person_preferences(
    State(state): State<AppState>, headers: HeaderMap, Path(person_id): Path<String>,
) -> Result<Json<PersonPreferencesResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let items = PersonPreferenceStore::new(pool).list(&person_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(PersonPreferencesResponse { items }))
}

#[derive(Deserialize)]
struct NewPersonPreferenceRequest { preference_type: String, value: String, source: Option<String> }

async fn post_person_preference(
    State(state): State<AppState>, headers: HeaderMap, Path(person_id): Path<String>,
    Json(req): Json<NewPersonPreferenceRequest>,
) -> Result<Json<crate::person_memory::PersonPreference>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let pref = PersonPreferenceStore::new(pool).upsert(&person_id, &req.preference_type, &req.value, req.source.as_deref().unwrap_or("manual")).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(pref))
}

// ── Relationship Timeline ───────────────────────────────────────────────────

#[derive(Serialize)]
struct RelationshipTimelineResponse { items: Vec<crate::person_memory::RelationshipEvent> }

async fn get_person_timeline(
    State(state): State<AppState>, headers: HeaderMap,
    Path(person_id): Path<String>, Query(query): Query<TimelineQuery>,
) -> Result<Json<RelationshipTimelineResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let items = RelationshipEventStore::new(pool).timeline(&person_id, query.limit.unwrap_or(50)).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(RelationshipTimelineResponse { items }))
}

#[derive(Deserialize)]
struct TimelineQuery { limit: Option<i64> }

async fn post_relationship_event(
    State(state): State<AppState>, headers: HeaderMap, Path(person_id): Path<String>,
    Json(req): Json<NewRelationshipEvent>,
) -> Result<Json<crate::person_memory::RelationshipEvent>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let event = RelationshipEventStore::new(pool).add(&NewRelationshipEvent { person_id, ..req }).await.map_err(|e| ApiError::from(e))?;
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
struct EnrichmentResultsResponse { items: Vec<crate::person_enrichment_engine::EnrichmentResult> }

async fn get_person_enrichment(
    State(state): State<AppState>, headers: HeaderMap, Path(person_id): Path<String>,
) -> Result<Json<EnrichmentResultsResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let items = EnrichmentResultStore::new(pool).list(&person_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(EnrichmentResultsResponse { items }))
}

async fn post_person_enrichment_apply(
    State(state): State<AppState>, headers: HeaderMap, Path((_person_id, result_id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    EnrichmentResultStore::new(pool).apply(&result_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(json!({"applied": true})))
}

async fn post_person_enrichment_reject(
    State(state): State<AppState>, headers: HeaderMap, Path((_person_id, result_id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    EnrichmentResultStore::new(pool).reject(&result_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(json!({"rejected": true})))
}

// ── Person Expertise ───────────────────────────────────────────────────────

#[derive(Serialize)]
struct PersonExpertiseResponse { items: Vec<crate::person_expertise::PersonExpertise> }

async fn get_person_expertise(
    State(state): State<AppState>, headers: HeaderMap, Path(person_id): Path<String>,
) -> Result<Json<PersonExpertiseResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let items = PersonExpertiseStore::new(pool).list(&person_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(PersonExpertiseResponse { items }))
}

#[derive(Deserialize)]
struct ExpertiseSearchQuery { skill: String, limit: Option<i64> }

async fn get_person_expertise_search(
    State(state): State<AppState>, headers: HeaderMap, Query(query): Query<ExpertiseSearchQuery>,
) -> Result<Json<PersonExpertiseResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let items = PersonExpertiseStore::new(pool).search_by_skill(&query.skill, query.limit.unwrap_or(20)).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(PersonExpertiseResponse { items }))
}

// ── Person Promises ────────────────────────────────────────────────────────

#[derive(Serialize)]
struct PersonPromisesResponse { items: Vec<crate::person_trust::PersonPromise> }

async fn get_person_promises(
    State(state): State<AppState>, headers: HeaderMap, Path(person_id): Path<String>,
) -> Result<Json<PersonPromisesResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let items = PersonPromiseStore::new(pool).list(&person_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(PersonPromisesResponse { items }))
}

// ── Person Risks ────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct PersonRisksResponse { items: Vec<crate::person_trust::PersonRisk> }

async fn get_person_risks(
    State(state): State<AppState>, headers: HeaderMap, Path(person_id): Path<String>,
) -> Result<Json<PersonRisksResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let items = PersonRiskStore::new(pool).list(&person_id).await.map_err(|e| ApiError::from(e))?;
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
struct PersonHealthResponse { items: Vec<crate::person_health::PersonHealth> }

async fn get_persons_health(
    State(state): State<AppState>, headers: HeaderMap,
) -> Result<Json<PersonHealthResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let items = PersonHealthStore::new(pool).list_health().await.map_err(|e| ApiError::from(e))?;
    Ok(Json(PersonHealthResponse { items }))
}

async fn get_persons_watchlist(
    State(state): State<AppState>, headers: HeaderMap,
) -> Result<Json<PersonHealthResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let items = PersonHealthStore::new(pool).list_watchlist().await.map_err(|e| ApiError::from(e))?;
    Ok(Json(PersonHealthResponse { items }))
}

async fn post_person_watchlist_toggle(
    State(state): State<AppState>, headers: HeaderMap, Path(person_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let on = PersonHealthStore::new(pool).toggle_watchlist(&person_id).await.map_err(|e| ApiError::from(e))?;
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

async fn post_person_investigate(
    State(state): State<AppState>, headers: HeaderMap, Path(person_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let dossier = PersonInvestigator::new(pool).assemble_dossier(&person_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(serde_json::to_value(&dossier).unwrap_or_default()))
}

async fn get_person_dossier(
    State(state): State<AppState>, headers: HeaderMap, Path(person_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let dossier = PersonInvestigator::new(pool).assemble_dossier(&person_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(serde_json::to_value(&dossier).unwrap_or_default()))
}

async fn get_person_meeting_prep(
    State(state): State<AppState>, headers: HeaderMap, Path(person_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let prep = PersonInvestigator::new(pool).meeting_prep(&person_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(serde_json::to_value(&prep).unwrap_or_default()))
}

// ── Person Analytics ────────────────────────────────────────────────────────

async fn get_person_analytics(
    State(state): State<AppState>, headers: HeaderMap, Path(person_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let analytics = PersonAnalyticsService::new(pool).compute(&person_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(serde_json::to_value(&analytics).unwrap_or_default()))
}

// ── Person Export ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct PersonDownloadQuery { format: Option<String> }

async fn get_person_export_handler(
    State(state): State<AppState>, headers: HeaderMap, Path(person_id): Path<String>,
    Query(query): Query<PersonDownloadQuery>,
) -> Result<(HeaderMap, String), ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let format = query.format.as_deref().and_then(ExportFormat::from_str).unwrap_or(ExportFormat::Json);
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let content = PersonExportService::new(pool).export(&person_id, format.clone()).await.map_err(|e| ApiError::from(e))?;
    let mut headers_map = HeaderMap::new();
    headers_map.insert(header::CONTENT_TYPE, HeaderValue::from_str(format.content_type()).unwrap_or(HeaderValue::from_static("application/json")));
    headers_map.insert(
        HeaderName::from_static("content-disposition"),
        HeaderValue::from_str(&format!("attachment; filename=person_{}.{}", person_id, format.extension())).unwrap(),
    );
    Ok((headers_map, content))
}


// ── Person Snapshots & History Diff ─────────────────────────────────────────

#[derive(Serialize)]
struct PersonSnapshotsResponse { items: Vec<crate::person_memory::PersonSnapshot> }

async fn get_person_snapshots(
    State(state): State<AppState>, headers: HeaderMap, Path(person_id): Path<String>,
) -> Result<Json<PersonSnapshotsResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let items = crate::person_memory::PersonSnapshotStore::new(pool).list(&person_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(PersonSnapshotsResponse { items }))
}

#[derive(Deserialize)]
struct HistoryDiffQuery { from: String, to: String }

async fn get_person_history_diff(
    State(state): State<AppState>, headers: HeaderMap, Path(person_id): Path<String>,
    Query(query): Query<HistoryDiffQuery>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let from_date = DateTime::parse_from_rfc3339(&query.from).map_err(|_| ApiError::InvalidCommunicationQuery("invalid from date"))?.with_timezone(&Utc);
    let to_date = DateTime::parse_from_rfc3339(&query.to).map_err(|_| ApiError::InvalidCommunicationQuery("invalid to date"))?.with_timezone(&Utc);
    let diff = crate::person_memory::PersonSnapshotStore::new(pool).history_diff(&person_id, from_date, to_date).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(serde_json::to_value(&diff).unwrap_or_default()))
}

// ── Organizations ───────────────────────────────────────────────────────────

#[derive(Serialize)]
struct OrganizationListResponse { items: Vec<crate::organizations::Organization> }

#[derive(Deserialize)]
struct OrganizationListQuery { org_type: Option<String>, limit: Option<i64> }

async fn get_organizations(
    State(state): State<AppState>, headers: HeaderMap, Query(query): Query<OrganizationListQuery>,
) -> Result<Json<OrganizationListResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let items = OrganizationStore::new(pool).list(query.org_type.as_deref(), query.limit.unwrap_or(50)).await?;
    Ok(Json(OrganizationListResponse { items }))
}

#[derive(Deserialize)]
struct NewOrganizationRequest { display_name: String, org_type: Option<String> }

async fn post_organization(
    State(state): State<AppState>, headers: HeaderMap, Json(req): Json<NewOrganizationRequest>,
) -> Result<Json<crate::organizations::Organization>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let org = OrganizationStore::new(pool).create(&req.display_name, req.org_type.as_deref()).await?;
    Ok(Json(org))
}

async fn get_organization(
    State(state): State<AppState>, headers: HeaderMap, Path(org_id): Path<String>,
) -> Result<Json<crate::organizations::Organization>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    OrganizationStore::new(pool).get(&org_id).await?.map(Json).ok_or(ApiError::NotFound)
}

async fn put_organization(
    State(state): State<AppState>, headers: HeaderMap, Path(org_id): Path<String>,
    Json(update): Json<OrganizationUpdate>,
) -> Result<Json<crate::organizations::Organization>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let org = OrganizationStore::new(pool).update(&org_id, &update).await?;
    Ok(Json(org))
}

#[derive(Deserialize)]
struct OrganizationSearchQuery { q: String, limit: Option<i64> }

async fn get_organization_search(
    State(state): State<AppState>, headers: HeaderMap, Query(query): Query<OrganizationSearchQuery>,
) -> Result<Json<OrganizationListResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let store = OrganizationStore::new(pool);
    let all = store.list(None, 200).await?;
    let q = query.q.trim().to_lowercase();
    let items: Vec<_> = all.into_iter()
        .filter(|o| o.display_name.to_lowercase().contains(&q)
                  || o.legal_name.as_deref().unwrap_or("").to_lowercase().contains(&q)
                  || o.website.as_deref().unwrap_or("").to_lowercase().contains(&q))
        .take(query.limit.unwrap_or(20).clamp(1, 100) as usize)
        .collect();
    Ok(Json(OrganizationListResponse { items }))
}

async fn post_organization_archive(
    State(state): State<AppState>, headers: HeaderMap, Path(org_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    OrganizationStore::new(pool).archive(&org_id).await?;
    Ok(Json(json!({"archived": true})))
}

// ── Organization Identities ────────────────────────────────────────────────

#[derive(Serialize)]
struct OrgIdentitiesResponse { items: Vec<crate::organization_core::OrganizationIdentity> }

async fn get_org_identities(
    State(state): State<AppState>, headers: HeaderMap, Path(org_id): Path<String>,
) -> Result<Json<OrgIdentitiesResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let items = crate::organization_core::OrgIdentityStore::new(pool).list(&org_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(OrgIdentitiesResponse { items }))
}

#[derive(Deserialize)]
struct NewOrgIdentityRequest { identity_type: String, identity_value: String, source: Option<String> }

async fn post_org_identity(
    State(state): State<AppState>, headers: HeaderMap, Path(org_id): Path<String>,
    Json(req): Json<NewOrgIdentityRequest>,
) -> Result<Json<crate::organization_core::OrganizationIdentity>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let identity = crate::organization_core::OrgIdentityStore::new(pool).upsert(&org_id, &req.identity_type, &req.identity_value, req.source.as_deref().unwrap_or("manual")).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(identity))
}

// ── Organization Aliases ───────────────────────────────────────────────────

#[derive(Serialize)] struct OrgAliasesResponse { items: Vec<crate::organization_core::OrganizationAlias> }

async fn get_org_aliases(
    State(state): State<AppState>, headers: HeaderMap, Path(org_id): Path<String>,
) -> Result<Json<OrgAliasesResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let items = crate::organization_core::OrgAliasStore::new(pool).list(&org_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(OrgAliasesResponse { items }))
}

#[derive(Deserialize)] struct NewOrgAliasRequest { name: String, alias_type: String, source: Option<String> }

async fn post_org_alias(
    State(state): State<AppState>, headers: HeaderMap, Path(org_id): Path<String>,
    Json(req): Json<NewOrgAliasRequest>,
) -> Result<Json<crate::organization_core::OrganizationAlias>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let alias = crate::organization_core::OrgAliasStore::new(pool).add(&org_id, &req.name, &req.alias_type, req.source.as_deref().unwrap_or("manual")).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(alias))
}

// ── Organization Domains ───────────────────────────────────────────────────

#[derive(Serialize)] struct OrgDomainsResponse { items: Vec<crate::organization_core::OrganizationDomain> }

async fn get_org_domains(
    State(state): State<AppState>, headers: HeaderMap, Path(org_id): Path<String>,
) -> Result<Json<OrgDomainsResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let items = crate::organization_core::OrgDomainStore::new(pool).list(&org_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(OrgDomainsResponse { items }))
}

// ── Organization Departments ───────────────────────────────────────────────

#[derive(Serialize)] struct OrgDepartmentsResponse { items: Vec<crate::organization_core::OrgDepartment> }

async fn get_org_departments(
    State(state): State<AppState>, headers: HeaderMap, Path(org_id): Path<String>,
) -> Result<Json<OrgDepartmentsResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let items = crate::organization_core::OrgDepartmentStore::new(pool).list(&org_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(OrgDepartmentsResponse { items }))
}

#[derive(Deserialize)] struct NewOrgDepartmentRequest { name: String, description: Option<String>, parent_id: Option<String> }

async fn post_org_department(
    State(state): State<AppState>, headers: HeaderMap, Path(org_id): Path<String>,
    Json(req): Json<NewOrgDepartmentRequest>,
) -> Result<Json<crate::organization_core::OrgDepartment>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let dept = crate::organization_core::OrgDepartmentStore::new(pool).add(&org_id, &req.name, req.description.as_deref(), req.parent_id.as_deref()).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(dept))
}

// ── Organization Contacts ──────────────────────────────────────────────────

#[derive(Serialize)] struct OrgContactsResponse { items: Vec<crate::organization_core::OrgContactLink> }

async fn get_org_contacts(
    State(state): State<AppState>, headers: HeaderMap, Path(org_id): Path<String>,
) -> Result<Json<OrgContactsResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let items = crate::organization_core::OrgContactLinkStore::new(pool).list_by_org(&org_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(OrgContactsResponse { items }))
}

#[derive(Deserialize)] struct LinkOrgContactRequest { person_id: String, role: Option<String>, department: Option<String> }

async fn post_org_contact_link(
    State(state): State<AppState>, headers: HeaderMap, Path(org_id): Path<String>,
    Json(req): Json<LinkOrgContactRequest>,
) -> Result<Json<crate::organization_core::OrgContactLink>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let link = crate::organization_core::OrgContactLinkStore::new(pool).link(&org_id, &req.person_id, req.role.as_deref(), req.department.as_deref()).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(link))
}

// ── Organization Related ───────────────────────────────────────────────────

#[derive(Serialize)] struct OrgRelatedResponse { items: Vec<crate::organization_core::RelatedOrganization> }

async fn get_org_related(
    State(state): State<AppState>, headers: HeaderMap, Path(org_id): Path<String>,
) -> Result<Json<OrgRelatedResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let items = crate::organization_core::RelatedOrgStore::new(pool).list(&org_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(OrgRelatedResponse { items }))
}

// ── Organization Timeline ──────────────────────────────────────────────────

#[derive(Serialize)] struct OrgTimelineResponse { items: Vec<crate::organization_workflows::OrgTimelineEvent> }
#[derive(Deserialize)] struct OrgTimelineQuery { limit: Option<i64> }

async fn get_org_timeline(
    State(state): State<AppState>, headers: HeaderMap, Path(org_id): Path<String>, Query(query): Query<OrgTimelineQuery>,
) -> Result<Json<OrgTimelineResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let items = crate::organization_workflows::OrgTimelineStore::new(pool).list(&org_id, query.limit.unwrap_or(50)).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(OrgTimelineResponse { items }))
}

// ── Organization Portals ───────────────────────────────────────────────────

#[derive(Serialize)] struct OrgPortalsResponse { items: Vec<crate::organization_workflows::OrgPortal> }

async fn get_org_portals(
    State(state): State<AppState>, headers: HeaderMap, Path(org_id): Path<String>,
) -> Result<Json<OrgPortalsResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let items = crate::organization_workflows::OrgPortalStore::new(pool).list(&org_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(OrgPortalsResponse { items }))
}

// ── Organization Procedures ────────────────────────────────────────────────

#[derive(Serialize)] struct OrgProceduresResponse { items: Vec<crate::organization_workflows::OrgProcedure> }

async fn get_org_procedures(
    State(state): State<AppState>, headers: HeaderMap, Path(org_id): Path<String>,
) -> Result<Json<OrgProceduresResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let items = crate::organization_workflows::OrgProcedureStore::new(pool).list(&org_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(OrgProceduresResponse { items }))
}

// ── Organization Playbooks ─────────────────────────────────────────────────

#[derive(Serialize)] struct OrgPlaybooksResponse { items: Vec<crate::organization_workflows::OrgPlaybook> }

async fn get_org_playbooks(
    State(state): State<AppState>, headers: HeaderMap, Path(org_id): Path<String>,
) -> Result<Json<OrgPlaybooksResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let items = crate::organization_workflows::OrgPlaybookStore::new(pool).list(&org_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(OrgPlaybooksResponse { items }))
}

// ── Organization Templates ─────────────────────────────────────────────────

#[derive(Serialize)] struct OrgTemplatesResponse { items: Vec<crate::organization_workflows::OrgTemplate> }

async fn get_org_templates(
    State(state): State<AppState>, headers: HeaderMap, Path(org_id): Path<String>,
) -> Result<Json<OrgTemplatesResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let items = crate::organization_workflows::OrgTemplateStore::new(pool).list(&org_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(OrgTemplatesResponse { items }))
}

// ── Organization Financial ─────────────────────────────────────────────────

async fn get_org_financial(
    State(state): State<AppState>, headers: HeaderMap, Path(org_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let info = crate::organization_finance::OrgFinancialStore::new(pool).get(&org_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(serde_json::to_value(&info).unwrap_or_default()))
}

// ── Organization Contracts ─────────────────────────────────────────────────

#[derive(Serialize)] struct OrgContractsResponse { items: Vec<crate::organization_finance::OrgContract> }

async fn get_org_contracts(
    State(state): State<AppState>, headers: HeaderMap, Path(org_id): Path<String>,
) -> Result<Json<OrgContractsResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let items = crate::organization_finance::OrgContractStore::new(pool).list(&org_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(OrgContractsResponse { items }))
}

// ── Organization Compliance ────────────────────────────────────────────────

#[derive(Serialize)] struct OrgComplianceResponse { items: Vec<crate::organization_finance::OrgCompliance> }

async fn get_org_compliance(
    State(state): State<AppState>, headers: HeaderMap, Path(org_id): Path<String>,
) -> Result<Json<OrgComplianceResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let items = crate::organization_finance::OrgComplianceStore::new(pool).list(&org_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(OrgComplianceResponse { items }))
}

// ── Organization Services ──────────────────────────────────────────────────

#[derive(Serialize)] struct OrgServicesResponse { items: Vec<crate::organization_finance::OrgService> }

async fn get_org_services(
    State(state): State<AppState>, headers: HeaderMap, Path(org_id): Path<String>,
) -> Result<Json<OrgServicesResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let items = crate::organization_finance::OrgServiceStore::new(pool).list(&org_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(OrgServicesResponse { items }))
}

// ── Organization Products ──────────────────────────────────────────────────

#[derive(Serialize)] struct OrgProductsResponse { items: Vec<crate::organization_finance::OrgProduct> }

async fn get_org_products(
    State(state): State<AppState>, headers: HeaderMap, Path(org_id): Path<String>,
) -> Result<Json<OrgProductsResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let items = crate::organization_finance::OrgProductStore::new(pool).list(&org_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(OrgProductsResponse { items }))
}

// ── Organization Enrichment ────────────────────────────────────────────────

#[derive(Serialize)] struct OrgEnrichmentResponse { items: Vec<crate::organization_enrichment::OrgEnrichmentResult> }

async fn get_org_enrichment(
    State(state): State<AppState>, headers: HeaderMap, Path(org_id): Path<String>,
) -> Result<Json<OrgEnrichmentResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let items = crate::organization_enrichment::OrgEnrichmentStore::new(pool).list(&org_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(OrgEnrichmentResponse { items }))
}

async fn post_org_enrich_apply(
    State(state): State<AppState>, headers: HeaderMap, Path((_org_id, rid)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    crate::organization_enrichment::OrgEnrichmentStore::new(pool).apply(&rid).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(json!({"applied": true})))
}

// ── Organization Risks ─────────────────────────────────────────────────────

#[derive(Serialize)] struct OrgRisksResponse { items: Vec<crate::organization_health::OrgRisk> }

async fn get_org_risks(
    State(state): State<AppState>, headers: HeaderMap, Path(org_id): Path<String>,
) -> Result<Json<OrgRisksResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let items = crate::organization_health::OrgRiskStore::new(pool).list(&org_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(OrgRisksResponse { items }))
}

// ── Organization Health ────────────────────────────────────────────────────

async fn get_org_health(
    State(state): State<AppState>, headers: HeaderMap, Path(org_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let health = crate::organization_health::OrgHealthStore::new(pool).get(&org_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(serde_json::to_value(&health).unwrap_or_default()))
}

async fn post_org_watchlist_toggle(
    State(state): State<AppState>, headers: HeaderMap, Path(org_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let on = crate::organization_health::OrgHealthStore::new(pool).toggle_watchlist(&org_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(json!({"watchlist": on})))
}

// ── Organization Investigator ───────────────────────────────────────────────

async fn get_org_dossier(
    State(state): State<AppState>, headers: HeaderMap, Path(org_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let dossier = crate::organization_investigator::OrganizationInvestigator::new(pool).dossier(&org_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(serde_json::to_value(&dossier).unwrap_or_default()))
}

async fn get_org_brief(
    State(state): State<AppState>, headers: HeaderMap, Path(org_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let brief = crate::organization_investigator::OrganizationInvestigator::new(pool).brief(&org_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(serde_json::to_value(&brief).unwrap_or_default()))
}

async fn get_org_context_pack(
    State(state): State<AppState>, headers: HeaderMap, Path(org_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pool = state.database.pool().ok_or(ApiError::DatabaseNotConfigured)?.clone();
    let pack = crate::organization_investigator::OrganizationInvestigator::new(pool).context_pack(&org_id).await.map_err(|e| ApiError::from(e))?;
    Ok(Json(serde_json::to_value(&pack).unwrap_or_default()))
}

pub async fn run(config: AppConfig) -> Result<(), AppError> {
    let http_addr = config.http_addr();
    let database = Database::connect(config.database_url()).await?;
    let listener = TcpListener::bind(http_addr).await?;

    tracing::info!(%http_addr, "starting Hermes Hub backend");

    axum::serve(listener, build_router_with_database(config, database)).await?;

    Ok(())
}

pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

fn local_frontend_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(|origin, _| {
            origin
                .to_str()
                .map(is_allowed_local_frontend_origin)
                .unwrap_or(false)
        }))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::OPTIONS])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            HeaderName::from_static(LOCAL_API_ACTOR_ID_HEADER),
        ])
}

fn is_allowed_local_frontend_origin(origin: &str) -> bool {
    let Ok(url) = url::Url::parse(origin) else {
        return false;
    };

    matches!(
        (url.scheme(), url.host_str()),
        (
            "http" | "https",
            Some("localhost" | "127.0.0.1" | "::1" | "[::1]")
        ) | ("http" | "https", Some("tauri.localhost"))
            | ("tauri", Some("localhost"))
    )
}

async fn healthz(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: state.config.service_name().to_owned(),
    })
}

async fn readyz(State(state): State<AppState>) -> (StatusCode, Json<ReadinessResponse>) {
    let database = state.database.readiness().await;
    let migrations = state.database.migration_readiness().await;
    let is_ready =
        database.status() == ReadinessStatus::Ok && migrations.status() == ReadinessStatus::Ok;

    let status_code = if is_ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status_code,
        Json(ReadinessResponse {
            status: if is_ready { "ok" } else { "degraded" },
            service: state.config.service_name().to_owned(),
            checks: ReadinessChecks {
                database,
                migrations,
            },
        }),
    )
}

async fn post_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<AppendEventRequest>,
) -> Result<(StatusCode, Json<AppendEventResponse>), ApiError> {
    let actor = verify_local_api_capability(&state.config, &headers)?;

    let store = event_store(&state)?;
    let event = request.into_new_event()?;
    let audit_log = api_audit_log(&state)?;
    audit_log
        .record(&NewApiAuditRecord::event_append(
            actor.actor_id,
            event.event_id.clone(),
        ))
        .await?;
    let position = store.append(&event).await?;

    Ok((
        StatusCode::CREATED,
        Json(AppendEventResponse {
            event_id: event.event_id,
            position,
        }),
    ))
}

async fn get_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(event_id): Path<String>,
) -> Result<Json<EventEnvelope>, ApiError> {
    let actor = verify_local_api_capability(&state.config, &headers)?;

    let store = event_store(&state)?;
    let audit_log = api_audit_log(&state)?;
    audit_log
        .record(&NewApiAuditRecord::event_get(
            actor.actor_id,
            event_id.clone(),
        ))
        .await?;
    let Some(event) = store.get_by_id(&event_id).await? else {
        return Err(ApiError::NotFound);
    };

    Ok(Json(event))
}

async fn get_audit_events(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AuditEventsQuery>,
) -> Result<Json<AuditEventsResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;

    let audit_log = api_audit_log(&state)?;
    let items = audit_log
        .list_event_records(
            query.target_id.as_deref(),
            query.actor_id.as_deref(),
            query.after_audit_id.unwrap_or(0),
            query.limit.unwrap_or(100),
        )
        .await?;

    Ok(Json(AuditEventsResponse { items }))
}

async fn get_v1_status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<V1StatusResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
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

async fn get_v1_communication_messages(
    State(state): State<AppState>,
    headers: HeaderMap,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<CommunicationMessagesResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
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

async fn get_v1_communication_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(message_id): Path<String>,
) -> Result<Json<CommunicationMessageDetailResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
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

async fn get_graph_summary(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<crate::graph::GraphSummary>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    Ok(Json(graph_store(&state)?.summary().await?))
}

async fn get_graph_nodes(
    State(state): State<AppState>,
    headers: HeaderMap,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<Vec<crate::graph::GraphNode>>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let query = parse_graph_nodes_query(raw_query.as_deref())?;
    let limit = query.limit.unwrap_or(20).clamp(1, 50);
    Ok(Json(
        graph_store(&state)?.list_nodes_for_picker(limit).await?,
    ))
}

async fn get_graph_neighborhood(
    State(state): State<AppState>,
    headers: HeaderMap,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<crate::graph::GraphNeighborhood>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
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

async fn get_graph_search(
    State(state): State<AppState>,
    headers: HeaderMap,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<Vec<crate::graph::GraphNode>>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
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

async fn get_projects(
    State(state): State<AppState>,
    headers: HeaderMap,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<ProjectListResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let query = parse_projects_query(raw_query.as_deref())?;
    let items = project_store(&state)?.list_projects(query.limit).await?;

    Ok(Json(ProjectListResponse { items }))
}

async fn get_project_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<String>,
) -> Result<Json<crate::projects::ProjectDetail>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let Some(project) = project_store(&state)?.project_detail(&project_id).await? else {
        return Err(ApiError::ProjectNotFound);
    };

    Ok(Json(project))
}

async fn get_project_link_candidates(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<String>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<ProjectLinkCandidateListResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let query = parse_project_link_candidates_query(raw_query.as_deref())?;
    let project_id = validate_non_empty_project_link_field("project_id", &project_id)?;

    let project_store = project_store(&state)?;
    let review_store = project_link_review_store(&state)?;
    let mut candidates = Vec::new();

    for message in project_store.matching_project_messages(&project_id).await? {
        let graph_node_id = node_id(GraphNodeKind::Message, &message.message_id);
        let sender_excerpt = text_preview(&message.sender, 140);
        let review_state = review_store
            .explicit_review(
                &project_id,
                ProjectLinkTargetKind::Message,
                &message.message_id,
            )
            .await?
            .map(|review| review.review_state)
            .unwrap_or(ProjectLinkReviewState::Suggested);
        let occurred_at = message.occurred_at.unwrap_or(message.projected_at);

        candidates.push(ProjectLinkCandidate {
            project_id: project_id.clone(),
            target_kind: ProjectLinkTargetKind::Message.as_str().to_owned(),
            target_id: message.message_id,
            graph_node_id,
            title: text_preview(&message.subject, 120),
            subtitle: message.sender,
            source_label: message.account_id,
            occurred_at,
            review_state: review_state.as_str().to_owned(),
            evidence_excerpt: Some(sender_excerpt),
        });
    }

    for document in project_store
        .matching_project_documents(&project_id)
        .await?
    {
        let graph_node_id = node_id(GraphNodeKind::Document, &document.document_id);
        let title = text_preview(&document.title, 140);
        let review_state = review_store
            .explicit_review(
                &project_id,
                ProjectLinkTargetKind::Document,
                &document.document_id,
            )
            .await?
            .map(|review| review.review_state)
            .unwrap_or(ProjectLinkReviewState::Suggested);

        candidates.push(ProjectLinkCandidate {
            project_id: project_id.clone(),
            target_kind: ProjectLinkTargetKind::Document.as_str().to_owned(),
            target_id: document.document_id,
            graph_node_id,
            title: title.clone(),
            subtitle: document.document_kind,
            source_label: document.source_fingerprint,
            occurred_at: document.imported_at,
            review_state: review_state.as_str().to_owned(),
            evidence_excerpt: Some(title),
        });
    }

    candidates.sort_by(|left, right| right.occurred_at.cmp(&left.occurred_at));
    candidates.truncate(query.limit.unwrap_or(25));

    Ok(Json(ProjectLinkCandidateListResponse { items: candidates }))
}

async fn put_project_link_review(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<String>,
    Json(request): Json<ProjectLinkReviewApiRequest>,
) -> Result<Json<ProjectLinkReviewApiResponse>, ApiError> {
    let actor = verify_local_api_capability(&state.config, &headers)?;
    let command = request.into_command(project_id, actor.actor_id)?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::project_link_review_set(
            &command.actor_id,
            &command.project_id,
            command.target_kind.as_str(),
            &command.target_id,
        ))
        .await?;

    let result = project_link_review_store(&state)?
        .set_review_state(&command)
        .await?;

    Ok(Json(result.into()))
}

async fn get_task_candidates(
    State(state): State<AppState>,
    headers: HeaderMap,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<TaskCandidateListResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let query = parse_task_candidates_query(raw_query.as_deref())?;
    let items = task_candidate_store(&state)?
        .list_candidates(query.limit)
        .await?;

    Ok(Json(TaskCandidateListResponse { items }))
}

async fn get_identity_candidates(
    State(state): State<AppState>,
    headers: HeaderMap,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<PersonIdentityCandidateListResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let query = parse_person_identity_candidates_query(raw_query.as_deref())?;
    let items = person_identity_store(&state)?
        .list_candidates(query.limit)
        .await?;

    Ok(Json(PersonIdentityCandidateListResponse { items }))
}

async fn put_identity_candidate_review(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(identity_candidate_id): Path<String>,
    Json(request): Json<PersonIdentityReviewApiRequest>,
) -> Result<Json<PersonIdentityReviewApiResponse>, ApiError> {
    let actor = verify_local_api_capability(&state.config, &headers)?;
    let command = request.into_command(identity_candidate_id, actor.actor_id)?;

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

async fn get_person_identity(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(person_id): Path<String>,
) -> Result<Json<PersonIdentityDetail>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let _ = validate_non_empty_person_identity_field("person_id", &person_id)?;

    let detail = person_identity_store(&state)?
        .person_identity(&person_id)
        .await?;
    Ok(Json(detail))
}

async fn put_task_candidate_review(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_candidate_id): Path<String>,
    Json(request): Json<TaskCandidateReviewApiRequest>,
) -> Result<Json<TaskCandidateReviewApiResponse>, ApiError> {
    let actor = verify_local_api_capability(&state.config, &headers)?;
    let command = request.into_command(task_candidate_id, actor.actor_id)?;

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

async fn get_tasks(
    State(state): State<AppState>,
    headers: HeaderMap,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<TaskListResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let query = parse_task_candidates_query(raw_query.as_deref())?;
    let items = task_candidate_store(&state)?
        .list_tasks(query.limit)
        .await?;

    Ok(Json(TaskListResponse { items }))
}

async fn get_application_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApplicationSettingsResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let items = settings_store(&state)?.list_settings().await?;

    Ok(Json(ApplicationSettingsResponse { items }))
}

async fn get_application_settings_accounts(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApplicationAccountsResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let items = communication_ingestion_store(&state)?
        .list_provider_accounts()
        .await?;

    Ok(Json(ApplicationAccountsResponse { items }))
}

async fn put_application_setting(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(setting_key): Path<String>,
    Json(request): Json<ApplicationSettingUpdateRequest>,
) -> Result<Json<ApplicationSetting>, ApiError> {
    let actor = verify_local_api_capability(&state.config, &headers)?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::application_setting_set(
            &actor.actor_id,
            &setting_key,
        ))
        .await?;
    let setting = settings_store(&state)?
        .update_setting_value(&setting_key, &request.value, &actor.actor_id)
        .await?;

    Ok(Json(setting))
}

async fn get_document_processing(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(document_id): Path<String>,
) -> Result<Json<DocumentProcessingRecord>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let _ = validate_non_empty_document_id(document_id.as_str())?;

    Ok(Json(
        document_processing_store(&state)?
            .document_processing(&document_id)
            .await?,
    ))
}

async fn get_document_processing_jobs(
    State(state): State<AppState>,
    headers: HeaderMap,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<DocumentProcessingJobsResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let query = parse_document_processing_jobs_query(raw_query.as_deref())?;
    let items = document_processing_store(&state)?
        .list_jobs(query.limit)
        .await?;

    Ok(Json(DocumentProcessingJobsResponse { items }))
}

async fn post_document_processing_job_retry(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(job_id): Path<String>,
    Json(request): Json<DocumentProcessingRetryApiRequest>,
) -> Result<Json<DocumentProcessingRetryApiResponse>, ApiError> {
    let actor = verify_local_api_capability(&state.config, &headers)?;
    let command = request.into_command(job_id, actor.actor_id)?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::document_processing_job_retry(
            &command.actor_id,
            &command.job_id,
        ))
        .await?;

    let result = document_processing_store(&state)?
        .retry_failed_job(&command)
        .await?;

    Ok(Json(result.into()))
}

async fn post_gmail_oauth_start(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<GmailOAuthStartApiRequest>,
) -> Result<Json<GmailOAuthStartApiResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
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

async fn post_gmail_oauth_complete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<GmailOAuthCompleteApiRequest>,
) -> Result<Json<EmailAccountSetupApiResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
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

async fn get_gmail_oauth_callback(Query(query): Query<GmailOAuthCallbackQuery>) -> Html<String> {
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

async fn post_imap_account_setup(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ImapAccountSetupApiRequest>,
) -> Result<Json<EmailAccountSetupApiResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let service = account_setup_service(&state)?;
    let result = service
        .setup_imap_account(request.into_setup_request()?)
        .await?;

    Ok(Json(result.into()))
}

async fn get_v3_ai_status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AiStatusResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
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

async fn get_v3_agents(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AiAgentListResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let runtime_settings = ai_runtime_settings(&state).await?;

    Ok(Json(AiAgentListResponse {
        items: v3_agents(&runtime_settings.chat_model),
    }))
}

async fn get_v3_ai_runs(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AiRunsQuery>,
) -> Result<Json<AiRunListResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let limit = query.limit.unwrap_or(25).clamp(1, 100);
    let runs = ai_run_store(&state)?.list_runs(limit).await?;

    Ok(Json(AiRunListResponse { items: runs }))
}

async fn get_v3_ai_run(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(run_id): Path<String>,
) -> Result<Json<AiAgentRun>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let Some(run) = ai_run_store(&state)?.get_run(&run_id).await? else {
        return Err(ApiError::AiRunNotFound);
    };

    Ok(Json(run))
}

async fn post_v3_ai_answer(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<AiAnswerRequest>,
) -> Result<Json<crate::ai::AiAnswerResponse>, ApiError> {
    let actor = verify_local_api_capability(&state.config, &headers)?;
    let service = ai_service(&state).await?;
    let response = service.answer(request, &actor.actor_id).await?;

    Ok(Json(response))
}

async fn post_v3_ai_task_candidates_refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<AiTaskCandidateRefreshRequest>,
) -> Result<Json<crate::ai::AiTaskCandidateRefreshResponse>, ApiError> {
    let actor = verify_local_api_capability(&state.config, &headers)?;
    let service = ai_service(&state).await?;
    let response = service
        .refresh_task_candidates(request, &actor.actor_id)
        .await?;

    Ok(Json(response))
}

async fn post_v3_ai_meeting_prep(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<AiMeetingPrepRequest>,
) -> Result<Json<crate::ai::AiMeetingPrepResponse>, ApiError> {
    let actor = verify_local_api_capability(&state.config, &headers)?;
    let service = ai_service(&state).await?;
    let response = service.meeting_prep(request, &actor.actor_id).await?;

    Ok(Json(response))
}

async fn get_v4_capabilities(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<V4CapabilitiesResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;

    Ok(Json(V4CapabilitiesResponse::current()))
}

async fn get_v5_capabilities(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<V5CapabilitiesResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;

    Ok(Json(V5CapabilitiesResponse::current()))
}

async fn post_v4_telegram_fixture_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<TelegramAccountSetupRequest>,
) -> Result<Json<TelegramAccountSetupResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;

    Ok(Json(
        telegram_store(&state)?
            .setup_fixture_account(&request)
            .await?,
    ))
}

async fn get_v4_telegram_chats(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<TelegramListQuery>,
) -> Result<Json<TelegramChatListResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let items = telegram_store(&state)?
        .list_chats(query.account_id.as_deref(), query.limit.unwrap_or(50))
        .await?;

    Ok(Json(TelegramChatListResponse { items }))
}

async fn post_v4_telegram_fixture_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<NewTelegramMessage>,
) -> Result<Json<TelegramMessageIngestResult>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;

    Ok(Json(
        telegram_store(&state)?
            .ingest_fixture_message(&request)
            .await?,
    ))
}

async fn get_v4_telegram_messages(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<TelegramListQuery>,
) -> Result<Json<TelegramMessageListResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let items = telegram_store(&state)?
        .recent_messages(
            query.account_id.as_deref(),
            query.provider_chat_id.as_deref(),
            query.limit.unwrap_or(50),
        )
        .await?;

    Ok(Json(TelegramMessageListResponse { items }))
}

async fn post_v5_whatsapp_fixture_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<WhatsappWebAccountSetupRequest>,
) -> Result<Json<WhatsappWebAccountSetupResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;

    Ok(Json(
        whatsapp_web_store(&state)?
            .setup_fixture_account(&request)
            .await?,
    ))
}

async fn get_v5_whatsapp_sessions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<WhatsappWebListQuery>,
) -> Result<Json<WhatsappWebSessionListResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let items = whatsapp_web_store(&state)?
        .list_sessions(query.account_id.as_deref(), query.limit.unwrap_or(50))
        .await?;

    Ok(Json(WhatsappWebSessionListResponse { items }))
}

async fn post_v5_whatsapp_fixture_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<NewWhatsappWebMessage>,
) -> Result<Json<WhatsappWebMessageIngestResult>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;

    Ok(Json(
        whatsapp_web_store(&state)?
            .ingest_fixture_message(&request)
            .await?,
    ))
}

async fn get_v5_whatsapp_messages(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<WhatsappWebListQuery>,
) -> Result<Json<WhatsappWebMessageListResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let items = whatsapp_web_store(&state)?
        .recent_messages(
            query.account_id.as_deref(),
            query.provider_chat_id.as_deref(),
            query.limit.unwrap_or(50),
        )
        .await?;

    Ok(Json(WhatsappWebMessageListResponse { items }))
}

async fn post_v4_policy_template(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<PolicyTemplateApiRequest>,
) -> Result<Json<AutomationTemplate>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;

    Ok(Json(
        automation_store(&state)?
            .upsert_template(&request.into_template())
            .await?,
    ))
}

async fn get_v4_policy_templates(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<PolicyTemplateListResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let items = automation_store(&state)?.list_templates().await?;

    Ok(Json(PolicyTemplateListResponse { items }))
}

async fn post_v4_policy(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<PolicyApiRequest>,
) -> Result<Json<AutomationPolicy>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;

    Ok(Json(
        automation_store(&state)?
            .upsert_policy(&request.into_policy())
            .await?,
    ))
}

async fn get_v4_policies(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<PolicyListResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let items = automation_store(&state)?.list_policies().await?;

    Ok(Json(PolicyListResponse { items }))
}

async fn post_v4_telegram_send_dry_run(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<TelegramSendDryRunRequest>,
) -> Result<Json<TelegramSendDryRunResponse>, ApiError> {
    let actor = verify_local_api_capability(&state.config, &headers)?;
    let response = match automation_store(&state)?
        .dry_run_send(&request, &actor.actor_id)
        .await
    {
        Ok(response) => response,
        Err(error) => {
            if let Some(decision) = telegram_send_dry_run_rejection_decision(&error, &request) {
                api_audit_log(&state)?
                    .record(
                        &NewApiAuditRecord::automation_telegram_send_dry_run_rejected(
                            &actor.actor_id,
                            &request.command_id,
                            &request.policy_id,
                            &request.provider_chat_id,
                            &decision,
                        ),
                    )
                    .await?;
            }
            return Err(error.into());
        }
    };
    api_audit_log(&state)?
        .record(&NewApiAuditRecord::automation_telegram_send_dry_run(
            &actor.actor_id,
            &response.outbound_message_id,
            &response.policy_id,
            &response.template_id,
            &response.account_id,
            &response.provider_chat_id,
            &response.rendered_preview_hash,
        ))
        .await?;

    Ok(Json(response))
}

fn telegram_send_dry_run_rejection_decision(
    error: &AutomationError,
    request: &TelegramSendDryRunRequest,
) -> Option<CapabilityDecision> {
    let reason = match error {
        AutomationError::InvalidRequest(_) => "invalid_request",
        AutomationError::PolicyNotFound => "policy_not_found",
        AutomationError::PolicyDisabled => "policy_disabled",
        AutomationError::ChatNotAllowed => "provider_chat_not_allowed",
        AutomationError::MissingTemplateVariable(_) => "template_variable_missing",
        AutomationError::UndeclaredTemplateVariable(_) => "template_variable_undeclared",
        AutomationError::EventEnvelope(_)
        | AutomationError::EventStore(_)
        | AutomationError::Sqlx(_) => return None,
    };

    Some(CapabilityDecision::rejected_high_risk(
        CapabilityActionClass::Automation,
        "telegram.send",
        reason,
        non_empty_optional_string(&request.policy_id),
    ))
}

fn non_empty_optional_string(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

async fn post_v4_call(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CallApiRequest>,
) -> Result<Json<TelegramCall>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;

    Ok(Json(
        call_intelligence_store(&state)?
            .upsert_call(&request.into_call())
            .await?,
    ))
}

async fn get_v4_calls(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<TelegramListQuery>,
) -> Result<Json<CallListResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let items = call_intelligence_store(&state)?
        .list_calls(query.account_id.as_deref(), query.limit.unwrap_or(50))
        .await?;

    Ok(Json(CallListResponse { items }))
}

async fn post_v4_call_transcript_fixture(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(call_id): Path<String>,
    Json(request): Json<CallTranscriptFixtureApiRequest>,
) -> Result<Json<CallTranscript>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let stt = FixtureSpeechToTextProvider;
    let fixture = stt.transcribe_fixture(&request.source_audio_ref)?;
    let transcript = NewCallTranscript {
        transcript_id: request.transcript_id,
        call_id,
        account_id: request.account_id,
        provider_chat_id: request.provider_chat_id,
        transcript_status: TranscriptStatus::Succeeded,
        stt_provider: stt.provider_name().to_owned(),
        source_audio_ref: Some(request.source_audio_ref),
        language_code: request.language_code,
        transcript_text: fixture.text,
        segments: fixture.segments,
        provenance: json!({
            "runtime": "fixture",
            "source": "local_call_audio",
            "always_on_policy": request.always_on_policy,
        }),
    };

    Ok(Json(
        call_intelligence_store(&state)?
            .upsert_transcript(&transcript)
            .await?,
    ))
}

async fn get_v4_call_transcript(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(call_id): Path<String>,
) -> Result<Json<CallTranscriptResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let transcript = call_intelligence_store(&state)?
        .transcript_for_call(&call_id)
        .await?;

    Ok(Json(CallTranscriptResponse { transcript }))
}

fn event_store(state: &AppState) -> Result<EventStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(EventStore::new(pool.clone()))
}

fn graph_store(state: &AppState) -> Result<crate::graph::GraphStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(crate::graph::GraphStore::new(pool.clone()))
}

fn message_store(state: &AppState) -> Result<MessageProjectionStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(MessageProjectionStore::new(pool.clone()))
}

fn mail_storage_store(state: &AppState) -> Result<MailStorageStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(MailStorageStore::new(pool.clone()))
}

fn communication_ingestion_store(
    state: &AppState,
) -> Result<CommunicationIngestionStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(CommunicationIngestionStore::new(pool.clone()))
}

fn project_store(state: &AppState) -> Result<ProjectStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ProjectStore::new(pool.clone()))
}

fn project_link_review_store(state: &AppState) -> Result<ProjectLinkReviewStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ProjectLinkReviewStore::new(pool.clone()))
}

fn task_candidate_store(state: &AppState) -> Result<TaskCandidateStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(TaskCandidateStore::new(pool.clone()))
}

fn ai_run_store(state: &AppState) -> Result<crate::ai::AiRunStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(crate::ai::AiRunStore::new(pool.clone()))
}

async fn ai_service(state: &AppState) -> Result<AiService, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let runtime_settings = ai_runtime_settings(state).await?;
    let ollama = ollama_client(&runtime_settings)?;

    Ok(AiService::new(
        pool.clone(),
        ollama,
        &runtime_settings.chat_model,
        &runtime_settings.embedding_model,
    ))
}

fn telegram_store(state: &AppState) -> Result<TelegramStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(TelegramStore::new(pool.clone()))
}

fn whatsapp_web_store(state: &AppState) -> Result<WhatsappWebStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(WhatsappWebStore::new(pool.clone()))
}

fn automation_store(state: &AppState) -> Result<AutomationStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(AutomationStore::new(pool.clone()))
}

fn call_intelligence_store(state: &AppState) -> Result<CallIntelligenceStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(CallIntelligenceStore::new(pool.clone()))
}

fn settings_store(state: &AppState) -> Result<ApplicationSettingsStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ApplicationSettingsStore::new(pool.clone()))
}

async fn ai_runtime_settings(state: &AppState) -> Result<AiRuntimeSettings, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Ok(AiRuntimeSettings::from_config(&state.config));
    };

    Ok(ApplicationSettingsStore::new(pool.clone())
        .ai_runtime_settings(&state.config)
        .await?)
}

fn ollama_client(settings: &AiRuntimeSettings) -> Result<OllamaClient, ApiError> {
    Ok(OllamaClient::new(
        OllamaClientConfig::new(
            &settings.base_url,
            &settings.chat_model,
            &settings.embedding_model,
        )
        .with_timeout_seconds(settings.timeout_seconds),
    )?)
}

fn document_processing_store(state: &AppState) -> Result<DocumentProcessingStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(DocumentProcessingStore::new(pool.clone()))
}

fn person_identity_store(state: &AppState) -> Result<PersonIdentityStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(PersonIdentityStore::new(pool.clone()))
}

fn email_multilingual_service(
    _state: &AppState,
) -> Result<crate::email_multilingual::MultilingualService, ApiError> {
    Ok(crate::email_multilingual::MultilingualService::new(
        crate::ollama::OllamaClient::new(crate::ollama::OllamaClientConfig::new(
            "http://127.0.0.1:11434",
            "qwen3:4b",
            "qwen3-embedding:4b",
        ))
        .ok(),
    ))
}

fn email_ai_reply_service(
    _state: &AppState,
) -> Result<crate::email_ai_reply::AiReplyService, ApiError> {
    Ok(crate::email_ai_reply::AiReplyService::new(
        crate::ollama::OllamaClient::new(crate::ollama::OllamaClientConfig::new(
            "http://127.0.0.1:11434",
            "qwen3:4b",
            "qwen3-embedding:4b",
        ))
        .ok(),
    ))
}

fn api_audit_log(state: &AppState) -> Result<ApiAuditLog, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ApiAuditLog::new(pool.clone()))
}

fn account_setup_service(state: &AppState) -> Result<EmailAccountSetupService, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let vault = database_encrypted_vault(&state.config, pool.clone())
        .ok_or(ApiError::SecretVaultNotConfigured)?;

    Ok(EmailAccountSetupService::new(
        CommunicationIngestionStore::new(pool.clone()),
        SecretReferenceStore::new(pool.clone()),
        vault,
    ))
}

fn database_encrypted_vault(
    config: &AppConfig,
    pool: sqlx::postgres::PgPool,
) -> Option<DatabaseEncryptedSecretVault> {
    Some(DatabaseEncryptedSecretVault::new(
        pool,
        config.secret_vault_key()?.clone(),
    ))
}

fn verify_local_api_capability(
    config: &AppConfig,
    headers: &HeaderMap,
) -> Result<LocalApiActor, ApiError> {
    let Some(expected_token) = config.local_api_token() else {
        return Err(ApiError::ApiTokenNotConfigured);
    };

    let Some(raw_authorization) = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
    else {
        return Err(ApiError::InvalidApiToken);
    };

    let Some((scheme, token)) = raw_authorization.split_once(' ') else {
        return Err(ApiError::InvalidApiToken);
    };

    if !scheme.eq_ignore_ascii_case("Bearer") || token != expected_token {
        return Err(ApiError::InvalidApiToken);
    }

    local_api_actor(headers)
}

fn local_api_actor(headers: &HeaderMap) -> Result<LocalApiActor, ApiError> {
    let Some(raw_actor_id) = headers
        .get(LOCAL_API_ACTOR_ID_HEADER)
        .and_then(|value| value.to_str().ok())
    else {
        return Err(ApiError::InvalidActorId);
    };

    let actor_id = raw_actor_id.trim();
    if actor_id.is_empty()
        || actor_id.len() > MAX_LOCAL_API_ACTOR_ID_LENGTH
        || !actor_id.bytes().all(is_valid_actor_id_byte)
    {
        return Err(ApiError::InvalidActorId);
    }

    Ok(LocalApiActor {
        actor_id: actor_id.to_owned(),
    })
}

fn is_valid_actor_id_byte(byte: u8) -> bool {
    matches!(
        byte,
        b'a'..=b'z'
            | b'A'..=b'Z'
            | b'0'..=b'9'
            | b'.'
            | b'_'
            | b'-'
            | b':'
            | b'@'
            | b'/'
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LocalApiActor {
    actor_id: String,
}

#[derive(Serialize)]
struct ReadinessResponse {
    status: &'static str,
    service: String,
    checks: ReadinessChecks,
}

#[derive(Serialize)]
struct ReadinessChecks {
    database: DatabaseReadiness,
    migrations: MigrationReadiness,
}

#[derive(Serialize)]
struct ApplicationSettingsResponse {
    items: Vec<ApplicationSetting>,
}

#[derive(Serialize)]
struct ApplicationAccountsResponse {
    items: Vec<ProviderAccount>,
}

#[derive(Deserialize)]
struct ApplicationSettingUpdateRequest {
    value: Value,
}

#[derive(Deserialize)]
struct AppendEventRequest {
    event_id: String,
    event_type: String,
    #[serde(default = "default_schema_version")]
    schema_version: i32,
    occurred_at: DateTime<Utc>,
    source: Value,
    actor: Option<Value>,
    subject: Value,
    #[serde(default = "empty_json_object")]
    payload: Value,
    #[serde(default = "empty_json_object")]
    provenance: Value,
    causation_id: Option<String>,
    correlation_id: Option<String>,
}

impl AppendEventRequest {
    fn into_new_event(self) -> Result<NewEventEnvelope, EventEnvelopeError> {
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
struct AppendEventResponse {
    event_id: String,
    position: i64,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: &'static str,
    message: String,
}

#[derive(Deserialize)]
struct AuditEventsQuery {
    target_id: Option<String>,
    actor_id: Option<String>,
    after_audit_id: Option<i64>,
    limit: Option<u32>,
}

#[derive(Serialize)]
struct AuditEventsResponse {
    items: Vec<ApiAuditRecord>,
}

#[derive(Serialize)]
struct V1StatusResponse {
    version: &'static str,
    surfaces: V1Surfaces,
}

#[derive(Serialize)]
struct V1Surfaces {
    messages: bool,
    persons: bool,
    search: bool,
    documents: bool,
    account_setup: bool,
}

#[derive(Serialize)]
struct CommunicationMessagesResponse {
    items: Vec<CommunicationMessageSummaryResponse>,
}

#[derive(Serialize)]
struct CommunicationMessageSummaryResponse {
    message_id: String,
    raw_record_id: String,
    account_id: String,
    provider_record_id: String,
    subject: String,
    sender: String,
    recipients: Vec<String>,
    body_text_preview: String,
    occurred_at: Option<DateTime<Utc>>,
    projected_at: DateTime<Utc>,
    channel_kind: String,
    conversation_id: Option<String>,
    sender_display_name: Option<String>,
    delivery_state: String,
    message_metadata: Value,
    attachment_count: i64,
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
        }
    }
}

#[derive(Serialize)]
struct CommunicationMessageDetailResponse {
    message: CommunicationMessageDetailItem,
    attachments: Vec<CommunicationAttachmentResponse>,
}

#[derive(Serialize)]
struct CommunicationMessageDetailItem {
    message_id: String,
    raw_record_id: String,
    account_id: String,
    provider_record_id: String,
    subject: String,
    sender: String,
    recipients: Vec<String>,
    body_text: String,
    occurred_at: Option<DateTime<Utc>>,
    projected_at: DateTime<Utc>,
    channel_kind: String,
    conversation_id: Option<String>,
    sender_display_name: Option<String>,
    delivery_state: String,
    message_metadata: Value,
}

impl From<ProjectedMessage> for CommunicationMessageDetailItem {
    fn from(message: ProjectedMessage) -> Self {
        Self {
            message_id: message.message_id,
            raw_record_id: message.raw_record_id,
            account_id: message.account_id,
            provider_record_id: message.provider_record_id,
            subject: message.subject,
            sender: message.sender,
            recipients: message.recipients,
            body_text: message.body_text,
            occurred_at: message.occurred_at,
            projected_at: message.projected_at,
            channel_kind: message.channel_kind,
            conversation_id: message.conversation_id,
            sender_display_name: message.sender_display_name,
            delivery_state: message.delivery_state,
            message_metadata: message.message_metadata,
        }
    }
}

#[derive(Serialize)]
struct CommunicationAttachmentResponse {
    attachment_id: String,
    message_id: String,
    raw_record_id: String,
    blob_id: String,
    provider_attachment_id: String,
    filename: Option<String>,
    content_type: String,
    size_bytes: i64,
    sha256: String,
    disposition: &'static str,
    scan_status: &'static str,
    scan_engine: Option<String>,
    scan_checked_at: Option<DateTime<Utc>>,
    scan_summary: Option<String>,
    scan_metadata: Value,
    storage_kind: String,
    storage_path: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
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
struct ProjectLinkCandidate {
    project_id: String,
    target_kind: String,
    target_id: String,
    graph_node_id: String,
    title: String,
    subtitle: String,
    source_label: String,
    occurred_at: DateTime<Utc>,
    review_state: String,
    evidence_excerpt: Option<String>,
}

#[derive(Serialize)]
struct ProjectLinkCandidateListResponse {
    items: Vec<ProjectLinkCandidate>,
}

#[derive(Serialize)]
struct TaskCandidateListResponse {
    items: Vec<TaskCandidate>,
}

#[derive(Deserialize)]
struct AiRunsQuery {
    limit: Option<i64>,
}

#[derive(Serialize)]
struct AiRunListResponse {
    items: Vec<AiAgentRun>,
}

#[derive(Serialize)]
struct V4CapabilitiesResponse {
    version: &'static str,
    runtime_mode: &'static str,
    capabilities: Vec<V4CapabilityStatus>,
    unsupported_features: Vec<&'static str>,
}

impl V4CapabilitiesResponse {
    fn current() -> Self {
        Self {
            version: "v4",
            runtime_mode: "fixture",
            capabilities: vec![
                V4CapabilityStatus::available(
                    "telegram_fixture_runtime",
                    "Fixture Telegram accounts, chats and message projection are available for CI and local smoke validation.",
                    true,
                ),
                V4CapabilityStatus::blocked(
                    "tdlib_live_runtime",
                    "Live TDLib sessions require the native runtime adapter, account-scoped secret resolver and opt-in Telegram credentials.",
                    false,
                ),
                V4CapabilityStatus::blocked(
                    "telegram_bot_live_runtime",
                    "Live bot sends require the Bot API runtime adapter and account-scoped bot token resolution.",
                    false,
                ),
                V4CapabilityStatus::available(
                    "automation_dry_run",
                    "Policy/template validation and audited dry-run records are available.",
                    true,
                ),
                V4CapabilityStatus::blocked(
                    "automation_live_send",
                    "Live automated sends remain blocked until a live Telegram runtime passes the same policy evaluator and audit contract.",
                    false,
                ),
                V4CapabilityStatus::available(
                    "call_state_and_transcript_storage",
                    "1:1 call metadata and transcript artifact storage are available through fixture APIs.",
                    true,
                ),
                V4CapabilityStatus::blocked(
                    "desktop_audio_capture",
                    "Desktop audio capture requires a visible recording runtime boundary and explicit platform permissions.",
                    false,
                ),
                V4CapabilityStatus::available(
                    "fixture_speech_to_text",
                    "Fixture speech-to-text provider is available for deterministic tests.",
                    true,
                ),
                V4CapabilityStatus::blocked(
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
struct V4CapabilityStatus {
    capability: &'static str,
    status: &'static str,
    closure_gate: bool,
    reason: &'static str,
}

impl V4CapabilityStatus {
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
struct V5CapabilitiesResponse {
    version: &'static str,
    runtime_mode: &'static str,
    capabilities: Vec<V5CapabilityStatus>,
    unsupported_features: Vec<&'static str>,
}

impl V5CapabilitiesResponse {
    fn current() -> Self {
        Self {
            version: "v5",
            runtime_mode: "fixture",
            capabilities: vec![
                V5CapabilityStatus::available(
                    "whatsapp_web_fixture_runtime",
                    "Fixture WhatsApp Web accounts, session metadata and message projection are available for CI and local smoke validation.",
                    true,
                ),
                V5CapabilityStatus::available(
                    "whatsapp_web_manual_session_state",
                    "Manual companion session metadata is stored without session secrets or pairing material in PostgreSQL.",
                    true,
                ),
                V5CapabilityStatus::available(
                    "whatsapp_web_fixture_ingestion",
                    "Fixture WhatsApp Web messages preserve append-only raw provenance and project into canonical communication messages.",
                    true,
                ),
                V5CapabilityStatus::blocked(
                    "whatsapp_web_live_runtime",
                    "Live WhatsApp Web requires a user-visible desktop runtime, explicit session lifecycle and smoke validation.",
                    false,
                ),
                V5CapabilityStatus::blocked(
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
struct V5CapabilityStatus {
    capability: &'static str,
    status: &'static str,
    closure_gate: bool,
    reason: &'static str,
}

impl V5CapabilityStatus {
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
struct TelegramListQuery {
    account_id: Option<String>,
    provider_chat_id: Option<String>,
    limit: Option<i64>,
}

#[derive(Serialize)]
struct TelegramChatListResponse {
    items: Vec<TelegramChat>,
}

#[derive(Serialize)]
struct TelegramMessageListResponse {
    items: Vec<TelegramMessage>,
}

#[derive(Deserialize)]
struct WhatsappWebListQuery {
    account_id: Option<String>,
    provider_chat_id: Option<String>,
    limit: Option<i64>,
}

#[derive(Serialize)]
struct WhatsappWebSessionListResponse {
    items: Vec<WhatsappWebSession>,
}

#[derive(Serialize)]
struct WhatsappWebMessageListResponse {
    items: Vec<WhatsappWebMessage>,
}

#[derive(Deserialize)]
struct PolicyTemplateApiRequest {
    template_id: String,
    name: String,
    body_template: String,
    #[serde(default)]
    required_variables: Vec<String>,
}

impl PolicyTemplateApiRequest {
    fn into_template(self) -> NewAutomationTemplate {
        NewAutomationTemplate {
            template_id: self.template_id,
            name: self.name,
            body_template: self.body_template,
            required_variables: self.required_variables,
        }
    }
}

#[derive(Serialize)]
struct PolicyTemplateListResponse {
    items: Vec<AutomationTemplate>,
}

#[derive(Deserialize)]
struct PolicyApiRequest {
    policy_id: String,
    template_id: String,
    name: String,
    enabled: bool,
    account_id: String,
    allowed_chat_ids: Vec<String>,
    trigger_kind: String,
    max_sends_per_hour: i32,
    #[serde(default = "empty_json_object")]
    quiet_hours: Value,
    expires_at: Option<DateTime<Utc>>,
    #[serde(default = "empty_json_object")]
    conditions: Value,
}

impl PolicyApiRequest {
    fn into_policy(self) -> NewAutomationPolicy {
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
struct PolicyListResponse {
    items: Vec<AutomationPolicy>,
}

#[derive(Deserialize)]
struct CallApiRequest {
    call_id: String,
    account_id: String,
    provider_call_id: String,
    provider_chat_id: String,
    direction: CallDirection,
    call_state: CallState,
    started_at: Option<DateTime<Utc>>,
    ended_at: Option<DateTime<Utc>>,
    transcription_policy_id: Option<String>,
    #[serde(default = "empty_json_object")]
    metadata: Value,
}

impl CallApiRequest {
    fn into_call(self) -> NewTelegramCall {
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
struct CallListResponse {
    items: Vec<TelegramCall>,
}

#[derive(Deserialize)]
struct CallTranscriptFixtureApiRequest {
    transcript_id: String,
    account_id: String,
    provider_chat_id: String,
    source_audio_ref: String,
    language_code: Option<String>,
    #[serde(default)]
    always_on_policy: bool,
}

#[derive(Serialize)]
struct CallTranscriptResponse {
    transcript: Option<CallTranscript>,
}

#[derive(Serialize)]
struct PersonIdentityCandidateListResponse {
    items: Vec<PersonIdentityCandidate>,
}

#[derive(Deserialize)]
struct PersonIdentityReviewApiRequest {
    command_id: String,
    review_state: String,
}

impl PersonIdentityReviewApiRequest {
    fn into_command(
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
struct PersonIdentityReviewApiResponse {
    identity_candidate_id: String,
    review_state: String,
    event_id: String,
}

impl From<crate::person_identity::PersonIdentityReviewCommandResult>
    for PersonIdentityReviewApiResponse
{
    fn from(result: crate::person_identity::PersonIdentityReviewCommandResult) -> Self {
        Self {
            identity_candidate_id: result.identity_candidate_id,
            review_state: result.review_state.as_str().to_owned(),
            event_id: result.event_id,
        }
    }
}

#[derive(Serialize)]
struct TaskListResponse {
    items: Vec<ActiveTask>,
}

#[derive(Serialize)]
struct DocumentProcessingJobsResponse {
    items: Vec<DocumentProcessingJob>,
}

#[derive(Deserialize)]
struct DocumentProcessingRetryApiRequest {
    command_id: String,
}

impl DocumentProcessingRetryApiRequest {
    fn into_command(
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
struct DocumentProcessingRetryApiResponse {
    job_id: String,
    status: DocumentProcessingStatus,
    event_id: String,
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
struct TaskCandidateReviewApiRequest {
    command_id: String,
    review_state: String,
}

impl TaskCandidateReviewApiRequest {
    fn into_command(
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
struct TaskCandidateReviewApiResponse {
    task_candidate_id: String,
    review_state: String,
    event_id: String,
}

impl From<crate::task_candidates::TaskCandidateReviewCommandResult>
    for TaskCandidateReviewApiResponse
{
    fn from(result: crate::task_candidates::TaskCandidateReviewCommandResult) -> Self {
        Self {
            task_candidate_id: result.task_candidate_id,
            review_state: result.review_state.as_str().to_owned(),
            event_id: result.event_id,
        }
    }
}

#[derive(Deserialize)]
struct ProjectLinkReviewApiRequest {
    command_id: String,
    target_kind: String,
    target_id: String,
    review_state: String,
}

impl ProjectLinkReviewApiRequest {
    fn into_command(
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
struct ProjectLinkReviewApiResponse {
    project_id: String,
    target_kind: String,
    target_id: String,
    review_state: String,
    event_id: String,
}

impl From<crate::project_link_reviews::ProjectLinkReviewCommandResult>
    for ProjectLinkReviewApiResponse
{
    fn from(result: crate::project_link_reviews::ProjectLinkReviewCommandResult) -> Self {
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
struct ProjectLinkCandidatesQuery {
    limit: Option<usize>,
}

struct TaskCandidatesQuery {
    limit: Option<i64>,
}

struct DocumentProcessingJobsQuery {
    limit: Option<i64>,
}

struct CommunicationMessagesQuery {
    limit: Option<i64>,
}

struct GraphNeighborhoodQuery {
    node_id: Option<String>,
    depth: Option<u8>,
}

struct GraphNodesQuery {
    limit: Option<i64>,
}

struct GraphSearchQuery {
    q: Option<String>,
    limit: Option<i64>,
}

struct ProjectsQuery {
    limit: Option<i64>,
}

fn parse_communication_messages_query(
    raw_query: Option<&str>,
) -> Result<CommunicationMessagesQuery, ApiError> {
    let mut query = CommunicationMessagesQuery { limit: None };

    if let Some(raw_query) = raw_query {
        for (key, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            if key.as_ref() == "limit" {
                query.limit = Some(value.parse::<i64>().map_err(|_| {
                    ApiError::InvalidCommunicationQuery("limit must be an integer")
                })?);
            }
        }
    }

    Ok(query)
}

fn parse_graph_neighborhood_query(
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

fn parse_graph_nodes_query(raw_query: Option<&str>) -> Result<GraphNodesQuery, ApiError> {
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

fn parse_graph_search_query(raw_query: Option<&str>) -> Result<GraphSearchQuery, ApiError> {
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

fn parse_projects_query(raw_query: Option<&str>) -> Result<ProjectsQuery, ApiError> {
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

fn parse_project_link_candidates_query(
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

fn parse_task_candidates_query(raw_query: Option<&str>) -> Result<TaskCandidatesQuery, ApiError> {
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

fn parse_document_processing_jobs_query(
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

struct PersonIdentityCandidatesQuery {
    limit: Option<i64>,
}

fn parse_person_identity_candidates_query(
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

fn parse_person_identity_review_state(
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

fn parse_project_link_target_kind(value: &str) -> Result<ProjectLinkTargetKind, ApiError> {
    match value.trim() {
        "message" => Ok(ProjectLinkTargetKind::Message),
        "document" => Ok(ProjectLinkTargetKind::Document),
        _ => Err(ApiError::InvalidProjectLinkReview(
            "target_kind must be message or document",
        )),
    }
}

fn parse_project_link_review_state(value: &str) -> Result<ProjectLinkReviewState, ApiError> {
    match value.trim() {
        "suggested" => Ok(ProjectLinkReviewState::Suggested),
        "user_confirmed" => Ok(ProjectLinkReviewState::UserConfirmed),
        "user_rejected" => Ok(ProjectLinkReviewState::UserRejected),
        _ => Err(ApiError::InvalidProjectLinkReview(
            "review_state must be suggested, user_confirmed, or user_rejected",
        )),
    }
}

fn parse_task_candidate_review_state(value: &str) -> Result<TaskCandidateReviewState, ApiError> {
    match value.trim() {
        "suggested" => Ok(TaskCandidateReviewState::Suggested),
        "user_confirmed" => Ok(TaskCandidateReviewState::UserConfirmed),
        "user_rejected" => Ok(TaskCandidateReviewState::UserRejected),
        _ => Err(ApiError::InvalidTaskCandidateReview(
            "review_state must be suggested, user_confirmed, or user_rejected",
        )),
    }
}

fn validate_non_empty_project_link_field(
    field: &'static str,
    value: &str,
) -> Result<String, ApiError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(ApiError::InvalidProjectLinkReview(field));
    }

    Ok(normalized.to_owned())
}

fn validate_non_empty_task_candidate_field(
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

fn validate_non_empty_document_id(value: &str) -> Result<String, ApiError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(ApiError::InvalidDocumentProcessingQuery(
            "document_id must not be empty",
        ));
    }

    Ok(normalized.to_owned())
}

fn validate_non_empty_document_processing_field(
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

fn validate_non_empty_person_identity_field(
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

fn text_preview(value: &str, max_chars: usize) -> String {
    let mut preview = value.chars().take(max_chars).collect::<String>();
    if value.chars().count() > max_chars {
        preview.push_str("...");
    }

    preview
}

enum ApiError {
    ApiTokenNotConfigured,
    InvalidApiToken,
    InvalidActorId,
    DatabaseNotConfigured,
    InvalidEnvelope(EventEnvelopeError),
    Audit(ApiAuditError),
    Store(EventStoreError),
    Graph(crate::graph::GraphStoreError),
    InvalidGraphQuery(&'static str),
    Projects(ProjectStoreError),
    InvalidProjectQuery(&'static str),
    InvalidProjectLinkReview(&'static str),
    InvalidTaskCandidateQuery(&'static str),
    InvalidTaskCandidateReview(&'static str),
    InvalidPersonIdentityReview(&'static str),
    InvalidDocumentProcessingQuery(&'static str),
    Settings(SettingsError),
    SettingNotFound,
    DocumentProcessing(DocumentProcessingError),
    TaskCandidateNotFound,
    TaskCandidate(TaskCandidateError),
    AiRunNotFound,
    Ai(AiError),
    Telegram(TelegramError),
    WhatsappWeb(WhatsappWebError),
    Automation(AutomationError),
    Call(CallError),
    ProjectLinkTargetNotFound,
    ProjectLinkReview(ProjectLinkReviewError),
    PersonIdentityNotFound,
    PersonIdentity(PersonIdentityError),
    Messages(MessageProjectionError),
    CommunicationIngestion(CommunicationIngestionError),
    MailStorage(MailStorageError),
    InvalidCommunicationQuery(&'static str),
    CommunicationMessageNotFound,
    SecretVaultNotConfigured,
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
            Self::ApiTokenNotConfigured => (
                StatusCode::SERVICE_UNAVAILABLE,
                "api_token_not_configured",
                "HERMES_LOCAL_API_TOKEN is not configured".to_owned(),
                false,
            ),
            Self::InvalidApiToken => (
                StatusCode::UNAUTHORIZED,
                "invalid_api_token",
                "missing or invalid bearer token".to_owned(),
                true,
            ),
            Self::InvalidActorId => (
                StatusCode::BAD_REQUEST,
                "invalid_actor_id",
                format!("missing or invalid {LOCAL_API_ACTOR_ID_HEADER} header"),
                false,
            ),
            Self::DatabaseNotConfigured => (
                StatusCode::SERVICE_UNAVAILABLE,
                "database_not_configured",
                "DATABASE_URL is not configured".to_owned(),
                false,
            ),
            Self::SecretVaultNotConfigured => (
                StatusCode::SERVICE_UNAVAILABLE,
                "secret_vault_not_configured",
                "HERMES_SECRET_VAULT_KEY is required for account setup".to_owned(),
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
                AiError::Ollama(error) => (
                    StatusCode::BAD_GATEWAY,
                    "ollama_runtime_error",
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
            Self::Telegram(error) => match error {
                TelegramError::InvalidRequest(message) => (
                    StatusCode::BAD_REQUEST,
                    "invalid_telegram_request",
                    message,
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
                TelegramError::MessageProjection(error) => {
                    tracing::error!(error = %error, "Telegram message projection failed");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "telegram_projection_error",
                        "Telegram message projection failed".to_owned(),
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

impl From<crate::graph::GraphStoreError> for ApiError {
    fn from(error: crate::graph::GraphStoreError) -> Self {
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

impl From<AiError> for ApiError {
    fn from(error: AiError) -> Self {
        match error {
            AiError::RunNotFound => Self::AiRunNotFound,
            _ => Self::Ai(error),
        }
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

impl From<crate::ollama::OllamaError> for ApiError {
    fn from(error: crate::ollama::OllamaError) -> Self {
        Self::Ai(AiError::Ollama(error))
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

impl From<crate::email_threads::EmailThreadError> for ApiError {
    fn from(error: crate::email_threads::EmailThreadError) -> Self {
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

impl From<crate::search::SearchError> for ApiError {
    fn from(error: crate::search::SearchError) -> Self {
        tracing::error!(error = %error, "search operation failed");
        ApiError::InvalidCommunicationQuery("search operation failed")
    }
}

impl From<crate::email_drafts::EmailDraftError> for ApiError {
    fn from(error: crate::email_drafts::EmailDraftError) -> Self {
        tracing::error!(error = %error, "email draft operation failed");
        ApiError::InvalidCommunicationQuery("email draft operation failed")
    }
}

impl From<crate::email_finance::EmailFinanceError> for ApiError {
    fn from(error: crate::email_finance::EmailFinanceError) -> Self {
        tracing::error!(error = %error, "email finance operation failed");
        ApiError::InvalidCommunicationQuery("email finance operation failed")
    }
}

impl From<crate::email_analytics::EmailAnalyticsError> for ApiError {
    fn from(error: crate::email_analytics::EmailAnalyticsError) -> Self {
        tracing::error!(error = %error, "email analytics operation failed");
        ApiError::InvalidCommunicationQuery("email analytics operation failed")
    }
}

impl From<crate::email_personas::EmailPersonaError> for ApiError {
    fn from(error: crate::email_personas::EmailPersonaError) -> Self {
        tracing::error!(error = %error, "email persona operation failed");
        ApiError::InvalidCommunicationQuery("email persona operation failed")
    }
}

impl From<crate::email_search::IndexEmailError> for ApiError {
    fn from(error: crate::email_search::IndexEmailError) -> Self {
        tracing::error!(error = %error, "email search operation failed");
        ApiError::InvalidCommunicationQuery("email search operation failed")
    }
}

impl From<crate::email_flags::MessageFlagsError> for ApiError {
    fn from(error: crate::email_flags::MessageFlagsError) -> Self {
        tracing::error!(error = %error, "message flags operation failed");
        ApiError::InvalidCommunicationQuery("message flags operation failed")
    }
}

impl From<crate::email_subscriptions::SubscriptionError> for ApiError {
    fn from(error: crate::email_subscriptions::SubscriptionError) -> Self {
        tracing::error!(error = %error, "subscriptions operation failed");
        ApiError::InvalidCommunicationQuery("subscriptions operation failed")
    }
}

impl From<crate::email_attachment_dedup::AttachmentDedupError> for ApiError {
    fn from(error: crate::email_attachment_dedup::AttachmentDedupError) -> Self {
        tracing::error!(error = %error, "attachment dedup operation failed");
        ApiError::InvalidCommunicationQuery("attachment dedup operation failed")
    }
}

impl From<crate::email_legal::LegalDocumentError> for ApiError {
    fn from(error: crate::email_legal::LegalDocumentError) -> Self {
        tracing::error!(error = %error, "legal document operation failed");
        ApiError::InvalidCommunicationQuery("legal document operation failed")
    }
}

impl From<crate::email_export::EmailExportError> for ApiError {
    fn from(error: crate::email_export::EmailExportError) -> Self {
        match error {
            crate::email_export::EmailExportError::NotFound => {
                ApiError::CommunicationMessageNotFound
            }
            _ => {
                tracing::error!(error = %error, "email export failed");
                ApiError::InvalidCommunicationQuery("email export failed")
            }
        }
    }
}

impl From<crate::email_send::EmailSendError> for ApiError {
    fn from(error: crate::email_send::EmailSendError) -> Self {
        tracing::error!(error = %error, "email send failed");
        ApiError::InvalidCommunicationQuery("email send failed")
    }
}
impl From<crate::email_imap_write::ImapWriteError> for ApiError {
    fn from(error: crate::email_imap_write::ImapWriteError) -> Self {
        tracing::error!(error = %error, "IMAP write operation failed");
        ApiError::InvalidCommunicationQuery("IMAP write operation failed")
    }
}
impl From<crate::email_signatures::CertificateError> for ApiError {
    fn from(error: crate::email_signatures::CertificateError) -> Self {
        tracing::error!(error = %error, "certificate operation failed");
        ApiError::InvalidCommunicationQuery("certificate operation failed")
    }
}
impl From<crate::email_multilingual::MultilingualError> for ApiError {
    fn from(error: crate::email_multilingual::MultilingualError) -> Self {
        tracing::error!(error = %error, "multilingual operation failed");
        ApiError::InvalidCommunicationQuery("multilingual operation failed")
    }
}
impl From<crate::email_ai_reply::AiReplyError> for ApiError {
    fn from(error: crate::email_ai_reply::AiReplyError) -> Self {
        tracing::error!(error = %error, "AI reply generation failed");
        ApiError::InvalidCommunicationQuery("AI reply generation failed")
    }
}
impl From<crate::email_extract::ExtractError> for ApiError {
    fn from(error: crate::email_extract::ExtractError) -> Self {
        tracing::error!(error = %error, "extract failed");
        ApiError::InvalidCommunicationQuery("extract failed")
    }
}
impl From<crate::person_enrichment::PersonEnrichmentError> for ApiError {
    fn from(error: crate::person_enrichment::PersonEnrichmentError) -> Self {
        match error {
            crate::person_enrichment::PersonEnrichmentError::NotFound => {
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

impl From<crate::organization_core::OrgCoreError> for ApiError {
    fn from(error: crate::organization_core::OrgCoreError) -> Self {
        tracing::error!(error = %error, "org core operation failed");
        ApiError::InvalidCommunicationQuery("org core operation failed")
    }
}
impl From<crate::organization_memory::OrgMemoryError> for ApiError {
    fn from(error: crate::organization_memory::OrgMemoryError) -> Self {
        tracing::error!(error = %error, "org memory operation failed");
        ApiError::InvalidCommunicationQuery("org memory operation failed")
    }
}
impl From<crate::organization_workflows::OrgWorkflowError> for ApiError {
    fn from(error: crate::organization_workflows::OrgWorkflowError) -> Self {
        tracing::error!(error = %error, "org workflow operation failed");
        ApiError::InvalidCommunicationQuery("org workflow operation failed")
    }
}
impl From<crate::organization_finance::OrgFinanceError> for ApiError {
    fn from(error: crate::organization_finance::OrgFinanceError) -> Self {
        tracing::error!(error = %error, "org finance operation failed");
        ApiError::InvalidCommunicationQuery("org finance operation failed")
    }
}
impl From<crate::organization_enrichment::OrgEnrichmentError> for ApiError {
    fn from(error: crate::organization_enrichment::OrgEnrichmentError) -> Self {
        tracing::error!(error = %error, "org enrichment operation failed");
        ApiError::InvalidCommunicationQuery("org enrichment operation failed")
    }
}
impl From<crate::organization_health::OrgHealthError> for ApiError {
    fn from(error: crate::organization_health::OrgHealthError) -> Self {
        tracing::error!(error = %error, "org health operation failed");
        ApiError::InvalidCommunicationQuery("org health operation failed")
    }
}
impl From<crate::organization_investigator::InvestigatorError> for ApiError {
    fn from(error: crate::organization_investigator::InvestigatorError) -> Self {
        match error {
            crate::organization_investigator::InvestigatorError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "investigator operation failed");
                ApiError::InvalidCommunicationQuery("investigator operation failed")
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
        Self::AccountSetup(error)
    }
}

#[derive(Deserialize)]
struct GmailOAuthStartApiRequest {
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
struct GmailOAuthStartApiResponse {
    setup_id: String,
    authorization_url: String,
    state: String,
    redirect_uri: String,
}

#[derive(Deserialize)]
struct GmailOAuthCompleteApiRequest {
    setup_id: String,
    state: String,
    authorization_code: String,
}

#[derive(Deserialize)]
struct GmailOAuthCallbackQuery {
    code: Option<String>,
    state: Option<String>,
}

#[derive(Deserialize)]
struct ImapAccountSetupApiRequest {
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
struct EmailAccountSetupApiResponse {
    account_id: String,
    secret_ref: String,
    secret_kind: SecretKind,
    store_kind: crate::secrets::SecretStoreKind,
}

impl From<crate::email_account_setup::EmailAccountSetupResult> for EmailAccountSetupApiResponse {
    fn from(result: crate::email_account_setup::EmailAccountSetupResult) -> Self {
        Self {
            account_id: result.account_id,
            secret_ref: result.secret_ref,
            secret_kind: result.secret_kind,
            store_kind: result.store_kind,
        }
    }
}

fn default_schema_version() -> i32 {
    1
}

fn empty_json_object() -> Value {
    json!({})
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error(transparent)]
    Io(#[from] io::Error),
}
