//! WhatsApp provider runtime. WebView execution is supplied by a host-owned port.

pub mod bootstrap;
pub mod client_port;
pub mod client_transport;
pub mod managed_control;
pub mod process;
pub mod vault_credentials;

use hermes_whatsapp_api::{
    capabilities::{capability_catalog, WhatsAppCapability, WhatsAppCapabilityScope},
    WhatsAppAccount, WhatsAppAccountSetup, WhatsAppContractError, WhatsAppProviderCommand,
    WhatsAppProviderEvent, WhatsAppRuntimeState, WhatsAppAccountState, validate_account_setup, validate_event,
};
use hermes_whatsapp_api::host_bridge::{
    WhatsAppHostBridgeEnvelopeV1, WhatsAppHostObservationV1, validate_host_bridge_envelope,
};
use hermes_whatsapp_core::{
    WhatsAppOperation, WhatsAppTransportError, accept_operation, operation_awaiting_provider,
    operation_completed, operation_failed, operation_running, validate_account_runtime,
};
use hermes_whatsapp_persistence::WhatsAppPersistence;
use hermes_whatsapp_persistence::{
    WhatsAppDurablePersistence, WhatsAppDurablePersistenceError,
};
use std::time::{SystemTime, UNIX_EPOCH};

pub const PACKAGE: &str = "hermes-whatsapp-runtime";

pub trait WhatsAppProviderTransport {
    fn execute(&mut self, command: &WhatsAppProviderCommand) -> Result<WhatsAppTransportResponse, WhatsAppTransportError>;
    fn poll_events(&mut self) -> Result<Vec<WhatsAppProviderEvent>, WhatsAppTransportError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WhatsAppTransportResponse {
    Accepted,
    Completed,
}

pub struct WhatsAppRuntime<T> {
    transport: T,
    persistence: WhatsAppPersistence,
}

impl<T: WhatsAppProviderTransport> WhatsAppRuntime<T> {
    pub fn capabilities(&self, scope: Option<&WhatsAppCapabilityScope>) -> Vec<WhatsAppCapability> {
        capability_catalog(scope)
    }

    pub fn runtime_status(
        &self,
        account_id: &str,
    ) -> hermes_whatsapp_api::WhatsAppRuntimeStatus {
        let account = self.persistence.account(account_id);
        let scope = account.map(|value| WhatsAppCapabilityScope {
            runtime_kind: Some(value.runtime_kind.as_str().to_owned()),
            lifecycle_state: Some(value.account_state.as_str().to_owned()),
            live_runtime_available: false,
            live_send_available: false,
            media_download_available: false,
            media_upload_available: false,
        });
        hermes_whatsapp_api::WhatsAppRuntimeStatus {
            account_id: account_id.to_owned(),
            account_state: account.map(|value| value.account_state),
            runtime_state: account.map(|value| value.runtime_state),
            capabilities: capability_catalog(scope.as_ref()),
            host_command_queue_available: true,
        }
    }
    #[must_use]
    pub fn new(transport: T) -> Self {
        Self { transport, persistence: WhatsAppPersistence::new() }
    }

    pub fn provision_account(
        &mut self,
        setup: WhatsAppAccountSetup,
    ) -> Result<WhatsAppAccount, WhatsAppContractError> {
        validate_account_setup(&setup)?;
        let account = WhatsAppAccount {
            account_id: setup.account_id,
            display_name: setup.display_name,
            external_account_id: setup.external_account_id,
            provider_shape: setup.provider_shape,
            runtime_kind: setup.runtime_kind,
            account_state: hermes_whatsapp_api::WhatsAppAccountState::LinkRequired,
            runtime_state: WhatsAppRuntimeState::Stopped,
            credentials: setup.credentials,
        };
        self.persistence.put_account(account.clone());
        Ok(account)
    }

