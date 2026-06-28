mod automation;
mod call;
mod telegram;
mod whatsapp;
mod yandex_telemost;
mod zoom;

use crate::application::provider_runtime_contracts::{
    TelegramError, WhatsappWebError, YandexTelemostError, ZoomError,
};
use crate::engines::automation::AutomationError;
use crate::platform::calls::CallError;

use super::ErrorParts;

pub(super) fn telegram_error_parts(error: TelegramError) -> ErrorParts {
    telegram::telegram_error_parts(error)
}

pub(super) fn whatsapp_web_error_parts(error: WhatsappWebError) -> ErrorParts {
    whatsapp::whatsapp_web_error_parts(error)
}

pub(super) fn zoom_error_parts(error: ZoomError) -> ErrorParts {
    zoom::zoom_error_parts(error)
}

pub(super) fn yandex_telemost_error_parts(error: YandexTelemostError) -> ErrorParts {
    yandex_telemost::yandex_telemost_error_parts(error)
}

pub(super) fn automation_error_parts(error: AutomationError) -> ErrorParts {
    automation::automation_error_parts(error)
}

pub(super) fn call_error_parts(error: CallError) -> ErrorParts {
    call::call_error_parts(error)
}
