use serde_json::{Value, json};
use thiserror::Error;

use super::client::{
    ZulipApiClient, ZulipClientError, ZulipReactionRequest, ZulipUpdateMessageRequest,
};
use super::models::{ZulipBasicResponse, ZulipSendMessageResponse, ZulipUploadFileResponse};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipExecutableCommand {
    pub command_id: String,
    pub command_kind: String,
    pub provider_message_id: Option<String>,
    pub payload: Value,
    pub prepared_upload: Option<ZulipPreparedUpload>,
}

impl ZulipExecutableCommand {
    pub fn new(
        command_id: impl Into<String>,
        command_kind: impl Into<String>,
        provider_message_id: Option<String>,
        payload: Value,
    ) -> Self {
        Self {
            command_id: command_id.into(),
            command_kind: command_kind.into(),
            provider_message_id,
            payload,
            prepared_upload: None,
        }
    }

    pub fn prepared_upload(mut self, prepared_upload: ZulipPreparedUpload) -> Self {
        self.prepared_upload = Some(prepared_upload);
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipPreparedUpload {
    pub filename: String,
    pub bytes: Vec<u8>,
    pub attachment_id: Option<String>,
    pub blob_id: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipCommandExecutionOutcome {
    pub result_payload: Value,
}

#[async_trait::async_trait]
pub trait ZulipCommandTransport: Send + Sync {
    async fn send_stream_message(
        &self,
        stream: &str,
        topic: &str,
        content: &str,
    ) -> Result<ZulipSendMessageResponse, ZulipClientError>;

    async fn send_direct_message(
        &self,
        recipients: &[&str],
        content: &str,
    ) -> Result<ZulipSendMessageResponse, ZulipClientError>;

    async fn send_direct_message_to_user_ids(
        &self,
        recipient_user_ids: &[i64],
        content: &str,
    ) -> Result<ZulipSendMessageResponse, ZulipClientError>;

    async fn update_message(
        &self,
        message_id: i64,
        request: &ZulipUpdateMessageRequest,
    ) -> Result<ZulipBasicResponse, ZulipClientError>;

    async fn delete_message(&self, message_id: i64)
    -> Result<ZulipBasicResponse, ZulipClientError>;

    async fn add_reaction(
        &self,
        message_id: i64,
        reaction: &ZulipReactionRequest,
    ) -> Result<ZulipBasicResponse, ZulipClientError>;

    async fn remove_reaction(
        &self,
        message_id: i64,
        reaction: &ZulipReactionRequest,
    ) -> Result<ZulipBasicResponse, ZulipClientError>;

    async fn upload_file_bytes(
        &self,
        filename: &str,
        bytes: Vec<u8>,
    ) -> Result<ZulipUploadFileResponse, ZulipClientError>;
}

#[async_trait::async_trait]
impl ZulipCommandTransport for ZulipApiClient {
    async fn send_stream_message(
        &self,
        stream: &str,
        topic: &str,
        content: &str,
    ) -> Result<ZulipSendMessageResponse, ZulipClientError> {
        ZulipApiClient::send_stream_message(self, stream, topic, content).await
    }

    async fn send_direct_message(
        &self,
        recipients: &[&str],
        content: &str,
    ) -> Result<ZulipSendMessageResponse, ZulipClientError> {
        ZulipApiClient::send_direct_message(self, recipients, content).await
    }

    async fn send_direct_message_to_user_ids(
        &self,
        recipient_user_ids: &[i64],
        content: &str,
    ) -> Result<ZulipSendMessageResponse, ZulipClientError> {
        ZulipApiClient::send_direct_message_to_user_ids(self, recipient_user_ids, content).await
    }

    async fn update_message(
        &self,
        message_id: i64,
        request: &ZulipUpdateMessageRequest,
    ) -> Result<ZulipBasicResponse, ZulipClientError> {
        ZulipApiClient::update_message(self, message_id, request).await
    }

    async fn delete_message(
        &self,
        message_id: i64,
    ) -> Result<ZulipBasicResponse, ZulipClientError> {
        ZulipApiClient::delete_message(self, message_id).await
    }

    async fn add_reaction(
        &self,
        message_id: i64,
        reaction: &ZulipReactionRequest,
    ) -> Result<ZulipBasicResponse, ZulipClientError> {
        ZulipApiClient::add_reaction(self, message_id, reaction).await
    }

    async fn remove_reaction(
        &self,
        message_id: i64,
        reaction: &ZulipReactionRequest,
    ) -> Result<ZulipBasicResponse, ZulipClientError> {
        ZulipApiClient::remove_reaction(self, message_id, reaction).await
    }

    async fn upload_file_bytes(
        &self,
        filename: &str,
        bytes: Vec<u8>,
    ) -> Result<ZulipUploadFileResponse, ZulipClientError> {
        ZulipApiClient::upload_file_bytes(self, filename, bytes).await
    }
}

pub async fn execute_zulip_command(
    transport: &impl ZulipCommandTransport,
    command: &ZulipExecutableCommand,
) -> Result<ZulipCommandExecutionOutcome, ZulipCommandExecutionError> {
    match command.command_kind.as_str() {
        "send_stream_message" => {
            let response = transport
                .send_stream_message(
                    required_string(command, "stream")?,
                    required_string(command, "topic")?,
                    required_string(command, "content")?,
                )
                .await
                .map_err(|error| execution_error_from_client(command, error))?;
            Ok(message_outcome(response.id))
        }
        "send_direct_message" => {
            let recipients = required_string_array(command, "recipients")?;
            let response =
                send_direct_message(transport, &recipients, required_string(command, "content")?)
                    .await
                    .map_err(|error| execution_error_from_client(command, error))?;
            Ok(message_outcome(response.id))
        }
        "upload_file" => {
            let upload = prepared_upload(command)?;
            let response = transport
                .upload_file_bytes(&upload.filename, upload.bytes.clone())
                .await
                .map_err(|error| execution_error_from_client(command, error))?;
            Ok(upload_outcome(&response.uri, upload))
        }
        "send_stream_message_with_upload" => {
            let upload = prepared_upload(command)?;
            let upload_response = transport
                .upload_file_bytes(&upload.filename, upload.bytes.clone())
                .await
                .map_err(|error| execution_error_from_client(command, error))?;
            let content =
                content_with_upload_uri(required_string(command, "content")?, &upload_response.uri);
            let response = transport
                .send_stream_message(
                    required_string(command, "stream")?,
                    required_string(command, "topic")?,
                    &content,
                )
                .await
                .map_err(|error| execution_error_from_client(command, error))?;
            Ok(message_with_upload_outcome(
                response.id,
                &upload_response.uri,
                upload,
            ))
        }
        "send_direct_message_with_upload" => {
            let upload = prepared_upload(command)?;
            let recipients = required_string_array(command, "recipients")?;
            let upload_response = transport
                .upload_file_bytes(&upload.filename, upload.bytes.clone())
                .await
                .map_err(|error| execution_error_from_client(command, error))?;
            let content =
                content_with_upload_uri(required_string(command, "content")?, &upload_response.uri);
            let response = send_direct_message(transport, &recipients, &content)
                .await
                .map_err(|error| execution_error_from_client(command, error))?;
            Ok(message_with_upload_outcome(
                response.id,
                &upload_response.uri,
                upload,
            ))
        }
        "update_message" => {
            let message_id = provider_message_id(command)?;
            let request = update_message_request(command)?;
            transport
                .update_message(message_id, &request)
                .await
                .map_err(|error| execution_error_from_client(command, error))?;
            Ok(basic_outcome())
        }
        "delete_message" => {
            let message_id = provider_message_id(command)?;
            transport
                .delete_message(message_id)
                .await
                .map_err(|error| execution_error_from_client(command, error))?;
            Ok(basic_outcome())
        }
        "add_reaction" => {
            let message_id = provider_message_id(command)?;
            let reaction = reaction_request(command)?;
            transport
                .add_reaction(message_id, &reaction)
                .await
                .map_err(|error| execution_error_from_client(command, error))?;
            Ok(basic_outcome())
        }
        "remove_reaction" => {
            let message_id = provider_message_id(command)?;
            let reaction = reaction_request(command)?;
            transport
                .remove_reaction(message_id, &reaction)
                .await
                .map_err(|error| execution_error_from_client(command, error))?;
            Ok(basic_outcome())
        }
        _ => Err(ZulipCommandExecutionError::InvalidCommand {
            command_id: command.command_id.clone(),
            reason: "unsupported Zulip command kind".to_owned(),
        }),
    }
}

async fn send_direct_message(
    transport: &impl ZulipCommandTransport,
    recipients: &[String],
    content: &str,
) -> Result<ZulipSendMessageResponse, ZulipClientError> {
    if let Some(user_ids) = numeric_recipient_user_ids(recipients) {
        transport
            .send_direct_message_to_user_ids(&user_ids, content)
            .await
    } else {
        let recipient_refs = recipients.iter().map(String::as_str).collect::<Vec<_>>();
        transport
            .send_direct_message(&recipient_refs, content)
            .await
    }
}

fn numeric_recipient_user_ids(recipients: &[String]) -> Option<Vec<i64>> {
    recipients
        .iter()
        .map(|recipient| recipient.parse::<i64>().ok())
        .collect()
}

fn prepared_upload(
    command: &ZulipExecutableCommand,
) -> Result<&ZulipPreparedUpload, ZulipCommandExecutionError> {
    command.prepared_upload.as_ref().ok_or_else(|| {
        invalid(
            command,
            "provider command requires a prepared local attachment upload".to_owned(),
        )
    })
}

fn message_outcome(provider_message_id: Option<i64>) -> ZulipCommandExecutionOutcome {
    ZulipCommandExecutionOutcome {
        result_payload: json!({
            "provider": "zulip",
            "result": "success",
            "provider_message_id": provider_message_id,
        }),
    }
}

fn upload_outcome(upload_uri: &str, upload: &ZulipPreparedUpload) -> ZulipCommandExecutionOutcome {
    ZulipCommandExecutionOutcome {
        result_payload: json!({
            "provider": "zulip",
            "result": "success",
            "upload_uri": upload_uri,
            "attachment_id": upload.attachment_id,
            "blob_id": upload.blob_id,
            "filename": upload.filename,
            "content_type": upload.content_type,
            "size_bytes": upload.size_bytes,
            "sha256": upload.sha256,
        }),
    }
}

fn message_with_upload_outcome(
    provider_message_id: Option<i64>,
    upload_uri: &str,
    upload: &ZulipPreparedUpload,
) -> ZulipCommandExecutionOutcome {
    let mut outcome = upload_outcome(upload_uri, upload).result_payload;
    outcome["provider_message_id"] = json!(provider_message_id);
    ZulipCommandExecutionOutcome {
        result_payload: outcome,
    }
}

fn content_with_upload_uri(content: &str, upload_uri: &str) -> String {
    format!("{}\n{}", content.trim(), upload_uri.trim())
}

fn basic_outcome() -> ZulipCommandExecutionOutcome {
    ZulipCommandExecutionOutcome {
        result_payload: json!({
            "provider": "zulip",
            "result": "success",
        }),
    }
}

fn required_string<'a>(
    command: &'a ZulipExecutableCommand,
    field: &'static str,
) -> Result<&'a str, ZulipCommandExecutionError> {
    command
        .payload
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| invalid(command, format!("payload field `{field}` must be a string")))
}

