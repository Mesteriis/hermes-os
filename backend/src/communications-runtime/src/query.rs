//! Communications runtime composition for provider-neutral evidence reads.

use hermes_communications_api::{
    CommunicationAccountSummaryV1, CommunicationAttachmentAnchorSummaryV1,
    CommunicationConversationSummaryV1, CommunicationMessageReferenceSummaryV1,
    CommunicationMessageSummaryV1, CommunicationObservedParticipantSummaryV1, CommunicationSummary,
    CommunicationsClientError, GetCommunicationConversationV1, GetCommunicationEvidenceV1,
    ListCommunicationAccountsV1, ListCommunicationConversationsV1, ListConversationMessagesV1,
    ListConversationParticipantsV1, ListMessageAttachmentAnchorsV1, ListMessageEvidenceV1,
    ListMessageReferencesV1,
};
use hermes_communications_persistence::CommunicationsDurablePersistence;

pub async fn get_communication_evidence(
    persistence: &CommunicationsDurablePersistence,
    request: GetCommunicationEvidenceV1,
) -> Result<CommunicationSummary, CommunicationsClientError> {
    persistence
        .summary(request.evidence_id)
        .await
        .map_err(|_| CommunicationsClientError::Unavailable)?
        .ok_or(CommunicationsClientError::UnknownCommunication)
}

pub async fn list_message_evidence(
    persistence: &CommunicationsDurablePersistence,
    request: ListMessageEvidenceV1,
) -> Result<Vec<CommunicationSummary>, CommunicationsClientError> {
    let ids = persistence
        .message_evidence_ids(request.message_id, request.limit)
        .await
        .map_err(|_| CommunicationsClientError::Unavailable)?;
    let mut result = Vec::with_capacity(ids.len());
    for evidence_id in ids {
        result.push(
            get_communication_evidence(persistence, GetCommunicationEvidenceV1 { evidence_id })
                .await?,
        );
    }
    Ok(result)
}

pub async fn get_communication_conversation(
    persistence: &CommunicationsDurablePersistence,
    request: GetCommunicationConversationV1,
) -> Result<CommunicationConversationSummaryV1, CommunicationsClientError> {
    persistence
        .conversation(request.conversation_id)
        .await
        .map_err(|_| CommunicationsClientError::Unavailable)?
        .ok_or(CommunicationsClientError::UnknownCommunication)
}

pub async fn list_communication_accounts(
    persistence: &CommunicationsDurablePersistence,
    request: ListCommunicationAccountsV1,
) -> Result<Vec<CommunicationAccountSummaryV1>, CommunicationsClientError> {
    persistence
        .accounts(request.limit)
        .await
        .map_err(|_| CommunicationsClientError::Unavailable)
}

pub async fn list_communication_conversations(
    persistence: &CommunicationsDurablePersistence,
    request: ListCommunicationConversationsV1,
) -> Result<Vec<CommunicationConversationSummaryV1>, CommunicationsClientError> {
    persistence
        .conversations(request.account_cursor, request.limit)
        .await
        .map_err(|_| CommunicationsClientError::Unavailable)
}

pub async fn list_conversation_messages(
    persistence: &CommunicationsDurablePersistence,
    request: ListConversationMessagesV1,
) -> Result<Vec<CommunicationMessageSummaryV1>, CommunicationsClientError> {
    persistence
        .conversation_messages(request.conversation_id, request.limit)
        .await
        .map_err(|_| CommunicationsClientError::Unavailable)
}

pub async fn list_conversation_participants(
    persistence: &CommunicationsDurablePersistence,
    request: ListConversationParticipantsV1,
) -> Result<Vec<CommunicationObservedParticipantSummaryV1>, CommunicationsClientError> {
    persistence
        .conversation_participants(request.conversation_id, request.limit)
        .await
        .map_err(|_| CommunicationsClientError::Unavailable)
}

pub async fn list_message_attachment_anchors(
    persistence: &CommunicationsDurablePersistence,
    request: ListMessageAttachmentAnchorsV1,
) -> Result<Vec<CommunicationAttachmentAnchorSummaryV1>, CommunicationsClientError> {
    persistence
        .message_attachment_anchors(request.message_id, request.limit)
        .await
        .map_err(|_| CommunicationsClientError::Unavailable)
}

pub async fn list_message_references(
    persistence: &CommunicationsDurablePersistence,
    request: ListMessageReferencesV1,
) -> Result<Vec<CommunicationMessageReferenceSummaryV1>, CommunicationsClientError> {
    persistence
        .message_references(request.message_id, request.limit)
        .await
        .map_err(|_| CommunicationsClientError::Unavailable)
}
