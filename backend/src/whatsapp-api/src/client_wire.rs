//! Generated WhatsApp client payload adapters. No domain types cross this wire.

use prost::Message;

use crate::{
    WhatsAppAccount, WhatsAppAccountState, WhatsAppConversationCommandKind, WhatsAppDialog,
    WhatsAppMedia, WhatsAppMessage, WhatsAppParticipant, WhatsAppProviderCommand,
    WhatsAppProviderEvent, WhatsAppProviderEventKind, WhatsAppProviderQuery,
    WhatsAppProviderQueryResponse, WhatsAppProviderShape, WhatsAppRealtimeFrame,
    WhatsAppRuntimeKind, WhatsAppRuntimeState, WhatsAppRuntimeStatus,
    capabilities::{WhatsAppActionClass, WhatsAppCapability, WhatsAppCapabilityState},
    wire::{self, whats_app_provider_command_v1::Command, whats_app_provider_query_v1::Query},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientWireError {
    InvalidPayload,
    MissingVariant,
}

pub fn encode_command(command: &WhatsAppProviderCommand) -> Vec<u8> {
    command_message(command).encode_to_vec()
}

pub fn decode_command(bytes: &[u8]) -> Result<WhatsAppProviderCommand, ClientWireError> {
    let message = wire::WhatsAppProviderCommandV1::decode(bytes)
        .map_err(|_| ClientWireError::InvalidPayload)?;
    match message.command.ok_or(ClientWireError::MissingVariant)? {
        Command::SendText(value) => Ok(WhatsAppProviderCommand::SendText {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            text: value.text,
        }),
        Command::Reply(value) => Ok(WhatsAppProviderCommand::Reply {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            reply_to_provider_message_id: value.reply_to_provider_message_id,
            text: value.text,
        }),
        Command::Forward(value) => Ok(WhatsAppProviderCommand::Forward {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            source_provider_chat_id: value.source_provider_chat_id,
            source_provider_message_id: value.source_provider_message_id,
        }),
        Command::Edit(value) => Ok(WhatsAppProviderCommand::Edit {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            provider_message_id: value.provider_message_id,
            text: value.text,
        }),
        Command::Delete(value) => Ok(WhatsAppProviderCommand::Delete {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            provider_message_id: value.provider_message_id,
        }),
        Command::React(value) => Ok(WhatsAppProviderCommand::React {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            provider_message_id: value.provider_message_id,
            emoji: value.emoji,
        }),
        Command::Unreact(value) => Ok(WhatsAppProviderCommand::Unreact {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            provider_message_id: value.provider_message_id,
            emoji: value.emoji,
        }),
        Command::SendMedia(value) => Ok(WhatsAppProviderCommand::SendMedia {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            blob_ref: value.blob_ref,
            media_kind: value.media_kind,
            caption: value.caption,
            filename: value.filename,
        }),
        Command::SendVoiceNote(value) => Ok(WhatsAppProviderCommand::SendVoiceNote {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            attachment_id: value.attachment_id,
            blob_ref: value.blob_ref,
            content_type: value.content_type,
            declared_size: value.declared_size,
            sha256: value.sha256,
            scan_status: value.scan_status,
            filename: value.filename,
        }),
        Command::DownloadMedia(value) => Ok(WhatsAppProviderCommand::DownloadMedia {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            provider_message_id: value.provider_message_id,
            provider_media_id: value.provider_media_id,
        }),
        Command::PublishStatus(value) => Ok(WhatsAppProviderCommand::PublishStatus {
            operation_id: value.operation_id,
            account_id: value.account_id,
            text: value.text,
        }),
        Command::JoinConversation(value) => Ok(WhatsAppProviderCommand::JoinConversation {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            invite_link: value.invite_link,
        }),
        Command::LeaveConversation(value) => Ok(WhatsAppProviderCommand::LeaveConversation {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
        }),
        Command::Conversation(value) => Ok(WhatsAppProviderCommand::Conversation {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            action: conversation_action_from_wire(value.action)?,
        }),
    }
}

pub fn encode_query(query: &WhatsAppProviderQuery) -> Vec<u8> {
    query_message(query).encode_to_vec()
}

pub fn decode_query(bytes: &[u8]) -> Result<WhatsAppProviderQuery, ClientWireError> {
    let message = wire::WhatsAppProviderQueryV1::decode(bytes)
        .map_err(|_| ClientWireError::InvalidPayload)?;
    match message.query.ok_or(ClientWireError::MissingVariant)? {
        Query::Account(value) => Ok(WhatsAppProviderQuery::Account {
            account_id: value.account_id,
        }),
        Query::RuntimeStatus(value) => Ok(WhatsAppProviderQuery::RuntimeStatus {
            account_id: value.account_id,
        }),
        Query::CachedMessages(value) => Ok(WhatsAppProviderQuery::CachedMessages {
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            limit: value.limit,
        }),
        Query::SearchMessages(value) => Ok(WhatsAppProviderQuery::SearchMessages {
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            query: value.query,
            limit: value.limit,
        }),
        Query::Dialogs(value) => Ok(WhatsAppProviderQuery::Dialogs {
            account_id: value.account_id,
            limit: value.limit,
        }),
        Query::Participants(value) => Ok(WhatsAppProviderQuery::Participants {
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            limit: value.limit,
        }),
        Query::Replay(value) => Ok(WhatsAppProviderQuery::Replay {
            account_id: value.account_id,
            after_sequence: value.after_sequence,
            limit: value.limit,
        }),
        Query::PendingCommands(value) => Ok(WhatsAppProviderQuery::PendingCommands {
            account_id: value.account_id,
            limit: value.limit,
        }),
        Query::ClaimPendingCommands(value) => Ok(WhatsAppProviderQuery::ClaimPendingCommands {
            account_id: value.account_id,
            host_claim_id: value.host_claim_id,
            lease_seconds: value.lease_seconds,
            limit: value.limit,
        }),
        Query::Events(value) => Ok(WhatsAppProviderQuery::Events {
            account_id: value.account_id,
            kind: event_kind_from_wire(value.kind)?,
            provider_chat_id: value.provider_chat_id,
            limit: value.limit,
        }),
    }
}

pub fn encode_query_response(response: &WhatsAppProviderQueryResponse) -> Option<Vec<u8>> {
    use wire::whats_app_query_response_v1::Response;
    let response = match response {
        WhatsAppProviderQueryResponse::Account(value) => {
            Response::Account(wire::WhatsAppAccountList {
                account: value.iter().map(account_to_wire).collect(),
            })
        }
        WhatsAppProviderQueryResponse::RuntimeStatus(value) => {
            Response::RuntimeStatus(runtime_status_to_wire(value))
        }
        WhatsAppProviderQueryResponse::Messages(values) => {
            Response::Messages(wire::WhatsAppMessageList {
                message: values.iter().map(message_to_wire).collect(),
            })
        }
        WhatsAppProviderQueryResponse::Dialogs(values) => {
            Response::Dialogs(wire::WhatsAppDialogList {
                dialog: values.iter().map(dialog_to_wire).collect(),
            })
        }
        WhatsAppProviderQueryResponse::Participants(values) => {
            Response::Participants(wire::WhatsAppParticipantList {
                participant: values.iter().map(participant_to_wire).collect(),
            })
        }
        WhatsAppProviderQueryResponse::Realtime(values) => {
            Response::Realtime(wire::WhatsAppRealtimeList {
                frame: values.iter().map(realtime_to_wire).collect(),
            })
        }
        WhatsAppProviderQueryResponse::Commands(values) => {
            Response::Commands(wire::WhatsAppCommandList {
                command: values.iter().map(command_message).collect(),
            })
        }
        WhatsAppProviderQueryResponse::Events(values) => {
            Response::Events(wire::WhatsAppProviderEventList {
                event: values.iter().map(event_to_wire).collect(),
            })
        }
    };
    Some(
        wire::WhatsAppQueryResponseV1 {
            response: Some(response),
        }
        .encode_to_vec(),
    )
}

fn parse_account(
    value: wire::WhatsAppAccountResponseV1,
) -> Result<WhatsAppAccount, ClientWireError> {
    let account_state = match value.account_state.as_str() {
        "provisioning" => WhatsAppAccountState::Provisioning,
        "link_required" => WhatsAppAccountState::LinkRequired,
        "linked" => WhatsAppAccountState::Linked,
        "degraded" => WhatsAppAccountState::Degraded,
        "revoked" => WhatsAppAccountState::Revoked,
        "retired" => WhatsAppAccountState::Retired,
        _ => return Err(ClientWireError::InvalidPayload),
    };
    let runtime_state = match value.runtime_state.as_str() {
        "stopped" => WhatsAppRuntimeState::Stopped,
        "starting" => WhatsAppRuntimeState::Starting,
        "running" => WhatsAppRuntimeState::Running,
        "degraded" => WhatsAppRuntimeState::Degraded,
        "blocked" => WhatsAppRuntimeState::Blocked,
        _ => return Err(ClientWireError::InvalidPayload),
    };
    if value.provider_shape != WhatsAppProviderShape::WebCompanion.as_str()
        || value.runtime_kind != WhatsAppRuntimeKind::HiddenWebView.as_str()
    {
        return Err(ClientWireError::InvalidPayload);
    }
    Ok(WhatsAppAccount {
        account_id: value.account_id,
        display_name: value.display_name,
        external_account_id: value.external_account_id,
        provider_shape: WhatsAppProviderShape::WebCompanion,
        runtime_kind: WhatsAppRuntimeKind::HiddenWebView,
        account_state,
        runtime_state,
        credentials: Vec::new(),
    })
}

fn parse_message(value: wire::WhatsAppMessage) -> WhatsAppMessage {
    WhatsAppMessage {
        account_id: value.account_id,
        provider_chat_id: value.provider_chat_id,
        provider_message_id: value.provider_message_id,
        sender_id: value.sender_id,
        sender_display_name: value.sender_display_name,
        text: value.text,
        reply_to_provider_message_id: value.reply_to_provider_message_id,
        occurred_at_unix_seconds: value.occurred_at_unix_seconds,
    }
}

fn parse_dialog(value: wire::WhatsAppDialog) -> WhatsAppDialog {
    WhatsAppDialog {
        account_id: value.account_id,
        provider_chat_id: value.provider_chat_id,
        title: value.title,
        kind: value.kind,
        is_archived: value.is_archived,
        is_pinned: value.is_pinned,
        is_muted: value.is_muted,
        is_unread: value.is_unread,
        unread_count: value.unread_count,
        participant_count: value.participant_count,
        observed_at_unix_seconds: value.observed_at_unix_seconds,
    }
}

fn parse_participant(value: wire::WhatsAppParticipant) -> WhatsAppParticipant {
    WhatsAppParticipant {
        account_id: value.account_id,
        provider_chat_id: value.provider_chat_id,
        provider_identity_id: value.provider_identity_id,
        display_name: value.display_name,
        role: value.role,
        status: value.status,
        is_self: value.is_self,
        observed_at_unix_seconds: value.observed_at_unix_seconds,
    }
}

fn parse_capability(
    value: wire::WhatsAppCapability,
) -> Result<WhatsAppCapability, ClientWireError> {
    let status = match value.status.as_str() {
        "available" => WhatsAppCapabilityState::Available,
        "blocked" => WhatsAppCapabilityState::Blocked,
        "degraded" => WhatsAppCapabilityState::Degraded,
        "planned" => WhatsAppCapabilityState::Planned,
        "unsupported" => WhatsAppCapabilityState::Unsupported,
        _ => return Err(ClientWireError::InvalidPayload),
    };
    let action_class = match value.action_class.as_str() {
        "read" => WhatsAppActionClass::Read,
        "provider_write" => WhatsAppActionClass::ProviderWrite,
        "destructive" => WhatsAppActionClass::Destructive,
        "secret_access" => WhatsAppActionClass::SecretAccess,
        _ => return Err(ClientWireError::InvalidPayload),
    };
    Ok(WhatsAppCapability {
        capability: value.capability,
        category: value.category,
        status,
        action_class,
        confirmation_required: value.confirmation_required,
        closure_gate: value.closure_gate,
        reason: value.reason,
    })
}

fn parse_runtime_status(
    value: wire::WhatsAppRuntimeStatusV1,
) -> Result<WhatsAppRuntimeStatus, ClientWireError> {
    Ok(WhatsAppRuntimeStatus {
        account_id: value.account_id,
        account_state: value
            .account_state
            .as_deref()
            .map(|value| match value {
                "provisioning" => Ok(WhatsAppAccountState::Provisioning),
                "link_required" => Ok(WhatsAppAccountState::LinkRequired),
                "linked" => Ok(WhatsAppAccountState::Linked),
                "degraded" => Ok(WhatsAppAccountState::Degraded),
                "revoked" => Ok(WhatsAppAccountState::Revoked),
                "retired" => Ok(WhatsAppAccountState::Retired),
                _ => Err(ClientWireError::InvalidPayload),
            })
            .transpose()?,
        runtime_state: value
            .runtime_state
            .as_deref()
            .map(parse_runtime_state)
            .transpose()?,
        capabilities: value
            .capability
            .into_iter()
            .map(parse_capability)
            .collect::<Result<Vec<_>, _>>()?,
        host_command_queue_available: value.host_command_queue_available,
    })
}

pub fn decode_query_response(
    bytes: &[u8],
) -> Result<WhatsAppProviderQueryResponse, ClientWireError> {
    use wire::whats_app_query_response_v1::Response;
    let message = wire::WhatsAppQueryResponseV1::decode(bytes)
        .map_err(|_| ClientWireError::InvalidPayload)?;
    match message.response.ok_or(ClientWireError::MissingVariant)? {
        Response::Account(value) => {
            let mut accounts = value.account.into_iter();
            let account = accounts.next().map(parse_account).transpose()?;
            if accounts.next().is_some() {
                return Err(ClientWireError::InvalidPayload);
            }
            Ok(WhatsAppProviderQueryResponse::Account(account))
        }
        Response::Messages(value) => Ok(WhatsAppProviderQueryResponse::Messages(
            value.message.into_iter().map(parse_message).collect(),
        )),
        Response::Dialogs(value) => Ok(WhatsAppProviderQueryResponse::Dialogs(
            value.dialog.into_iter().map(parse_dialog).collect(),
        )),
        Response::Participants(value) => Ok(WhatsAppProviderQueryResponse::Participants(
            value
                .participant
                .into_iter()
                .map(parse_participant)
                .collect(),
        )),
        Response::Realtime(value) => Ok(WhatsAppProviderQueryResponse::Realtime(
            value
                .frame
                .into_iter()
                .map(parse_realtime)
                .collect::<Result<Vec<_>, _>>()?,
        )),
        Response::Commands(value) => Ok(WhatsAppProviderQueryResponse::Commands(
            value
                .command
                .into_iter()
                .map(|value| decode_command(&value.encode_to_vec()))
                .collect::<Result<Vec<_>, _>>()?,
        )),
        Response::Events(value) => Ok(WhatsAppProviderQueryResponse::Events(
            value
                .event
                .into_iter()
                .map(parse_event)
                .collect::<Result<Vec<_>, _>>()?,
        )),
        Response::RuntimeStatus(value) => Ok(WhatsAppProviderQueryResponse::RuntimeStatus(
            parse_runtime_status(value)?,
        )),
    }
}

fn event_kind_from_wire(value: i32) -> Result<WhatsAppProviderEventKind, ClientWireError> {
    match wire::ProviderEventKind::try_from(value).map_err(|_| ClientWireError::MissingVariant)? {
        wire::ProviderEventKind::RuntimeState => Ok(WhatsAppProviderEventKind::RuntimeState),
        wire::ProviderEventKind::Message => Ok(WhatsAppProviderEventKind::Message),
        wire::ProviderEventKind::MessageEdited => Ok(WhatsAppProviderEventKind::MessageEdited),
        wire::ProviderEventKind::MessageDeleted => Ok(WhatsAppProviderEventKind::MessageDeleted),
        wire::ProviderEventKind::Receipt => Ok(WhatsAppProviderEventKind::Receipt),
        wire::ProviderEventKind::Reaction => Ok(WhatsAppProviderEventKind::Reaction),
        wire::ProviderEventKind::Dialog => Ok(WhatsAppProviderEventKind::Dialog),
        wire::ProviderEventKind::Participant => Ok(WhatsAppProviderEventKind::Participant),
        wire::ProviderEventKind::Presence => Ok(WhatsAppProviderEventKind::Presence),
        wire::ProviderEventKind::Call => Ok(WhatsAppProviderEventKind::Call),
        wire::ProviderEventKind::Status => Ok(WhatsAppProviderEventKind::Status),
        wire::ProviderEventKind::StatusView => Ok(WhatsAppProviderEventKind::StatusView),
        wire::ProviderEventKind::StatusDeleted => Ok(WhatsAppProviderEventKind::StatusDeleted),
        wire::ProviderEventKind::Media => Ok(WhatsAppProviderEventKind::Media),
        wire::ProviderEventKind::Session => Ok(WhatsAppProviderEventKind::Session),
        wire::ProviderEventKind::CommandResult => Ok(WhatsAppProviderEventKind::CommandResult),
        wire::ProviderEventKind::Unspecified => Err(ClientWireError::MissingVariant),
    }
}

fn account_to_wire(value: &WhatsAppAccount) -> wire::WhatsAppAccountResponseV1 {
    wire::WhatsAppAccountResponseV1 {
        account_id: value.account_id.clone(),
        display_name: value.display_name.clone(),
        external_account_id: value.external_account_id.clone(),
        provider_shape: value.provider_shape.as_str().to_owned(),
        runtime_kind: value.runtime_kind.as_str().to_owned(),
        account_state: value.account_state.as_str().to_owned(),
        runtime_state: format!("{:?}", value.runtime_state).to_lowercase(),
    }
}

fn runtime_status_to_wire(value: &WhatsAppRuntimeStatus) -> wire::WhatsAppRuntimeStatusV1 {
    wire::WhatsAppRuntimeStatusV1 {
        account_id: value.account_id.clone(),
        account_state: value
            .account_state
            .map(|state| format!("{state:?}").to_lowercase()),
        runtime_state: value
            .runtime_state
            .map(|state| format!("{state:?}").to_lowercase()),
        capability: value
            .capabilities
            .iter()
            .map(|item| wire::WhatsAppCapability {
                capability: item.capability.clone(),
                category: item.category.clone(),
                status: format!("{:?}", item.status).to_lowercase(),
                action_class: format!("{:?}", item.action_class).to_lowercase(),
                confirmation_required: item.confirmation_required,
                closure_gate: item.closure_gate,
                reason: item.reason.clone(),
            })
            .collect(),
        host_command_queue_available: value.host_command_queue_available,
    }
}

fn event_kind_to_wire(value: WhatsAppProviderEventKind) -> i32 {
    match value {
        WhatsAppProviderEventKind::RuntimeState => wire::ProviderEventKind::RuntimeState as i32,
        WhatsAppProviderEventKind::Message => wire::ProviderEventKind::Message as i32,
        WhatsAppProviderEventKind::MessageEdited => wire::ProviderEventKind::MessageEdited as i32,
        WhatsAppProviderEventKind::MessageDeleted => wire::ProviderEventKind::MessageDeleted as i32,
        WhatsAppProviderEventKind::Receipt => wire::ProviderEventKind::Receipt as i32,
        WhatsAppProviderEventKind::Reaction => wire::ProviderEventKind::Reaction as i32,
        WhatsAppProviderEventKind::Dialog => wire::ProviderEventKind::Dialog as i32,
        WhatsAppProviderEventKind::Participant => wire::ProviderEventKind::Participant as i32,
        WhatsAppProviderEventKind::Presence => wire::ProviderEventKind::Presence as i32,
        WhatsAppProviderEventKind::Call => wire::ProviderEventKind::Call as i32,
        WhatsAppProviderEventKind::Status => wire::ProviderEventKind::Status as i32,
        WhatsAppProviderEventKind::StatusView => wire::ProviderEventKind::StatusView as i32,
        WhatsAppProviderEventKind::StatusDeleted => wire::ProviderEventKind::StatusDeleted as i32,
        WhatsAppProviderEventKind::Media => wire::ProviderEventKind::Media as i32,
        WhatsAppProviderEventKind::Session => wire::ProviderEventKind::Session as i32,
        WhatsAppProviderEventKind::CommandResult => wire::ProviderEventKind::CommandResult as i32,
    }
}

fn realtime_to_wire(value: &WhatsAppRealtimeFrame) -> wire::WhatsAppRealtimeFrame {
    wire::WhatsAppRealtimeFrame {
        account_id: value.account_id.clone(),
        sequence: value.sequence,
        event: Some(event_to_wire(&value.event)),
    }
}

fn event_to_wire(value: &WhatsAppProviderEvent) -> wire::WhatsAppProviderEventV1 {
    use wire::whats_app_provider_event_v1::Event;
    let event = match value {
        WhatsAppProviderEvent::RuntimeStateChanged {
            account_id,
            state,
            observed_at_unix_seconds,
        } => Event::RuntimeStateChanged(wire::RuntimeStateChangedEvent {
            account_id: account_id.clone(),
            state: format!("{state:?}").to_ascii_lowercase(),
            observed_at_unix_seconds: *observed_at_unix_seconds,
        }),
        WhatsAppProviderEvent::SessionStateChanged {
            account_id,
            linked,
            secret_ref,
            revision,
            observed_at_unix_seconds,
        } => Event::SessionStateChanged(wire::SessionStateChangedEvent {
            account_id: account_id.clone(),
            linked: *linked,
            secret_ref: secret_ref.clone(),
            revision: *revision,
            observed_at_unix_seconds: *observed_at_unix_seconds,
        }),
        WhatsAppProviderEvent::CommandResultObserved {
            account_id,
            operation_id,
            provider_request_id,
            succeeded,
            observed_at_unix_seconds,
        } => Event::CommandResultObserved(wire::CommandResultObservedEvent {
            account_id: account_id.clone(),
            operation_id: operation_id.clone(),
            provider_request_id: provider_request_id.clone(),
            succeeded: *succeeded,
            observed_at_unix_seconds: *observed_at_unix_seconds,
        }),
        WhatsAppProviderEvent::MessageObserved(value) => {
            Event::MessageObserved(message_to_wire(value))
        }
        WhatsAppProviderEvent::MessageEdited {
            account_id,
            provider_chat_id,
            provider_message_id,
            text,
            observed_at_unix_seconds,
        } => Event::MessageEdited(wire::MessageEditedEvent {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            text: text.clone(),
            observed_at_unix_seconds: *observed_at_unix_seconds,
        }),
        WhatsAppProviderEvent::MessageDeleted {
            account_id,
            provider_chat_id,
            provider_message_id,
            observed_at_unix_seconds,
        } => Event::MessageDeleted(wire::MessageDeletedEvent {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            observed_at_unix_seconds: *observed_at_unix_seconds,
        }),
        WhatsAppProviderEvent::ReceiptChanged {
            account_id,
            provider_chat_id,
            provider_message_id,
            delivery_state,
            observed_at_unix_seconds,
        } => Event::ReceiptChanged(wire::ReceiptChangedEvent {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            delivery_state: delivery_state.clone(),
            observed_at_unix_seconds: *observed_at_unix_seconds,
        }),
        WhatsAppProviderEvent::ReactionChanged {
            account_id,
            provider_chat_id,
            provider_message_id,
            actor_id,
            emoji,
            is_active,
            observed_at_unix_seconds,
        } => Event::ReactionChanged(wire::ReactionChangedEvent {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            actor_id: actor_id.clone(),
            emoji: emoji.clone(),
            is_active: *is_active,
            observed_at_unix_seconds: *observed_at_unix_seconds,
        }),
        WhatsAppProviderEvent::DialogObserved(value) => {
            Event::DialogObserved(dialog_to_wire(value))
        }
        WhatsAppProviderEvent::ParticipantObserved(value) => {
            Event::ParticipantObserved(participant_to_wire(value))
        }
        WhatsAppProviderEvent::PresenceChanged {
            account_id,
            provider_chat_id,
            provider_identity_id,
            state,
            observed_at_unix_seconds,
        } => Event::PresenceChanged(wire::PresenceChangedEvent {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_identity_id: provider_identity_id.clone(),
            state: state.clone(),
            observed_at_unix_seconds: *observed_at_unix_seconds,
        }),
        WhatsAppProviderEvent::CallObserved {
            account_id,
            provider_call_id,
            provider_chat_id,
            direction,
            state,
            observed_at_unix_seconds,
        } => Event::CallObserved(wire::CallObservedEvent {
            account_id: account_id.clone(),
            provider_call_id: provider_call_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            direction: direction.clone(),
            state: state.clone(),
            observed_at_unix_seconds: *observed_at_unix_seconds,
        }),
        WhatsAppProviderEvent::StatusObserved {
            account_id,
            provider_status_id,
            sender_id,
            text,
            observed_at_unix_seconds,
        } => Event::StatusObserved(wire::StatusObservedEvent {
            account_id: account_id.clone(),
            provider_status_id: provider_status_id.clone(),
            sender_id: sender_id.clone(),
            text: text.clone(),
            observed_at_unix_seconds: *observed_at_unix_seconds,
        }),
        WhatsAppProviderEvent::StatusViewObserved {
            account_id,
            provider_status_id,
            viewer_id,
            observed_at_unix_seconds,
        } => Event::StatusViewObserved(wire::StatusViewObservedEvent {
            account_id: account_id.clone(),
            provider_status_id: provider_status_id.clone(),
            viewer_id: viewer_id.clone(),
            observed_at_unix_seconds: *observed_at_unix_seconds,
        }),
        WhatsAppProviderEvent::StatusDeleted {
            account_id,
            provider_status_id,
            observed_at_unix_seconds,
        } => Event::StatusDeleted(wire::StatusDeletedEvent {
            account_id: account_id.clone(),
            provider_status_id: provider_status_id.clone(),
            observed_at_unix_seconds: *observed_at_unix_seconds,
        }),
        WhatsAppProviderEvent::MediaObserved(value) => Event::MediaObserved(media_to_wire(value)),
    };
    wire::WhatsAppProviderEventV1 { event: Some(event) }
}

fn media_to_wire(value: &WhatsAppMedia) -> wire::WhatsAppMedia {
    wire::WhatsAppMedia {
        account_id: value.account_id.clone(),
        provider_chat_id: value.provider_chat_id.clone(),
        provider_message_id: value.provider_message_id.clone(),
        provider_media_id: value.provider_media_id.clone(),
        media_kind: value.media_kind.clone(),
        filename: value.filename.clone(),
        content_type: value.content_type.clone(),
        declared_size: value.declared_size,
        observed_at_unix_seconds: value.observed_at_unix_seconds,
    }
}

fn message_to_wire(value: &WhatsAppMessage) -> wire::WhatsAppMessage {
    wire::WhatsAppMessage {
        account_id: value.account_id.clone(),
        provider_chat_id: value.provider_chat_id.clone(),
        provider_message_id: value.provider_message_id.clone(),
        sender_id: value.sender_id.clone(),
        sender_display_name: value.sender_display_name.clone(),
        text: value.text.clone(),
        reply_to_provider_message_id: value.reply_to_provider_message_id.clone(),
        occurred_at_unix_seconds: value.occurred_at_unix_seconds,
    }
}

fn dialog_to_wire(value: &WhatsAppDialog) -> wire::WhatsAppDialog {
    wire::WhatsAppDialog {
        account_id: value.account_id.clone(),
        provider_chat_id: value.provider_chat_id.clone(),
        title: value.title.clone(),
        kind: value.kind.clone(),
        is_archived: value.is_archived,
        is_pinned: value.is_pinned,
        is_muted: value.is_muted,
        is_unread: value.is_unread,
        unread_count: value.unread_count,
        participant_count: value.participant_count,
        observed_at_unix_seconds: value.observed_at_unix_seconds,
    }
}

fn participant_to_wire(value: &WhatsAppParticipant) -> wire::WhatsAppParticipant {
    wire::WhatsAppParticipant {
        account_id: value.account_id.clone(),
        provider_chat_id: value.provider_chat_id.clone(),
        provider_identity_id: value.provider_identity_id.clone(),
        display_name: value.display_name.clone(),
        role: value.role.clone(),
        status: value.status.clone(),
        is_self: value.is_self,
        observed_at_unix_seconds: value.observed_at_unix_seconds,
    }
}

fn parse_runtime_state(value: &str) -> Result<WhatsAppRuntimeState, ClientWireError> {
    match value {
        "stopped" => Ok(WhatsAppRuntimeState::Stopped),
        "starting" => Ok(WhatsAppRuntimeState::Starting),
        "running" => Ok(WhatsAppRuntimeState::Running),
        "degraded" => Ok(WhatsAppRuntimeState::Degraded),
        "blocked" => Ok(WhatsAppRuntimeState::Blocked),
        _ => Err(ClientWireError::InvalidPayload),
    }
}

fn parse_event(
    value: wire::WhatsAppProviderEventV1,
) -> Result<WhatsAppProviderEvent, ClientWireError> {
    use wire::whats_app_provider_event_v1::Event;
    match value.event.ok_or(ClientWireError::MissingVariant)? {
        Event::RuntimeStateChanged(value) => Ok(WhatsAppProviderEvent::RuntimeStateChanged {
            account_id: value.account_id,
            state: parse_runtime_state(&value.state)?,
            observed_at_unix_seconds: value.observed_at_unix_seconds,
        }),
        Event::SessionStateChanged(value) => Ok(WhatsAppProviderEvent::SessionStateChanged {
            account_id: value.account_id,
            linked: value.linked,
            secret_ref: value.secret_ref,
            revision: value.revision,
            observed_at_unix_seconds: value.observed_at_unix_seconds,
        }),
        Event::CommandResultObserved(value) => Ok(WhatsAppProviderEvent::CommandResultObserved {
            account_id: value.account_id,
            operation_id: value.operation_id,
            provider_request_id: value.provider_request_id,
            succeeded: value.succeeded,
            observed_at_unix_seconds: value.observed_at_unix_seconds,
        }),
        Event::MessageObserved(value) => {
            Ok(WhatsAppProviderEvent::MessageObserved(parse_message(value)))
        }
        Event::MessageEdited(value) => Ok(WhatsAppProviderEvent::MessageEdited {
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            provider_message_id: value.provider_message_id,
            text: value.text,
            observed_at_unix_seconds: value.observed_at_unix_seconds,
        }),
        Event::MessageDeleted(value) => Ok(WhatsAppProviderEvent::MessageDeleted {
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            provider_message_id: value.provider_message_id,
            observed_at_unix_seconds: value.observed_at_unix_seconds,
        }),
        Event::ReceiptChanged(value) => Ok(WhatsAppProviderEvent::ReceiptChanged {
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            provider_message_id: value.provider_message_id,
            delivery_state: value.delivery_state,
            observed_at_unix_seconds: value.observed_at_unix_seconds,
        }),
        Event::ReactionChanged(value) => Ok(WhatsAppProviderEvent::ReactionChanged {
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            provider_message_id: value.provider_message_id,
            actor_id: value.actor_id,
            emoji: value.emoji,
            is_active: value.is_active,
            observed_at_unix_seconds: value.observed_at_unix_seconds,
        }),
        Event::DialogObserved(value) => {
            Ok(WhatsAppProviderEvent::DialogObserved(parse_dialog(value)))
        }
        Event::ParticipantObserved(value) => Ok(WhatsAppProviderEvent::ParticipantObserved(
            parse_participant(value),
        )),
        Event::PresenceChanged(value) => Ok(WhatsAppProviderEvent::PresenceChanged {
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            provider_identity_id: value.provider_identity_id,
            state: value.state,
            observed_at_unix_seconds: value.observed_at_unix_seconds,
        }),
        Event::CallObserved(value) => Ok(WhatsAppProviderEvent::CallObserved {
            account_id: value.account_id,
            provider_call_id: value.provider_call_id,
            provider_chat_id: value.provider_chat_id,
            direction: value.direction,
            state: value.state,
            observed_at_unix_seconds: value.observed_at_unix_seconds,
        }),
        Event::StatusObserved(value) => Ok(WhatsAppProviderEvent::StatusObserved {
            account_id: value.account_id,
            provider_status_id: value.provider_status_id,
            sender_id: value.sender_id,
            text: value.text,
            observed_at_unix_seconds: value.observed_at_unix_seconds,
        }),
        Event::StatusViewObserved(value) => Ok(WhatsAppProviderEvent::StatusViewObserved {
            account_id: value.account_id,
            provider_status_id: value.provider_status_id,
            viewer_id: value.viewer_id,
            observed_at_unix_seconds: value.observed_at_unix_seconds,
        }),
        Event::StatusDeleted(value) => Ok(WhatsAppProviderEvent::StatusDeleted {
            account_id: value.account_id,
            provider_status_id: value.provider_status_id,
            observed_at_unix_seconds: value.observed_at_unix_seconds,
        }),
        Event::MediaObserved(value) => Ok(WhatsAppProviderEvent::MediaObserved(WhatsAppMedia {
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            provider_message_id: value.provider_message_id,
            provider_media_id: value.provider_media_id,
            media_kind: value.media_kind,
            filename: value.filename,
            content_type: value.content_type,
            declared_size: value.declared_size,
            observed_at_unix_seconds: value.observed_at_unix_seconds,
        })),
    }
}

fn parse_realtime(
    value: wire::WhatsAppRealtimeFrame,
) -> Result<WhatsAppRealtimeFrame, ClientWireError> {
    Ok(WhatsAppRealtimeFrame {
        account_id: value.account_id,
        sequence: value.sequence,
        event: parse_event(value.event.ok_or(ClientWireError::MissingVariant)?)?,
    })
}

fn command_message(command: &WhatsAppProviderCommand) -> wire::WhatsAppProviderCommandV1 {
    use wire::whats_app_provider_command_v1::Command;
    let command = match command {
        WhatsAppProviderCommand::SendText {
            operation_id,
            account_id,
            provider_chat_id,
            text,
        } => Command::SendText(wire::SendTextCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            text: text.clone(),
        }),
        WhatsAppProviderCommand::Reply {
            operation_id,
            account_id,
            provider_chat_id,
            reply_to_provider_message_id,
            text,
        } => Command::Reply(wire::ReplyCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            reply_to_provider_message_id: reply_to_provider_message_id.clone(),
            text: text.clone(),
        }),
        WhatsAppProviderCommand::Forward {
            operation_id,
            account_id,
            provider_chat_id,
            source_provider_chat_id,
            source_provider_message_id,
        } => Command::Forward(wire::ForwardCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            source_provider_chat_id: source_provider_chat_id.clone(),
            source_provider_message_id: source_provider_message_id.clone(),
        }),
        WhatsAppProviderCommand::Edit {
            operation_id,
            account_id,
            provider_chat_id,
            provider_message_id,
            text,
        } => Command::Edit(wire::EditCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            text: text.clone(),
        }),
        WhatsAppProviderCommand::Delete {
            operation_id,
            account_id,
            provider_chat_id,
            provider_message_id,
        } => Command::Delete(wire::DeleteCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
        }),
        WhatsAppProviderCommand::React {
            operation_id,
            account_id,
            provider_chat_id,
            provider_message_id,
            emoji,
        } => Command::React(wire::ReactCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            emoji: emoji.clone(),
        }),
        WhatsAppProviderCommand::Unreact {
            operation_id,
            account_id,
            provider_chat_id,
            provider_message_id,
            emoji,
        } => Command::Unreact(wire::UnreactCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            emoji: emoji.clone(),
        }),
        WhatsAppProviderCommand::SendMedia {
            operation_id,
            account_id,
            provider_chat_id,
            blob_ref,
            media_kind,
            caption,
            filename,
        } => Command::SendMedia(wire::SendMediaCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            blob_ref: blob_ref.clone(),
            media_kind: media_kind.clone(),
            caption: caption.clone(),
            filename: filename.clone(),
        }),
        WhatsAppProviderCommand::SendVoiceNote {
            operation_id,
            account_id,
            provider_chat_id,
            attachment_id,
            blob_ref,
            content_type,
            declared_size,
            sha256,
            scan_status,
            filename,
        } => Command::SendVoiceNote(wire::SendVoiceNoteCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            attachment_id: attachment_id.clone(),
            blob_ref: blob_ref.clone(),
            content_type: content_type.clone(),
            declared_size: *declared_size,
            sha256: sha256.clone(),
            scan_status: scan_status.clone(),
            filename: filename.clone(),
        }),
        WhatsAppProviderCommand::DownloadMedia {
            operation_id,
            account_id,
            provider_chat_id,
            provider_message_id,
            provider_media_id,
        } => Command::DownloadMedia(wire::DownloadMediaCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            provider_media_id: provider_media_id.clone(),
        }),
        WhatsAppProviderCommand::PublishStatus {
            operation_id,
            account_id,
            text,
        } => Command::PublishStatus(wire::PublishStatusCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            text: text.clone(),
        }),
        WhatsAppProviderCommand::JoinConversation {
            operation_id,
            account_id,
            provider_chat_id,
            invite_link,
        } => Command::JoinConversation(wire::JoinConversationCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            invite_link: invite_link.clone(),
        }),
        WhatsAppProviderCommand::LeaveConversation {
            operation_id,
            account_id,
            provider_chat_id,
        } => Command::LeaveConversation(wire::LeaveConversationCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
        }),
        WhatsAppProviderCommand::Conversation {
            operation_id,
            account_id,
            provider_chat_id,
            action,
        } => Command::Conversation(wire::ConversationCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            action: conversation_action_to_wire(*action),
        }),
    };
    wire::WhatsAppProviderCommandV1 {
        command: Some(command),
    }
}

