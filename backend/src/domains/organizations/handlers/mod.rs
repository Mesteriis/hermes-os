// ADR-0073: organization handlers are grouped for the first handlers.rs
// extraction; split profile, enrichment and operations handlers next.
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

// ── Organizations ───────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct OrganizationListResponse {
    items: Vec<crate::domains::organizations::api::Organization>,
}

#[derive(Deserialize)]
pub(crate) struct OrganizationListQuery {
    org_type: Option<String>,
    limit: Option<i64>,
}

pub(crate) async fn get_organizations(
    State(state): State<AppState>,
    Query(query): Query<OrganizationListQuery>,
) -> Result<Json<OrganizationListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = OrganizationStore::new(pool)
        .list(query.org_type.as_deref(), query.limit.unwrap_or(50))
        .await?;
    Ok(Json(OrganizationListResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewOrganizationRequest {
    display_name: String,
    org_type: Option<String>,
}

pub(crate) async fn post_organization(
    State(state): State<AppState>,
    Json(req): Json<NewOrganizationRequest>,
) -> Result<Json<crate::domains::organizations::api::Organization>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let org = OrganizationStore::new(pool)
        .create(&req.display_name, req.org_type.as_deref())
        .await?;
    Ok(Json(org))
}

pub(crate) async fn get_organization(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<crate::domains::organizations::api::Organization>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    OrganizationStore::new(pool)
        .get(&org_id)
        .await?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

pub(crate) async fn put_organization(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
    Json(update): Json<OrganizationUpdate>,
) -> Result<Json<crate::domains::organizations::api::Organization>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let org = OrganizationStore::new(pool)
        .update(&org_id, &update)
        .await?;
    Ok(Json(org))
}

#[derive(Deserialize)]
pub(crate) struct OrganizationSearchQuery {
    q: String,
    limit: Option<i64>,
}

pub(crate) async fn get_organization_search(
    State(state): State<AppState>,
    Query(query): Query<OrganizationSearchQuery>,
) -> Result<Json<OrganizationListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = OrganizationStore::new(pool);
    let all = store.list(None, 200).await?;
    let q = query.q.trim().to_lowercase();
    let items: Vec<_> = all
        .into_iter()
        .filter(|o| {
            o.display_name.to_lowercase().contains(&q)
                || o.legal_name
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&q)
                || o.website
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&q)
        })
        .take(query.limit.unwrap_or(20).clamp(1, 100) as usize)
        .collect();
    Ok(Json(OrganizationListResponse { items }))
}

pub(crate) async fn post_organization_archive(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    OrganizationStore::new(pool).archive(&org_id).await?;
    Ok(Json(json!({"archived": true})))
}

// ── Organization Identities ────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct OrgIdentitiesResponse {
    items: Vec<crate::domains::organizations::core::OrganizationIdentity>,
}

pub(crate) async fn get_org_identities(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgIdentitiesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::domains::organizations::core::OrgIdentityStore::new(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgIdentitiesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewOrgIdentityRequest {
    identity_type: String,
    identity_value: String,
    source: Option<String>,
}

pub(crate) async fn post_org_identity(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
    Json(req): Json<NewOrgIdentityRequest>,
) -> Result<Json<crate::domains::organizations::core::OrganizationIdentity>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let identity = crate::domains::organizations::core::OrgIdentityStore::new(pool)
        .upsert(
            &org_id,
            &req.identity_type,
            &req.identity_value,
            req.source.as_deref().unwrap_or("manual"),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(identity))
}

// ── Organization Aliases ───────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct OrgAliasesResponse {
    items: Vec<crate::domains::organizations::core::OrganizationAlias>,
}

