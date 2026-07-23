//! Generated Zulip operational client payload adapters.

use prost::Message;

use crate::{ZulipBlobIntentV1, ZulipClientRequestV1, ZulipClientResponseV1, ZulipCommandOperationOutcomeV1, ZulipCommandOperationStatusV1, ZulipCommandReceiptV1, ZulipCommandV1, ZulipReactionOperationV1, ZulipReactionV1, wire};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZulipClientWireErrorV1 { InvalidPayload, MissingVariant }

pub fn encode_request(request: &ZulipClientRequestV1) -> Vec<u8> {
    use wire::zulip_client_request_v1::Request;
    let request = match request {
        ZulipClientRequestV1::Command(command) => Request::Command(command_message(command)),
        ZulipClientRequestV1::OperationStatus { operation_id } => Request::OperationStatus(wire::ZulipOperationStatusQueryV1 { operation_id: operation_id.clone() }),
    };
    wire::ZulipClientRequestV1 { request: Some(request) }.encode_to_vec()
}

pub fn decode_request(bytes: &[u8]) -> Result<ZulipClientRequestV1, ZulipClientWireErrorV1> {
    use wire::zulip_client_request_v1::Request;
    let message = wire::ZulipClientRequestV1::decode(bytes).map_err(|_| ZulipClientWireErrorV1::InvalidPayload)?;
    match message.request.ok_or(ZulipClientWireErrorV1::MissingVariant)? {
        Request::Command(command) => decode_command_message(command).map(ZulipClientRequestV1::Command),
        Request::OperationStatus(query) => Ok(ZulipClientRequestV1::OperationStatus { operation_id: query.operation_id }),
    }
}

pub fn encode_command(command: &ZulipCommandV1) -> Vec<u8> { command_message(command).encode_to_vec() }

pub fn decode_command(bytes: &[u8]) -> Result<ZulipCommandV1, ZulipClientWireErrorV1> {
    wire::ZulipProviderCommandV1::decode(bytes).map_err(|_| ZulipClientWireErrorV1::InvalidPayload).and_then(decode_command_message)
}

pub fn encode_response(response: &ZulipClientResponseV1) -> Vec<u8> {
    use wire::zulip_client_response_v1::Response;
    let response = match response {
        ZulipClientResponseV1::CommandReceipt(receipt) => Response::CommandReceipt(wire::ZulipCommandReceiptV1 { operation_id: receipt.operation_id.clone(), account_id: receipt.account_id.clone() }),
        ZulipClientResponseV1::OperationStatus(status) => Response::OperationStatus(wire::ZulipOperationStatusResponseV1 { status: status.as_ref().map(status_message) }),
    };
    wire::ZulipClientResponseV1 { response: Some(response) }.encode_to_vec()
}

pub fn decode_response(bytes: &[u8]) -> Result<ZulipClientResponseV1, ZulipClientWireErrorV1> {
    use wire::zulip_client_response_v1::Response;
    let message = wire::ZulipClientResponseV1::decode(bytes).map_err(|_| ZulipClientWireErrorV1::InvalidPayload)?;
    match message.response.ok_or(ZulipClientWireErrorV1::MissingVariant)? {
        Response::CommandReceipt(value) => Ok(ZulipClientResponseV1::CommandReceipt(ZulipCommandReceiptV1 { operation_id: value.operation_id, account_id: value.account_id })),
        Response::OperationStatus(value) => value.status.map(decode_status_message).transpose().map(ZulipClientResponseV1::OperationStatus),
    }
}

