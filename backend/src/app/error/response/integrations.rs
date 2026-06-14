mod automation;
mod call;
mod telegram;
mod whatsapp;

use crate::engines::automation::AutomationError;
use crate::integrations::telegram::client::TelegramError;
use crate::integrations::whatsapp::client::WhatsappWebError;
use crate::platform::calls::CallError;

use super::ErrorParts;

pub(super) fn telegram_error_parts(error: TelegramError) -> ErrorParts {
    telegram::telegram_error_parts(error)
}

pub(super) fn whatsapp_web_error_parts(error: WhatsappWebError) -> ErrorParts {
    whatsapp::whatsapp_web_error_parts(error)
}

pub(super) fn automation_error_parts(error: AutomationError) -> ErrorParts {
    automation::automation_error_parts(error)
}

pub(super) fn call_error_parts(error: CallError) -> ErrorParts {
    call::call_error_parts(error)
}
