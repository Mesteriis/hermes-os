use serde::Serialize;
use serde_json::json;
use thiserror::Error;

use crate::application::communication_fixture_ingest::{
    build_event, telegram_message_snapshot_payload,
};
use crate::application::provider_communication_projection::{
    ProviderCommunicationProjectionError, record_and_project_telegram_message,
};
use crate::application::telegram_runtime::{self, TelegramRuntimeUseCaseContext};
use crate::integrations::telegram::client::lifecycle;
use crate::integrations::telegram::client::models::messages::{
    TelegramForwardRequest, TelegramManualSendRequest, TelegramManualSendResponse,
    TelegramReplyRequest,
};
use crate::integrations::telegram::client::{TelegramError, TelegramStore};
use crate::platform::audit::{ApiAuditError, ApiAuditLog, NewApiAuditRecord};
use crate::platform::events::bus::telegram_event_types;
use crate::platform::events::{EventBus, EventStore, NewEventEnvelope};

const AUDIT_ACTOR_ID: &str = "hermes-frontend";

#[derive(Clone)]
pub(crate) struct TelegramMessageWriteApplicationService {
    store: TelegramStore,
    audit_log: ApiAuditLog,
    event_store: EventStore,
    event_bus: EventBus,
}

impl TelegramMessageWriteApplicationService {
    pub(crate) fn new(
        store: TelegramStore,
        audit_log: ApiAuditLog,
        event_store: EventStore,
        event_bus: EventBus,
    ) -> Self {
        Self {
            store,
            audit_log,
            event_store,
            event_bus,
        }
    }

    pub(crate) async fn send_manual_message(
        &self,
        context: &TelegramRuntimeUseCaseContext<'_>,
        request: &TelegramManualSendRequest,
    ) -> Result<TelegramManualSendResponse, TelegramMessageWriteError> {
        let mut response = telegram_runtime::send_manual_message(context, request).await?;
        self.finish_created_message(&mut response, &request.command_id, "send_text", json!({}))
            .await?;
        Ok(response)
    }

    pub(crate) async fn send_reply_message(
        &self,
        context: &TelegramRuntimeUseCaseContext<'_>,
        request: &TelegramReplyRequest,
    ) -> Result<TelegramManualSendResponse, TelegramMessageWriteError> {
        let mut response = telegram_runtime::send_reply_message(context, request).await?;
        self.finish_created_message(
            &mut response,
            &request.command_id,
            "reply",
            json!({
                "reply_to_provider_message_id": &request.reply_to_provider_message_id,
            }),
        )
        .await?;
        Ok(response)
    }

    pub(crate) async fn send_forward_message(
        &self,
        context: &TelegramRuntimeUseCaseContext<'_>,
        request: &TelegramForwardRequest,
    ) -> Result<TelegramManualSendResponse, TelegramMessageWriteError> {
        let mut response = telegram_runtime::send_forward_message(context, request).await?;
        self.finish_created_message(
            &mut response,
            &request.command_id,
            "forward",
            json!({
                "from_provider_chat_id": &request.from_provider_chat_id,
                "from_provider_message_id": &request.from_provider_message_id,
            }),
        )
        .await?;
        Ok(response)
    }

    pub(crate) async fn send_conversation_message(
        &self,
        context: &TelegramRuntimeUseCaseContext<'_>,
        conversation_id: &str,
        request: CommunicationConversationMessageRequest,
    ) -> Result<TelegramManualSendResponse, TelegramMessageWriteError> {
        let command = TelegramManualSendRequest {
            command_id: request.command_id.unwrap_or_else(lifecycle::new_command_id),
            account_id: request.account_id,
            provider_chat_id: conversation_id.trim().to_owned(),
            text: request.text,
        };
        command.validate()?;
        self.send_manual_message(context, &command).await
    }

    pub(crate) async fn reply_to_message(
        &self,
        context: &TelegramRuntimeUseCaseContext<'_>,
        message_id: &str,
        request: CommunicationReplyRequest,
    ) -> Result<TelegramManualSendResponse, TelegramMessageWriteError> {
        let message = self.telegram_message(message_id).await?;
        let provider_chat_id = required_provider_chat_id(&message)?;
        let command = TelegramReplyRequest {
            command_id: request.command_id.unwrap_or_else(lifecycle::new_command_id),
            account_id: message.account_id.clone(),
            provider_chat_id,
            reply_to_provider_message_id: message.provider_message_id.clone(),
            text: request.text,
        };
        command.validate()?;
        self.send_reply_message(context, &command).await
    }

    pub(crate) async fn forward_message(
        &self,
        context: &TelegramRuntimeUseCaseContext<'_>,
        message_id: &str,
        request: CommunicationForwardRequest,
    ) -> Result<TelegramManualSendResponse, TelegramMessageWriteError> {
        let message = self.telegram_message(message_id).await?;
        let from_provider_chat_id = required_provider_chat_id(&message)?;
        let command = TelegramForwardRequest {
            command_id: request.command_id.unwrap_or_else(lifecycle::new_command_id),
            account_id: message.account_id.clone(),
            provider_chat_id: request.conversation_id.trim().to_owned(),
            from_provider_chat_id,
            from_provider_message_id: message.provider_message_id.clone(),
        };
        command.validate()?;
        self.send_forward_message(context, &command).await
    }

