use super::super::types::ApiError;
use crate::application::communication_fixture_ingest::CommunicationFixtureIngestError;
use crate::application::communication_provider_writes::TelegramMessageWriteError;
use crate::engines::automation::errors::AutomationError;
use crate::integrations::telegram::client::TelegramError;
use crate::integrations::whatsapp::client::errors::WhatsappWebError;
use crate::integrations::yandex_telemost::client::errors::YandexTelemostError;
use crate::integrations::zoom::client::errors::ZoomError;
use crate::platform::calls::CallError;
use crate::workflows::review_inbox::ReviewInboxWorkflowError;

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

impl From<ZoomError> for ApiError {
    fn from(error: ZoomError) -> Self {
        Self::Zoom(error)
    }
}

impl From<YandexTelemostError> for ApiError {
    fn from(error: YandexTelemostError) -> Self {
        Self::YandexTelemost(error)
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
    fn from(_error: CommunicationFixtureIngestError) -> Self {
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
