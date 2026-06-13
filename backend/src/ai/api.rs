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

use crate::ai::control_center::{
    AiControlCenterStore, AiModelCatalogItem, AiModelRoute, AiModelRouteUpdateRequest,
    AiPromptActivateRequest, AiPromptCreateRequest, AiPromptEvalRun, AiPromptTemplate,
    AiPromptTestRequest, AiPromptVersion, AiPromptVersionCreateRequest, AiProviderAccount,
    AiProviderCommandKind, AiProviderCommandResponse, AiProviderConsentRequest,
    AiProviderCreateRequest, AiProviderPatchRequest, AiSettingsOverviewResponse,
    store_api_key_in_host_vault,
};
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
use crate::domains::persons::api::PersonProjectionStore;

pub(crate) async fn get_ai_status(
    State(state): State<AppState>,
) -> Result<Json<AiStatusResponse>, ApiError> {
    let runtime_settings = ai_runtime_settings(&state).await?;
    let runtime = ai_runtime_client(&state, &runtime_settings)?;
    let version = runtime.version().await;
    let models = runtime.models().await;
    let chat_model = runtime_settings.chat_model;
    let embedding_model = runtime_settings.embedding_model;
    let chat_model_available = models
        .as_ref()
        .map(|models| models.iter().any(|model| model == &chat_model))
        .unwrap_or(false);
    let embedding_model_available = models
        .as_ref()
        .map(|models| models.iter().any(|model| model == &embedding_model))
        .unwrap_or(false);

    Ok(Json(AiStatusResponse {
        runtime: runtime.runtime_name().to_owned(),
        status: if version.is_ok()
            && models.is_ok()
            && chat_model_available
            && embedding_model_available
        {
            "ok"
        } else {
            "unavailable"
        }
        .to_owned(),
        version: version.ok().flatten(),
        chat_model,
        embedding_model,
        embedding_dimension: AI_EMBEDDING_DIMENSION,
        chat_model_available,
        embedding_model_available,
    }))
}

pub(crate) async fn get_ai_agents(
    State(state): State<AppState>,
) -> Result<Json<AiAgentListResponse>, ApiError> {
    let runtime_settings = ai_runtime_settings(&state).await?;
    let mut items = v3_agents(&runtime_settings.chat_model);

    if let Some(pool) = state.database.pool() {
        let store = PersonProjectionStore::new(pool.clone());
        for item in &mut items {
            let persona = store
                .upsert_ai_agent_persona(item.agent_id, item.display_name)
                .await?;
            item.persona_id = Some(persona.person_id);
            item.persona_type = Some(persona.persona_type.as_str());
            item.persona_email = Some(persona.email_address);
        }
    }

    Ok(Json(AiAgentListResponse { items }))
}

pub(crate) async fn get_ai_runs(
    State(state): State<AppState>,
    Query(query): Query<AiRunsQuery>,
) -> Result<Json<AiRunListResponse>, ApiError> {
    let limit = query.limit.unwrap_or(25).clamp(1, 100);
    let runs = ai_run_store(&state)?.list_runs(limit).await?;

    Ok(Json(AiRunListResponse { items: runs }))
}

pub(crate) async fn get_ai_run(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<AiAgentRun>, ApiError> {
    let Some(run) = ai_run_store(&state)?.get_run(&run_id).await? else {
        return Err(ApiError::AiRunNotFound);
    };

    Ok(Json(run))
}

pub(crate) async fn post_ai_answer(
    State(state): State<AppState>,
    Json(request): Json<AiAnswerRequest>,
) -> Result<Json<crate::ai::core::AiAnswerResponse>, ApiError> {
    let actor_id = "hermes-frontend".to_string();
    let service = ai_service(&state).await?;
    let response = service.answer(request, &actor_id).await?;

    Ok(Json(response))
}

pub(crate) async fn post_ai_task_candidates_refresh(
    State(state): State<AppState>,
    Json(request): Json<AiTaskCandidateRefreshRequest>,
) -> Result<Json<crate::ai::core::AiTaskCandidateRefreshResponse>, ApiError> {
    let actor_id = "hermes-frontend".to_string();
    let service = ai_service(&state).await?;
    let response = service.refresh_task_candidates(request, &actor_id).await?;

    Ok(Json(response))
}

pub(crate) async fn post_ai_meeting_prep(
    State(state): State<AppState>,
    Json(request): Json<AiMeetingPrepRequest>,
) -> Result<Json<crate::ai::core::AiMeetingPrepResponse>, ApiError> {
    let actor_id = "hermes-frontend".to_string();
    let service = ai_service(&state).await?;
    let response = service.meeting_prep(request, &actor_id).await?;

    Ok(Json(response))
}

pub(crate) async fn get_ai_settings_overview(
    State(state): State<AppState>,
) -> Result<Json<AiSettingsOverviewResponse>, ApiError> {
    Ok(Json(ai_control_center_store(&state)?.overview().await?))
}

pub(crate) async fn get_ai_providers(
    State(state): State<AppState>,
) -> Result<Json<AiProviderListResponse>, ApiError> {
    Ok(Json(AiProviderListResponse {
        items: ai_control_center_store(&state)?.list_providers().await?,
    }))
}

pub(crate) async fn post_ai_provider(
    State(state): State<AppState>,
    Json(request): Json<AiProviderCreateRequest>,
) -> Result<Json<AiProviderAccount>, ApiError> {
    let store = ai_control_center_store(&state)?;
    let provider = store.create_provider(&request).await?;
    if let Some(api_key) = request
        .api_key
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        let Some(pool) = state.database.pool() else {
            return Err(ApiError::DatabaseNotConfigured);
        };
        store_api_key_in_host_vault(pool, &state.vault, &provider.provider_id, api_key).await?;
        let Some(provider) = store.provider(&provider.provider_id).await? else {
            return Err(ApiError::NotFound);
        };
        return Ok(Json(provider));
    }

    Ok(Json(provider))
}