    pub fn start_account(
        &mut self,
        account_id: &str,
    ) -> Result<WhatsAppAccount, WhatsAppTransportError> {
        let mut account = self
            .persistence
            .account(account_id)
            .cloned()
            .ok_or(WhatsAppTransportError::Rejected)?;
        if matches!(account.account_state, WhatsAppAccountState::Revoked | WhatsAppAccountState::Retired) {
            return Err(WhatsAppTransportError::Rejected);
        }
        account.runtime_state = WhatsAppRuntimeState::Starting;
        self.persistence.put_account(account.clone());
        Ok(account)
    }

    pub fn stop_account(
        &mut self,
        account_id: &str,
    ) -> Result<WhatsAppAccount, WhatsAppTransportError> {
        let mut account = self
            .persistence
            .account(account_id)
            .cloned()
            .ok_or(WhatsAppTransportError::Rejected)?;
        account.runtime_state = WhatsAppRuntimeState::Stopped;
        self.persistence.put_account(account.clone());
        Ok(account)
    }

    pub fn revoke_account(
        &mut self,
        account_id: &str,
    ) -> Result<WhatsAppAccount, WhatsAppTransportError> {
        let mut account = self
            .persistence
            .account(account_id)
            .cloned()
            .ok_or(WhatsAppTransportError::Rejected)?;
        account.account_state = WhatsAppAccountState::Revoked;
        account.runtime_state = WhatsAppRuntimeState::Stopped;
        account.credentials.clear();
        self.persistence.put_account(account.clone());
        self.record_session_revocation(account_id)?;
        Ok(account)
    }

    pub fn relink_account(
        &mut self,
        account_id: &str,
    ) -> Result<WhatsAppAccount, WhatsAppTransportError> {
        let mut account = self
            .persistence
            .account(account_id)
            .cloned()
            .ok_or(WhatsAppTransportError::Rejected)?;
        if account.account_state == WhatsAppAccountState::Retired {
            return Err(WhatsAppTransportError::Rejected);
        }
        account.account_state = WhatsAppAccountState::LinkRequired;
        account.runtime_state = WhatsAppRuntimeState::Stopped;
        account.credentials.clear();
        self.persistence.put_account(account.clone());
        self.record_session_revocation(account_id)?;
        Ok(account)
    }

    pub fn remove_account(
        &mut self,
        account_id: &str,
    ) -> Result<WhatsAppAccount, WhatsAppTransportError> {
        let mut account = self
            .persistence
            .account(account_id)
            .cloned()
            .ok_or(WhatsAppTransportError::Rejected)?;
        account.account_state = WhatsAppAccountState::Retired;
        account.runtime_state = WhatsAppRuntimeState::Stopped;
        account.credentials.clear();
        self.persistence.put_account(account.clone());
        self.record_session_revocation(account_id)?;
        Ok(account)
    }

    fn record_session_revocation(&mut self, account_id: &str) -> Result<(), WhatsAppTransportError> {
        let observed_at_unix_seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| WhatsAppTransportError::Protocol)?
            .as_secs();
        let observed_at_unix_seconds = i64::try_from(observed_at_unix_seconds)
            .map_err(|_| WhatsAppTransportError::Protocol)?;
        self.persistence.apply_event(WhatsAppProviderEvent::SessionStateChanged {
            account_id: account_id.to_owned(),
            linked: false,
            secret_ref: None,
            revision: None,
            observed_at_unix_seconds,
        });
        Ok(())
    }