fn optional_string<'a>(payload: &'a Value, field: &'static str) -> Option<&'a str> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn required_string_array(
    command: &ZulipExecutableCommand,
    field: &'static str,
) -> Result<Vec<String>, ZulipCommandExecutionError> {
    let values = command
        .payload
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| invalid(command, format!("payload field `{field}` must be an array")))?;
    let recipients = values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .ok_or_else(|| {
                    invalid(
                        command,
                        format!("payload field `{field}` must contain only non-empty strings"),
                    )
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if recipients.is_empty() {
        return Err(invalid(
            command,
            format!("payload field `{field}` must not be empty"),
        ));
    }
    Ok(recipients)
}

fn provider_message_id(
    command: &ZulipExecutableCommand,
) -> Result<i64, ZulipCommandExecutionError> {
    if let Some(message_id) = command
        .provider_message_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return message_id
            .parse::<i64>()
            .map_err(|_| invalid(command, "provider_message_id must be an integer".to_owned()));
    }

    if let Some(message_id) = command.payload.get("message_id").and_then(Value::as_i64) {
        return Ok(message_id);
    }
    if let Some(message_id) = optional_string(&command.payload, "message_id") {
        return message_id.parse::<i64>().map_err(|_| {
            invalid(
                command,
                "payload field `message_id` must be an integer".to_owned(),
            )
        });
    }

    Err(invalid(
        command,
        "provider_message_id or payload field `message_id` is required".to_owned(),
    ))
}

