pub mod ai;
pub mod audit;
pub mod automation;
pub mod calls;
pub mod communications;
pub mod config;
pub mod contact_identity;
pub mod contacts;
pub mod document_processing;
pub mod documents;
pub mod email_account_setup;
pub mod email_fixture_export;
pub mod email_fixture_pipeline;
pub mod email_import;
pub mod email_provider_network;
pub mod email_rfc822;
pub mod email_sources;
pub mod email_sync;
pub mod email_sync_pipeline;
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
use axum::routing::{get, post, put};
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
use crate::communications::{CommunicationIngestionStore, EmailProviderKind};
use crate::config::AppConfig;
use crate::contact_identity::{
    ContactIdentityCandidate, ContactIdentityDetail, ContactIdentityError,
    ContactIdentityReviewCommand, ContactIdentityReviewState, ContactIdentityStore,
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
use crate::event_log::{
    EventEnvelope, EventEnvelopeError, EventStore, EventStoreError, NewEventEnvelope,
};
use crate::graph::{GraphNodeKind, node_id};
use crate::mail_storage::{MailStorageError, MailStorageStore, StoredMailAttachmentWithBlob};
use crate::messages::{
    MessageProjectionError, MessageProjectionStore, ProjectedMessage, ProjectedMessageSummary,
};
use crate::ollama::{OllamaClient, OllamaClientConfig};
use crate::project_link_reviews::{
    ProjectLinkReviewCommand, ProjectLinkReviewError, ProjectLinkReviewState,
    ProjectLinkReviewStore, ProjectLinkTargetKind,
};
use crate::projects::{ProjectListResponse, ProjectStore, ProjectStoreError};
use crate::secret_vault::EncryptedSecretVault;
use crate::secrets::{SecretKind, SecretReferenceStore};
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
        .route("/api/v2/identity-candidates", get(get_identity_candidates))
        .route(
            "/api/v2/identity-candidates/{identity_candidate_id}/review",
            put(put_identity_candidate_review),
        )
        .route(
            "/api/v2/contacts/{contact_id}/identity",
            get(get_contact_identity),
        )
        .route("/api/v2/task-candidates", get(get_task_candidates))
        .route(
            "/api/v2/task-candidates/{task_candidate_id}/review",
            put(put_task_candidate_review),
        )
        .route("/api/v2/tasks", get(get_tasks))
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
            contacts: true,
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
) -> Result<Json<ContactIdentityCandidateListResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let query = parse_contact_identity_candidates_query(raw_query.as_deref())?;
    let items = contact_identity_store(&state)?
        .list_candidates(query.limit)
        .await?;

    Ok(Json(ContactIdentityCandidateListResponse { items }))
}

async fn put_identity_candidate_review(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(identity_candidate_id): Path<String>,
    Json(request): Json<ContactIdentityReviewApiRequest>,
) -> Result<Json<ContactIdentityReviewApiResponse>, ApiError> {
    let actor = verify_local_api_capability(&state.config, &headers)?;
    let command = request.into_command(identity_candidate_id, actor.actor_id)?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::contact_identity_review_set(
            &command.actor_id,
            &command.identity_candidate_id,
        ))
        .await?;

    let result = contact_identity_store(&state)?
        .set_review_state(&command)
        .await?;

    Ok(Json(result.into()))
}

