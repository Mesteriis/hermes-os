use super::super::types::ApiError;
use crate::application::provider_communication_projection::ProviderCommunicationProjectionError;
use crate::application::review_inbox::ReviewInboxWorkflowError;
use crate::engines::automation::AutomationError;
use crate::integrations::telegram::client::TelegramError;
use crate::integrations::whatsapp::client::WhatsappWebError;
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

impl From<ProviderCommunicationProjectionError> for ApiError {
    fn from(error: ProviderCommunicationProjectionError) -> Self {
        tracing::error!(error = %error, "provider communication projection failed");
        Self::InvalidCommunicationQuery("provider communication projection failed")
    }
}

impl From<ReviewInboxWorkflowError> for ApiError {
    fn from(error: ReviewInboxWorkflowError) -> Self {
        tracing::error!(error = %error, "communication review inbox sync failed");
        Self::InvalidCommunicationQuery("communication review inbox sync failed")
    }
}