fn conversation_action_from_wire(
    value: i32,
) -> Result<WhatsAppConversationCommandKind, ClientWireError> {
    match wire::DialogCommandAction::try_from(value).map_err(|_| ClientWireError::InvalidPayload)? {
        wire::DialogCommandAction::MarkRead => Ok(WhatsAppConversationCommandKind::MarkRead),
        wire::DialogCommandAction::MarkUnread => Ok(WhatsAppConversationCommandKind::MarkUnread),
        wire::DialogCommandAction::Archive => Ok(WhatsAppConversationCommandKind::Archive),
        wire::DialogCommandAction::Unarchive => Ok(WhatsAppConversationCommandKind::Unarchive),
        wire::DialogCommandAction::Mute => Ok(WhatsAppConversationCommandKind::Mute),
        wire::DialogCommandAction::Unmute => Ok(WhatsAppConversationCommandKind::Unmute),
        wire::DialogCommandAction::Pin => Ok(WhatsAppConversationCommandKind::Pin),
        wire::DialogCommandAction::Unpin => Ok(WhatsAppConversationCommandKind::Unpin),
        wire::DialogCommandAction::Unspecified => Err(ClientWireError::MissingVariant),
    }
}

fn conversation_action_to_wire(value: WhatsAppConversationCommandKind) -> i32 {
    match value {
        WhatsAppConversationCommandKind::MarkRead => wire::DialogCommandAction::MarkRead as i32,
        WhatsAppConversationCommandKind::MarkUnread => wire::DialogCommandAction::MarkUnread as i32,
        WhatsAppConversationCommandKind::Archive => wire::DialogCommandAction::Archive as i32,
        WhatsAppConversationCommandKind::Unarchive => wire::DialogCommandAction::Unarchive as i32,
        WhatsAppConversationCommandKind::Mute => wire::DialogCommandAction::Mute as i32,
        WhatsAppConversationCommandKind::Unmute => wire::DialogCommandAction::Unmute as i32,
        WhatsAppConversationCommandKind::Pin => wire::DialogCommandAction::Pin as i32,
        WhatsAppConversationCommandKind::Unpin => wire::DialogCommandAction::Unpin as i32,
    }
}