fn update_message_request(
    command: &ZulipExecutableCommand,
) -> Result<ZulipUpdateMessageRequest, ZulipCommandExecutionError> {
    let mut request = ZulipUpdateMessageRequest::new();
    let mut has_update = false;
    if let Some(content) = optional_string(&command.payload, "content") {
        request = request.content(content);
        has_update = true;
    }
    if let Some(topic) = optional_string(&command.payload, "topic") {
        request = request.topic(topic);
        has_update = true;
    }
    if let Some(stream_id) = command.payload.get("stream_id").and_then(Value::as_i64) {
        request = request.stream_id(stream_id);
        has_update = true;
    }
    if let Some(propagate_mode) = optional_string(&command.payload, "propagate_mode") {
        request = request.propagate_mode(propagate_mode);
        has_update = true;
    }
    if !has_update {
        return Err(invalid(
            command,
            "message update command must include at least one update field".to_owned(),
        ));
    }
    Ok(request)
}

fn reaction_request(
    command: &ZulipExecutableCommand,
) -> Result<ZulipReactionRequest, ZulipCommandExecutionError> {
    let mut reaction = ZulipReactionRequest::new(required_string(command, "emoji_name")?);
    if let Some(emoji_code) = optional_string(&command.payload, "emoji_code") {
        reaction = reaction.emoji_code(emoji_code);
    }
    if let Some(reaction_type) = optional_string(&command.payload, "reaction_type") {
        reaction = reaction.reaction_type(reaction_type);
    }
    Ok(reaction)
}