    pub fn execute_command(
        &mut self,
        command: WhatsAppProviderCommand,
    ) -> Result<WhatsAppOperation, WhatsAppTransportError> {
        let account_id = hermes_whatsapp_api::provider_command_account_id(&command);
        let account = self.persistence.account(account_id).ok_or(WhatsAppTransportError::Rejected)?;
        validate_account_runtime(account).map_err(|_| WhatsAppTransportError::Rejected)?;
        let operation = accept_operation(&command).map_err(|_| WhatsAppTransportError::Rejected)?;
        if let Some(existing) = self.persistence.operation(&operation.operation_id) {
            return Ok(existing.clone());
        }
        self.persistence.put_command(command.clone());
        let running = operation_running(&operation);
        self.persistence.put_operation(running.clone());
        match self.transport.execute(&command) {
            Ok(WhatsAppTransportResponse::Completed) => {
                let completed = operation_completed(&running);
                self.persistence.put_operation(completed.clone());
                Ok(completed)
            }
            Ok(WhatsAppTransportResponse::Accepted) => {
                let waiting = operation_awaiting_provider(&running);
                self.persistence.put_operation(waiting.clone());
                Ok(waiting)
            }
            Err(error) => {
                let failed = operation_failed(&running, "WhatsApp provider execution failed");
                self.persistence.put_operation(failed);
                Err(error)
            }
        }
    }

    pub fn poll_events(&mut self) -> Result<usize, WhatsAppTransportError> {
        let events = self.transport.poll_events()?;
        for event in events.iter().cloned() {
            validate_event(&event).map_err(|_| WhatsAppTransportError::Protocol)?;
            self.persistence.reconcile_event(&event);
            self.persistence.apply_event(event);
        }
        Ok(events.len())
    }

