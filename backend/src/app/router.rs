// ADR-0073: route registration remains centralized in app during the hard-v1
// migration so endpoint paths and shared middleware stay auditable in one place.
use std::collections::HashSet;
use std::io;
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

use axum::extract::{Path, Query, RawQuery, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, header};
use axum::response::Html;
use axum::routing::{delete, get, patch, post, put};
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
use crate::vault::{HostVault, HostVaultConfig};
use crate::workflows::email_intelligence::{EmailIntelligenceError, EmailIntelligenceService};

use crate::ai::api::*;
use crate::app::guard;
use crate::app::vault_reconciliation::spawn_host_vault_manifest_reconciliation;
use crate::app::{AccountSetupState, AppError, AppState};
use crate::domains::calendar::handlers::*;
use crate::domains::documents::api::*;
use crate::domains::graph::api::*;
use crate::domains::mail::handlers::*;
use crate::domains::organizations::handlers::*;
use crate::domains::persons::handlers::*;
use crate::domains::projects::api::*;
use crate::domains::settings::api::*;
use crate::domains::tasks::handlers::*;
use crate::engines::automation_api::*;
use crate::integrations::telegram::api::*;
use crate::integrations::telegram::runtime::TelegramRuntimeManager;
use crate::integrations::whatsapp::api::*;
use crate::platform::calls_api::*;
use crate::platform::events_api::*;
use axum::middleware;

static MAIL_BACKGROUND_SYNC_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

pub fn build_router(config: AppConfig) -> Router {
    build_router_with_database(config, Database::disabled())
}