fn execution_error_from_client(
    command: &ZulipExecutableCommand,
    error: ZulipClientError,
) -> ZulipCommandExecutionError {
    match error {
        ZulipClientError::Api { status, .. } => ZulipCommandExecutionError::ProviderApi { status },
        ZulipClientError::InvalidRequest(_) => ZulipCommandExecutionError::InvalidCommand {
            command_id: command.command_id.clone(),
            reason: "provider command payload is invalid".to_owned(),
        },
        ZulipClientError::Json(_) => ZulipCommandExecutionError::InvalidProviderResponse,
        ZulipClientError::Http(_) => ZulipCommandExecutionError::Transport,
        ZulipClientError::Url(_) => ZulipCommandExecutionError::InvalidClientConfiguration,
    }
}

fn invalid(command: &ZulipExecutableCommand, reason: String) -> ZulipCommandExecutionError {
    ZulipCommandExecutionError::InvalidCommand {
        command_id: command.command_id.clone(),
        reason,
    }
}

#[derive(Debug, Error)]
pub enum ZulipCommandExecutionError {
    #[error("invalid Zulip command `{command_id}`: {reason}")]
    InvalidCommand { command_id: String, reason: String },
    #[error("Zulip API returned HTTP {status}")]
    ProviderApi { status: u16 },
    #[error("Zulip HTTP request failed")]
    Transport,
    #[error("Zulip API response was invalid")]
    InvalidProviderResponse,
    #[error("Zulip client configuration is invalid")]
    InvalidClientConfiguration,
}

