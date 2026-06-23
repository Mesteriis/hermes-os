use super::super::types::ApiError;
use crate::application::communication_fixture_ingest::CommunicationFixtureIngestError;
use crate::application::communication_provider_writes::TelegramMessageWriteError;
use crate::application::provider_runtime_contracts::{TelegramError, WhatsappWebError};
use crate::application::review_inbox::ReviewInboxWorkflowError;
use crate::engines::automation::AutomationError;
use crate::platform::calls::CallError;

impl From<TelegramError> for ApiError {
    fn from(error: TelegramError) -> Self {
        Self::Telegram(error)
    }
}

impl From<WhatsappWebError> for ApiError {
    fn from(error: WhatsappWebError) -> Self {
        Self::WhatsappWeb(error)
    }
}

impl From<AutomationError> for ApiError {
    fn from(error: AutomationError) -> Self {
        Self::Automation(error)
    }
}

impl From<CallError> for ApiError {
    fn from(error: CallError) -> Self {
        Self::Call(error)
    }
}

impl From<CommunicationFixtureIngestError> for ApiError {
    fn from(error: CommunicationFixtureIngestError) -> Self {
        tracing::error!(error = %error, "communication fixture ingest failed");
        Self::InvalidCommunicationQuery("communication fixture ingest failed")
    }
}

impl From<TelegramMessageWriteError> for ApiError {
    fn from(error: TelegramMessageWriteError) -> Self {
        tracing::error!(error = %error, "telegram message write failed");
        Self::InvalidCommunicationQuery("telegram message write failed")
    }
}

impl From<ReviewInboxWorkflowError> for ApiError {
    fn from(error: ReviewInboxWorkflowError) -> Self {
        tracing::error!(error = %error, "communication review inbox sync failed");
        Self::InvalidCommunicationQuery("communication review inbox sync failed")
    }
}
