//! Telegram integration policy and provider-neutral orchestration.

use hermes_communications_ingress::SourceEnvelope;
use hermes_telegram_api::{
    TelegramAccount, TelegramAccountState, TelegramContractError, TelegramCredentialBinding,
    TelegramChatStateProjection, TelegramDeliveryState, TelegramMessageMutation,
    TelegramMessageObservation, TelegramMessageProjection, TelegramOperation,
    TelegramOperationState, TelegramProviderCommand, TelegramProviderEvent,
    TelegramQrLoginSession, TelegramQrLoginState,
    TelegramReconciliationState, TelegramRuntimeLease, TelegramRuntimeLeaseState,
    provider_command_account_id, provider_command_kind,
    provider_command_operation_id,
    TelegramRuntimeState, validate_text,
};
use hermes_vault_protocol::{
    CredentialLeaseV1, DEFAULT_LEASE_TTL_SECONDS, SecretClassV1, VaultActionV1,
};

pub use hermes_vault_protocol::VaultPurposeRequestV1;
pub use hermes_vault_protocol::{CredentialLeaseV1 as TelegramCredentialLeaseV1, LeaseAudienceV1};

pub const PACKAGE: &str = "hermes-telegram-core";

pub const TELEGRAM_CREDENTIAL_PURPOSE_PREFIX: &str = "telegram.account";

pub fn credential_lease_purpose(
    account_id: &str,
    configuration_instance_id: &str,
    binding: &TelegramCredentialBinding,
) -> Result<VaultPurposeRequestV1, TelegramContractError> {
    let purpose_id = format!(
        "{TELEGRAM_CREDENTIAL_PURPOSE_PREFIX}.{account_id}.{}",
        binding.purpose.as_str()
    );
    let secret_class = if binding.purpose.is_session_store_key() {
        SecretClassV1::SessionStoreKey
    } else {
        SecretClassV1::ProviderCredential
    };
    VaultPurposeRequestV1::new(
        purpose_id,
        configuration_instance_id.to_owned(),
        vec![secret_class],
        vec![VaultActionV1::Resolve],
        DEFAULT_LEASE_TTL_SECONDS,
    )
    .map_err(|_| TelegramContractError::InvalidTransition)
}

pub fn credential_lease_purposes(
    account_id: &str,
    configuration_instance_id: &str,
    bindings: &[TelegramCredentialBinding],
) -> Result<Vec<VaultPurposeRequestV1>, TelegramContractError> {
    bindings
        .iter()
        .map(|binding| credential_lease_purpose(account_id, configuration_instance_id, binding))
        .collect()
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
    binding: &TelegramCredentialBinding,
    lease: &CredentialLeaseV1,
    now_unix_seconds: u64,
) -> Result<(), TelegramContractError> {
    let expected_purpose = credential_lease_purpose(
        account_id,
        configuration_instance_id,
        binding,
    )
    .map_err(|_| TelegramContractError::CredentialLeaseRejected)?;
    let request = lease.request();
    let audience = request.audience();
    let purpose = request.purpose();
    if request.logical_owner_id() != logical_owner_id
        || request.secret_revision() != binding.revision
        || request.vault_runtime_generation() != vault_runtime_generation
        || purpose != &expected_purpose
        || audience.module_registration_id() != module_registration_id
        || audience.runtime_instance_id() != runtime_instance_id
        || audience.runtime_generation() != runtime_generation
        || audience.grant_epoch() != grant_epoch
        || lease.issued_at_unix_seconds() > now_unix_seconds
        || lease.expires_at_unix_seconds() <= now_unix_seconds
    {
        return Err(TelegramContractError::CredentialLeaseRejected);
    }
    Ok(())
}

pub fn qr_login_preparing(
    setup_id: impl Into<String>,
    account_id: impl Into<String>,
    expires_at_unix_seconds: u64,
) -> TelegramQrLoginSession {
    TelegramQrLoginSession {
        setup_id: setup_id.into(),
        account_id: account_id.into(),
        state: TelegramQrLoginState::Preparing,
        qr_link: None,
        password_attempts: 0,
        expires_at_unix_seconds,
    }
}

