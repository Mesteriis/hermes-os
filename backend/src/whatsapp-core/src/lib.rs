//! WhatsApp-owned lifecycle and operation policy.

use hermes_whatsapp_api::{
    WhatsAppAccount, WhatsAppAccountState, WhatsAppContractError, WhatsAppProviderCommand,
    WhatsAppProviderShape, WhatsAppRuntimeKind, WhatsAppRuntimeState,
    WhatsAppConversationCommandKind,
    provider_command_account_id, provider_command_operation_id, validate_provider_command,
    WhatsAppProviderEvent,
    WhatsAppCredentialBinding,
};
use hermes_vault_protocol::{
    CredentialLeaseV1, DEFAULT_LEASE_TTL_SECONDS, SecretClassV1, VaultActionV1,
    VaultPurposeRequestV1,
};
use serde::{Deserialize, Serialize};

pub const PACKAGE: &str = "hermes-whatsapp-core";
pub const WHATSAPP_CREDENTIAL_PURPOSE_PREFIX: &str = "whatsapp.account";

pub fn credential_lease_purpose(
    account_id: &str,
    configuration_instance_id: &str,
    binding: &WhatsAppCredentialBinding,
) -> Result<VaultPurposeRequestV1, WhatsAppContractError> {
    let purpose_id = format!(
        "{WHATSAPP_CREDENTIAL_PURPOSE_PREFIX}.{account_id}.{}",
        binding.purpose.as_str()
    );
    VaultPurposeRequestV1::new(
        purpose_id,
        configuration_instance_id.to_owned(),
        vec![SecretClassV1::SessionCredentialBlob],
        vec![VaultActionV1::Resolve],
        DEFAULT_LEASE_TTL_SECONDS,
    )
    .map_err(|_| WhatsAppContractError::InvalidTransition)
}

