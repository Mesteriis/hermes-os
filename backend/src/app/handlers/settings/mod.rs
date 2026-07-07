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
use crate::domains::communications::core::{
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

use crate::application::email_intelligence::{EmailIntelligenceError, EmailIntelligenceService};
use crate::application::mail_background_sync::MailSyncStore;
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
use crate::domains::communications::core::CommunicationProviderAccountStore;
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
    EmailAccountSetupError, EmailAccountSetupService, GmailOAuthPendingGrant,
    GmailOAuthSetupRequest, ImapAccountSetupRequest,
};
use crate::integrations::ollama::client::{OllamaClient, OllamaClientConfig};
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

use crate::app::api_support::*;
use crate::app::{ApiError, AppState};

pub(crate) async fn get_application_settings(
    State(state): State<AppState>,
) -> Result<Json<ApplicationSettingsResponse>, ApiError> {
    let items = settings_store(&state)?.list_public_settings().await?;

    Ok(Json(ApplicationSettingsResponse { items }))
}

pub(crate) async fn get_application_settings_accounts(
    State(state): State<AppState>,
) -> Result<Json<ApplicationAccountsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let accounts =
        crate::app::api_support::app_store::<CommunicationProviderAccountStore>(pool.clone())
            .list()
            .await?
            .into_iter()
            .filter(|account| !account.is_deleted())
            .collect::<Vec<_>>();
    let mail_sync_error_codes =
        match crate::app::api_support::app_store::<MailSyncStore>(pool.clone())
            .sync_statuses()
            .await
        {
            Ok(statuses) => statuses
                .into_iter()
                .map(|status| (status.account_id, status.last_error_code))
                .collect::<HashMap<_, _>>(),
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    "failed to inspect mail sync statuses for provider account credential state"
                );
                HashMap::new()
            }
        };
    let mut items = Vec::with_capacity(accounts.len());
    for account in accounts {
        let credential_state = application_account_credential_state_from_sync_failure(
            account.provider_kind,
            mail_sync_error_codes
                .get(&account.account_id)
                .and_then(|error_code| error_code.as_deref()),
        );
        items.push(ApplicationAccountView {
            account,
            credential_state,
        });
    }

    Ok(Json(ApplicationAccountsResponse { items }))
}

fn application_account_credential_state_from_sync_failure(
    provider_kind: EmailProviderKind,
    last_error_code: Option<&str>,
) -> ApplicationAccountCredentialState {
    if provider_kind != EmailProviderKind::Gmail {
        return ApplicationAccountCredentialState::not_applicable();
    }

    if last_error_code == Some("oauth_refresh_failed") {
        ApplicationAccountCredentialState::expired()
    } else {
        ApplicationAccountCredentialState::valid()
    }
}

pub(crate) async fn patch_application_settings_account(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
    Json(request): Json<ApplicationAccountUpdateRequest>,
) -> Result<Json<ProviderAccount>, ApiError> {
    let Some(display_name) = request
        .display_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Err(ApiError::InvalidCommunicationQuery(
            "account display_name is required",
        ));
    };
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let account = crate::app::api_support::app_store::<CommunicationProviderAccountStore>(pool)
        .update_display_name(&account_id, display_name)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(account))
}

pub(crate) async fn put_application_setting(
    State(state): State<AppState>,
    Path(setting_key): Path<String>,
    Json(request): Json<ApplicationSettingUpdateRequest>,
) -> Result<Json<ApplicationSetting>, ApiError> {
    let actor_id = "hermes-frontend".to_string();

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::application_setting_set(
            &actor_id,
            &setting_key,
        ))
        .await?;
    let setting = settings_store(&state)?
        .update_setting_value(&setting_key, &request.value, &actor_id)
        .await?;

    Ok(Json(setting))
}

#[cfg(test)]
mod account_credential_state_tests {
    use super::*;

    #[test]
    fn gmail_oauth_credential_state_requests_reauth_after_refresh_failure() {
        let state = application_account_credential_state_from_sync_failure(
            EmailProviderKind::Gmail,
            Some("oauth_refresh_failed"),
        );

        assert_eq!(state, ApplicationAccountCredentialState::expired());
    }

    #[test]
    fn gmail_oauth_credential_state_does_not_request_reauth_without_refresh_failure() {
        let state = application_account_credential_state_from_sync_failure(
            EmailProviderKind::Gmail,
            Some("provider_network_error"),
        );

        assert_eq!(state, ApplicationAccountCredentialState::valid());
    }

    #[test]
    fn non_gmail_provider_credential_state_is_not_applicable() {
        let state = application_account_credential_state_from_sync_failure(
            EmailProviderKind::Icloud,
            Some("oauth_refresh_failed"),
        );

        assert_eq!(state, ApplicationAccountCredentialState::not_applicable());
    }
}
