use serde::{Deserialize, Serialize};

use crate::integrations::telegram::client::models::messages::TelegramManualSendResponse;

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct CommunicationConversationMessageRequest {
    pub(crate) account_id: String,
    pub(crate) text: String,
    pub(crate) command_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct CommunicationReplyRequest {
    pub(crate) text: String,
    pub(crate) command_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct CommunicationForwardRequest {
    pub(crate) conversation_id: String,
    pub(crate) command_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct TelegramMessageMarkReadRequest {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct TelegramMessageMarkReadResponse {
    pub(crate) telegram_chat_id: String,
    pub(crate) action: String,
    pub(crate) status: String,
    pub(crate) metadata: serde_json::Value,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct CommunicationProviderMessageCommandResponse {
    pub(crate) message_id: String,
    pub(crate) raw_record_id: String,
    pub(crate) conversation_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) provider_message_id: Option<String>,
    pub(crate) channel_kind: &'static str,
    pub(crate) status: String,
    pub(crate) command_id: String,
    pub(crate) provider: &'static str,
}

impl CommunicationProviderMessageCommandResponse {
    pub(crate) fn telegram(command_id: String, response: &TelegramManualSendResponse) -> Self {
        Self {
            message_id: response.message_id.clone(),
            raw_record_id: response.raw_record_id.clone(),
            conversation_id: response.provider_chat_id.clone(),
            provider_chat_id: response.provider_chat_id.clone(),
            provider_message_id: None,
            channel_kind: "telegram",
            status: response.status.clone(),
            command_id,
            provider: "telegram",
        }
    }
}
