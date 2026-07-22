//! WhatsApp-owned projections and operation state. No business-domain storage.

pub mod durable;

pub use durable::{WHATSAPP_SCHEMA_V1, WhatsAppDurablePersistence, WhatsAppDurablePersistenceError};

use std::collections::HashMap;

use hermes_whatsapp_api::{
    WhatsAppAccount, WhatsAppDialog, WhatsAppMedia, WhatsAppMessage, WhatsAppParticipant,
    WhatsAppProviderCommand, WhatsAppProviderEvent, WhatsAppRealtimeFrame,
};
use hermes_whatsapp_core::{WhatsAppOperation, event_reconciles_command, operation_completed, operation_failed, operation_host_claimed};

pub const PACKAGE: &str = "hermes-whatsapp-persistence";

pub struct WhatsAppPersistence {
    accounts: HashMap<String, WhatsAppAccount>,
    operations: HashMap<String, WhatsAppOperation>,
    commands: HashMap<String, WhatsAppProviderCommand>,
    messages: HashMap<String, WhatsAppMessage>,
    dialogs: HashMap<String, WhatsAppDialog>,
    participants: HashMap<String, WhatsAppParticipant>,
    media: HashMap<String, WhatsAppMedia>,
    events: HashMap<String, Vec<WhatsAppProviderEvent>>,
}