pub fn build_router_with_database(config: AppConfig, database: Database) -> Router {
    let api_secret = config.local_api_secret().unwrap_or_default().to_owned();
    let vault = HostVault::new(HostVaultConfig {
        home: config.vault_home().to_path_buf(),
        dev_mode: config.dev_mode(),
        dev_key_path: config.dev_key_path().to_path_buf(),
    })
    .expect("host vault runtime must initialize");
    if let Err(error) = vault.unlock_existing() {
        tracing::warn!(error = %error, "host vault auto-unlock skipped");
    }
    let state = AppState {
        config,
        database,
        vault,
        account_setup: AccountSetupState::default(),
        telegram_runtime: TelegramRuntimeManager::default(),
    };
    spawn_host_vault_manifest_reconciliation(&state);
    spawn_mail_background_sync_scheduler(&state);

    let api_routes = Router::new()
        .route("/api/v1/status", get(get_v1_status))
        .route("/api/v1/vault/status", get(get_v1_vault_status))
        .route(
            "/api/v1/vault/collect-entropy",
            post(post_v1_vault_collect_entropy),
        )
        .route("/api/v1/vault/create", post(post_v1_vault_create))
        .route("/api/v1/vault/unlock", post(post_v1_vault_unlock))
        .route(
            "/api/v1/vault/recovery/export",
            post(post_v1_vault_recovery_export),
        )
        .route(
            "/api/v1/vault/recovery/import",
            post(post_v1_vault_recovery_import),
        )
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
        .route("/api/v1/workflow-actions", post(post_v1_workflow_action))
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
            "/api/v1/communications/messages/{message_id}/trash",
            post(post_v1_message_trash),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/restore",
            post(post_v1_message_restore),
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
        .route("/api/v1/graph/summary", get(get_graph_summary))
        .route("/api/v1/graph/nodes", get(get_graph_nodes))
        .route("/api/v1/graph/neighborhood", get(get_graph_neighborhood))
        .route("/api/v1/graph/search", get(get_graph_search))
        .route("/api/v1/projects", get(get_projects))
        .route("/api/v1/projects/{project_id}", get(get_project_detail))
        .route(
            "/api/v1/projects/{project_id}/link-candidates",
            get(get_project_link_candidates),
        )
        .route(
            "/api/v1/projects/{project_id}/link-reviews",
            put(put_project_link_review),
        )
        .route(
            "/api/v1/documents/{document_id}/processing",
            get(get_document_processing),
        )
        .route(
            "/api/v1/document-processing/jobs",
            get(get_document_processing_jobs),
        )
        .route(
            "/api/v1/document-processing/jobs/{job_id}/retry",
            post(post_document_processing_job_retry),
        )
        .route("/api/v1/persons", get(get_persons))
        .route("/api/v1/persons/{person_id}", get(get_person))
        .route(
            "/api/v1/persons/{person_id}/fingerprint",
            post(post_person_fingerprint),
        )
        .route(
            "/api/v1/persons/{person_id}/favorite",
            post(post_person_favorite),
        )
        .route("/api/v1/persons/{person_id}/notes", put(put_person_notes))
        .route("/api/v1/persons/search", get(get_person_search))
        .route("/api/v1/identity-candidates", get(get_identity_candidates))
        .route(
            "/api/v1/identity-candidates/{identity_candidate_id}/review",
            put(put_identity_candidate_review),
        )
        .route(
            "/api/v1/persons/{person_id}/identity",
            get(get_person_identity),
        )
        .route(
            "/api/v1/persons/{person_id}/identities",
            get(get_person_identities),
        )
        .route(
            "/api/v1/persons/{person_id}/identities",
            post(post_person_identity),
        )
        .route(
            "/api/v1/persons/{person_id}/identities/{identity_id}",
            delete(delete_person_identity),
        )
        .route("/api/v1/persons/{person_id}/roles", get(get_person_roles))
        .route("/api/v1/persons/{person_id}/roles", post(post_person_role))
        .route(
            "/api/v1/persons/{person_id}/roles/{role}",
            delete(delete_person_role),
        )
        .route(
            "/api/v1/persons/{person_id}/personas",
            get(get_person_personas),
        )
        .route(
            "/api/v1/persons/{person_id}/personas",
            post(post_person_persona),
        )
        .route(
            "/api/v1/persons/{person_id}/personas/{persona_id}",
            delete(delete_person_persona),
        )
        .route(
            "/api/v1/persons/{person_id}/facts",
            get(get_person_facts).post(post_person_fact),
        )
        .route(
            "/api/v1/persons/{person_id}/memory-cards",
            get(get_person_memory_cards).post(post_person_memory_card),
        )
        .route(
            "/api/v1/persons/{person_id}/preferences",
            get(get_person_preferences).post(post_person_preference),
        )
        .route(
            "/api/v1/persons/{person_id}/timeline",
            get(get_person_timeline).post(post_relationship_event),
        )
        .route(
            "/api/v1/persons/{person_id}/snapshots",
            get(get_person_snapshots),
        )
        .route(
            "/api/v1/persons/{person_id}/history-diff",
            get(get_person_history_diff),
        )
        .route(
            "/api/v1/persons/{person_id}/enrichment",
            get(get_person_enrichment),
        )
        .route(
            "/api/v1/persons/{person_id}/enrichment/{result_id}/apply",
            post(post_person_enrichment_apply),
        )
        .route(
            "/api/v1/persons/{person_id}/enrichment/{result_id}/reject",
            post(post_person_enrichment_reject),
        )
        .route(
            "/api/v1/persons/{person_id}/expertise",
            get(get_person_expertise),
        )
        .route(
            "/api/v1/persons/search/expertise",
            get(get_person_expertise_search),
        )
        .route(
            "/api/v1/persons/{person_id}/promises",
            get(get_person_promises),
        )
        .route("/api/v1/persons/{person_id}/risks", get(get_person_risks))
        .route(
            "/api/v1/persons/{person_id}/investigate",
            post(post_person_investigate),
        )
        .route(
            "/api/v1/persons/{person_id}/dossier",
            get(get_person_dossier),
        )
        .route(
            "/api/v1/persons/{person_id}/meeting-prep",
            get(get_person_meeting_prep),
        )
        .route(
            "/api/v1/persons/{person_id}/analytics",
            get(get_person_analytics),
        )
        .route(
            "/api/v1/persons/{person_id}/export",
            get(get_person_export_handler),
        )
        .route("/api/v1/persons/{person_id}/health", get(get_person_health))
        .route("/api/v1/persons/health", get(get_persons_health))
        .route("/api/v1/persons/watchlist", get(get_persons_watchlist))
        .route(
            "/api/v1/persons/{person_id}/watchlist",
            post(post_person_watchlist_toggle),
        )
        .route(
            "/api/v1/calendar/accounts",
            get(get_calendar_accounts).post(post_calendar_account),
        )
        .route(
            "/api/v1/calendar/accounts/{account_id}",
            get(get_calendar_account)
                .put(put_calendar_account)
                .delete(delete_calendar_account),
        )
        .route(
            "/api/v1/calendar/accounts/{account_id}/sources",
            get(get_calendar_sources).post(post_calendar_source),
        )
        .route(
            "/api/v1/calendar/events",
            get(get_calendar_events).post(post_calendar_event),
        )
        .route(
            "/api/v1/calendar/events/{event_id}",
            get(get_calendar_event)
                .put(put_calendar_event)
                .delete(delete_calendar_event),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/reschedule",
            post(post_calendar_event_reschedule),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/cancel",
            post(post_calendar_event_cancel),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/participants",
            get(get_event_participants).post(post_event_participant),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/relations",
            get(get_event_relations).post(post_event_relation),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/context-pack",
            get(get_event_context_pack).post(post_event_context_pack),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/agenda",
            get(get_event_agenda).post(post_event_agenda),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/checklist",
            get(get_event_checklist).post(post_event_checklist),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/classify",
            post(post_event_classify),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/analyze",
            post(post_event_analyze),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/risks",
            get(get_event_risks),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/notes",
            get(get_meeting_notes).post(post_meeting_note),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/outcomes",
            get(get_meeting_outcomes).post(post_meeting_outcome),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/recording",
            get(get_event_recordings).post(post_event_recording),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/transcript",
            get(get_event_transcript),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/follow-up",
            post(post_event_follow_up),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/follow-up-status",
            get(get_event_follow_up_status),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/brief",
            get(get_event_brief),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/generate-agenda",
            post(post_generate_agenda),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/export",
            get(get_event_export),
        )
        .route(
            "/api/v1/calendar/deadlines",
            get(get_deadlines).post(post_deadline),
        )
        .route(
            "/api/v1/calendar/focus-blocks",
            get(get_focus_blocks).post(post_focus_block),
        )
        .route("/api/v1/calendar/smart-schedule", post(post_smart_schedule))
        .route("/api/v1/calendar/watchtower", get(get_calendar_watchtower))
        .route("/api/v1/calendar/health", get(get_calendar_health))
        .route("/api/v1/calendar/weekly-brief", get(get_weekly_brief))
        .route("/api/v1/calendar/analytics", get(get_calendar_analytics))
        .route("/api/v1/calendar/brain", post(post_calendar_brain))
        .route("/api/v1/calendar/search", get(get_calendar_search))
        .route(
            "/api/v1/calendar/rules",
            get(get_calendar_rules).post(post_calendar_rule),
        )
        .route(
            "/api/v1/calendar/rules/{rule_id}",
            put(put_calendar_rule).delete(delete_calendar_rule),
        )
        .route("/api/v1/calendar/import", post(post_calendar_import))
        .route(
            "/api/v1/calendar/accounts/{account_id}/sync",
            post(post_calendar_sync),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/reminders",
            get(get_event_reminders).post(post_event_reminder),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/reminders/{reminder_id}/toggle",
            post(post_event_reminder_toggle),
        )
        .route(
            "/api/v1/calendar/analytics/distribution",
            get(get_time_distribution),
        )
        .route(
            "/api/v1/calendar/analytics/focus-balance",
            get(get_focus_balance),
        )
        .route(
            "/api/v1/calendar/analytics/back-to-back",
            get(get_back_to_back),
        )
        .route(
            "/api/v1/organizations",
            get(get_organizations).post(post_organization),
        )
        .route("/api/v1/organizations/search", get(get_organization_search))
        .route(
            "/api/v1/organizations/{org_id}",
            get(get_organization).put(put_organization),
        )
        .route(
            "/api/v1/organizations/{org_id}/archive",
            post(post_organization_archive),
        )
        .route(
            "/api/v1/organizations/{org_id}/identities",
            get(get_org_identities).post(post_org_identity),
        )
        .route(
            "/api/v1/organizations/{org_id}/aliases",
            get(get_org_aliases).post(post_org_alias),
        )
        .route(
            "/api/v1/organizations/{org_id}/domains",
            get(get_org_domains),
        )
        .route(
            "/api/v1/organizations/{org_id}/departments",
            get(get_org_departments).post(post_org_department),
        )
        .route(
            "/api/v1/organizations/{org_id}/contacts",
            get(get_org_contacts).post(post_org_contact_link),
        )
        .route(
            "/api/v1/organizations/{org_id}/related",
            get(get_org_related),
        )
        .route(
            "/api/v1/organizations/{org_id}/timeline",
            get(get_org_timeline),
        )
        .route(
            "/api/v1/organizations/{org_id}/portals",
            get(get_org_portals),
        )
        .route(
            "/api/v1/organizations/{org_id}/procedures",
            get(get_org_procedures),
        )
        .route(
            "/api/v1/organizations/{org_id}/playbooks",
            get(get_org_playbooks),
        )
        .route(
            "/api/v1/organizations/{org_id}/templates",
            get(get_org_templates),
        )
        .route(
            "/api/v1/organizations/{org_id}/financial",
            get(get_org_financial),
        )
        .route(
            "/api/v1/organizations/{org_id}/contracts",
            get(get_org_contracts),
        )
        .route(
            "/api/v1/organizations/{org_id}/compliance",
            get(get_org_compliance),
        )
        .route(
            "/api/v1/organizations/{org_id}/services",
            get(get_org_services),
        )
        .route(
            "/api/v1/organizations/{org_id}/products",
            get(get_org_products),
        )
        .route(
            "/api/v1/organizations/{org_id}/enrichment",
            get(get_org_enrichment),
        )
        .route(
            "/api/v1/organizations/{org_id}/enrichment/{rid}/apply",
            post(post_org_enrich_apply),
        )
        .route("/api/v1/organizations/{org_id}/risks", get(get_org_risks))
        .route("/api/v1/organizations/{org_id}/health", get(get_org_health))
        .route(
            "/api/v1/organizations/{org_id}/watchlist",
            post(post_org_watchlist_toggle),
        )
        .route(
            "/api/v1/organizations/{org_id}/dossier",
            get(get_org_dossier),
        )
        .route("/api/v1/organizations/{org_id}/brief", get(get_org_brief))
        .route(
            "/api/v1/organizations/{org_id}/context-pack",
            get(get_org_context_pack),
        )
        .route("/api/v1/tasks", get(get_tasks).post(post_task))
        .route("/api/v1/tasks/{task_id}", get(get_task).put(put_task))
        .route("/api/v1/tasks/{task_id}/archive", post(post_task_archive))
        .route("/api/v1/tasks/{task_id}/status", post(post_task_status))
        .route(
            "/api/v1/tasks/{task_id}/context-pack",
            get(get_task_context_pack).post(post_task_context_pack),
        )
        .route(
            "/api/v1/tasks/{task_id}/evidence",
            get(get_task_evidence).post(post_task_evidence),
        )
        .route(
            "/api/v1/tasks/{task_id}/relations",
            get(get_task_relations).post(post_task_relation),
        )
        .route(
            "/api/v1/tasks/{task_id}/checklist",
            get(get_task_checklist).post(post_task_checklist),
        )
        .route(
            "/api/v1/tasks/{task_id}/subtasks",
            get(get_task_subtasks).post(post_task_subtask),
        )
        .route("/api/v1/tasks/{task_id}/analyze", post(post_task_analyze))
        .route("/api/v1/tasks/{task_id}/export", get(get_task_export))
        .route("/api/v1/tasks/{task_id}/external", get(get_task_external))
        .route(
            "/api/v1/tasks/providers",
            get(get_task_providers).post(post_task_provider),
        )
        .route("/api/v1/tasks/brain", post(post_task_brain))
        .route("/api/v1/tasks/search", get(get_task_search))
        .route("/api/v1/tasks/daily-brief", get(get_task_daily_brief))
        .route(
            "/api/v1/tasks/rules",
            get(get_task_rules).post(post_task_rule),
        )
        .route("/api/v1/tasks/rules/{rule_id}", delete(delete_task_rule))
        .route("/api/v1/tasks/templates", get(get_task_templates))
        .route("/api/v1/tasks/watchtower", get(get_task_watchtower))
        .route("/api/v1/tasks/health", get(get_task_health))
        .route("/api/v1/tasks/analytics", get(get_task_analytics))
        .route("/api/v1/task-candidates", get(get_task_candidates))
        .route(
            "/api/v1/task-candidates/{task_candidate_id}/review",
            put(put_task_candidate_review),
        )
        .route("/api/v1/settings", get(get_application_settings))
        .route(
            "/api/v1/settings/accounts",
            get(get_application_settings_accounts),
        )
        .route(
            "/api/v1/settings/{setting_key}",
            put(put_application_setting),
        )
        .route("/api/v1/ai/status", get(get_ai_status))
        .route(
            "/api/v1/ai/settings/overview",
            get(get_ai_settings_overview),
        )
        .route(
            "/api/v1/ai/providers",
            get(get_ai_providers).post(post_ai_provider),
        )
        .route(
            "/api/v1/ai/providers/{provider_id}",
            patch(patch_ai_provider),
        )
        .route(
            "/api/v1/ai/providers/{provider_id}/test",
            post(post_ai_provider_test),
        )
        .route(
            "/api/v1/ai/providers/{provider_id}/sync-models",
            post(post_ai_provider_sync_models),
        )
        .route(
            "/api/v1/ai/providers/{provider_id}/consent",
            post(post_ai_provider_consent),
        )
        .route("/api/v1/ai/models", get(get_ai_models))
        .route("/api/v1/ai/model-routes/{slot}", put(put_ai_model_route))
        .route(
            "/api/v1/ai/prompts",
            get(get_ai_prompts).post(post_ai_prompt),
        )
        .route(
            "/api/v1/ai/prompts/{prompt_id}/versions",
            post(post_ai_prompt_version),
        )
        .route(
            "/api/v1/ai/prompts/{prompt_id}/activate",
            post(post_ai_prompt_activate),
        )
        .route(
            "/api/v1/ai/prompts/{prompt_id}/test",
            post(post_ai_prompt_test),
        )
        .route("/api/v1/ai/agents", get(get_ai_agents))
        .route("/api/v1/ai/runs", get(get_ai_runs))
        .route("/api/v1/ai/runs/{run_id}", get(get_ai_run))
        .route("/api/v1/ai/answers", post(post_ai_answer))
        .route(
            "/api/v1/ai/task-candidates/refresh",
            post(post_ai_task_candidates_refresh),
        )
        .route("/api/v1/ai/meeting-prep", post(post_ai_meeting_prep))
        .route(
            "/api/v1/telegram/capabilities",
            get(get_telegram_capabilities),
        )
        .route(
            "/api/v1/whatsapp/capabilities",
            get(get_whatsapp_capabilities),
        )
        .route(
            "/api/v1/telegram/accounts/fixture",
            post(post_telegram_fixture_account),
        )
        .route("/api/v1/telegram/accounts", post(post_telegram_account))
        .route(
            "/api/v1/telegram/runtime/status",
            get(get_telegram_runtime_status),
        )
        .route(
            "/api/v1/telegram/runtime/start",
            post(post_telegram_runtime_start),
        )
        .route(
            "/api/v1/telegram/login/qr/start",
            post(post_telegram_qr_login_start),
        )
        .route(
            "/api/v1/telegram/login/qr/{setup_id}",
            get(get_telegram_qr_login_status).delete(delete_telegram_qr_login),
        )
        .route(
            "/api/v1/telegram/login/qr/{setup_id}/password",
            post(post_telegram_qr_login_password),
        )
        .route("/api/v1/telegram/chats", get(get_telegram_chats))
        .route(
            "/api/v1/telegram/sync/chats",
            post(post_telegram_sync_chats),
        )
        .route(
            "/api/v1/telegram/sync/history",
            post(post_telegram_sync_history),
        )
        .route(
            "/api/v1/telegram/messages",
            get(get_telegram_messages).post(post_telegram_fixture_message),
        )
        .route(
            "/api/v1/telegram/messages/send",
            post(post_telegram_manual_send),
        )
        .route(
            "/api/v1/telegram/media/download",
            post(post_telegram_media_download),
        )
        .route(
            "/api/v1/policies/templates",
            get(get_policy_templates).post(post_policy_template),
        )
        .route("/api/v1/policies", get(get_policies).post(post_policy))
        .route(
            "/api/v1/policies/telegram-send/dry-run",
            post(post_telegram_send_dry_run),
        )
        .route("/api/v1/calls", get(get_calls).post(post_call))
        .route(
            "/api/v1/calls/{call_id}/transcript",
            get(get_call_transcript).post(post_call_transcript_fixture),
        )
        .route(
            "/api/v1/whatsapp/accounts/fixture",
            post(post_whatsapp_fixture_account),
        )
        .route("/api/v1/whatsapp/sessions", get(get_whatsapp_sessions))
        .route(
            "/api/v1/whatsapp/messages",
            get(get_whatsapp_messages).post(post_whatsapp_fixture_message),
        )
        .route(
            "/api/v1/email-accounts/gmail/oauth/start",
            post(post_gmail_oauth_start),
        )
        .route(
            "/api/v1/email-accounts/gmail/oauth/complete",
            post(post_gmail_oauth_complete),
        )
        .route("/api/v1/email-accounts/imap", post(post_imap_account_setup))
        .route(
            "/api/v1/email-accounts/sync-status",
            get(get_v1_email_account_sync_status),
        )
        .route(
            "/api/v1/email-accounts/{account_id}/sync-settings",
            get(get_v1_email_account_sync_settings).put(put_v1_email_account_sync_settings),
        )
        .route(
            "/api/v1/email-accounts/{account_id}/sync-now",
            post(post_v1_email_account_sync_now),
        )
        .route(
            "/api/v1/email-accounts/{account_id}/sync-full-resync",
            post(post_v1_email_account_sync_full_resync),
        )
        .route("/api/v1/audit/events", get(get_audit_events))
        .route("/api/v1/events", post(post_event))
        .route("/api/v1/events/{event_id}", get(get_event))
        .route_layer(middleware::from_fn_with_state(
            api_secret,
            guard::require_secret,
        ));

    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route(
            "/api/v1/email-accounts/gmail/oauth/callback",
            get(get_gmail_oauth_callback),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/remote-image",
            get(get_v1_communication_message_remote_image),
        )
        .merge(api_routes)
        .with_state(state)
        .layer(local_frontend_cors_layer())
}