pub fn validate_credential_lease(
    account_id: &str,
    logical_owner_id: &str,
    configuration_instance_id: &str,
    module_registration_id: &str,
    runtime_instance_id: &str,
    runtime_generation: u64,
    grant_epoch: u64,
    vault_runtime_generation: u64,
    binding: &WhatsAppCredentialBinding,
    lease: &CredentialLeaseV1,
    now_unix_seconds: u64,
) -> Result<(), WhatsAppContractError> {
    let expected_purpose = credential_lease_purpose(
        account_id,
        configuration_instance_id,
        binding,
    )
    .map_err(|_| WhatsAppContractError::CredentialLeaseRejected)?;
    let request = lease.request();
    let audience = request.audience();
    if request.logical_owner_id() != logical_owner_id
        || request.secret_revision() != binding.revision
        || request.vault_runtime_generation() != vault_runtime_generation
        || request.purpose() != &expected_purpose
        || audience.module_registration_id() != module_registration_id
        || audience.runtime_instance_id() != runtime_instance_id
        || audience.runtime_generation() != runtime_generation
        || audience.grant_epoch() != grant_epoch
        || lease.issued_at_unix_seconds() > now_unix_seconds
        || lease.expires_at_unix_seconds() <= now_unix_seconds
    {
        return Err(WhatsAppContractError::CredentialLeaseRejected);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WhatsAppOperationState {
    Accepted,
    Running,
    AwaitingProvider,
    HostClaimed,
    Completed,
    Failed,
    RetryScheduled,
    DeadLettered,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WhatsAppReconciliationState {
    NotObserved,
    AwaitingProvider,
    Observed,
    Mismatch,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WhatsAppOperation {
    pub operation_id: String,
    pub account_id: String,
    pub command_kind: String,
    pub state: WhatsAppOperationState,
    pub reconciliation: WhatsAppReconciliationState,
    pub last_error: Option<String>,
    pub host_claim_id: Option<String>,
    pub host_claimed_until_unix_seconds: Option<i64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhatsAppTransportError {
    Unavailable,
    Rejected,
    Protocol,
}

pub fn command_kind(command: &WhatsAppProviderCommand) -> &'static str {
    match command {
        WhatsAppProviderCommand::SendText { .. } => "send_text",
        WhatsAppProviderCommand::Reply { .. } => "reply",
        WhatsAppProviderCommand::Forward { .. } => "forward",
        WhatsAppProviderCommand::Edit { .. } => "edit",
        WhatsAppProviderCommand::Delete { .. } => "delete",
        WhatsAppProviderCommand::React { .. } => "react",
        WhatsAppProviderCommand::Unreact { .. } => "unreact",
        WhatsAppProviderCommand::SendMedia { .. } => "send_media",
        WhatsAppProviderCommand::SendVoiceNote { .. } => "send_voice_note",
        WhatsAppProviderCommand::DownloadMedia { .. } => "download_media",
        WhatsAppProviderCommand::PublishStatus { .. } => "publish_status",
        WhatsAppProviderCommand::JoinConversation { .. } => "join_group",
        WhatsAppProviderCommand::LeaveConversation { .. } => "leave_group",
        WhatsAppProviderCommand::Conversation { action, .. } => action.as_str(),
    }
}

pub fn accept_operation(command: &WhatsAppProviderCommand) -> Result<WhatsAppOperation, WhatsAppContractError> {
    validate_provider_command(command)?;
    Ok(WhatsAppOperation {
        operation_id: provider_command_operation_id(command).to_owned(),
        account_id: provider_command_account_id(command).to_owned(),
        command_kind: command_kind(command).to_owned(),
        state: WhatsAppOperationState::Accepted,
        reconciliation: WhatsAppReconciliationState::NotObserved,
        last_error: None,
        host_claim_id: None,
        host_claimed_until_unix_seconds: None,
    })
}

pub fn operation_running(operation: &WhatsAppOperation) -> WhatsAppOperation {
    WhatsAppOperation { state: WhatsAppOperationState::Running, ..operation.clone() }
}

pub fn operation_awaiting_provider(operation: &WhatsAppOperation) -> WhatsAppOperation {
    WhatsAppOperation {
        state: WhatsAppOperationState::AwaitingProvider,
        reconciliation: WhatsAppReconciliationState::AwaitingProvider,
        host_claim_id: None,
        host_claimed_until_unix_seconds: None,
        ..operation.clone()
    }
}

pub fn operation_host_claimed(
    operation: &WhatsAppOperation,
    claim_id: impl Into<String>,
    until_unix_seconds: i64,
) -> WhatsAppOperation {
    WhatsAppOperation {
        state: WhatsAppOperationState::HostClaimed,
        reconciliation: WhatsAppReconciliationState::AwaitingProvider,
        host_claim_id: Some(claim_id.into()),
        host_claimed_until_unix_seconds: Some(until_unix_seconds),
        ..operation.clone()
    }
}

pub fn operation_completed(operation: &WhatsAppOperation) -> WhatsAppOperation {
    WhatsAppOperation {
        state: WhatsAppOperationState::Completed,
        reconciliation: WhatsAppReconciliationState::Observed,
        host_claim_id: None,
        host_claimed_until_unix_seconds: None,
        ..operation.clone()
    }
}

pub fn operation_failed(operation: &WhatsAppOperation, error: impl Into<String>) -> WhatsAppOperation {
    WhatsAppOperation {
        state: WhatsAppOperationState::Failed,
        last_error: Some(error.into()),
        host_claim_id: None,
        host_claimed_until_unix_seconds: None,
        ..operation.clone()
    }
}

pub fn operation_retry_scheduled(operation: &WhatsAppOperation) -> WhatsAppOperation {
    WhatsAppOperation {
        state: WhatsAppOperationState::RetryScheduled,
        reconciliation: WhatsAppReconciliationState::AwaitingProvider,
        last_error: None,
        host_claim_id: None,
        host_claimed_until_unix_seconds: None,
        ..operation.clone()
    }
}

pub fn operation_dead_lettered(
    operation: &WhatsAppOperation,
    reason: impl Into<String>,
) -> WhatsAppOperation {
    WhatsAppOperation {
        state: WhatsAppOperationState::DeadLettered,
        last_error: Some(reason.into()),
        host_claim_id: None,
        host_claimed_until_unix_seconds: None,
        ..operation.clone()
    }
}

pub fn event_reconciles_command(
    event: &WhatsAppProviderEvent,
    command: &WhatsAppProviderCommand,
) -> bool {
    match (event, command) {
        (
            WhatsAppProviderEvent::CommandResultObserved {
                operation_id,
                ..
            },
            command,
        ) => operation_id == provider_command_operation_id(command),
        (
            WhatsAppProviderEvent::DialogObserved(dialog),
            WhatsAppProviderCommand::Conversation {
                account_id,
                provider_chat_id,
                action,
                ..
            },
        ) if &dialog.account_id == account_id && &dialog.provider_chat_id == provider_chat_id => {
            match action {
                WhatsAppConversationCommandKind::MarkRead => dialog.is_unread == Some(false),
                WhatsAppConversationCommandKind::MarkUnread => dialog.is_unread == Some(true),
                WhatsAppConversationCommandKind::Archive => dialog.is_archived == Some(true),
                WhatsAppConversationCommandKind::Unarchive => dialog.is_archived == Some(false),
                WhatsAppConversationCommandKind::Mute => dialog.is_muted == Some(true),
                WhatsAppConversationCommandKind::Unmute => dialog.is_muted == Some(false),
                WhatsAppConversationCommandKind::Pin => dialog.is_pinned == Some(true),
                WhatsAppConversationCommandKind::Unpin => dialog.is_pinned == Some(false),
            }
        }
        (
            WhatsAppProviderEvent::MessageEdited { account_id, provider_chat_id, provider_message_id, .. },
            WhatsAppProviderCommand::Edit { account_id: command_account_id, provider_chat_id: command_chat_id, provider_message_id: command_message_id, .. },
        ) => account_id == command_account_id && provider_chat_id == command_chat_id && provider_message_id == command_message_id,
        (
            WhatsAppProviderEvent::MessageDeleted { account_id, provider_chat_id, provider_message_id, .. },
            WhatsAppProviderCommand::Delete { account_id: command_account_id, provider_chat_id: command_chat_id, provider_message_id: command_message_id, .. },
        ) => account_id == command_account_id && provider_chat_id == command_chat_id && provider_message_id == command_message_id,
        (
            WhatsAppProviderEvent::ReactionChanged { account_id, provider_chat_id, provider_message_id, emoji, is_active, .. },
            WhatsAppProviderCommand::React { account_id: command_account_id, provider_chat_id: command_chat_id, provider_message_id: command_message_id, emoji: command_emoji, .. },
        ) => account_id == command_account_id && provider_chat_id == command_chat_id && provider_message_id == command_message_id && emoji.as_deref() == Some(command_emoji.as_str()) && *is_active,
        (
            WhatsAppProviderEvent::ReactionChanged { account_id, provider_chat_id, provider_message_id, emoji, is_active, .. },
            WhatsAppProviderCommand::Unreact { account_id: command_account_id, provider_chat_id: command_chat_id, provider_message_id: command_message_id, emoji: command_emoji, .. },
        ) => account_id == command_account_id && provider_chat_id == command_chat_id && provider_message_id == command_message_id && emoji.as_deref() == Some(command_emoji.as_str()) && !*is_active,
        (
            WhatsAppProviderEvent::MediaObserved(media),
            WhatsAppProviderCommand::DownloadMedia { account_id, provider_chat_id, provider_media_id, .. },
        ) => &media.account_id == account_id && &media.provider_chat_id == provider_chat_id && &media.provider_media_id == provider_media_id,
        _ => false,
    }
}

pub fn validate_account_runtime(account: &WhatsAppAccount) -> Result<(), WhatsAppContractError> {
    if account.provider_shape != WhatsAppProviderShape::WebCompanion
        || account.runtime_kind != WhatsAppRuntimeKind::HiddenWebView
    {
        return Err(WhatsAppContractError::InvalidTransition);
    }
    if account.account_state == WhatsAppAccountState::Retired
        || account.account_state == WhatsAppAccountState::Revoked
        || account.runtime_state == WhatsAppRuntimeState::Blocked
    {
        return Err(WhatsAppContractError::InvalidTransition);
    }
    Ok(())
}
