use super::super::*;
use super::database::database_pool;

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

pub(crate) fn mail_storage_store(state: &AppState) -> Result<MailStorageStore, ApiError> {
    Ok(MailStorageStore::new(database_pool(state)?))
}

pub(crate) fn communication_ingestion_store(
    state: &AppState,
) -> Result<CommunicationIngestionStore, ApiError> {
    Ok(CommunicationIngestionStore::new(database_pool(state)?))
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

pub(crate) fn person_identity_store(state: &AppState) -> Result<PersonIdentityStore, ApiError> {
    Ok(PersonIdentityStore::new(database_pool(state)?))
}

pub(crate) fn api_audit_log(state: &AppState) -> Result<ApiAuditLog, ApiError> {
    Ok(ApiAuditLog::new(database_pool(state)?))
}
