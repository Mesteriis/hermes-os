use super::super::*;
use super::database::database_pool;
use crate::domains::communications::storage::LocalCommunicationBlobStore;
use crate::workflows::mail_background_sync::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use sqlx::PgPool;

pub(crate) trait AppStoreFactory: Sized {
    fn from_pool(pool: PgPool) -> Self;
}

pub(crate) fn app_store<S: AppStoreFactory>(pool: PgPool) -> S {
    S::from_pool(pool)
}

macro_rules! impl_app_store_factory {
    ($($store:path),+ $(,)?) => {
        $(
            impl AppStoreFactory for $store {
                fn from_pool(pool: PgPool) -> Self {
                    <$store>::new(pool)
                }
            }
        )+
    };
}

impl_app_store_factory!(
    crate::domains::calendar::core::EventAgendaStore,
    crate::domains::calendar::core::EventChecklistStore,
    crate::domains::calendar::core::EventContextPackStore,
    crate::domains::calendar::core::EventParticipantStore,
    crate::domains::calendar::core::EventRelationStore,
    crate::domains::calendar::events::CalendarAccountStore,
    crate::domains::calendar::events::CalendarEventStore,
    crate::domains::calendar::events::CalendarSourceStore,
    crate::domains::calendar::meetings::EventRecordingStore,
    crate::domains::calendar::meetings::EventTranscriptStore,
    crate::domains::calendar::meetings::MeetingNoteStore,
    crate::domains::calendar::meetings::MeetingOutcomeStore,
    crate::domains::calendar::reminders::CalendarReminderStore,
    crate::domains::calendar::rules::CalendarRuleStore,
    crate::domains::calendar::scheduling::DeadlineStore,
    crate::domains::calendar::scheduling::FocusBlockStore,
    crate::domains::communications::ai_state::CommunicationAiStateStore,
    crate::domains::communications::analytics::EmailAnalyticsStore,
    crate::domains::communications::attachment_dedup::AttachmentDedupStore,
    crate::domains::communications::attachment_search::AttachmentSearchStore,
    crate::domains::communications::bulk_actions::BulkMessageActionStore,
    hermes_communications_postgres::provider_store::CommunicationProviderAccountStore,
    hermes_communications_postgres::provider_store::CommunicationProviderSecretBindingStore,
    crate::domains::communications::delivery_notifications::CommunicationDeliveryNotificationStore,
    crate::domains::communications::drafts::CommunicationDraftStore,
    crate::domains::communications::finance::CommunicationFinanceStore,
    crate::domains::communications::folders::CommunicationFolderStore,
    crate::domains::communications::legal::LegalDocumentStore,
    crate::domains::communications::messages::MessageProjectionStore,
    crate::domains::communications::outbox::CommunicationOutboxStore,
    crate::domains::communications::personas::CommunicationPersonaStore,
    crate::domains::communications::provider_resources::MailProviderResourceStore,
    crate::domains::communications::read_receipts::CommunicationReadReceiptStore,
    crate::domains::communications::saved_searches::CommunicationSavedSearchStore,
    crate::domains::communications::signatures::CertificateStore,
    crate::domains::communications::subscriptions::SubscriptionStore,
    crate::domains::communications::templates::CommunicationTemplateStore,
    crate::domains::communications::threads::CommunicationThreadStore,
    crate::domains::decisions::DecisionStore,
    crate::domains::documents::processing::DocumentProcessingStore,
    crate::domains::obligations::ObligationStore,
    crate::domains::organizations::api::OrganizationStore,
    crate::domains::organizations::core::OrgAliasStore,
    crate::domains::organizations::core::OrgPersonaLinkStore,
    crate::domains::organizations::core::OrgDepartmentStore,
    crate::domains::organizations::core::OrgDomainStore,
    crate::domains::organizations::core::OrgIdentityStore,
    crate::domains::organizations::core::RelatedOrgStore,
    crate::domains::organizations::enrichment::OrgEnrichmentStore,
    crate::domains::organizations::finance::OrgComplianceStore,
    crate::domains::organizations::finance::OrgContractStore,
    crate::domains::organizations::finance::OrgFinancialStore,
    crate::domains::organizations::finance::OrgProductStore,
    crate::domains::organizations::finance::OrgServiceStore,
    crate::domains::organizations::health::OrgHealthStore,
    crate::domains::organizations::health::OrgRiskStore,
    crate::domains::organizations::workflows::OrgPlaybookStore,
    crate::domains::organizations::workflows::OrgPortalStore,
    crate::domains::organizations::workflows::OrgProcedureStore,
    crate::domains::organizations::workflows::OrgTemplateStore,
    crate::domains::organizations::workflows::OrgTimelineStore,
    crate::domains::personas::api::PersonaProjectionStore,
    crate::domains::personas::core::PersonaInteractionContextStore,
    crate::domains::personas::core::PersonaRoleStore,
    crate::domains::personas::core::PersonaIdentityStore,
    crate::domains::personas::enrichment::PersonaEnrichmentStore,
    crate::domains::personas::enrichment_engine::EnrichmentResultStore,
    crate::domains::personas::expertise::PersonaExpertiseStore,
    crate::domains::personas::health::PersonaHealthStore,
    crate::domains::personas::memory::PersonaFactStore,
    crate::domains::personas::memory::PersonaMemoryCardStore,
    crate::domains::personas::memory::PersonaPreferenceStore,
    crate::domains::personas::memory::PersonaSnapshotStore,
    crate::domains::personas::memory::RelationshipEventStore,
    crate::domains::personas::trust::PersonaPromiseStore,
    crate::domains::personas::trust::PersonaRiskStore,
    crate::domains::relationships::store::RelationshipStore,
    crate::domains::review::ReviewInboxStore,
    crate::domains::tasks::api::TaskStore,
    crate::domains::tasks::core::ExternalTaskIdentityStore,
    crate::domains::tasks::core::TaskChecklistStore,
    crate::domains::tasks::core::TaskContextPackStore,
    crate::domains::tasks::core::TaskEvidenceStore,
    crate::domains::tasks::core::TaskProviderStore,
    crate::domains::tasks::core::TaskRelationStore,
    crate::domains::tasks::core::TaskSubtaskStore,
    crate::domains::tasks::rules::TaskRuleStore,
    crate::domains::tasks::rules::TaskTemplateStore,
    crate::engines::consistency::store::ContradictionObservationStore,
    hermes_events_postgres::store::EventStore,
    hermes_observations_postgres::store::ObservationStore,
    crate::workflows::mail_background_sync::MailSyncStore,
);

