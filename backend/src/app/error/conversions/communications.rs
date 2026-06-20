use super::super::types::ApiError;
use crate::domains::communications::core::CommunicationIngestionError;
use crate::domains::communications::messages::MessageProjectionError;
use crate::domains::communications::service::MailCommandServiceError;
use crate::domains::communications::storage::MailStorageError;
use crate::integrations::mail::accounts::EmailAccountSetupError;
use crate::workflows::email_intelligence::EmailIntelligenceError;

impl From<CommunicationIngestionError> for ApiError {
    fn from(error: CommunicationIngestionError) -> Self {
        Self::CommunicationIngestion(error)
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

impl From<crate::domains::communications::threads::CommunicationThreadError> for ApiError {
    fn from(error: crate::domains::communications::threads::CommunicationThreadError) -> Self {
        match error {
            crate::domains::communications::threads::CommunicationThreadError::InvalidCursor => {
                ApiError::InvalidCommunicationQuery("invalid thread cursor")
            }
            error => {
                tracing::error!(error = %error, "email thread operation failed");
                ApiError::InvalidCommunicationQuery("email thread operation failed")
            }
        }
    }
}

impl From<EmailIntelligenceError> for ApiError {
    fn from(error: EmailIntelligenceError) -> Self {
        match error {
            EmailIntelligenceError::ParseError(_msg) => {
                ApiError::InvalidCommunicationQuery("failed to parse AI analysis result")
            }
            _ => {
                tracing::error!(error = %error, "email intelligence operation failed");
                ApiError::InvalidCommunicationQuery("email intelligence operation failed")
            }
        }
    }
}

impl From<crate::domains::communications::drafts::CommunicationDraftError> for ApiError {
    fn from(error: crate::domains::communications::drafts::CommunicationDraftError) -> Self {
        match error {
            crate::domains::communications::drafts::CommunicationDraftError::Invalid(_) => {
                ApiError::InvalidCommunicationQuery("invalid draft request")
            }
            crate::domains::communications::drafts::CommunicationDraftError::InvalidCursor => {
                ApiError::InvalidCommunicationQuery("invalid draft cursor")
            }
            error => {
                tracing::error!(error = %error, "email draft operation failed");
                ApiError::InvalidCommunicationQuery("email draft operation failed")
            }
        }
    }
}

impl From<crate::domains::communications::outbox::CommunicationOutboxError> for ApiError {
    fn from(error: crate::domains::communications::outbox::CommunicationOutboxError) -> Self {
        match error {
            crate::domains::communications::outbox::CommunicationOutboxError::UndoUnavailable => {
                ApiError::NotFound
            }
            crate::domains::communications::outbox::CommunicationOutboxError::InvalidCursor => {
                ApiError::InvalidCommunicationQuery("invalid outbox cursor")
            }
            error => {
                tracing::error!(error = %error, "email outbox operation failed");
                ApiError::InvalidCommunicationQuery("email outbox operation failed")
            }
        }
    }
}

impl From<crate::domains::communications::bulk_actions::BulkMessageActionError> for ApiError {
    fn from(error: crate::domains::communications::bulk_actions::BulkMessageActionError) -> Self {
        match error {
            crate::domains::communications::bulk_actions::BulkMessageActionError::Invalid(_) => {
                ApiError::InvalidCommunicationQuery("invalid bulk message action request")
            }
            error => {
                tracing::error!(error = %error, "bulk message action failed");
                ApiError::InvalidCommunicationQuery("bulk message action failed")
            }
        }
    }
}

impl From<crate::domains::communications::saved_searches::MailSavedSearchError> for ApiError {
    fn from(error: crate::domains::communications::saved_searches::MailSavedSearchError) -> Self {
        match error {
            crate::domains::communications::saved_searches::MailSavedSearchError::Invalid(_) => {
                ApiError::InvalidCommunicationQuery("invalid saved search request")
            }
            crate::domains::communications::saved_searches::MailSavedSearchError::InvalidCursor => {
                ApiError::InvalidCommunicationQuery("invalid saved search cursor")
            }
            error => {
                tracing::error!(error = %error, "mail saved search operation failed");
                ApiError::InvalidCommunicationQuery("mail saved search operation failed")
            }
        }
    }
}

impl From<crate::domains::communications::folders::MailFolderError> for ApiError {
    fn from(error: crate::domains::communications::folders::MailFolderError) -> Self {
        match error {
            crate::domains::communications::folders::MailFolderError::Invalid(_) => {
                ApiError::InvalidCommunicationQuery("invalid mail folder request")
            }
            crate::domains::communications::folders::MailFolderError::InvalidCursor => {
                ApiError::InvalidCommunicationQuery("invalid mail folder cursor")
            }
            error => {
                tracing::error!(error = %error, "mail folder operation failed");
                ApiError::InvalidCommunicationQuery("mail folder operation failed")
            }
        }
    }
}

impl From<crate::domains::communications::ai_state::MailAiStateError> for ApiError {
    fn from(error: crate::domains::communications::ai_state::MailAiStateError) -> Self {
        match error {
            crate::domains::communications::ai_state::MailAiStateError::Invalid(_) => {
                ApiError::InvalidCommunicationQuery("invalid mail AI state request")
            }
            error => {
                tracing::error!(error = %error, "mail AI state operation failed");
                ApiError::InvalidCommunicationQuery("mail AI state operation failed")
            }
        }
    }
}

impl From<crate::domains::communications::read_receipts::MailReadReceiptError> for ApiError {
    fn from(error: crate::domains::communications::read_receipts::MailReadReceiptError) -> Self {
        match error {
            crate::domains::communications::read_receipts::MailReadReceiptError::Invalid(_) => {
                ApiError::InvalidCommunicationQuery("invalid mail read receipt request")
            }
            error => {
                tracing::error!(error = %error, "mail read receipt operation failed");
                ApiError::InvalidCommunicationQuery("mail read receipt operation failed")
            }
        }
    }
}

impl From<crate::domains::communications::templates::CommunicationTemplateError> for ApiError {
    fn from(error: crate::domains::communications::templates::CommunicationTemplateError) -> Self {
        match error {
            crate::domains::communications::templates::CommunicationTemplateError::InvalidTemplate(_) => {
                ApiError::InvalidCommunicationQuery("invalid email template request")
            }
            error => {
                tracing::error!(error = %error, "email template operation failed");
                ApiError::InvalidCommunicationQuery("email template operation failed")
            }
        }
    }
}

impl From<crate::domains::communications::delivery_notifications::MailDeliveryNotificationError>
    for ApiError
{
    fn from(
        error: crate::domains::communications::delivery_notifications::MailDeliveryNotificationError,
    ) -> Self {
        match error {
            crate::domains::communications::delivery_notifications::MailDeliveryNotificationError::Invalid(_) => {
                ApiError::InvalidCommunicationQuery("invalid mail delivery notification request")
            }
            error => {
                tracing::error!(error = %error, "mail delivery notification operation failed");
                ApiError::InvalidCommunicationQuery("mail delivery notification operation failed")
            }
        }
    }
}

impl From<crate::domains::communications::finance::CommunicationFinanceError> for ApiError {
    fn from(error: crate::domains::communications::finance::CommunicationFinanceError) -> Self {
        tracing::error!(error = %error, "email finance operation failed");
        ApiError::InvalidCommunicationQuery("email finance operation failed")
    }
}

impl From<crate::domains::communications::analytics::EmailAnalyticsError> for ApiError {
    fn from(error: crate::domains::communications::analytics::EmailAnalyticsError) -> Self {
        match error {
            crate::domains::communications::analytics::EmailAnalyticsError::InvalidCursor => {
                ApiError::InvalidCommunicationQuery("invalid analytics cursor")
            }
            error => {
                tracing::error!(error = %error, "email analytics operation failed");
                ApiError::InvalidCommunicationQuery("email analytics operation failed")
            }
        }
    }
}

impl From<crate::domains::communications::personas::CommunicationPersonaError> for ApiError {
    fn from(error: crate::domains::communications::personas::CommunicationPersonaError) -> Self {
        tracing::error!(error = %error, "email persona operation failed");
        ApiError::InvalidCommunicationQuery("email persona operation failed")
    }
}

impl From<crate::domains::communications::search::IndexEmailError> for ApiError {
    fn from(error: crate::domains::communications::search::IndexEmailError) -> Self {
        tracing::error!(error = %error, "email search operation failed");
        ApiError::InvalidCommunicationQuery("email search operation failed")
    }
}

impl From<crate::domains::communications::flags::MessageFlagsError> for ApiError {
    fn from(error: crate::domains::communications::flags::MessageFlagsError) -> Self {
        tracing::error!(error = %error, "message flags operation failed");
        ApiError::InvalidCommunicationQuery("message flags operation failed")
    }
}

impl From<crate::domains::communications::subscriptions::SubscriptionError> for ApiError {
    fn from(error: crate::domains::communications::subscriptions::SubscriptionError) -> Self {
        match error {
            crate::domains::communications::subscriptions::SubscriptionError::InvalidCursor => {
                ApiError::InvalidCommunicationQuery("invalid subscription cursor")
            }
            error => {
                tracing::error!(error = %error, "subscriptions operation failed");
                ApiError::InvalidCommunicationQuery("subscriptions operation failed")
            }
        }
    }
}

impl From<crate::domains::communications::attachment_dedup::AttachmentDedupError> for ApiError {
    fn from(error: crate::domains::communications::attachment_dedup::AttachmentDedupError) -> Self {
        tracing::error!(error = %error, "attachment dedup operation failed");
        ApiError::InvalidCommunicationQuery("attachment dedup operation failed")
    }
}

impl From<crate::domains::communications::attachment_search::AttachmentSearchError> for ApiError {
    fn from(
        error: crate::domains::communications::attachment_search::AttachmentSearchError,
    ) -> Self {
        match error {
            crate::domains::communications::attachment_search::AttachmentSearchError::InvalidCursor => {
                ApiError::InvalidCommunicationQuery("invalid attachment search cursor")
            }
            error => {
                tracing::error!(error = %error, "attachment search operation failed");
                ApiError::InvalidCommunicationQuery("attachment search operation failed")
            }
        }
    }
}

impl From<MailCommandServiceError> for ApiError {
    fn from(error: MailCommandServiceError) -> Self {
        match error {
            MailCommandServiceError::ObservationCapture { operation, source } => {
                tracing::error!(error = %source, operation, "mail command observation capture failed");
                ApiError::InvalidCommunicationQuery("mail command observation capture failed")
            }
            MailCommandServiceError::InvalidRequest(message) => {
                ApiError::InvalidCommunicationQuery(message)
            }
            MailCommandServiceError::Draft(inner) => ApiError::from(inner),
            MailCommandServiceError::Folder(inner) => ApiError::from(inner),
            MailCommandServiceError::SavedSearch(inner) => ApiError::from(inner),
            MailCommandServiceError::Outbox(inner) => ApiError::from(inner),
            MailCommandServiceError::MailStorage(inner) => ApiError::from(inner),
            MailCommandServiceError::AttachmentScan(source) => {
                tracing::warn!(error = %source, "attachment safety scan failed");
                ApiError::InvalidCommunicationQuery("attachment safety scan failed")
            }
            MailCommandServiceError::ProviderSendStore(source) => {
                tracing::error!(error = %source, "provider send observation persistence failed");
                ApiError::InvalidCommunicationQuery("provider send observation persistence failed")
            }
            MailCommandServiceError::MessageProjection(inner) => ApiError::from(inner),
            MailCommandServiceError::MailAiState(inner) => ApiError::from(inner),
            MailCommandServiceError::MessageFlags(inner) => ApiError::from(inner),
        }
    }
}

impl From<crate::domains::communications::legal::LegalDocumentError> for ApiError {
    fn from(error: crate::domains::communications::legal::LegalDocumentError) -> Self {
        tracing::error!(error = %error, "legal document operation failed");
        ApiError::InvalidCommunicationQuery("legal document operation failed")
    }
}

impl From<crate::domains::communications::export::CommunicationExportError> for ApiError {
    fn from(error: crate::domains::communications::export::CommunicationExportError) -> Self {
        match error {
            crate::domains::communications::export::CommunicationExportError::NotFound => {
                ApiError::CommunicationMessageNotFound
            }
            _ => {
                tracing::error!(error = %error, "email export failed");
                ApiError::InvalidCommunicationQuery("email export failed")
            }
        }
    }
}

impl From<crate::integrations::mail::send::EmailSendError> for ApiError {
    fn from(error: crate::integrations::mail::send::EmailSendError) -> Self {
        tracing::error!(error = %error, "email send failed");
        ApiError::InvalidCommunicationQuery("email send failed")
    }
}

impl From<crate::integrations::mail::imap_write::ImapWriteError> for ApiError {
    fn from(error: crate::integrations::mail::imap_write::ImapWriteError) -> Self {
        tracing::error!(error = %error, "IMAP write operation failed");
        ApiError::InvalidCommunicationQuery("IMAP write operation failed")
    }
}

impl From<crate::domains::communications::signatures::CertificateError> for ApiError {
    fn from(error: crate::domains::communications::signatures::CertificateError) -> Self {
        tracing::error!(error = %error, "certificate operation failed");
        ApiError::InvalidCommunicationQuery("certificate operation failed")
    }
}

impl From<crate::domains::communications::multilingual::MultilingualError> for ApiError {
    fn from(error: crate::domains::communications::multilingual::MultilingualError) -> Self {
        tracing::error!(error = %error, "multilingual operation failed");
        ApiError::InvalidCommunicationQuery("multilingual operation failed")
    }
}

impl From<crate::domains::communications::ai_reply::AiReplyError> for ApiError {
    fn from(error: crate::domains::communications::ai_reply::AiReplyError) -> Self {
        tracing::error!(error = %error, "AI reply generation failed");
        ApiError::InvalidCommunicationQuery("AI reply generation failed")
    }
}

impl From<crate::domains::communications::extract::ExtractError> for ApiError {
    fn from(error: crate::domains::communications::extract::ExtractError) -> Self {
        tracing::error!(error = %error, "extract failed");
        ApiError::InvalidCommunicationQuery("extract failed")
    }
}

impl From<EmailAccountSetupError> for ApiError {
    fn from(error: EmailAccountSetupError) -> Self {
        match error {
            EmailAccountSetupError::HostVault(error) => Self::HostVault(error),
            error => Self::AccountSetup(error),
        }
    }
}