pub(crate) async fn get_org_aliases(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgAliasesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::domains::organizations::core::OrgAliasStore::new(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgAliasesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewOrgAliasRequest {
    name: String,
    alias_type: String,
    source: Option<String>,
}

pub(crate) async fn post_org_alias(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
    Json(req): Json<NewOrgAliasRequest>,
) -> Result<Json<crate::domains::organizations::core::OrganizationAlias>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let alias = crate::domains::organizations::core::OrgAliasStore::new(pool)
        .add(
            &org_id,
            &req.name,
            &req.alias_type,
            req.source.as_deref().unwrap_or("manual"),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(alias))
}

// ── Organization Domains ───────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct OrgDomainsResponse {
    items: Vec<crate::domains::organizations::core::OrganizationDomain>,
}

pub(crate) async fn get_org_domains(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgDomainsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::domains::organizations::core::OrgDomainStore::new(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgDomainsResponse { items }))
}

// ── Organization Departments ───────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct OrgDepartmentsResponse {
    items: Vec<crate::domains::organizations::core::OrgDepartment>,
}

pub(crate) async fn get_org_departments(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgDepartmentsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::domains::organizations::core::OrgDepartmentStore::new(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgDepartmentsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewOrgDepartmentRequest {
    name: String,
    description: Option<String>,
    parent_id: Option<String>,
}

pub(crate) async fn post_org_department(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
    Json(req): Json<NewOrgDepartmentRequest>,
) -> Result<Json<crate::domains::organizations::core::OrgDepartment>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let dept = crate::domains::organizations::core::OrgDepartmentStore::new(pool)
        .add(
            &org_id,
            &req.name,
            req.description.as_deref(),
            req.parent_id.as_deref(),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(dept))
}

// ── Organization Contacts ──────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct OrgContactsResponse {
    items: Vec<crate::domains::organizations::core::OrgContactLink>,
}

pub(crate) async fn get_org_contacts(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgContactsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::domains::organizations::core::OrgContactLinkStore::new(pool)
        .list_by_org(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgContactsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct LinkOrgContactRequest {
    person_id: String,
    role: Option<String>,
    department: Option<String>,
}

pub(crate) async fn post_org_contact_link(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
    Json(req): Json<LinkOrgContactRequest>,
) -> Result<Json<crate::domains::organizations::core::OrgContactLink>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let link = crate::domains::organizations::core::OrgContactLinkStore::new(pool)
        .link(
            &org_id,
            &req.person_id,
            req.role.as_deref(),
            req.department.as_deref(),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(link))
}

// ── Organization Related ───────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct OrgRelatedResponse {
    items: Vec<crate::domains::organizations::core::RelatedOrganization>,
}

pub(crate) async fn get_org_related(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgRelatedResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::domains::organizations::core::RelatedOrgStore::new(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgRelatedResponse { items }))
}

// ── Organization Timeline ──────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct OrgTimelineResponse {
    items: Vec<crate::domains::organizations::workflows::OrgTimelineEvent>,
}
#[derive(Deserialize)]
pub(crate) struct OrgTimelineQuery {
    limit: Option<i64>,
}

pub(crate) async fn get_org_timeline(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
    Query(query): Query<OrgTimelineQuery>,
) -> Result<Json<OrgTimelineResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::domains::organizations::workflows::OrgTimelineStore::new(pool)
        .list(&org_id, query.limit.unwrap_or(50))
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgTimelineResponse { items }))
}

// ── Organization Portals ───────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct OrgPortalsResponse {
    items: Vec<crate::domains::organizations::workflows::OrgPortal>,
}

pub(crate) async fn get_org_portals(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgPortalsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::domains::organizations::workflows::OrgPortalStore::new(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgPortalsResponse { items }))
}

// ── Organization Procedures ────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct OrgProceduresResponse {
    items: Vec<crate::domains::organizations::workflows::OrgProcedure>,
}

pub(crate) async fn get_org_procedures(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgProceduresResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::domains::organizations::workflows::OrgProcedureStore::new(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgProceduresResponse { items }))
}

// ── Organization Playbooks ─────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct OrgPlaybooksResponse {
    items: Vec<crate::domains::organizations::workflows::OrgPlaybook>,
}