pub fn qr_login_qr_issued(
    session: &TelegramQrLoginSession,
    qr_link: impl Into<String>,
) -> Result<TelegramQrLoginSession, TelegramContractError> {
    if session.state != TelegramQrLoginState::Preparing {
        return Err(TelegramContractError::InvalidTransition);
    }
    Ok(TelegramQrLoginSession {
        state: TelegramQrLoginState::WaitingQrScan,
        qr_link: Some(qr_link.into()),
        ..session.clone()
    })
}

pub fn qr_login_password_required(
    session: &TelegramQrLoginSession,
) -> Result<TelegramQrLoginSession, TelegramContractError> {
    if session.state != TelegramQrLoginState::WaitingQrScan {
        return Err(TelegramContractError::InvalidTransition);
    }
    Ok(TelegramQrLoginSession {
        state: TelegramQrLoginState::WaitingPassword,
        ..session.clone()
    })
}

pub fn qr_login_password_submitted(
    session: &TelegramQrLoginSession,
) -> Result<TelegramQrLoginSession, TelegramContractError> {
    if session.state != TelegramQrLoginState::WaitingPassword || session.password_attempts >= 5 {
        return Err(TelegramContractError::InvalidTransition);
    }
    Ok(TelegramQrLoginSession {
        password_attempts: session.password_attempts + 1,
        ..session.clone()
    })
}

pub fn qr_login_ready(
    session: &TelegramQrLoginSession,
) -> Result<TelegramQrLoginSession, TelegramContractError> {
    if !matches!(
        session.state,
        TelegramQrLoginState::WaitingQrScan | TelegramQrLoginState::WaitingPassword
    ) {
        return Err(TelegramContractError::InvalidTransition);
    }
    Ok(TelegramQrLoginSession {
        state: TelegramQrLoginState::Ready,
        ..session.clone()
    })
}

pub use hermes_communications_ingress::CommunicationObservationDraft;

#[derive(Default)]
pub struct TelegramLifecycle;

impl TelegramLifecycle {
    pub fn start(
        &self,
        account: &TelegramAccount,
        lease: &TelegramRuntimeLease,
        now_unix_seconds: u64,
    ) -> Result<TelegramAccount, TelegramContractError> {
        if account.state == TelegramAccountState::Retired {
            return Err(TelegramContractError::AccountRetired);
        }
        if lease.account_id != account.account_id
            || lease.state != TelegramRuntimeLeaseState::Active
            || lease.epoch == 0
            || lease.expires_at_unix_seconds <= now_unix_seconds
        {
            return Err(TelegramContractError::RuntimeBlocked);
        }
        Ok(TelegramAccount {
            runtime_state: TelegramRuntimeState::Starting,
            runtime_epoch: lease.epoch,
            ..account.clone()
        })
    }

    pub fn mark_running(&self, account: &TelegramAccount) -> TelegramAccount {
        TelegramAccount {
            state: TelegramAccountState::Ready,
            runtime_state: TelegramRuntimeState::Running,
            ..account.clone()
        }
    }

    pub fn stop(
        &self,
        account: &TelegramAccount,
        lease: &TelegramRuntimeLease,
    ) -> Result<TelegramAccount, TelegramContractError> {
        if lease.account_id != account.account_id || lease.epoch != account.runtime_epoch {
            return Err(TelegramContractError::RuntimeBlocked);
        }
        Ok(TelegramAccount {
            runtime_state: TelegramRuntimeState::Stopped,
            ..account.clone()
        })
    }

    pub fn retire(&self, account: &TelegramAccount) -> Result<TelegramAccount, TelegramContractError> {
        if account.state == TelegramAccountState::Retired {
            return Err(TelegramContractError::AccountRetired);
        }
        Ok(TelegramAccount {
            state: TelegramAccountState::Retired,
            runtime_state: TelegramRuntimeState::Stopped,
            ..account.clone()
        })
    }
}

pub fn accept_operation(command: &TelegramProviderCommand, lease_epoch: u64) -> TelegramOperation {
    let operation_id = provider_command_operation_id(command).to_owned();
    let command_kind = provider_command_kind(command);
    TelegramOperation {
        operation_id: operation_id.clone(),
        account_id: provider_command_account_id(command).to_owned(),
        command_kind,
        idempotency_key: format!("telegram:{}:{}", command_kind.as_str(), operation_id),
        state: TelegramOperationState::Accepted,
        retry_count: 0,
        max_retries: 3,
        lease_epoch,
        reconciliation: TelegramReconciliationState::NotObserved,
        last_error: None,
        next_attempt_at_unix_seconds: None,
        locked_at_unix_seconds: None,
        locked_by: None,
        provider_observed_at_unix_seconds: None,
        reconciled_at_unix_seconds: None,
    }
}