fn command_message(command: &ZulipCommandV1) -> wire::ZulipProviderCommandV1 {
    use wire::zulip_provider_command_v1::Command;
    let command = match command {
        ZulipCommandV1::SendStream { operation_id, account_id, stream, topic, content } => Command::SendStream(wire::SendStreamCommandV1 { operation_id: operation_id.clone(), account_id: account_id.clone(), stream: stream.clone(), topic: topic.clone(), content: content.clone() }),
        ZulipCommandV1::SendDirect { operation_id, account_id, recipients, content } => Command::SendDirect(wire::SendDirectCommandV1 { operation_id: operation_id.clone(), account_id: account_id.clone(), recipient: recipients.clone(), content: content.clone() }),
        ZulipCommandV1::UpdateMessage { operation_id, account_id, provider_message_id, content, topic } => Command::UpdateMessage(wire::UpdateMessageCommandV1 { operation_id: operation_id.clone(), account_id: account_id.clone(), provider_message_id: provider_message_id.clone(), content: content.clone(), topic: topic.clone() }),
        ZulipCommandV1::DeleteMessage { operation_id, account_id, provider_message_id } => Command::DeleteMessage(wire::DeleteMessageCommandV1 { operation_id: operation_id.clone(), account_id: account_id.clone(), provider_message_id: provider_message_id.clone() }),
        ZulipCommandV1::Reaction { operation_id, account_id, provider_message_id, reaction, operation } => Command::Reaction(wire::ReactionCommandV1 { operation_id: operation_id.clone(), account_id: account_id.clone(), provider_message_id: provider_message_id.clone(), emoji_name: reaction.emoji_name.clone(), emoji_code: reaction.emoji_code.clone(), reaction_type: reaction.reaction_type.clone(), add: *operation == ZulipReactionOperationV1::Add }),
        ZulipCommandV1::SendStreamWithUpload { operation_id, account_id, stream, topic, content, blob, filename } => Command::SendStreamWithUpload(wire::SendStreamWithUploadCommandV1 { operation_id: operation_id.clone(), account_id: account_id.clone(), stream: stream.clone(), topic: topic.clone(), content: content.clone(), blob: Some(blob_message(blob)), filename: filename.clone() }),
        ZulipCommandV1::SendDirectWithUpload { operation_id, account_id, recipients, content, blob, filename } => Command::SendDirectWithUpload(wire::SendDirectWithUploadCommandV1 { operation_id: operation_id.clone(), account_id: account_id.clone(), recipient: recipients.clone(), content: content.clone(), blob: Some(blob_message(blob)), filename: filename.clone() }),
        ZulipCommandV1::DownloadAttachment { operation_id, account_id, upload_path, blob } => Command::DownloadAttachment(wire::DownloadAttachmentCommandV1 { operation_id: operation_id.clone(), account_id: account_id.clone(), upload_path: upload_path.clone(), blob: Some(blob_message(blob)) }),
    };
    wire::ZulipProviderCommandV1 { command: Some(command) }
}

fn decode_command_message(message: wire::ZulipProviderCommandV1) -> Result<ZulipCommandV1, ZulipClientWireErrorV1> {
    use wire::zulip_provider_command_v1::Command;
    match message.command.ok_or(ZulipClientWireErrorV1::MissingVariant)? {
        Command::SendStream(value) => Ok(ZulipCommandV1::SendStream { operation_id: value.operation_id, account_id: value.account_id, stream: value.stream, topic: value.topic, content: value.content }),
        Command::SendDirect(value) => Ok(ZulipCommandV1::SendDirect { operation_id: value.operation_id, account_id: value.account_id, recipients: value.recipient, content: value.content }),
        Command::UpdateMessage(value) => Ok(ZulipCommandV1::UpdateMessage { operation_id: value.operation_id, account_id: value.account_id, provider_message_id: value.provider_message_id, content: value.content, topic: value.topic }),
        Command::DeleteMessage(value) => Ok(ZulipCommandV1::DeleteMessage { operation_id: value.operation_id, account_id: value.account_id, provider_message_id: value.provider_message_id }),
        Command::Reaction(value) => Ok(ZulipCommandV1::Reaction { operation_id: value.operation_id, account_id: value.account_id, provider_message_id: value.provider_message_id, reaction: ZulipReactionV1 { emoji_name: value.emoji_name, emoji_code: value.emoji_code, reaction_type: value.reaction_type }, operation: if value.add { ZulipReactionOperationV1::Add } else { ZulipReactionOperationV1::Remove } }),
        Command::SendStreamWithUpload(value) => Ok(ZulipCommandV1::SendStreamWithUpload { operation_id: value.operation_id, account_id: value.account_id, stream: value.stream, topic: value.topic, content: value.content, blob: decode_blob(value.blob)?, filename: value.filename }),
        Command::SendDirectWithUpload(value) => Ok(ZulipCommandV1::SendDirectWithUpload { operation_id: value.operation_id, account_id: value.account_id, recipients: value.recipient, content: value.content, blob: decode_blob(value.blob)?, filename: value.filename }),
        Command::DownloadAttachment(value) => Ok(ZulipCommandV1::DownloadAttachment { operation_id: value.operation_id, account_id: value.account_id, upload_path: value.upload_path, blob: decode_blob(value.blob)? }),
    }
}

