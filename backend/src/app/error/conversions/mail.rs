use super::super::types::ApiError;
use crate::domains::mail::accounts::EmailAccountSetupError;
use crate::domains::mail::core::CommunicationIngestionError;
use crate::domains::mail::messages::MessageProjectionError;
use crate::domains::mail::service::MailCommandServiceError;
use crate::domains::mail::storage::MailStorageError;
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

impl From<crate::domains::mail::threads::EmailThreadError> for ApiError {
    fn from(error: crate::domains::mail::threads::EmailThreadError) -> Self {
        match error {
            crate::domains::mail::threads::EmailThreadError::InvalidCursor => {
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

impl From<crate::domains::mail::drafts::EmailDraftError> for ApiError {
    fn from(error: crate::domains::mail::drafts::EmailDraftError) -> Self {
        match error {
            crate::domains::mail::drafts::EmailDraftError::Invalid(_) => {
                ApiError::InvalidCommunicationQuery("invalid draft request")
            }
            crate::domains::mail::drafts::EmailDraftError::InvalidCursor => {
                ApiError::InvalidCommunicationQuery("invalid draft cursor")
            }
            error => {
                tracing::error!(error = %error, "email draft operation failed");
                ApiError::InvalidCommunicationQuery("email draft operation failed")
            }
        }
    }
}

impl From<crate::domains::mail::outbox::EmailOutboxError> for ApiError {
    fn from(error: crate::domains::mail::outbox::EmailOutboxError) -> Self {
        match error {
            crate::domains::mail::outbox::EmailOutboxError::UndoUnavailable => ApiError::NotFound,
            crate::domains::mail::outbox::EmailOutboxError::InvalidCursor => {
                ApiError::InvalidCommunicationQuery("invalid outbox cursor")
            }
            error => {
                tracing::error!(error = %error, "email outbox operation failed");
                ApiError::InvalidCommunicationQuery("email outbox operation failed")
            }
        }
    }
}

impl From<crate::domains::mail::bulk_actions::BulkMessageActionError> for ApiError {
    fn from(error: crate::domains::mail::bulk_actions::BulkMessageActionError) -> Self {
        match error {
            crate::domains::mail::bulk_actions::BulkMessageActionError::Invalid(_) => {
                ApiError::InvalidCommunicationQuery("invalid bulk message action request")
            }
            error => {
                tracing::error!(error = %error, "bulk message action failed");
                ApiError::InvalidCommunicationQuery("bulk message action failed")
            }
        }
    }
}

impl From<crate::domains::mail::saved_searches::MailSavedSearchError> for ApiError {
    fn from(error: crate::domains::mail::saved_searches::MailSavedSearchError) -> Self {
        match error {
            crate::domains::mail::saved_searches::MailSavedSearchError::Invalid(_) => {
                ApiError::InvalidCommunicationQuery("invalid saved search request")
            }
            crate::domains::mail::saved_searches::MailSavedSearchError::InvalidCursor => {
                ApiError::InvalidCommunicationQuery("invalid saved search cursor")
            }
            error => {
                tracing::error!(error = %error, "mail saved search operation failed");
                ApiError::InvalidCommunicationQuery("mail saved search operation failed")
            }
        }
    }
}

impl From<crate::domains::mail::folders::MailFolderError> for ApiError {
    fn from(error: crate::domains::mail::folders::MailFolderError) -> Self {
        match error {
            crate::domains::mail::folders::MailFolderError::Invalid(_) => {
                ApiError::InvalidCommunicationQuery("invalid mail folder request")
            }
            crate::domains::mail::folders::MailFolderError::InvalidCursor => {
                ApiError::InvalidCommunicationQuery("invalid mail folder cursor")
            }
            error => {
                tracing::error!(error = %error, "mail folder operation failed");
                ApiError::InvalidCommunicationQuery("mail folder operation failed")
            }
        }
    }
}

impl From<crate::domains::mail::ai_state::MailAiStateError> for ApiError {
    fn from(error: crate::domains::mail::ai_state::MailAiStateError) -> Self {
        match error {
            crate::domains::mail::ai_state::MailAiStateError::Invalid(_) => {
                ApiError::InvalidCommunicationQuery("invalid mail AI state request")
            }
            error => {
                tracing::error!(error = %error, "mail AI state operation failed");
                ApiError::InvalidCommunicationQuery("mail AI state operation failed")
            }
        }
    }
}

impl From<crate::domains::mail::read_receipts::MailReadReceiptError> for ApiError {
    fn from(error: crate::domains::mail::read_receipts::MailReadReceiptError) -> Self {
        match error {
            crate::domains::mail::read_receipts::MailReadReceiptError::Invalid(_) => {
                ApiError::InvalidCommunicationQuery("invalid mail read receipt request")
            }
            error => {
                tracing::error!(error = %error, "mail read receipt operation failed");
                ApiError::InvalidCommunicationQuery("mail read receipt operation failed")
            }
        }
    }
}

impl From<crate::domains::mail::templates::EmailTemplateError> for ApiError {
    fn from(error: crate::domains::mail::templates::EmailTemplateError) -> Self {
        match error {
            crate::domains::mail::templates::EmailTemplateError::InvalidTemplate(_) => {
                ApiError::InvalidCommunicationQuery("invalid email template request")
            }
            error => {
                tracing::error!(error = %error, "email template operation failed");
                ApiError::InvalidCommunicationQuery("email template operation failed")
            }
        }
    }
}

impl From<crate::domains::mail::delivery_notifications::MailDeliveryNotificationError>
    for ApiError
{
    fn from(
        error: crate::domains::mail::delivery_notifications::MailDeliveryNotificationError,
    ) -> Self {
        match error {
            crate::domains::mail::delivery_notifications::MailDeliveryNotificationError::Invalid(_) => {
                ApiError::InvalidCommunicationQuery("invalid mail delivery notification request")
            }
            error => {
                tracing::error!(error = %error, "mail delivery notification operation failed");
                ApiError::InvalidCommunicationQuery("mail delivery notification operation failed")
            }
        }
    }
}

impl From<crate::domains::mail::finance::EmailFinanceError> for ApiError {
    fn from(error: crate::domains::mail::finance::EmailFinanceError) -> Self {
        tracing::error!(error = %error, "email finance operation failed");
        ApiError::InvalidCommunicationQuery("email finance operation failed")
    }
}

impl From<crate::domains::mail::analytics::EmailAnalyticsError> for ApiError {
    fn from(error: crate::domains::mail::analytics::EmailAnalyticsError) -> Self {
        match error {
            crate::domains::mail::analytics::EmailAnalyticsError::InvalidCursor => {
                ApiError::InvalidCommunicationQuery("invalid analytics cursor")
            }
            error => {
                tracing::error!(error = %error, "email analytics operation failed");
                ApiError::InvalidCommunicationQuery("email analytics operation failed")
            }
        }
    }
}

impl From<crate::domains::mail::personas::EmailPersonaError> for ApiError {
    fn from(error: crate::domains::mail::personas::EmailPersonaError) -> Self {
        tracing::error!(error = %error, "email persona operation failed");
        ApiError::InvalidCommunicationQuery("email persona operation failed")
    }
}

impl From<crate::domains::mail::search::IndexEmailError> for ApiError {
    fn from(error: crate::domains::mail::search::IndexEmailError) -> Self {
        tracing::error!(error = %error, "email search operation failed");
        ApiError::InvalidCommunicationQuery("email search operation failed")
    }
}

impl From<crate::domains::mail::flags::MessageFlagsError> for ApiError {
    fn from(error: crate::domains::mail::flags::MessageFlagsError) -> Self {
        tracing::error!(error = %error, "message flags operation failed");
        ApiError::InvalidCommunicationQuery("message flags operation failed")
    }
}

impl From<crate::domains::mail::subscriptions::SubscriptionError> for ApiError {
    fn from(error: crate::domains::mail::subscriptions::SubscriptionError) -> Self {
        match error {
            crate::domains::mail::subscriptions::SubscriptionError::InvalidCursor => {
                ApiError::InvalidCommunicationQuery("invalid subscription cursor")
            }
            error => {
                tracing::error!(error = %error, "subscriptions operation failed");
                ApiError::InvalidCommunicationQuery("subscriptions operation failed")
            }
        }
    }
}

impl From<crate::domains::mail::attachment_dedup::AttachmentDedupError> for ApiError {
    fn from(error: crate::domains::mail::attachment_dedup::AttachmentDedupError) -> Self {
        tracing::error!(error = %error, "attachment dedup operation failed");
        ApiError::InvalidCommunicationQuery("attachment dedup operation failed")
    }
}

impl From<crate::domains::mail::attachment_search::AttachmentSearchError> for ApiError {
    fn from(error: crate::domains::mail::attachment_search::AttachmentSearchError) -> Self {
        match error {
            crate::domains::mail::attachment_search::AttachmentSearchError::InvalidCursor => {
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

impl From<crate::domains::mail::legal::LegalDocumentError> for ApiError {
    fn from(error: crate::domains::mail::legal::LegalDocumentError) -> Self {
        tracing::error!(error = %error, "legal document operation failed");
        ApiError::InvalidCommunicationQuery("legal document operation failed")
    }
}

impl From<crate::domains::mail::export::EmailExportError> for ApiError {
    fn from(error: crate::domains::mail::export::EmailExportError) -> Self {
        match error {
            crate::domains::mail::export::EmailExportError::NotFound => {
                ApiError::CommunicationMessageNotFound
            }
            _ => {
                tracing::error!(error = %error, "email export failed");
                ApiError::InvalidCommunicationQuery("email export failed")
            }
        }
    }
}

impl From<crate::domains::mail::send::EmailSendError> for ApiError {
    fn from(error: crate::domains::mail::send::EmailSendError) -> Self {
        tracing::error!(error = %error, "email send failed");
        ApiError::InvalidCommunicationQuery("email send failed")
    }
}

impl From<crate::domains::mail::imap_write::ImapWriteError> for ApiError {
    fn from(error: crate::domains::mail::imap_write::ImapWriteError) -> Self {
        tracing::error!(error = %error, "IMAP write operation failed");
        ApiError::InvalidCommunicationQuery("IMAP write operation failed")
    }
}

impl From<crate::domains::mail::signatures::CertificateError> for ApiError {
    fn from(error: crate::domains::mail::signatures::CertificateError) -> Self {
        tracing::error!(error = %error, "certificate operation failed");
        ApiError::InvalidCommunicationQuery("certificate operation failed")
    }
}

impl From<crate::domains::mail::multilingual::MultilingualError> for ApiError {
    fn from(error: crate::domains::mail::multilingual::MultilingualError) -> Self {
        tracing::error!(error = %error, "multilingual operation failed");
        ApiError::InvalidCommunicationQuery("multilingual operation failed")
    }
}

impl From<crate::domains::mail::ai_reply::AiReplyError> for ApiError {
    fn from(error: crate::domains::mail::ai_reply::AiReplyError) -> Self {
        tracing::error!(error = %error, "AI reply generation failed");
        ApiError::InvalidCommunicationQuery("AI reply generation failed")
    }
}

impl From<crate::domains::mail::extract::ExtractError> for ApiError {
    fn from(error: crate::domains::mail::extract::ExtractError) -> Self {
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