pub fn operation_running(operation: &TelegramOperation) -> TelegramOperation {
    TelegramOperation {
        state: TelegramOperationState::Running,
        ..operation.clone()
    }
}

pub fn operation_awaiting_provider(operation: &TelegramOperation) -> TelegramOperation {
    TelegramOperation {
        state: TelegramOperationState::AwaitingProvider,
        reconciliation: TelegramReconciliationState::AwaitingProvider,
        ..operation.clone()
    }
}

pub fn operation_completed(operation: &TelegramOperation) -> TelegramOperation {
    TelegramOperation {
        state: TelegramOperationState::Completed,
        reconciliation: TelegramReconciliationState::Observed,
        ..operation.clone()
    }
}

pub fn operation_failed(operation: &TelegramOperation, error: impl Into<String>) -> TelegramOperation {
    TelegramOperation {
        state: TelegramOperationState::Failed,
        last_error: Some(error.into()),
        ..operation.clone()
    }
}

pub fn operation_retry_scheduled(
    operation: &TelegramOperation,
    next_attempt_at_unix_seconds: u64,
    error: impl Into<String>,
) -> TelegramOperation {
    let state = if operation.retry_count >= operation.max_retries {
        TelegramOperationState::DeadLetter
    } else {
        TelegramOperationState::RetryScheduled
    };
    TelegramOperation {
        state,
        next_attempt_at_unix_seconds: (state == TelegramOperationState::RetryScheduled)
            .then_some(next_attempt_at_unix_seconds),
        locked_at_unix_seconds: None,
        locked_by: None,
        last_error: Some(error.into()),
        ..operation.clone()
    }
}

pub fn observation_draft(
    observation: TelegramMessageObservation,
) -> Result<CommunicationObservationDraft, TelegramContractError> {
    let source_id = format!(
        "telegram:{}:{}:{}",
        observation.account_id, observation.provider_chat_id, observation.provider_message_id
    );
    let text_preview = observation.text.and_then(|text| {
        let trimmed = text.trim().to_owned();
        (!trimmed.is_empty()).then_some(trimmed)
    });
    if let Some(text) = &text_preview {
        validate_text(text)?;
    }
    Ok(CommunicationObservationDraft {
        operation_id: source_id.clone(),
        source_id,
        source_kind: "telegram".to_owned(),
        text_preview,
        has_body: true,
        is_final_window: true,
    })
}

pub fn project_message(
    observation: &TelegramMessageObservation,
) -> Result<TelegramMessageProjection, TelegramContractError> {
    if observation.account_id.trim().is_empty()
        || observation.provider_chat_id.trim().is_empty()
        || observation.provider_message_id.trim().is_empty()
        || observation.sender_id.trim().is_empty()
    {
        return Err(TelegramContractError::EmptyField);
    }
    Ok(TelegramMessageProjection {
        message_id: format!(
            "telegram:{}:{}:{}",
            observation.account_id, observation.provider_chat_id, observation.provider_message_id
        ),
        account_id: observation.account_id.clone(),
        provider_chat_id: observation.provider_chat_id.clone(),
        provider_message_id: observation.provider_message_id.clone(),
        provider_topic_id: observation.provider_topic_id.clone(),
        sender_id: observation.sender_id.clone(),
        sender_display_name: observation.sender_display_name.clone(),
        text: observation.text.clone(),
        media: observation.media.clone(),
        references: observation.references.clone(),
        observed_at_unix_seconds: observation.observed_at_unix_seconds,
        delivery_state: TelegramDeliveryState::Received,
    })
}

