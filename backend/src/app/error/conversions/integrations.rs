use super::super::types::ApiError;
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