impl ZulipCommandExecutionError {
    pub fn error_kind(&self) -> &'static str {
        match self {
            Self::InvalidCommand { .. } => "invalid_command",
            Self::ProviderApi { .. } => "provider_api",
            Self::Transport => "transport",
            Self::InvalidProviderResponse => "invalid_provider_response",
            Self::InvalidClientConfiguration => "invalid_client_configuration",
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    #[derive(Default)]
    struct RecordingTransport {
        direct_recipients: Mutex<Vec<Vec<String>>>,
        direct_user_ids: Mutex<Vec<Vec<i64>>>,
    }

    #[async_trait::async_trait]
    impl ZulipCommandTransport for RecordingTransport {
        async fn send_stream_message(
            &self,
            _stream: &str,
            _topic: &str,
            _content: &str,
        ) -> Result<ZulipSendMessageResponse, ZulipClientError> {
            panic!("stream messages are not used by these tests")
        }

        async fn send_direct_message(
            &self,
            recipients: &[&str],
            _content: &str,
        ) -> Result<ZulipSendMessageResponse, ZulipClientError> {
            self.direct_recipients
                .lock()
                .expect("direct recipient lock")
                .push(
                    recipients
                        .iter()
                        .map(|recipient| (*recipient).to_owned())
                        .collect(),
                );
            Ok(ZulipSendMessageResponse {
                result: "success".to_owned(),
                msg: String::new(),
                id: Some(8801),
            })
        }

        async fn send_direct_message_to_user_ids(
            &self,
            recipient_user_ids: &[i64],
            _content: &str,
        ) -> Result<ZulipSendMessageResponse, ZulipClientError> {
            self.direct_user_ids
                .lock()
                .expect("direct user id lock")
                .push(recipient_user_ids.to_vec());
            Ok(ZulipSendMessageResponse {
                result: "success".to_owned(),
                msg: String::new(),
                id: Some(9901),
            })
        }

        async fn update_message(
            &self,
            _message_id: i64,
            _request: &ZulipUpdateMessageRequest,
        ) -> Result<ZulipBasicResponse, ZulipClientError> {
            panic!("message updates are not used by these tests")
        }

        async fn delete_message(
            &self,
            _message_id: i64,
        ) -> Result<ZulipBasicResponse, ZulipClientError> {
            panic!("message deletes are not used by these tests")
        }

        async fn add_reaction(
            &self,
            _message_id: i64,
            _reaction: &ZulipReactionRequest,
        ) -> Result<ZulipBasicResponse, ZulipClientError> {
            panic!("reactions are not used by these tests")
        }

        async fn remove_reaction(
            &self,
            _message_id: i64,
            _reaction: &ZulipReactionRequest,
        ) -> Result<ZulipBasicResponse, ZulipClientError> {
            panic!("reactions are not used by these tests")
        }

        async fn upload_file_bytes(
            &self,
            _filename: &str,
            _bytes: Vec<u8>,
        ) -> Result<ZulipUploadFileResponse, ZulipClientError> {
            panic!("uploads are not used by these tests")
        }
    }

    #[tokio::test]
    async fn direct_command_uses_user_id_recipient_payload_when_all_recipients_are_numeric() {
        let transport = RecordingTransport::default();
        let command = ZulipExecutableCommand::new(
            "zulip-direct-user-id",
            "send_direct_message",
            None,
            json!({
                "recipients": ["101", "202"],
                "content": "Direct by user id"
            }),
        );

        let outcome = execute_zulip_command(&transport, &command)
            .await
            .expect("execute direct user id command");

        assert_eq!(outcome.result_payload["provider_message_id"], json!(9901));
        assert_eq!(
            transport
                .direct_user_ids
                .lock()
                .expect("direct user id calls")
                .as_slice(),
            &[vec![101, 202]]
        );
        assert!(
            transport
                .direct_recipients
                .lock()
                .expect("direct recipient calls")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn direct_command_keeps_email_recipient_payload_for_non_numeric_recipients() {
        let transport = RecordingTransport::default();
        let command = ZulipExecutableCommand::new(
            "zulip-direct-email",
            "send_direct_message",
            None,
            json!({
                "recipients": ["alice@example.test"],
                "content": "Direct by email"
            }),
        );

        let outcome = execute_zulip_command(&transport, &command)
            .await
            .expect("execute direct email command");

        assert_eq!(outcome.result_payload["provider_message_id"], json!(8801));
        assert_eq!(
            transport
                .direct_recipients
                .lock()
                .expect("direct recipient calls")
                .as_slice(),
            &[vec!["alice@example.test".to_owned()]]
        );
        assert!(
            transport
                .direct_user_ids
                .lock()
                .expect("direct user id calls")
                .is_empty()
        );
    }
}