pub(crate) async fn get_org_playbooks(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgPlaybooksResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::domains::organizations::workflows::OrgPlaybookStore::new(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgPlaybooksResponse { items }))
}

// ── Organization Templates ─────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct OrgTemplatesResponse {
    items: Vec<crate::domains::organizations::workflows::OrgTemplate>,
}

pub(crate) async fn get_org_templates(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgTemplatesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::domains::organizations::workflows::OrgTemplateStore::new(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgTemplatesResponse { items }))
}

// ── Organization Financial ─────────────────────────────────────────────────

pub(crate) async fn get_org_financial(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let info = crate::domains::organizations::finance::OrgFinancialStore::new(pool)
        .get(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&info).unwrap_or_default()))
}

// ── Organization Contracts ─────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct OrgContractsResponse {
    items: Vec<crate::domains::organizations::finance::OrgContract>,
}

pub(crate) async fn get_org_contracts(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgContractsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::domains::organizations::finance::OrgContractStore::new(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgContractsResponse { items }))
}

// ── Organization Compliance ────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct OrgComplianceResponse {
    items: Vec<crate::domains::organizations::finance::OrgCompliance>,
}

pub(crate) async fn get_org_compliance(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgComplianceResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::domains::organizations::finance::OrgComplianceStore::new(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgComplianceResponse { items }))
}

// ── Organization Services ──────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct OrgServicesResponse {
    items: Vec<crate::domains::organizations::finance::OrgService>,
}

pub(crate) async fn get_org_services(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgServicesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::domains::organizations::finance::OrgServiceStore::new(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgServicesResponse { items }))
}

// ── Organization Products ──────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct OrgProductsResponse {
    items: Vec<crate::domains::organizations::finance::OrgProduct>,
}

pub(crate) async fn get_org_products(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgProductsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::domains::organizations::finance::OrgProductStore::new(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgProductsResponse { items }))
}

// ── Organization Enrichment ────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct OrgEnrichmentResponse {
    items: Vec<crate::domains::organizations::enrichment::OrgEnrichmentResult>,
}

pub(crate) async fn get_org_enrichment(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgEnrichmentResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::domains::organizations::enrichment::OrgEnrichmentStore::new(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgEnrichmentResponse { items }))
}

pub(crate) async fn post_org_enrich_apply(
    State(state): State<AppState>,
    Path((_org_id, rid)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    crate::domains::organizations::enrichment::OrgEnrichmentStore::new(pool)
        .apply(&rid)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(json!({"applied": true})))
}

// ── Organization Risks ─────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct OrgRisksResponse {
    items: Vec<crate::domains::organizations::health::OrgRisk>,
}

pub(crate) async fn get_org_risks(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgRisksResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::domains::organizations::health::OrgRiskStore::new(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgRisksResponse { items }))
}

// ── Organization Health ────────────────────────────────────────────────────

pub(crate) async fn get_org_health(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let health = crate::domains::organizations::health::OrgHealthStore::new(pool)
        .get(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&health).unwrap_or_default()))
}

pub(crate) async fn post_org_watchlist_toggle(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let on = crate::domains::organizations::health::OrgHealthStore::new(pool)
        .toggle_watchlist(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(json!({"watchlist": on})))
}

// ── Organization Investigator ───────────────────────────────────────────────

pub(crate) async fn get_org_dossier(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let dossier = crate::domains::organizations::investigator::OrganizationInvestigator::new(pool)
        .dossier(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&dossier).unwrap_or_default()))
}

pub(crate) async fn get_org_brief(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let brief = crate::domains::organizations::investigator::OrganizationInvestigator::new(pool)
        .brief(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&brief).unwrap_or_default()))
}

pub(crate) async fn get_org_context_pack(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let pack = crate::domains::organizations::investigator::OrganizationInvestigator::new(pool)
        .context_pack(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&pack).unwrap_or_default()))
}
