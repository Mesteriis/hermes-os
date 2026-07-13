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

use crate::platform::formatting::text_preview;

use crate::ai::core::{
    AI_EMBEDDING_DIMENSION, AiAgentListResponse, AiAgentRun, AiAnswerRequest, AiError,
    AiMeetingPrepRequest, AiService, AiStatusResponse, AiTaskCandidateRefreshRequest, v3_agents,
};
use crate::domains::personas::analytics::{AnalyticsError, PersonaAnalyticsService};
use crate::domains::personas::enrichment_engine::{EnrichmentEngineError, EnrichmentResultStore};
use crate::domains::personas::expertise::{PersonaExpertiseError, PersonaExpertiseStore};
use crate::domains::personas::export::{ExportError, ExportFormat, PersonaExportService};
use crate::domains::personas::health::{PersonaHealthError, PersonaHealthStore};
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

pub(crate) async fn get_projects(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<ProjectListResponse>, ApiError> {
    let query = parse_projects_query(raw_query.as_deref())?;
    let items = project_store(&state)?.list_projects(query.limit).await?;

    Ok(Json(ProjectListResponse { items }))
}

pub(crate) async fn get_project_detail(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<crate::domains::projects::core::ProjectDetail>, ApiError> {
    let Some(project) = project_store(&state)?.project_detail(&project_id).await? else {
        return Err(ApiError::ProjectNotFound);
    };

    Ok(Json(project))
}

pub(crate) async fn get_project_link_candidates(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<ProjectLinkCandidateListResponse>, ApiError> {
    let query = parse_project_link_candidates_query(raw_query.as_deref())?;
    let project_id = validate_non_empty_project_link_field("project_id", &project_id)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();

    let project_store = project_store(&state)?;
    let review_store = project_link_review_store(&state)?;
    let mut candidates = Vec::new();

    for message in project_store.matching_project_messages(&project_id).await? {
        let graph_node_id = node_id(GraphNodeKind::Message, &message.message_id);
        let title = text_preview(&message.subject, 120);
        let sender_excerpt = text_preview(&message.sender, 140);
        crate::application::project_link_review_mirror::ensure_project_link_candidate_review_item(
            &pool,
            &project_id,
            ProjectLinkTargetKind::Message,
            &message.message_id,
            &title,
            &sender_excerpt,
            1.0,
            &message.observation_id,
            Some(&graph_node_id),
        )
        .await
        .map_err(|error| ApiError::FailedPrecondition(error.to_string()))?;
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
            title,
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
        crate::application::project_link_review_mirror::ensure_project_link_candidate_review_item(
            &pool,
            &project_id,
            ProjectLinkTargetKind::Document,
            &document.document_id,
            &title,
            &title,
            1.0,
            &document.observation_id,
            Some(&graph_node_id),
        )
        .await
        .map_err(|error| ApiError::FailedPrecondition(error.to_string()))?;
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

pub(crate) async fn put_project_link_review(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(request): Json<ProjectLinkReviewApiRequest>,
) -> Result<Json<ProjectLinkReviewApiResponse>, ApiError> {
    let actor_id = "hermes-frontend".to_string();
    let command = request.into_command(project_id, actor_id)?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::project_link_review_set(
            &command.actor_id,
            &command.project_id,
            command.target_kind.as_str(),
            &command.target_id,
        ))
        .await?;

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();

    let result =
        crate::domains::projects::link_reviews::ProjectLinkReviewService::new(pool.clone())
            .review_manual(&command)
            .await?;
    let stored_event = hermes_events_postgres::store::EventStore::new(pool.clone())
        .get_by_id(&result.event_id)
        .await?
        .ok_or_else(|| {
            ApiError::Store(
                hermes_events_postgres::errors::EventStoreError::ConsumerHandlerFailed(
                    "project link review event was not stored".to_owned(),
                ),
            )
        })?;
    crate::workflows::project_link_review_effects::project_link_review_effect(&pool, &stored_event)
        .await
        .map_err(|error| ApiError::FailedPrecondition(error.to_string()))?;
    let observation_id = sqlx::query_scalar::<_, String>(
        r#"
        SELECT observation_id
        FROM observation_links
        WHERE domain = 'projects'
          AND entity_kind = 'project_link_review'
          AND entity_id = $1
          AND relationship_kind = 'review_transition'
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&result.event_id)
    .fetch_optional(&pool)
    .await
    .map_err(ProjectStoreError::from)
    .map_err(ApiError::Projects)?
    .ok_or_else(|| {
        ApiError::Store(
            hermes_events_postgres::errors::EventStoreError::ConsumerHandlerFailed(
                "project link review observation link was not stored".to_owned(),
            ),
        )
    })?;
    let project_store = project_store(&state)?;
    let mut transaction = pool
        .begin()
        .await
        .map_err(ProjectStoreError::from)
        .map_err(ApiError::Projects)?;
    match command.target_kind {
        ProjectLinkTargetKind::Message => {
            let message = project_store
                .matching_project_messages(&command.project_id)
                .await?
                .into_iter()
                .find(|item| item.message_id == command.target_id)
                .ok_or(ApiError::ProjectLinkTargetNotFound)?;
            let title = text_preview(&message.subject, 120);
            let summary = text_preview(&message.sender, 140);
            crate::application::project_link_review_mirror::sync_project_link_review_state_in_transaction(
                &mut transaction,
                &command.project_id,
                command.target_kind,
                &command.target_id,
                command.review_state,
                &title,
                &summary,
                1.0,
                &observation_id,
            )
            .await
            .map_err(|error| ApiError::FailedPrecondition(error.to_string()))?;
        }
        ProjectLinkTargetKind::Document => {
            let document = project_store
                .matching_project_documents(&command.project_id)
                .await?
                .into_iter()
                .find(|item| item.document_id == command.target_id)
                .ok_or(ApiError::ProjectLinkTargetNotFound)?;
            let title = text_preview(&document.title, 140);
            crate::application::project_link_review_mirror::sync_project_link_review_state_in_transaction(
                &mut transaction,
                &command.project_id,
                command.target_kind,
                &command.target_id,
                command.review_state,
                &title,
                &title,
                1.0,
                &observation_id,
            )
            .await
            .map_err(|error| ApiError::FailedPrecondition(error.to_string()))?;
        }
    }
    transaction
        .commit()
        .await
        .map_err(ProjectStoreError::from)
        .map_err(ApiError::Projects)?;

    Ok(Json(result.into()))
}