fn query_message(query: &WhatsAppProviderQuery) -> wire::WhatsAppProviderQueryV1 {
    use wire::whats_app_provider_query_v1::Query;
    let query = match query {
        WhatsAppProviderQuery::Account { account_id } => Query::Account(wire::AccountQuery {
            account_id: account_id.clone(),
        }),
        WhatsAppProviderQuery::RuntimeStatus { account_id } => {
            Query::RuntimeStatus(wire::RuntimeStatusQuery {
                account_id: account_id.clone(),
            })
        }
        WhatsAppProviderQuery::CachedMessages {
            account_id,
            provider_chat_id,
            limit,
        } => Query::CachedMessages(wire::CachedMessagesQuery {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            limit: *limit,
        }),
        WhatsAppProviderQuery::SearchMessages {
            account_id,
            provider_chat_id,
            query,
            limit,
        } => Query::SearchMessages(wire::SearchMessagesQuery {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            query: query.clone(),
            limit: *limit,
        }),
        WhatsAppProviderQuery::Dialogs { account_id, limit } => {
            Query::Dialogs(wire::DialogsQuery {
                account_id: account_id.clone(),
                limit: *limit,
            })
        }
        WhatsAppProviderQuery::Participants {
            account_id,
            provider_chat_id,
            limit,
        } => Query::Participants(wire::ParticipantsQuery {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            limit: *limit,
        }),
        WhatsAppProviderQuery::Replay {
            account_id,
            after_sequence,
            limit,
        } => Query::Replay(wire::ReplayQuery {
            account_id: account_id.clone(),
            after_sequence: *after_sequence,
            limit: *limit,
        }),
        WhatsAppProviderQuery::PendingCommands { account_id, limit } => {
            Query::PendingCommands(wire::PendingCommandsQuery {
                account_id: account_id.clone(),
                limit: *limit,
            })
        }
        WhatsAppProviderQuery::Events {
            account_id,
            kind,
            provider_chat_id,
            limit,
        } => Query::Events(wire::EventsQuery {
            account_id: account_id.clone(),
            kind: event_kind_to_wire(*kind),
            provider_chat_id: provider_chat_id.clone(),
            limit: *limit,
        }),
        WhatsAppProviderQuery::ClaimPendingCommands {
            account_id,
            host_claim_id,
            lease_seconds,
            limit,
        } => Query::ClaimPendingCommands(wire::ClaimPendingCommandsQuery {
            account_id: account_id.clone(),
            host_claim_id: host_claim_id.clone(),
            lease_seconds: *lease_seconds,
            limit: *limit,
        }),
    };
    wire::WhatsAppProviderQueryV1 { query: Some(query) }
}