pub fn event_message_mutation(
    event: &TelegramProviderEvent,
) -> Option<(&str, &str, &str, TelegramMessageMutation)> {
    match event {
        TelegramProviderEvent::MessageEdited {
            account_id,
            provider_chat_id,
            provider_message_id,
            text,
            observed_at_unix_seconds,
        } => Some((
            account_id,
            provider_chat_id,
            provider_message_id,
            TelegramMessageMutation::Edit {
                text: text.clone(),
                observed_at_unix_seconds: *observed_at_unix_seconds,
            },
        )),
        TelegramProviderEvent::MessageDeleted {
            account_id,
            provider_chat_id,
            provider_message_id,
            is_permanent,
        } => Some((
            account_id,
            provider_chat_id,
            provider_message_id,
            TelegramMessageMutation::Delete {
                is_permanent: *is_permanent,
            },
        )),
        TelegramProviderEvent::MessagePinned {
            account_id,
            provider_chat_id,
            provider_message_id,
            is_pinned,
        } => Some((
            account_id,
            provider_chat_id,
            provider_message_id,
            TelegramMessageMutation::Pin {
                is_pinned: *is_pinned,
            },
        )),
        TelegramProviderEvent::ReactionChanged {
            account_id,
            provider_chat_id,
            provider_message_id,
            emoji,
            is_active,
        } => Some((
            account_id,
            provider_chat_id,
            provider_message_id,
            TelegramMessageMutation::Reaction {
                emoji: emoji.clone(),
                is_active: *is_active,
            },
        )),
        _ => None,
    }
}

pub fn event_chat_state(event: &TelegramProviderEvent) -> Option<(&str, &str, TelegramChatStateProjection)> {
    match event {
        TelegramProviderEvent::ChatUnreadChanged {
            account_id,
            provider_chat_id,
            unread_count,
            unread_mention_count,
            last_read_inbox_message_id,
        } => Some((
            account_id,
            provider_chat_id,
            TelegramChatStateProjection {
                unread_count: *unread_count,
                unread_mention_count: *unread_mention_count,
                last_read_inbox_message_id: last_read_inbox_message_id.clone(),
                is_marked_as_unread: false,
            },
        )),
        TelegramProviderEvent::ChatMarkedUnreadChanged {
            account_id,
            provider_chat_id,
            is_marked_as_unread,
        } => Some((
            account_id,
            provider_chat_id,
            TelegramChatStateProjection {
                is_marked_as_unread: *is_marked_as_unread,
                ..TelegramChatStateProjection::default()
            },
        )),
        _ => None,
    }
}

pub fn provider_event_draft(
    event: &TelegramProviderEvent,
) -> Result<Option<CommunicationObservationDraft>, TelegramContractError> {
    match event {
        TelegramProviderEvent::MessageCreated(observation) => observation_draft(observation.clone()).map(Some),
        TelegramProviderEvent::MessageEdited {
            account_id,
            provider_chat_id,
            provider_message_id,
            text,
            ..
        } => event_draft(
            &format!("telegram:event:edited:{account_id}:{provider_chat_id}:{provider_message_id}"),
            text.clone(),
        ),
        TelegramProviderEvent::MessageDeleted {
            account_id,
            provider_chat_id,
            provider_message_id,
            ..
        }
        | TelegramProviderEvent::MessagePinned {
            account_id,
            provider_chat_id,
            provider_message_id,
            ..
        }
        | TelegramProviderEvent::ReactionChanged {
            account_id,
            provider_chat_id,
            provider_message_id,
            ..
        } => event_draft(
            &format!(
                "telegram:event:{}:{account_id}:{provider_chat_id}:{provider_message_id}",
                event_kind(event)
            ),
            None,
        ),
        TelegramProviderEvent::ReactionsObserved {
            account_id,
            provider_chat_id,
            provider_message_id,
            ..
        } => event_draft(
            &format!(
                "telegram:event:reactions:{account_id}:{provider_chat_id}:{provider_message_id}"
            ),
            None,
        ),
        TelegramProviderEvent::MessageSendFailed { account_id, provider_chat_id, old_provider_message_id, .. }
        | TelegramProviderEvent::MessageSendSucceeded { account_id, provider_chat_id, old_provider_message_id, .. } => event_draft(
            &format!("telegram:event:send-state:{account_id}:{provider_chat_id}:{old_provider_message_id}"),
            None,
        ),
        TelegramProviderEvent::ChatUnreadChanged { account_id, provider_chat_id, .. }
        | TelegramProviderEvent::ChatMarkedUnreadChanged { account_id, provider_chat_id, .. } => {
            event_draft(
                &format!("telegram:event:chat-state:{account_id}:{provider_chat_id}"),
                None,
            )
        }
        TelegramProviderEvent::TypingChanged(state) => event_draft(
            &format!(
                "telegram:event:typing:{}:{}:{}",
                state.account_id, state.provider_chat_id, state.sender_id
            ),
            None,
        ),
        TelegramProviderEvent::TopicChanged(topic) => event_draft(
            &format!(
                "telegram:event:topic:{}:{}:{}",
                topic.account_id, topic.provider_chat_id, topic.provider_topic_id
            ),
            Some(topic.title.clone()),
        ),
        TelegramProviderEvent::ChatPositionChanged(position) => event_draft(
            &format!(
                "telegram:event:chat-position:{}:{}",
                position.account_id, position.provider_chat_id
            ),
            None,
        ),
        TelegramProviderEvent::ChatFoldersChanged { account_id, .. } => event_draft(
            &format!("telegram:event:chat-folders:{account_id}"),
            None,
        ),
        TelegramProviderEvent::ChatNotificationChanged {
            account_id,
            provider_chat_id,
            ..
        } => event_draft(
            &format!("telegram:event:chat-notifications:{account_id}:{provider_chat_id}"),
            None,
        ),
        TelegramProviderEvent::ChatAvatarChanged(avatar) => event_draft(
            &format!(
                "telegram:event:chat-avatar:{}:{}",
                avatar.account_id, avatar.provider_chat_id
            ),
            None,
        ),
        TelegramProviderEvent::ParticipantChanged(participant) => event_draft(
            &format!(
                "telegram:event:participant:{}:{}:{}",
                participant.account_id,
                participant.provider_chat_id,
                participant.provider_member_id
            ),
            None,
        ),
        TelegramProviderEvent::FileChanged(file) => event_draft(
            &format!("telegram:event:file:{}:{}", file.account_id, file.provider_file_id),
            None,
        ),
    }
}