pub(crate) async fn patch_ai_provider(
    State(state): State<AppState>,
    Path(provider_id): Path<String>,
    Json(request): Json<AiProviderPatchRequest>,
) -> Result<Json<AiProviderAccount>, ApiError> {
    let store = ai_control_center_store(&state)?;
    let provider = store.update_provider(&provider_id, &request).await?;
    if let Some(api_key) = request
        .api_key
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        let Some(pool) = state.database.pool() else {
            return Err(ApiError::DatabaseNotConfigured);
        };
        store_api_key_in_host_vault(pool, &state.vault, &provider.provider_id, api_key).await?;
        let Some(provider) = store.provider(&provider.provider_id).await? else {
            return Err(ApiError::NotFound);
        };
        return Ok(Json(provider));
    }

    Ok(Json(provider))
}

pub(crate) async fn post_ai_provider_test(
    State(state): State<AppState>,
    Path(provider_id): Path<String>,
) -> Result<Json<AiProviderCommandResponse>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .provider_command(&provider_id, AiProviderCommandKind::Test)
            .await?,
    ))
}

pub(crate) async fn post_ai_provider_sync_models(
    State(state): State<AppState>,
    Path(provider_id): Path<String>,
) -> Result<Json<AiProviderCommandResponse>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .provider_command(&provider_id, AiProviderCommandKind::SyncModels)
            .await?,
    ))
}

pub(crate) async fn post_ai_provider_consent(
    State(state): State<AppState>,
    Path(provider_id): Path<String>,
    Json(request): Json<AiProviderConsentRequest>,
) -> Result<Json<AiProviderAccount>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .record_consent(&provider_id, &request)
            .await?,
    ))
}

pub(crate) async fn get_ai_models(
    State(state): State<AppState>,
) -> Result<Json<AiModelListResponse>, ApiError> {
    Ok(Json(AiModelListResponse {
        items: ai_control_center_store(&state)?.list_models().await?,
    }))
}

pub(crate) async fn put_ai_model_route(
    State(state): State<AppState>,
    Path(slot): Path<String>,
    Json(request): Json<AiModelRouteUpdateRequest>,
) -> Result<Json<AiModelRoute>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .put_model_route(&slot, &request)
            .await?,
    ))
}

pub(crate) async fn get_ai_prompts(
    State(state): State<AppState>,
) -> Result<Json<AiPromptListResponse>, ApiError> {
    Ok(Json(AiPromptListResponse {
        items: ai_control_center_store(&state)?.list_prompts().await?,
    }))
}

pub(crate) async fn post_ai_prompt(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<AiPromptCreateRequest>,
) -> Result<Json<AiPromptTemplate>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .create_prompt(&request, &request_actor_id(&headers))
            .await?,
    ))
}

pub(crate) async fn post_ai_prompt_version(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(prompt_id): Path<String>,
    Json(request): Json<AiPromptVersionCreateRequest>,
) -> Result<Json<AiPromptVersion>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .create_prompt_version(&prompt_id, &request, &request_actor_id(&headers))
            .await?,
    ))
}

pub(crate) async fn post_ai_prompt_activate(
    State(state): State<AppState>,
    Path(prompt_id): Path<String>,
    Json(request): Json<AiPromptActivateRequest>,
) -> Result<Json<AiPromptTemplate>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .activate_prompt_version(&prompt_id, &request)
            .await?,
    ))
}

pub(crate) async fn post_ai_prompt_test(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(prompt_id): Path<String>,
    Json(request): Json<AiPromptTestRequest>,
) -> Result<Json<AiPromptEvalRun>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .test_prompt(&prompt_id, &request, &request_actor_id(&headers))
            .await?,
    ))
}

fn ai_control_center_store(state: &AppState) -> Result<AiControlCenterStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(AiControlCenterStore::new(pool.clone()))
}

fn request_actor_id(headers: &HeaderMap) -> String {
    headers
        .get("x-hermes-actor-id")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("hermes-frontend")
        .to_owned()
}

#[derive(Serialize)]
pub(crate) struct AiProviderListResponse {
    pub(crate) items: Vec<AiProviderAccount>,
}

#[derive(Serialize)]
pub(crate) struct AiModelListResponse {
    pub(crate) items: Vec<AiModelCatalogItem>,
}

#[derive(Serialize)]
pub(crate) struct AiPromptListResponse {
    pub(crate) items: Vec<AiPromptTemplate>,
}