    pub fn ingest_host_observation(
        &mut self,
        envelope: WhatsAppHostBridgeEnvelopeV1,
    ) -> Result<Vec<WhatsAppOperation>, WhatsAppTransportError> {
        validate_host_bridge_envelope(&envelope)
            .map_err(|_| WhatsAppTransportError::Protocol)?;
        let account_id = envelope.account_id.clone();
        let observed_at = envelope.observed_at_unix_seconds;
        let event = match envelope.observation {
            WhatsAppHostObservationV1::RuntimeState { state } => {
                WhatsAppProviderEvent::RuntimeStateChanged {
                    account_id,
                    state: parse_runtime_state(&state)?,
                    observed_at_unix_seconds: observed_at,
                }
            }
            WhatsAppHostObservationV1::SessionLinked { secret_ref, revision } => {
                WhatsAppProviderEvent::SessionStateChanged {
                    account_id,
                    linked: true,
                    secret_ref: Some(secret_ref),
                    revision: Some(revision),
                    observed_at_unix_seconds: observed_at,
                }
            }
            WhatsAppHostObservationV1::SessionRevoked => {
                WhatsAppProviderEvent::SessionStateChanged {
                    account_id,
                    linked: false,
                    secret_ref: None,
                    revision: None,
                    observed_at_unix_seconds: observed_at,
                }
            }
            WhatsAppHostObservationV1::CommandResult {
                operation_id,
                provider_request_id,
                succeeded,
            } => WhatsAppProviderEvent::CommandResultObserved {
                account_id,
                operation_id,
                provider_request_id,
                succeeded,
                observed_at_unix_seconds: observed_at,
            },
            WhatsAppHostObservationV1::MessageIdentity {
                provider_chat_id,
                provider_message_id,
                sender_id,
            } => WhatsAppProviderEvent::MessageObserved(hermes_whatsapp_api::WhatsAppMessage {
                account_id,
                provider_chat_id,
                provider_message_id,
                sender_id,
                sender_display_name: String::new(),
                text: None,
                reply_to_provider_message_id: None,
                occurred_at_unix_seconds: observed_at,
            }),
            WhatsAppHostObservationV1::MessageUpdated {
                provider_chat_id,
                provider_message_id,
            } => WhatsAppProviderEvent::MessageEdited {
                account_id,
                provider_chat_id,
                provider_message_id,
                text: None,
                observed_at_unix_seconds: observed_at,
            },
            WhatsAppHostObservationV1::MessageDeleted {
                provider_chat_id,
                provider_message_id,
            } => WhatsAppProviderEvent::MessageDeleted {
                account_id,
                provider_chat_id,
                provider_message_id,
                observed_at_unix_seconds: observed_at,
            },
            WhatsAppHostObservationV1::Receipt {
                provider_chat_id,
                provider_message_id,
                delivery_state,
            } => WhatsAppProviderEvent::ReceiptChanged {
                account_id,
                provider_chat_id,
                provider_message_id,
                delivery_state,
                observed_at_unix_seconds: observed_at,
            },
            WhatsAppHostObservationV1::Reaction {
                provider_chat_id,
                provider_message_id,
                actor_id,
                emoji,
                is_active,
            } => WhatsAppProviderEvent::ReactionChanged {
                account_id,
                provider_chat_id,
                provider_message_id,
                actor_id,
                emoji,
                is_active,
                observed_at_unix_seconds: observed_at,
            },
            WhatsAppHostObservationV1::Dialog {
                provider_chat_id,
                title,
                kind,
            } => WhatsAppProviderEvent::DialogObserved(hermes_whatsapp_api::WhatsAppDialog {
                account_id,
                provider_chat_id,
                title,
                kind,
                is_archived: None,
                is_pinned: None,
                is_muted: None,
                is_unread: None,
                unread_count: None,
                participant_count: None,
                observed_at_unix_seconds: observed_at,
            }),
            WhatsAppHostObservationV1::Participant {
                provider_chat_id,
                provider_identity_id,
                display_name,
            } => WhatsAppProviderEvent::ParticipantObserved(hermes_whatsapp_api::WhatsAppParticipant {
                account_id,
                provider_chat_id,
                provider_identity_id,
                display_name,
                role: String::new(),
                status: String::new(),
                is_self: false,
                observed_at_unix_seconds: observed_at,
            }),
            WhatsAppHostObservationV1::Presence {
                provider_chat_id,
                provider_identity_id,
                state,
            } => WhatsAppProviderEvent::PresenceChanged {
                account_id,
                provider_chat_id,
                provider_identity_id,
                state,
                observed_at_unix_seconds: observed_at,
            },
            WhatsAppHostObservationV1::MediaMetadata {
                provider_chat_id,
                provider_message_id,
                provider_media_id,
                media_kind,
            } => WhatsAppProviderEvent::MediaObserved(hermes_whatsapp_api::WhatsAppMedia {
                account_id,
                provider_chat_id,
                provider_message_id,
                provider_media_id,
                media_kind,
                filename: None,
                content_type: None,
                declared_size: None,
                observed_at_unix_seconds: observed_at,
            }),
            WhatsAppHostObservationV1::CallMetadata {
                provider_call_id,
                provider_chat_id,
                direction,
                state,
            } => WhatsAppProviderEvent::CallObserved {
                account_id,
                provider_call_id,
                provider_chat_id,
                direction,
                state,
                observed_at_unix_seconds: observed_at,
            },
            WhatsAppHostObservationV1::StatusMetadata {
                provider_status_id,
                sender_id,
            } => WhatsAppProviderEvent::StatusObserved {
                account_id,
                provider_status_id,
                sender_id,
                text: None,
                observed_at_unix_seconds: observed_at,
            },
            WhatsAppHostObservationV1::StatusViewMetadata {
                provider_status_id,
                viewer_id,
            } => WhatsAppProviderEvent::StatusViewObserved {
                account_id,
                provider_status_id,
                viewer_id,
                observed_at_unix_seconds: observed_at,
            },
            WhatsAppHostObservationV1::StatusDeletedMetadata { provider_status_id } => {
                WhatsAppProviderEvent::StatusDeleted {
                    account_id,
                    provider_status_id,
                    observed_at_unix_seconds: observed_at,
                }
            }
        };
        validate_event(&event).map_err(|_| WhatsAppTransportError::Protocol)?;
        let reconciled = self.persistence.reconcile_event(&event);
        self.persistence.apply_event(event);
        Ok(reconciled)
    }