async fn get_contact_identity(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(contact_id): Path<String>,
) -> Result<Json<ContactIdentityDetail>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let _ = validate_non_empty_contact_identity_field("contact_id", &contact_id)?;

    let detail = contact_identity_store(&state)?
        .contact_identity(&contact_id)
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
    let service = EmailAccountSetupService::new_for_vault_only(
        encrypted_vault(&state.config).ok_or(ApiError::SecretVaultNotConfigured)?,
    );
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
    let ollama = ollama_client(&state.config)?;
    let version = ollama.version().await;
    let tags = ollama.tags().await;
    let chat_model = state.config.ollama_chat_model().to_owned();
    let embedding_model = state.config.ollama_embed_model().to_owned();
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

    Ok(Json(AiAgentListResponse {
        items: v3_agents(state.config.ollama_chat_model()),
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
    let service = ai_service(&state)?;
    let response = service.answer(request, &actor.actor_id).await?;

    Ok(Json(response))
}

async fn post_v3_ai_task_candidates_refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<AiTaskCandidateRefreshRequest>,
) -> Result<Json<crate::ai::AiTaskCandidateRefreshResponse>, ApiError> {
    let actor = verify_local_api_capability(&state.config, &headers)?;
    let service = ai_service(&state)?;
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
    let service = ai_service(&state)?;
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
    let response = automation_store(&state)?
        .dry_run_send(&request, &actor.actor_id)
        .await?;
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

fn ai_service(state: &AppState) -> Result<AiService, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let ollama = ollama_client(&state.config)?;

    Ok(AiService::new(
        pool.clone(),
        ollama,
        state.config.ollama_chat_model(),
        state.config.ollama_embed_model(),
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

fn ollama_client(config: &AppConfig) -> Result<OllamaClient, ApiError> {
    Ok(OllamaClient::new(
        OllamaClientConfig::new(
            config.ollama_base_url(),
            config.ollama_chat_model(),
            config.ollama_embed_model(),
        )
        .with_timeout_seconds(config.ollama_timeout_seconds()),
    )?)
}

fn document_processing_store(state: &AppState) -> Result<DocumentProcessingStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(DocumentProcessingStore::new(pool.clone()))
}

fn contact_identity_store(state: &AppState) -> Result<ContactIdentityStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ContactIdentityStore::new(pool.clone()))
}

fn api_audit_log(state: &AppState) -> Result<ApiAuditLog, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ApiAuditLog::new(pool.clone()))
}

fn account_setup_service(state: &AppState) -> Result<EmailAccountSetupService, ApiError> {
    let vault = encrypted_vault(&state.config).ok_or(ApiError::SecretVaultNotConfigured)?;
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(EmailAccountSetupService::new(
        CommunicationIngestionStore::new(pool.clone()),
        SecretReferenceStore::new(pool.clone()),
        vault,
    ))
}