pub(crate) fn communication_blob_store() -> LocalCommunicationBlobStore {
    LocalCommunicationBlobStore::new(DEFAULT_MAIL_SYNC_BLOB_ROOT)
}

pub(crate) fn event_store(state: &AppState) -> Result<EventStore, ApiError> {
    Ok(EventStore::new(database_pool(state)?))
}

pub(crate) fn graph_store(
    state: &AppState,
) -> Result<crate::domains::graph::core::GraphStore, ApiError> {
    Ok(crate::domains::graph::core::GraphStore::new(database_pool(
        state,
    )?))
}

pub(crate) fn message_store(state: &AppState) -> Result<MessageProjectionStore, ApiError> {
    Ok(MessageProjectionStore::new(database_pool(state)?))
}

pub(crate) fn observation_store(
    state: &AppState,
) -> Result<hermes_observations_postgres::store::ObservationStore, ApiError> {
    Ok(hermes_observations_postgres::store::ObservationStore::new(
        database_pool(state)?,
    ))
}

pub(crate) fn communication_storage_store(
    state: &AppState,
) -> Result<CommunicationStorageStore, ApiError> {
    Ok(CommunicationStorageStore::new(database_pool(state)?))
}

pub(crate) fn communication_ingestion_store(
    state: &AppState,
) -> Result<CommunicationIngestionStore, ApiError> {
    Ok(CommunicationIngestionStore::new(database_pool(state)?))
}

pub(crate) fn communication_provider_account_store(
    state: &AppState,
) -> Result<
    hermes_communications_postgres::provider_store::CommunicationProviderAccountStore,
    ApiError,
> {
    Ok(
        hermes_communications_postgres::provider_store::CommunicationProviderAccountStore::new(
            database_pool(state)?,
        ),
    )
}

pub(crate) fn communication_provider_secret_binding_store(
    state: &AppState,
) -> Result<
    hermes_communications_postgres::provider_store::CommunicationProviderSecretBindingStore,
    ApiError,
> {
    Ok(
        hermes_communications_postgres::provider_store::CommunicationProviderSecretBindingStore::new(
            database_pool(state)?,
        ),
    )
}

pub(crate) fn project_store(state: &AppState) -> Result<ProjectStore, ApiError> {
    Ok(ProjectStore::new(database_pool(state)?))
}

pub(crate) fn project_link_review_store(
    state: &AppState,
) -> Result<ProjectLinkReviewStore, ApiError> {
    Ok(ProjectLinkReviewStore::new(database_pool(state)?))
}

pub(crate) fn task_candidate_store(state: &AppState) -> Result<TaskCandidateStore, ApiError> {
    Ok(TaskCandidateStore::new(database_pool(state)?))
}

pub(crate) fn document_processing_store(
    state: &AppState,
) -> Result<DocumentProcessingStore, ApiError> {
    Ok(DocumentProcessingStore::new(database_pool(state)?))
}

pub(crate) fn persona_identity_review_store(
    state: &AppState,
) -> Result<PersonaIdentityReviewStore, ApiError> {
    Ok(PersonaIdentityReviewStore::new(database_pool(state)?))
}

pub(crate) fn api_audit_log(state: &AppState) -> Result<ApiAuditLog, ApiError> {
    Ok(ApiAuditLog::new(database_pool(state)?))
}