fn spawn_mail_background_sync_scheduler(state: &AppState) {
    let Some(pool) = state.database.pool().cloned() else {
        return;
    };
    let Some(database_url) = state.database.database_url() else {
        return;
    };
    if !register_mail_background_sync_scheduler(database_url) {
        return;
    }
    let vault = state.vault.clone();

    tokio::spawn(async move {
        let store = crate::domains::mail::background_sync::MailSyncStore::new(pool.clone());
        let service = crate::domains::mail::background_sync::MailBackgroundSyncService::new(
            pool,
            vault,
            crate::domains::mail::background_sync::DEFAULT_MAIL_SYNC_BLOB_ROOT,
        );
        if let Err(error) = store.mark_orphaned_active_runs_failed(Utc::now()).await {
            tracing::warn!(error = %error, "mail background sync startup recovery failed");
        }
        let mut tick = tokio::time::interval(Duration::from_secs(30));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tick.tick().await;
            if let Err(error) = service.run_due_accounts().await {
                tracing::warn!(error = %error, "mail background sync scheduler tick failed");
            }
        }
    });
}

fn register_mail_background_sync_scheduler(database_url: &str) -> bool {
    match MAIL_BACKGROUND_SYNC_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "mail background sync scheduler registry is unavailable"
            );
            false
        }
    }
}

#[derive(Serialize)]
pub(crate) struct HealthResponse {
    status: &'static str,
    service: String,
}

#[derive(Serialize)]
pub(crate) struct ReadinessResponse {
    status: &'static str,
    service: String,
    checks: ReadinessChecks,
}

#[derive(Serialize)]
pub(crate) struct ReadinessChecks {
    database: DatabaseReadiness,
    migrations: MigrationReadiness,
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

pub(crate) fn local_frontend_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(|origin, _| {
            origin
                .to_str()
                .map(is_allowed_local_frontend_origin)
                .unwrap_or(false)
        }))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::OPTIONS])
        .allow_headers([
            header::CONTENT_TYPE,
            HeaderName::from_static("x-hermes-secret"),
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

pub(crate) async fn healthz(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: state.config.service_name().to_owned(),
    })
}

pub(crate) async fn readyz(State(state): State<AppState>) -> (StatusCode, Json<ReadinessResponse>) {
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