    pub async fn provision_account_durable(
        &mut self,
        durable: &WhatsAppDurablePersistence,
        setup: WhatsAppAccountSetup,
    ) -> Result<WhatsAppAccount, WhatsAppDurableRuntimeError> {
        let account = self
            .provision_account(setup)
            .map_err(WhatsAppDurableRuntimeError::Contract)?;
        durable
            .upsert_account(&account)
            .await
            .map_err(WhatsAppDurableRuntimeError::Persistence)?;
        Ok(account)
    }

    pub async fn ingest_host_observation_durable(
        &mut self,
        durable: &WhatsAppDurablePersistence,
        envelope: WhatsAppHostBridgeEnvelopeV1,
    ) -> Result<(), WhatsAppDurableRuntimeError> {
        let account_id = envelope.account_id.clone();
        if self.persistence.account(&account_id).is_none() {
            if let Some(account) = durable
                .account(&account_id)
                .await
                .map_err(WhatsAppDurableRuntimeError::Persistence)?
            {
                self.persistence.put_account(account);
            }
        }
        let reconciled = self
            .ingest_host_observation(envelope)
            .map_err(WhatsAppDurableRuntimeError::Transport)?;
        for operation in reconciled {
            durable
                .save_operation(&operation)
                .await
                .map_err(WhatsAppDurableRuntimeError::Persistence)?;
        }
        let frame = self
            .persistence
            .events_after(&account_id, self.persistence.next_event_sequence(&account_id).saturating_sub(2), 1)
            .into_iter()
            .last()
            .ok_or(WhatsAppDurableRuntimeError::Persistence(
                WhatsAppDurablePersistenceError::InvalidRow,
            ))?;
        durable
            .append_event(&frame)
            .await
            .map_err(WhatsAppDurableRuntimeError::Persistence)?;
        match &frame.event {
            hermes_whatsapp_api::WhatsAppProviderEvent::MessageObserved(message) => durable
                .upsert_message(message)
                .await
                .map_err(WhatsAppDurableRuntimeError::Persistence)?,
            hermes_whatsapp_api::WhatsAppProviderEvent::DialogObserved(dialog) => durable
                .upsert_dialog(dialog)
                .await
                .map_err(WhatsAppDurableRuntimeError::Persistence)?,
            hermes_whatsapp_api::WhatsAppProviderEvent::ParticipantObserved(participant) => durable
                .upsert_participant(participant)
                .await
                .map_err(WhatsAppDurableRuntimeError::Persistence)?,
            hermes_whatsapp_api::WhatsAppProviderEvent::MediaObserved(media) => durable
                .upsert_media(media)
                .await
                .map_err(WhatsAppDurableRuntimeError::Persistence)?,
            hermes_whatsapp_api::WhatsAppProviderEvent::RuntimeStateChanged { .. } => {
                if let Some(account) = self.persistence.account(&account_id) {
                    durable
                        .upsert_account(account)
                        .await
                        .map_err(WhatsAppDurableRuntimeError::Persistence)?;
                }
            }
            hermes_whatsapp_api::WhatsAppProviderEvent::SessionStateChanged { .. } => {
                if let Some(account) = self.persistence.account(&account_id) {
                    durable
                        .upsert_account(account)
                        .await
                        .map_err(WhatsAppDurableRuntimeError::Persistence)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub async fn execute_command_durable(
        &mut self,
        durable: &WhatsAppDurablePersistence,
        command: WhatsAppProviderCommand,
    ) -> Result<WhatsAppOperation, WhatsAppDurableRuntimeError> {
        let account_id = hermes_whatsapp_api::provider_command_account_id(&command);
        if self.persistence.account(account_id).is_none() {
            if let Some(account) = durable
                .account(account_id)
                .await
                .map_err(WhatsAppDurableRuntimeError::Persistence)?
            {
                self.persistence.put_account(account);
            }
        }
        let command_copy = command.clone();
        let result = self.queue_command_for_host(command);
        durable
            .save_command(&command_copy)
            .await
            .map_err(WhatsAppDurableRuntimeError::Persistence)?;
        let operation_id = hermes_whatsapp_api::provider_command_operation_id(&command_copy);
        if let Some(operation) = self.persistence.operation(operation_id) {
            durable
                .save_operation(operation)
                .await
                .map_err(WhatsAppDurableRuntimeError::Persistence)?;
        }
        result.map_err(WhatsAppDurableRuntimeError::Transport)
    }

    fn queue_command_for_host(
        &mut self,
        command: WhatsAppProviderCommand,
    ) -> Result<WhatsAppOperation, WhatsAppTransportError> {
        let account_id = hermes_whatsapp_api::provider_command_account_id(&command);
        let account = self
            .persistence
            .account(account_id)
            .ok_or(WhatsAppTransportError::Rejected)?;
        validate_account_runtime(account).map_err(|_| WhatsAppTransportError::Rejected)?;
        let operation = accept_operation(&command).map_err(|_| WhatsAppTransportError::Rejected)?;
        if let Some(existing) = self.persistence.operation(&operation.operation_id) {
            return Ok(existing.clone());
        }
        self.persistence.put_command(command);
        let waiting = operation_awaiting_provider(&operation_running(&operation));
        self.persistence.put_operation(waiting.clone());
        Ok(waiting)
    }

    pub async fn claim_pending_commands_durable(
        &self,
        durable: &WhatsAppDurablePersistence,
        account_id: &str,
        claim_id: &str,
        lease_seconds: i64,
        limit: i64,
    ) -> Result<Vec<WhatsAppProviderCommand>, WhatsAppDurableRuntimeError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| WhatsAppDurableRuntimeError::Transport(WhatsAppTransportError::Protocol))?
            .as_secs();
        let now = i64::try_from(now)
            .map_err(|_| WhatsAppDurableRuntimeError::Transport(WhatsAppTransportError::Protocol))?;
        durable
            .claim_pending_commands(account_id, claim_id, now, lease_seconds, limit)
            .await
            .map_err(WhatsAppDurableRuntimeError::Persistence)
    }

    pub async fn fail_claimed_command_durable(
        &self,
        durable: &WhatsAppDurablePersistence,
        operation_id: &str,
        claim_id: &str,
        reason: impl Into<String>,
    ) -> Result<bool, WhatsAppDurableRuntimeError> {
        durable
            .fail_claimed_command(operation_id, claim_id, reason)
            .await
            .map_err(WhatsAppDurableRuntimeError::Persistence)
    }

    pub fn retry_command(&mut self, operation_id: &str) -> Result<(), WhatsAppTransportError> {
        self.persistence
            .retry_command(operation_id)
            .then_some(())
            .ok_or(WhatsAppTransportError::Rejected)
    }

    pub fn dead_letter_command(
        &mut self,
        operation_id: &str,
        reason: impl Into<String>,
    ) -> Result<(), WhatsAppTransportError> {
        self.persistence
            .dead_letter_command(operation_id, reason)
            .then_some(())
            .ok_or(WhatsAppTransportError::Rejected)
    }

    pub async fn retry_command_durable(
        &mut self,
        durable: &WhatsAppDurablePersistence,
        operation_id: &str,
    ) -> Result<(), WhatsAppDurableRuntimeError> {
        durable
            .retry_command(operation_id)
            .await
            .map_err(WhatsAppDurableRuntimeError::Persistence)?
            .then_some(())
            .ok_or(WhatsAppDurableRuntimeError::Transport(WhatsAppTransportError::Rejected))?;
        if let Some(operation) = durable
            .operation(operation_id)
            .await
            .map_err(WhatsAppDurableRuntimeError::Persistence)?
        {
            self.persistence.put_operation(operation);
        }
        Ok(())
    }

    pub async fn dead_letter_command_durable(
        &mut self,
        durable: &WhatsAppDurablePersistence,
        operation_id: &str,
        reason: impl Into<String>,
    ) -> Result<(), WhatsAppDurableRuntimeError> {
        durable
            .dead_letter_command(operation_id, reason)
            .await
            .map_err(WhatsAppDurableRuntimeError::Persistence)?
            .then_some(())
            .ok_or(WhatsAppDurableRuntimeError::Transport(WhatsAppTransportError::Rejected))?;
        if let Some(operation) = durable
            .operation(operation_id)
            .await
            .map_err(WhatsAppDurableRuntimeError::Persistence)?
        {
            self.persistence.put_operation(operation);
        }
        Ok(())
    }

    pub async fn poll_events_durable(
        &mut self,
        durable: &WhatsAppDurablePersistence,
    ) -> Result<usize, WhatsAppDurableRuntimeError> {
        let events = self
            .transport
            .poll_events()
            .map_err(WhatsAppDurableRuntimeError::Transport)?;
        for event in events.iter().cloned() {
            validate_event(&event).map_err(|_| WhatsAppDurableRuntimeError::Contract(
                WhatsAppContractError::InvalidTimestamp,
            ))?;
            let account_id = hermes_whatsapp_api::provider_event_account_id(&event).to_owned();
            let reconciled_operations = self.persistence.reconcile_event(&event);
            for operation in reconciled_operations {
                durable
                    .save_operation(&operation)
                    .await
                    .map_err(WhatsAppDurableRuntimeError::Persistence)?;
            }
            let frame = hermes_whatsapp_api::WhatsAppRealtimeFrame {
                account_id: account_id.clone(),
                sequence: self.persistence.next_event_sequence(&account_id),
                event: event.clone(),
            };
            durable
                .append_event(&frame)
                .await
                .map_err(WhatsAppDurableRuntimeError::Persistence)?;
            match &event {
                hermes_whatsapp_api::WhatsAppProviderEvent::MessageObserved(message) => durable
                    .upsert_message(message)
                    .await
                    .map_err(WhatsAppDurableRuntimeError::Persistence)?,
                hermes_whatsapp_api::WhatsAppProviderEvent::DialogObserved(dialog) => durable
                    .upsert_dialog(dialog)
                    .await
                    .map_err(WhatsAppDurableRuntimeError::Persistence)?,
                hermes_whatsapp_api::WhatsAppProviderEvent::ParticipantObserved(participant) => durable
                    .upsert_participant(participant)
                    .await
                    .map_err(WhatsAppDurableRuntimeError::Persistence)?,
                hermes_whatsapp_api::WhatsAppProviderEvent::MediaObserved(media) => durable
                    .upsert_media(media)
                    .await
                    .map_err(WhatsAppDurableRuntimeError::Persistence)?,
                _ => {}
            }
            self.persistence.apply_event(event);
        }
        Ok(events.len())
    }

    #[must_use]
    pub fn persistence(&self) -> &WhatsAppPersistence {
        &self.persistence
    }

    pub fn persistence_mut(&mut self) -> &mut WhatsAppPersistence {
        &mut self.persistence
    }
}

fn parse_runtime_state(value: &str) -> Result<WhatsAppRuntimeState, WhatsAppTransportError> {
    match value {
        "stopped" => Ok(WhatsAppRuntimeState::Stopped),
        "starting" => Ok(WhatsAppRuntimeState::Starting),
        "running" => Ok(WhatsAppRuntimeState::Running),
        "degraded" => Ok(WhatsAppRuntimeState::Degraded),
        "blocked" => Ok(WhatsAppRuntimeState::Blocked),
        _ => Err(WhatsAppTransportError::Protocol),
    }
}

#[derive(Debug)]
pub enum WhatsAppDurableRuntimeError {
    Contract(WhatsAppContractError),
    Persistence(WhatsAppDurablePersistenceError),
    Transport(WhatsAppTransportError),
}
