use super::super::types::ApiError;
use crate::domains::mail::accounts::EmailAccountSetupError;
use crate::domains::mail::core::CommunicationIngestionError;
use crate::domains::mail::messages::MessageProjectionError;
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
        tracing::error!(error = %error, "email thread operation failed");
        ApiError::InvalidCommunicationQuery("email thread operation failed")
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
        tracing::error!(error = %error, "email draft operation failed");
        ApiError::InvalidCommunicationQuery("email draft operation failed")
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
        tracing::error!(error = %error, "email analytics operation failed");
        ApiError::InvalidCommunicationQuery("email analytics operation failed")
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
        tracing::error!(error = %error, "subscriptions operation failed");
        ApiError::InvalidCommunicationQuery("subscriptions operation failed")
    }
}

impl From<crate::domains::mail::attachment_dedup::AttachmentDedupError> for ApiError {
    fn from(error: crate::domains::mail::attachment_dedup::AttachmentDedupError) -> Self {
        tracing::error!(error = %error, "attachment dedup operation failed");
        ApiError::InvalidCommunicationQuery("attachment dedup operation failed")
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