fn encrypted_vault(config: &AppConfig) -> Option<EncryptedSecretVault> {
    Some(EncryptedSecretVault::new(
        config.secret_vault_path()?.to_path_buf(),
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
    contacts: bool,
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
struct ContactIdentityCandidateListResponse {
    items: Vec<ContactIdentityCandidate>,
}

#[derive(Deserialize)]
struct ContactIdentityReviewApiRequest {
    command_id: String,
    review_state: String,
}

impl ContactIdentityReviewApiRequest {
    fn into_command(
        self,
        identity_candidate_id: String,
        actor_id: String,
    ) -> Result<ContactIdentityReviewCommand, ApiError> {
        let command_id = validate_non_empty_contact_identity_field("command_id", &self.command_id)?;
        let identity_candidate_id = validate_non_empty_contact_identity_field(
            "identity_candidate_id",
            &identity_candidate_id,
        )?;
        let review_state = parse_contact_identity_review_state(&self.review_state)?;

        Ok(ContactIdentityReviewCommand {
            command_id,
            identity_candidate_id,
            review_state,
            actor_id,
        })
    }
}

#[derive(Serialize)]
struct ContactIdentityReviewApiResponse {
    identity_candidate_id: String,
    review_state: String,
    event_id: String,
}

impl From<crate::contact_identity::ContactIdentityReviewCommandResult>
    for ContactIdentityReviewApiResponse
{
    fn from(result: crate::contact_identity::ContactIdentityReviewCommandResult) -> Self {
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

struct ContactIdentityCandidatesQuery {
    limit: Option<i64>,
}

fn parse_contact_identity_candidates_query(
    raw_query: Option<&str>,
) -> Result<ContactIdentityCandidatesQuery, ApiError> {
    let mut query = ContactIdentityCandidatesQuery { limit: None };

    if let Some(raw_query) = raw_query {
        for (key, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            if key.as_ref() == "limit" {
                query.limit = Some(
                    value
                        .parse::<i64>()
                        .map_err(|_| {
                            ApiError::InvalidContactIdentityReview("limit must be an integer")
                        })?
                        .clamp(1, 100),
                );
            }
        }
    }

    Ok(query)
}

fn parse_contact_identity_review_state(
    value: &str,
) -> Result<ContactIdentityReviewState, ApiError> {
    match value.trim() {
        "suggested" => Ok(ContactIdentityReviewState::Suggested),
        "user_confirmed" => Ok(ContactIdentityReviewState::UserConfirmed),
        "user_rejected" => Ok(ContactIdentityReviewState::UserRejected),
        _ => Err(ApiError::InvalidContactIdentityReview(
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

fn validate_non_empty_contact_identity_field(
    field: &'static str,
    value: &str,
) -> Result<String, ApiError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(ApiError::InvalidContactIdentityReview(match field {
            "command_id" => "command_id must not be empty",
            "identity_candidate_id" => "identity_candidate_id must not be empty",
            "contact_id" => "contact_id must not be empty",
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
    InvalidContactIdentityReview(&'static str),
    InvalidDocumentProcessingQuery(&'static str),
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
    ContactIdentityNotFound,
    ContactIdentity(ContactIdentityError),
    Messages(MessageProjectionError),
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
                "HERMES_SECRET_VAULT_PATH and HERMES_SECRET_VAULT_KEY are required for account setup".to_owned(),
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
            Self::InvalidContactIdentityReview(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_contact_identity_review",
                message.to_owned(),
                false,
            ),
            Self::InvalidDocumentProcessingQuery(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_document_processing_query",
                message.to_owned(),
                false,
            ),
            Self::DocumentProcessing(error) => {
                let (status, message) = match error {
                    DocumentProcessingError::InvalidLimit => {
                        (StatusCode::BAD_REQUEST, "document processing limit must be between 1 and 100")
                    }
                    DocumentProcessingError::EmptyField(_)
                    | DocumentProcessingError::InvalidStep(_)
                    | DocumentProcessingError::InvalidStatus(_)
                    | DocumentProcessingError::InvalidArtifactKind(_) => {
                        (StatusCode::BAD_REQUEST, "invalid document processing request payload")
                    }
                    DocumentProcessingError::DocumentNotFound | DocumentProcessingError::JobNotFound => {
                        (StatusCode::NOT_FOUND, "document processing job was not found")
                    }
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
                        (StatusCode::INTERNAL_SERVER_ERROR, "document processing store operation failed")
                    }
                };

                (status, "document_processing_store_error", message.to_owned(), false)
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
            Self::ContactIdentityNotFound => (
                StatusCode::NOT_FOUND,
                "contact_identity_candidate_not_found",
                "contact identity candidate was not found".to_owned(),
                false,
            ),
            Self::ContactIdentity(error) => {
                tracing::error!(
                    error = %error,
                    "contact identity store operation failed"
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "contact_identity_store_error",
                    "contact identity store operation failed".to_owned(),
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

impl From<ContactIdentityError> for ApiError {
    fn from(error: ContactIdentityError) -> Self {
        match error {
            ContactIdentityError::IdentityCandidateNotFound => Self::ContactIdentityNotFound,
            ContactIdentityError::InvalidLimit | ContactIdentityError::InvalidReviewState(_) => {
                Self::InvalidContactIdentityReview(
                    "review_state or limit must be valid for contact identity candidates",
                )
            }
            ContactIdentityError::InvalidPayload(_)
            | ContactIdentityError::MissingPayloadField(_)
            | ContactIdentityError::MissingActorId => {
                Self::InvalidContactIdentityReview("invalid identity candidate review payload")
            }
            _ => Self::ContactIdentity(error),
        }
    }
}

impl From<DocumentProcessingError> for ApiError {
    fn from(error: DocumentProcessingError) -> Self {
        Self::DocumentProcessing(error)
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
