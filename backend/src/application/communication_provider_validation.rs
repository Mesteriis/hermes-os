use crate::application::communication_provider_error::TelegramMessageWriteError;
use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::messages::TelegramMessage;

pub(crate) fn required_provider_chat_id(
    message: &TelegramMessage,
) -> Result<String, TelegramMessageWriteError> {
    message.provider_chat_id.clone().ok_or_else(|| {
        TelegramError::InvalidRequest(
            "Telegram message does not include provider chat id".to_owned(),
        )
        .into()
    })
}
