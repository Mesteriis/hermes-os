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
use crate::platform::calls::errors::CallError;
use crate::platform::calls::{
    CallDirection, CallIntelligenceStore, CallState, CallTranscript, FixtureSpeechToTextProvider,
    NewCallTranscript, NewTelegramCall, SpeechToTextProvider, TelegramCall, TranscriptStatus,
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

pub(crate) async fn post_policy_template(
    State(state): State<AppState>,
    Json(request): Json<PolicyTemplateApiRequest>,
) -> Result<Json<AutomationTemplate>, ApiError> {
    let actor_id = "hermes-frontend";
    Ok(Json(
        automation_store(&state)?
            .upsert_template(&request.into_template(), actor_id)
            .await?,
    ))
}

pub(crate) async fn get_policy_templates(
    State(state): State<AppState>,
) -> Result<Json<PolicyTemplateListResponse>, ApiError> {
    let items = automation_store(&state)?.list_templates().await?;

    Ok(Json(PolicyTemplateListResponse { items }))
}

pub(crate) async fn post_policy(
    State(state): State<AppState>,
    Json(request): Json<PolicyApiRequest>,
) -> Result<Json<AutomationPolicy>, ApiError> {
    let actor_id = "hermes-frontend";
    Ok(Json(
        automation_store(&state)?
            .upsert_policy(&request.into_policy(), actor_id)
            .await?,
    ))
}

pub(crate) async fn get_policies(
    State(state): State<AppState>,
) -> Result<Json<PolicyListResponse>, ApiError> {
    let items = automation_store(&state)?.list_policies().await?;

    Ok(Json(PolicyListResponse { items }))
}

pub(crate) async fn post_telegram_send_dry_run(
    State(state): State<AppState>,
    Json(request): Json<TelegramSendDryRunRequest>,
) -> Result<Json<TelegramSendDryRunResponse>, ApiError> {
    let actor_id = "hermes-frontend".to_string();
    let response = match automation_store(&state)?
        .dry_run_send(&request, &actor_id)
        .await
    {
        Ok(response) => response,
        Err(error) => {
            if let Some(decision) = telegram_send_dry_run_rejection_decision(&error, &request) {
                api_audit_log(&state)?
                    .record(
                        &NewApiAuditRecord::automation_telegram_send_dry_run_rejected(
                            &actor_id,
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
            &actor_id,
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

pub(crate) fn telegram_send_dry_run_rejection_decision(
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
        | AutomationError::ObservationStore(_)
        | AutomationError::Sqlx(_) => return None,
    };

    Some(CapabilityDecision::rejected_high_risk(
        CapabilityActionClass::Automation,
        "telegram.send",
        reason,
        non_empty_optional_string(&request.policy_id),
    ))
}

pub(crate) fn non_empty_optional_string(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}