fn event_draft(
    source_id: &str,
    text: Option<String>,
) -> Result<Option<CommunicationObservationDraft>, TelegramContractError> {
    if let Some(value) = &text {
        validate_text(value)?;
    }
    let has_body = text.is_some();
    Ok(Some(CommunicationObservationDraft {
        operation_id: source_id.to_owned(),
        source_id: source_id.to_owned(),
        source_kind: "telegram".to_owned(),
        text_preview: text,
        has_body,
        is_final_window: true,
    }))
}

fn event_kind(event: &TelegramProviderEvent) -> &'static str {
    match event {
        TelegramProviderEvent::MessageEdited { .. } => "edited",
        TelegramProviderEvent::MessageDeleted { .. } => "deleted",
        TelegramProviderEvent::MessagePinned { .. } => "pinned",
        TelegramProviderEvent::ReactionChanged { .. } => "reaction",
        TelegramProviderEvent::MessageSendFailed { .. } => "send-failed",
        TelegramProviderEvent::MessageSendSucceeded { .. } => "send-succeeded",
        _ => "other",
    }
}

pub fn source_envelope(observation: &TelegramMessageObservation) -> SourceEnvelope {
    SourceEnvelope {
        source_kind: "telegram".to_owned(),
        source_id: format!(
            "telegram:{}:{}:{}",
            observation.account_id, observation.provider_chat_id, observation.provider_message_id
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hermes_telegram_api::{
        TelegramAccountSetup, TelegramCredentialPurpose, TelegramProviderKind,
    };
    use hermes_vault_protocol::{
        CredentialLeaseV1, LeaseAudienceV1, LeaseIdV1, SecretClassV1, VaultActionV1,
        VaultLeaseIssueRequestV1,
    };

    fn account_setup() -> TelegramAccountSetup {
        TelegramAccountSetup {
            account_id: "telegram-account".to_owned(),
            provider_kind: TelegramProviderKind::User,
            display_name: "Personal Telegram".to_owned(),
            external_account_id: "telegram:42".to_owned(),
            credentials: vec![TelegramCredentialBinding {
                purpose: TelegramCredentialPurpose::ApiHash,
                secret_ref: "secret:telegram:api-hash".to_owned(),
                revision: 1,
            }],
            qr_authorized: false,
        }
    }

    #[test]
    fn credential_purpose_is_scoped_to_provider_credential_and_resolve() {
        let binding = &account_setup().credentials[0];
        let purpose = credential_lease_purpose("telegram-account", "cfg-1", binding)
            .expect("valid Telegram Vault purpose");
        assert_eq!(purpose.allowed_secret_classes(), &[SecretClassV1::ProviderCredential]);
        assert_eq!(purpose.actions(), &[VaultActionV1::Resolve]);
    }

    #[test]
    fn credential_lease_requires_exact_scope_and_current_fences() {
        let binding = &account_setup().credentials[0];
        let purpose = credential_lease_purpose("telegram-account", "cfg-1", binding)
            .expect("valid Telegram Vault purpose");
        let audience = LeaseAudienceV1::new(
            "registration-1".to_owned(),
            "runtime-1".to_owned(),
            4,
            9,
        )
        .expect("valid lease audience");
        let request = VaultLeaseIssueRequestV1::new(
            "vault-1".to_owned(),
            7,
            binding.revision,
            "owner-1".to_owned(),
            purpose,
            audience,
        )
        .expect("valid lease request");
        let lease = CredentialLeaseV1::new(
            LeaseIdV1::new("a".repeat(32)).expect("valid lease id"),
            request,
            100,
            60,
            true,
        )
        .expect("valid credential lease");

        assert_eq!(
            validate_credential_lease(
                "telegram-account",
                "owner-1",
                "cfg-1",
                "registration-1",
                "runtime-1",
                4,
                9,
                7,
                binding,
                &lease,
                120,
            ),
            Ok(())
        );
        assert_eq!(
            validate_credential_lease(
                "telegram-account",
                "owner-1",
                "cfg-1",
                "registration-1",
                "runtime-1",
                5,
                9,
                7,
                binding,
                &lease,
                120,
            ),
            Err(TelegramContractError::CredentialLeaseRejected)
        );
    }

    #[test]
    fn qr_password_attempts_stop_after_five() {
        let mut session = qr_login_preparing("setup-1", "telegram-account", 100);
        session = qr_login_qr_issued(&session, "tg://login?token=x").expect("QR issued");
        session = qr_login_password_required(&session).expect("password requested");
        for _ in 0..5 {
            session = qr_login_password_submitted(&session).expect("attempt accepted");
        }
        assert_eq!(session.password_attempts, 5);
        assert_eq!(
            qr_login_password_submitted(&session),
            Err(TelegramContractError::InvalidTransition)
        );
    }

    #[test]
    fn projection_and_evidence_use_provider_source_without_business_entity() {
        let observation = TelegramMessageObservation {
            account_id: "telegram-account".to_owned(),
            provider_chat_id: "100".to_owned(),
            provider_message_id: "200".to_owned(),
            provider_topic_id: None,
            sender_id: "42".to_owned(),
            sender_display_name: Some("Owner".to_owned()),
            text: Some("hello".to_owned()),
            media: None,
            references: hermes_telegram_api::TelegramMessageReferences::default(),
            observed_at_unix_seconds: 10,
        };
        let projection = project_message(&observation).expect("projection");
        let draft = observation_draft(observation).expect("neutral evidence draft");
        assert_eq!(projection.message_id, "telegram:telegram-account:100:200");
        assert_eq!(draft.source_kind, "telegram");
        assert_eq!(draft.source_id, "telegram:telegram-account:100:200");
    }

    #[test]
    fn runtime_lifecycle_rejects_expired_or_foreign_lease() {
        let account = TelegramAccount {
            account_id: "telegram-account".to_owned(),
            provider_kind: TelegramProviderKind::User,
            display_name: "Personal Telegram".to_owned(),
            external_account_id: "telegram:42".to_owned(),
            state: TelegramAccountState::Provisioning,
            runtime_state: TelegramRuntimeState::Stopped,
            runtime_epoch: 0,
        };
        let lifecycle = TelegramLifecycle;
        let expired = TelegramRuntimeLease {
            account_id: account.account_id.clone(),
            topology: "process".to_owned(),
            holder: "runtime-1".to_owned(),
            epoch: 1,
            state: TelegramRuntimeLeaseState::Active,
            expires_at_unix_seconds: 10,
        };
        assert_eq!(
            lifecycle.start(&account, &expired, 10),
            Err(TelegramContractError::RuntimeBlocked)
        );

        let foreign = TelegramRuntimeLease {
            account_id: "another-account".to_owned(),
            expires_at_unix_seconds: 100,
            ..expired
        };
        assert_eq!(
            lifecycle.start(&account, &foreign, 1),
            Err(TelegramContractError::RuntimeBlocked)
        );
    }
}