    async fn finish_created_message(
        &self,
        response: &mut TelegramManualSendResponse,
        command_id: &str,
        command_kind: &str,
        extra_payload: serde_json::Value,
    ) -> Result<(), TelegramMessageWriteError> {
        self.project_manual_send_response(response).await?;
        self.audit_log
            .record(&NewApiAuditRecord::telegram_message_send(
                AUDIT_ACTOR_ID,
                &response.message_id,
                &response.account_id,
                &response.provider_chat_id,
                &response.rendered_preview_hash,
            ))
            .await?;

        let event = build_event(
            telegram_event_types::MESSAGE_CREATED,
            &response.account_id,
            &response.message_id,
            telegram_message_snapshot_payload(
                &self.store,
                &response.message_id,
                merge_json_objects(
                    json!({
                        "provider_chat_id": &response.provider_chat_id,
                        "delivery_state": &response.delivery_state,
                        "runtime_kind": &response.runtime_kind,
                        "status": &response.status,
                    }),
                    extra_payload.clone(),
                ),
            )
            .await?,
        );
        self.publish_event(event).await;

        let command_event = build_command_event(CommandEventInput {
            account_id: &response.account_id,
            command_id,
            command_kind,
            provider_chat_id: &response.provider_chat_id,
            message_id: Some(&response.message_id),
            provider_message_id: None,
            status: &response.status,
            extra_payload: merge_json_objects(
                json!({
                    "telegram_message_id": &response.message_id,
                    "runtime_kind": &response.runtime_kind,
                }),
                extra_payload,
            ),
        });
        self.publish_event(command_event).await;
        Ok(())
    }

    async fn project_manual_send_response(
        &self,
        response: &mut TelegramManualSendResponse,
    ) -> Result<(), TelegramMessageWriteError> {
        let Some(raw) = response.raw.take() else {
            return Err(TelegramError::InvalidRequest(
                "Telegram send response is missing raw provider observation".to_owned(),
            )
            .into());
        };
        let projected = record_and_project_telegram_message(self.store.pool().clone(), raw).await?;
        response.raw_record_id = projected.raw_record_id;
        response.message_id = projected.message_id;
        Ok(())
    }

    async fn telegram_message(
        &self,
        message_id: &str,
    ) -> Result<crate::integrations::telegram::client::TelegramMessage, TelegramMessageWriteError>
    {
        self.store.message_by_id(message_id).await?.ok_or_else(|| {
            TelegramMessageWriteError::Telegram(TelegramError::InvalidRequest(format!(
                "Telegram message `{message_id}` was not found"
            )))
        })
    }

    async fn publish_event(&self, event: NewEventEnvelope) {
        if let Err(error) = self.event_store.append(&event).await {
            tracing::warn!(error = %error, "failed to append Telegram message write event");
        }
        let _ = self.event_bus.broadcast(event);
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub(crate) struct CommunicationConversationMessageRequest {
    pub(crate) account_id: String,
    pub(crate) text: String,
    pub(crate) command_id: Option<String>,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub(crate) struct CommunicationReplyRequest {
    pub(crate) text: String,
    pub(crate) command_id: Option<String>,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub(crate) struct CommunicationForwardRequest {
    pub(crate) conversation_id: String,
    pub(crate) command_id: Option<String>,
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

#[derive(Debug, Error)]
pub(crate) enum TelegramMessageWriteError {
    #[error(transparent)]
    Telegram(#[from] TelegramError),

    #[error(transparent)]
    Projection(#[from] ProviderCommunicationProjectionError),

    #[error(transparent)]
    Audit(#[from] ApiAuditError),
}

fn required_provider_chat_id(
    message: &crate::integrations::telegram::client::TelegramMessage,
) -> Result<String, TelegramMessageWriteError> {
    message.provider_chat_id.clone().ok_or_else(|| {
        TelegramError::InvalidRequest(
            "Telegram message does not include provider chat id".to_owned(),
        )
        .into()
    })
}

struct CommandEventInput<'a> {
    account_id: &'a str,
    command_id: &'a str,
    command_kind: &'a str,
    provider_chat_id: &'a str,
    message_id: Option<&'a str>,
    provider_message_id: Option<&'a str>,
    status: &'a str,
    extra_payload: serde_json::Value,
}

fn build_command_event(input: CommandEventInput<'_>) -> NewEventEnvelope {
    let mut payload = json!({
        "account_id": input.account_id,
        "command_id": input.command_id,
        "command_kind": input.command_kind,
        "action": input.command_kind,
        "provider_chat_id": input.provider_chat_id,
        "message_id": input.message_id,
        "provider_message_id": input.provider_message_id,
        "status": input.status,
    });
    payload = merge_json_objects(payload, input.extra_payload.clone());
    if let Some(payload_obj) = payload.as_object_mut() {
        payload_obj.insert("payload".to_owned(), input.extra_payload);
    }
    build_event(
        telegram_event_types::COMMAND_STATUS_CHANGED,
        input.account_id,
        input.command_id,
        payload,
    )
}

fn merge_json_objects(mut base: serde_json::Value, extra: serde_json::Value) -> serde_json::Value {
    if let (Some(base_obj), Some(extra_obj)) = (base.as_object_mut(), extra.as_object()) {
        for (key, value) in extra_obj {
            base_obj.insert(key.clone(), value.clone());
        }
    }
    base
}