fn blob_message(value: &ZulipBlobIntentV1) -> wire::ZulipBlobIntentV1 {
    wire::ZulipBlobIntentV1 { blob_ref: value.blob_ref.clone(), reference_id: value.reference_id.clone(), declared_size: value.declared_size, backup_class: value.backup_class }
}

fn decode_blob(value: Option<wire::ZulipBlobIntentV1>) -> Result<ZulipBlobIntentV1, ZulipClientWireErrorV1> {
    let value = value.ok_or(ZulipClientWireErrorV1::MissingVariant)?;
    (value.reference_id.len() == 16 && value.reference_id.iter().any(|byte| *byte != 0) && value.declared_size > 0 && (1..=3).contains(&value.backup_class) && !value.blob_ref.is_empty())
        .then_some(ZulipBlobIntentV1 { blob_ref: value.blob_ref, reference_id: value.reference_id, declared_size: value.declared_size, backup_class: value.backup_class })
        .ok_or(ZulipClientWireErrorV1::InvalidPayload)
}

fn status_message(status: &ZulipCommandOperationStatusV1) -> wire::ZulipCommandOperationStatusV1 {
    let (outcome, provider_message_id, blob_ref) = match &status.outcome { ZulipCommandOperationOutcomeV1::OutcomeUnknown => ("outcome_unknown", None, None), ZulipCommandOperationOutcomeV1::Accepted { provider_message_id, blob_ref } => ("accepted", *provider_message_id, blob_ref.clone()), ZulipCommandOperationOutcomeV1::Rejected => ("rejected", None, None) };
    wire::ZulipCommandOperationStatusV1 { operation_id: status.operation_id.clone(), account_id: status.account_id.clone(), outcome: outcome.to_owned(), provider_message_id, requested_at_unix_seconds: status.requested_at_unix_seconds, completed_at_unix_seconds: status.completed_at_unix_seconds, blob_ref }
}

fn decode_status_message(status: wire::ZulipCommandOperationStatusV1) -> Result<ZulipCommandOperationStatusV1, ZulipClientWireErrorV1> {
    let outcome = match status.outcome.as_str() {
        "outcome_unknown" if status.provider_message_id.is_none() && status.blob_ref.is_none() && status.completed_at_unix_seconds.is_none() => ZulipCommandOperationOutcomeV1::OutcomeUnknown,
        "accepted" if status.completed_at_unix_seconds.is_some() && !(status.provider_message_id.is_some() && status.blob_ref.is_some()) => ZulipCommandOperationOutcomeV1::Accepted { provider_message_id: status.provider_message_id, blob_ref: status.blob_ref },
        "rejected" if status.provider_message_id.is_none() && status.blob_ref.is_none() && status.completed_at_unix_seconds.is_some() => ZulipCommandOperationOutcomeV1::Rejected,
        _ => return Err(ZulipClientWireErrorV1::InvalidPayload),
    };
    Ok(ZulipCommandOperationStatusV1 { operation_id: status.operation_id, account_id: status.account_id, outcome, requested_at_unix_seconds: status.requested_at_unix_seconds, completed_at_unix_seconds: status.completed_at_unix_seconds })
}