impl WhatsAppPersistence {
    #[must_use]
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            operations: HashMap::new(),
            commands: HashMap::new(),
            messages: HashMap::new(),
            dialogs: HashMap::new(),
            participants: HashMap::new(),
            media: HashMap::new(),
            events: HashMap::new(),
        }
    }

    pub fn put_account(&mut self, account: WhatsAppAccount) {
        self.accounts.insert(account.account_id.clone(), account);
    }

    pub fn account(&self, account_id: &str) -> Option<&WhatsAppAccount> {
        self.accounts.get(account_id)
    }

    pub fn put_operation(&mut self, operation: WhatsAppOperation) {
        self.operations.insert(operation.operation_id.clone(), operation);
    }

    pub fn operation(&self, operation_id: &str) -> Option<&WhatsAppOperation> {
        self.operations.get(operation_id)
    }

    pub fn put_command(&mut self, command: WhatsAppProviderCommand) {
        let id = hermes_whatsapp_api::provider_command_operation_id(&command).to_owned();
        self.commands.insert(id, command);
    }

    pub fn command(&self, operation_id: &str) -> Option<&WhatsAppProviderCommand> {
        self.commands.get(operation_id)
    }

    pub fn pending_commands_for_account(&self, account_id: &str, limit: u32) -> Vec<WhatsAppProviderCommand> {
        self.commands
            .values()
            .filter(|command| hermes_whatsapp_api::provider_command_account_id(command) == account_id)
            .filter(|command| {
                self.operation(hermes_whatsapp_api::provider_command_operation_id(command))
                    .is_some_and(|operation| matches!(operation.state, hermes_whatsapp_core::WhatsAppOperationState::AwaitingProvider | hermes_whatsapp_core::WhatsAppOperationState::RetryScheduled))
            })
            .take(limit as usize)
            .cloned()
            .collect()
    }

    pub fn claim_pending_commands(
        &mut self,
        account_id: &str,
        claim_id: &str,
        until_unix_seconds: i64,
        limit: u32,
    ) -> Vec<WhatsAppProviderCommand> {
        let expired: Vec<String> = self
            .operations
            .values()
            .filter(|operation| {
                operation.state == hermes_whatsapp_core::WhatsAppOperationState::HostClaimed
                    && operation.host_claimed_until_unix_seconds.is_some_and(|until| until <= until_unix_seconds)
            })
            .map(|operation| operation.operation_id.clone())
            .collect();
        for operation_id in expired {
            if let Some(operation) = self.operations.get(&operation_id).cloned() {
                self.operations.insert(operation_id, hermes_whatsapp_core::operation_awaiting_provider(&operation));
            }
        }
        let operation_ids: Vec<String> = self
            .commands
            .values()
            .filter(|command| hermes_whatsapp_api::provider_command_account_id(command) == account_id)
            .filter_map(|command| {
                let operation_id = hermes_whatsapp_api::provider_command_operation_id(command);
                self.operation(operation_id).and_then(|operation| {
                    (matches!(operation.state, hermes_whatsapp_core::WhatsAppOperationState::AwaitingProvider | hermes_whatsapp_core::WhatsAppOperationState::RetryScheduled))
                        .then(|| operation_id.to_owned())
                })
            })
            .take(limit as usize)
            .collect();
        operation_ids
            .into_iter()
            .filter_map(|operation_id| {
                let operation = self.operations.get(&operation_id)?.clone();
                self.operations.insert(
                    operation_id.clone(),
                    operation_host_claimed(&operation, claim_id, until_unix_seconds),
                );
                self.commands.get(&operation_id).cloned()
            })
            .collect()
    }

    pub fn fail_claimed_command(
        &mut self,
        operation_id: &str,
        host_claim_id: &str,
        reason: impl Into<String>,
    ) -> bool {
        let Some(operation) = self.operations.get(operation_id).cloned() else {
            return false;
        };
        if operation.state != hermes_whatsapp_core::WhatsAppOperationState::HostClaimed
            || operation.host_claim_id.as_deref() != Some(host_claim_id)
        {
            return false;
        }
        self.operations.insert(
            operation_id.to_owned(),
            hermes_whatsapp_core::operation_failed(&operation, reason),
        );
        true
    }

    pub fn retry_command(&mut self, operation_id: &str) -> bool {
        let Some(operation) = self.operations.get(operation_id).cloned() else { return false; };
        if !matches!(operation.state, hermes_whatsapp_core::WhatsAppOperationState::Failed | hermes_whatsapp_core::WhatsAppOperationState::RetryScheduled) {
            return false;
        }
        self.operations.insert(
            operation_id.to_owned(),
            hermes_whatsapp_core::operation_retry_scheduled(&operation),
        );
        true
    }

    pub fn dead_letter_command(&mut self, operation_id: &str, reason: impl Into<String>) -> bool {
        let Some(operation) = self.operations.get(operation_id).cloned() else { return false; };
        if operation.state == hermes_whatsapp_core::WhatsAppOperationState::Completed
            || operation.state == hermes_whatsapp_core::WhatsAppOperationState::DeadLettered
        {
            return false;
        }
        self.operations.insert(
            operation_id.to_owned(),
            hermes_whatsapp_core::operation_dead_lettered(&operation, reason),
        );
        true
    }

    pub fn reconcile_event(&mut self, event: &WhatsAppProviderEvent) -> Vec<WhatsAppOperation> {
        let operation_ids: Vec<String> = self
            .commands
            .iter()
            .filter_map(|(operation_id, command)| {
                let operation = self.operations.get(operation_id)?;
                if matches!(operation.state, hermes_whatsapp_core::WhatsAppOperationState::AwaitingProvider | hermes_whatsapp_core::WhatsAppOperationState::HostClaimed)
                    && event_reconciles_command(event, command)
                {
                    Some(operation_id.clone())
                } else {
                    None
                }
            })
            .collect();
        operation_ids
            .into_iter()
            .filter_map(|operation_id| {
                let operation = self.operations.get(&operation_id).cloned()?;
                let updated = match event {
                    WhatsAppProviderEvent::CommandResultObserved { succeeded: true, .. } => {
                        operation_completed(&operation)
                    }
                    WhatsAppProviderEvent::CommandResultObserved { succeeded: false, .. } => {
                        operation_failed(&operation, "provider command failed")
                    }
                    _ => operation_completed(&operation),
                };
                self.operations.insert(operation_id, updated.clone());
                Some(updated)
            })
            .collect()
    }

    pub fn apply_event(&mut self, event: WhatsAppProviderEvent) {
        let account_id = hermes_whatsapp_api::provider_event_account_id(&event).to_owned();
        match &event {
            WhatsAppProviderEvent::RuntimeStateChanged { state, .. } => {
                if let Some(account) = self.accounts.get_mut(&account_id) {
                    account.runtime_state = *state;
                    account.account_state = match state {
                        hermes_whatsapp_api::WhatsAppRuntimeState::Running => {
                            hermes_whatsapp_api::WhatsAppAccountState::Linked
                        }
                        hermes_whatsapp_api::WhatsAppRuntimeState::Degraded => {
                            hermes_whatsapp_api::WhatsAppAccountState::Degraded
                        }
                        hermes_whatsapp_api::WhatsAppRuntimeState::Blocked => account.account_state,
                        hermes_whatsapp_api::WhatsAppRuntimeState::Stopped
                        | hermes_whatsapp_api::WhatsAppRuntimeState::Starting => {
                            account.account_state
                        }
                    };
                }
            }
            WhatsAppProviderEvent::SessionStateChanged { linked, secret_ref, revision, .. } => {
                if let Some(account) = self.accounts.get_mut(&account_id) {
                    if *linked {
                        if let (Some(secret_ref), Some(revision)) = (secret_ref, revision) {
                            account.credentials = vec![hermes_whatsapp_api::WhatsAppCredentialBinding {
                                purpose: hermes_whatsapp_api::WhatsAppCredentialPurpose::WebSessionKey,
                                secret_ref: secret_ref.clone(),
                                revision: *revision,
                            }];
                        }
                        account.account_state = hermes_whatsapp_api::WhatsAppAccountState::Linked;
                    } else {
                        account.credentials.clear();
                    }
                }
            }
            WhatsAppProviderEvent::MessageObserved(message) => {
                self.messages.insert(message.provider_message_id.clone(), message.clone());
            }
            WhatsAppProviderEvent::DialogObserved(dialog) => {
                self.dialogs.insert(dialog.provider_chat_id.clone(), dialog.clone());
            }
            WhatsAppProviderEvent::ParticipantObserved(participant) => {
                self.participants.insert(
                    format!("{}:{}", participant.provider_chat_id, participant.provider_identity_id),
                    participant.clone(),
                );
            }
            WhatsAppProviderEvent::MediaObserved(media) => {
                self.media.insert(media.provider_media_id.clone(), media.clone());
            }
            _ => {}
        }
        self.events.entry(account_id).or_default().push(event);
    }

    pub fn next_event_sequence(&self, account_id: &str) -> u64 {
        self.events
            .get(account_id)
            .map_or(1, |events| events.len() as u64 + 1)
    }

    pub fn messages_for_account(&self, account_id: &str) -> Vec<WhatsAppMessage> {
        self.messages.values().filter(|item| item.account_id == account_id).cloned().collect()
    }

    pub fn messages_for_query(
        &self,
        account_id: &str,
        provider_chat_id: Option<&str>,
        query: Option<&str>,
        limit: u32,
    ) -> Vec<WhatsAppMessage> {
        let query = query.map(str::to_lowercase);
        self.messages
            .values()
            .filter(|item| item.account_id == account_id)
            .filter(|item| provider_chat_id.is_none_or(|chat_id| item.provider_chat_id == chat_id))
            .filter(|item| query.as_ref().is_none_or(|text| item.text.as_ref().is_some_and(|value| value.to_lowercase().contains(text))))
            .take(limit as usize)
            .cloned()
            .collect()
    }

    pub fn dialogs_for_account(&self, account_id: &str) -> Vec<WhatsAppDialog> {
        self.dialogs.values().filter(|item| item.account_id == account_id).cloned().collect()
    }

    pub fn participants_for_chat(&self, account_id: &str, chat_id: &str) -> Vec<WhatsAppParticipant> {
        self.participants.values().filter(|item| item.account_id == account_id && item.provider_chat_id == chat_id).cloned().collect()
    }

    pub fn events_after(&self, account_id: &str, after_sequence: u64, limit: u32) -> Vec<WhatsAppRealtimeFrame> {
        self.events
            .get(account_id)
            .into_iter()
            .flatten()
            .enumerate()
            .map(|(index, event)| WhatsAppRealtimeFrame {
                account_id: account_id.to_owned(),
                sequence: index as u64 + 1,
                event: event.clone(),
            })
            .filter(|frame| frame.sequence > after_sequence)
            .take(limit as usize)
            .collect()
    }

    pub fn events_by_kind(
        &self,
        account_id: &str,
        kind: hermes_whatsapp_api::WhatsAppProviderEventKind,
        provider_chat_id: Option<&str>,
        limit: u32,
    ) -> Vec<WhatsAppProviderEvent> {
        self.events
            .get(account_id)
            .into_iter()
            .flatten()
            .filter(|event| hermes_whatsapp_api::provider_event_kind(event) == kind)
            .filter(|event| {
                provider_chat_id.is_none_or(|chat_id| {
                    hermes_whatsapp_api::provider_event_chat_id(event) == Some(chat_id)
                })
            })
            .take(limit as usize)
            .cloned()
            .collect()
    }
}

impl Default for WhatsAppPersistence {
    fn default() -> Self { Self::new() }
}
