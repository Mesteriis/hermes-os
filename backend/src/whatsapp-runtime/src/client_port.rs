//! WhatsApp-owned typed client port and module-envelope codec.

use hermes_runtime_protocol::v1::{ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1};
use hermes_whatsapp_api::{
    MAX_TEXT_BYTES, WhatsAppClientRequest, WhatsAppClientResponse, WhatsAppContractError,
    WhatsAppProviderQuery, WhatsAppProviderQueryResponse, validate_provider_query,
    provider_query_account_id,
    host_bridge::{decode_host_bridge_payload, encode_host_bridge_payload},
    client_wire::{decode_command, decode_query, encode_command, encode_query, encode_query_response},
};
use hermes_whatsapp_core::WhatsAppTransportError;
use hermes_whatsapp_persistence::{WhatsAppDurablePersistence, WhatsAppDurablePersistenceError};
use prost::Message;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{WhatsAppProviderTransport, WhatsAppRuntime};

const MODULE_CLIENT_PROTOCOL_MAJOR: u32 = 1;
const WHATSAPP_MODULE_ID: &str = "hermes-whatsapp-runtime";
const WHATSAPP_OWNER_ID: &str = "whatsapp";
const WHATSAPP_CLIENT_CONTRACT_NAME: &str = "whatsapp.client";

#[derive(Debug)]
pub enum WhatsAppClientPortError {
    Provider(WhatsAppTransportError),
    Protocol(String),
    Codec(String),
}

fn contract() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: WHATSAPP_OWNER_ID.to_owned(),
        name: WHATSAPP_CLIENT_CONTRACT_NAME.to_owned(),
        major: 1,
        revision: 1,
        schema_sha256: Vec::new(),
    }
}

pub struct WhatsAppClientPort<'a, T> {
    runtime: &'a mut WhatsAppRuntime<T>,
}

impl<'a, T: WhatsAppProviderTransport> WhatsAppClientPort<'a, T> {
    pub fn new(runtime: &'a mut WhatsAppRuntime<T>) -> Self {
        Self { runtime }
    }

    pub fn handle(
        &mut self,
        request: WhatsAppClientRequest,
    ) -> Result<WhatsAppClientResponse, WhatsAppClientPortError> {
        match request {
            WhatsAppClientRequest::Lifecycle(request) => self
                .apply_lifecycle(request)
                .map(WhatsAppClientResponse::Account)
                .map_err(WhatsAppClientPortError::Provider),
            WhatsAppClientRequest::HostObservation(envelope) => {
                let provider_event_id = envelope.provider_event_id.clone();
                self.runtime
                    .ingest_host_observation(envelope)
                    .map_err(WhatsAppClientPortError::Provider)?;
                Ok(WhatsAppClientResponse::ObservationAccepted { provider_event_id })
            }
            WhatsAppClientRequest::HostCommandFailed {
                operation_id,
                host_claim_id,
                reason,
            } => {
                if !self.runtime.persistence_mut().fail_claimed_command(
                    &operation_id,
                    &host_claim_id,
                    reason,
                ) {
                    return Err(WhatsAppClientPortError::Protocol(
                        "WhatsApp host command failure claim is not admitted".to_owned(),
                    ));
                }
                Ok(WhatsAppClientResponse::HostCommandFailureRecorded { operation_id })
            }
            WhatsAppClientRequest::RetryCommand { operation_id } => {
                self.runtime
                    .retry_command(&operation_id)
                    .map_err(WhatsAppClientPortError::Provider)?;
                Ok(WhatsAppClientResponse::CommandLifecycleUpdated {
                    operation_id,
                    state: "retry_scheduled".to_owned(),
                })
            }
            WhatsAppClientRequest::DeadLetterCommand { operation_id, reason } => {
                self.runtime
                    .dead_letter_command(&operation_id, reason)
                    .map_err(WhatsAppClientPortError::Provider)?;
                Ok(WhatsAppClientResponse::CommandLifecycleUpdated {
                    operation_id,
                    state: "dead_lettered".to_owned(),
                })
            }
            WhatsAppClientRequest::Command(command) => self
                .runtime
                .execute_command(command)
                .map(|operation| WhatsAppClientResponse::Accepted {
                    operation_id: operation.operation_id,
                })
                .map_err(WhatsAppClientPortError::Provider),
            WhatsAppClientRequest::Query(query) => self.query(query),
        }
    }

    fn query(
        &mut self,
        query: WhatsAppProviderQuery,
    ) -> Result<WhatsAppClientResponse, WhatsAppClientPortError> {
        validate_provider_query(&query).map_err(contract_error)?;
        let account_id = provider_query_account_id(&query).to_owned();
        let response = match query {
            WhatsAppProviderQuery::Account { .. } => {
                WhatsAppProviderQueryResponse::Account(
                    self.runtime.persistence().account(&account_id).cloned(),
                )
            }
            WhatsAppProviderQuery::RuntimeStatus { .. } => {
                WhatsAppProviderQueryResponse::RuntimeStatus(
                    self.runtime.runtime_status(&account_id),
                )
            }
            WhatsAppProviderQuery::CachedMessages {
                provider_chat_id,
                limit,
                ..
            } => WhatsAppProviderQueryResponse::Messages(self.runtime.persistence().messages_for_query(
                &account_id,
                provider_chat_id.as_deref(),
                None,
                limit,
            )),
            WhatsAppProviderQuery::SearchMessages {
                provider_chat_id,
                query,
                limit,
                ..
            } => WhatsAppProviderQueryResponse::Messages(self.runtime.persistence().messages_for_query(
                &account_id,
                provider_chat_id.as_deref(),
                Some(&query),
                limit,
            )),
            WhatsAppProviderQuery::Dialogs { limit, .. } => {
                let mut dialogs = self.runtime.persistence().dialogs_for_account(&account_id);
                dialogs.truncate(limit as usize);
                WhatsAppProviderQueryResponse::Dialogs(dialogs)
            }
            WhatsAppProviderQuery::Participants {
                provider_chat_id,
                limit,
                ..
            } => {
                let mut participants = self
                    .runtime
                    .persistence()
                    .participants_for_chat(&account_id, &provider_chat_id);
                participants.truncate(limit as usize);
                WhatsAppProviderQueryResponse::Participants(participants)
            }
            WhatsAppProviderQuery::Replay {
                after_sequence,
                limit,
                ..
            } => WhatsAppProviderQueryResponse::Realtime(
                self.runtime
                    .persistence()
                    .events_after(&account_id, after_sequence, limit),
            ),
            WhatsAppProviderQuery::PendingCommands { limit, .. } => {
                WhatsAppProviderQueryResponse::Commands(
                    self.runtime
                        .persistence()
                        .pending_commands_for_account(&account_id, limit),
                )
            }
            WhatsAppProviderQuery::Events {
                kind,
                provider_chat_id,
                limit,
                ..
            } => WhatsAppProviderQueryResponse::Events(
                self.runtime.persistence().events_by_kind(
                    &account_id,
                    kind,
                    provider_chat_id.as_deref(),
                    limit,
                ),
            ),
            WhatsAppProviderQuery::ClaimPendingCommands {
                host_claim_id,
                lease_seconds,
                limit,
                ..
            } => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|_| WhatsAppClientPortError::Protocol("WhatsApp host clock is invalid".to_owned()))?
                    .as_secs();
                let until = now
                    .checked_add(u64::from(lease_seconds))
                    .and_then(|value| i64::try_from(value).ok())
                    .ok_or_else(|| WhatsAppClientPortError::Protocol("WhatsApp host lease deadline is invalid".to_owned()))?;
                WhatsAppProviderQueryResponse::Commands(
                    self.runtime.persistence_mut().claim_pending_commands(
                        &account_id,
                        &host_claim_id,
                        until,
                        limit,
                    ),
                )
            }
        };
        Ok(WhatsAppClientResponse::Query(response))
    }

    pub fn handle_module_request(
        &mut self,
        bytes: &[u8],
    ) -> Result<Vec<u8>, WhatsAppClientPortError> {
        let (request_id, request) = decode_module_request(bytes)?;
        let response = self.handle(request)?;
        encode_module_response(request_id, &response)
    }

    pub async fn handle_module_request_durable(
        &mut self,
        bytes: &[u8],
        durable: &WhatsAppDurablePersistence,
    ) -> Result<Vec<u8>, WhatsAppClientPortError> {
        let (request_id, request) = decode_module_request(bytes)?;
        let response = match request {
            WhatsAppClientRequest::Lifecycle(
                hermes_whatsapp_api::WhatsAppLifecycleRequest::Provision(setup),
            ) => self
                .runtime
                .provision_account_durable(durable, setup)
                .await
                .map(WhatsAppClientResponse::Account)
                .map_err(|error| WhatsAppClientPortError::Protocol(format!("{error:?}")))?,
            WhatsAppClientRequest::Lifecycle(request) => {
                let account_id = lifecycle_account_id(&request).map(str::to_owned);
                if let Some(account_id) = lifecycle_account_id(&request) {
                    if self.runtime.persistence().account(account_id).is_none() {
                        if let Some(account) = durable
                            .account(account_id)
                            .await
                            .map_err(|error| WhatsAppClientPortError::Protocol(format!("{error:?}")))?
                        {
                            self.runtime.persistence_mut().put_account(account);
                        }
                    }
                }
                let before_sequence = account_id
                    .as_deref()
                    .map(|value| self.runtime.persistence().next_event_sequence(value));
                let account = self
                    .apply_lifecycle(request)
                    .map_err(WhatsAppClientPortError::Provider)?;
                if let (Some(account_id), Some(before_sequence)) = (account_id.as_deref(), before_sequence) {
                    if self.runtime.persistence().next_event_sequence(account_id) > before_sequence {
                        if let Some(frame) = self
                            .runtime
                            .persistence()
                            .events_after(account_id, before_sequence.saturating_sub(1), 1)
                            .into_iter()
                            .last()
                        {
                            durable
                                .append_event(&frame)
                                .await
                                .map_err(|error| WhatsAppClientPortError::Protocol(format!("{error:?}")))?;
                        }
                    }
                }
                durable
                    .upsert_account(&account)
                    .await
                    .map_err(|error| WhatsAppClientPortError::Protocol(format!("{error:?}")))?;
                WhatsAppClientResponse::Account(account)
            }
            WhatsAppClientRequest::HostObservation(envelope) => {
                let provider_event_id = envelope.provider_event_id.clone();
                self.runtime
                    .ingest_host_observation_durable(durable, envelope)
                    .await
                    .map_err(|error| WhatsAppClientPortError::Protocol(format!("{error:?}")))?;
                WhatsAppClientResponse::ObservationAccepted { provider_event_id }
            }
            WhatsAppClientRequest::HostCommandFailed {
                operation_id,
                host_claim_id,
                reason,
            } => {
                let recorded = self
                    .runtime
                    .fail_claimed_command_durable(
                        durable,
                        &operation_id,
                        &host_claim_id,
                        reason,
                    )
                    .await
                    .map_err(|error| WhatsAppClientPortError::Protocol(format!("{error:?}")))?;
                if !recorded {
                    return Err(WhatsAppClientPortError::Protocol(
                        "WhatsApp durable host command failure claim is not admitted".to_owned(),
                    ));
                }
                WhatsAppClientResponse::HostCommandFailureRecorded { operation_id }
            }
            WhatsAppClientRequest::RetryCommand { operation_id } => {
                self.runtime
                    .retry_command_durable(durable, &operation_id)
                    .await
                    .map_err(|error| WhatsAppClientPortError::Protocol(format!("{error:?}")))?;
                WhatsAppClientResponse::CommandLifecycleUpdated {
                    operation_id,
                    state: "retry_scheduled".to_owned(),
                }
            }
            WhatsAppClientRequest::DeadLetterCommand { operation_id, reason } => {
                self.runtime
                    .dead_letter_command_durable(durable, &operation_id, reason)
                    .await
                    .map_err(|error| WhatsAppClientPortError::Protocol(format!("{error:?}")))?;
                WhatsAppClientResponse::CommandLifecycleUpdated {
                    operation_id,
                    state: "dead_lettered".to_owned(),
                }
            }
            WhatsAppClientRequest::Command(command) => self
                .runtime
                .execute_command_durable(durable, command)
                .await
                .map(|operation| WhatsAppClientResponse::Accepted {
                    operation_id: operation.operation_id,
                })
                .map_err(|error| WhatsAppClientPortError::Protocol(format!("{error:?}")))?,
            WhatsAppClientRequest::Query(query) => self.query_durable(query, durable).await?,
        };
        encode_module_response(request_id, &response)
    }

    fn apply_lifecycle(
        &mut self,
        request: hermes_whatsapp_api::WhatsAppLifecycleRequest,
    ) -> Result<hermes_whatsapp_api::WhatsAppAccount, WhatsAppClientPortError> {
        match request {
            hermes_whatsapp_api::WhatsAppLifecycleRequest::Provision(setup) => self
                .runtime
                .provision_account(setup)
                .map_err(WhatsAppClientPortError::Provider),
            hermes_whatsapp_api::WhatsAppLifecycleRequest::Start { account_id } => self
                .runtime
                .start_account(&account_id)
                .map_err(WhatsAppClientPortError::Provider),
            hermes_whatsapp_api::WhatsAppLifecycleRequest::Stop { account_id } => self
                .runtime
                .stop_account(&account_id)
                .map_err(WhatsAppClientPortError::Provider),
            hermes_whatsapp_api::WhatsAppLifecycleRequest::Revoke { account_id } => self
                .runtime
                .revoke_account(&account_id)
                .map_err(WhatsAppClientPortError::Provider),
            hermes_whatsapp_api::WhatsAppLifecycleRequest::Relink { account_id } => self
                .runtime
                .relink_account(&account_id)
                .map_err(WhatsAppClientPortError::Provider),
            hermes_whatsapp_api::WhatsAppLifecycleRequest::Remove { account_id } => self
                .runtime
                .remove_account(&account_id)
                .map_err(WhatsAppClientPortError::Provider),
        }
    }

    async fn query_durable(
        &mut self,
        query: WhatsAppProviderQuery,
        durable: &WhatsAppDurablePersistence,
    ) -> Result<WhatsAppClientResponse, WhatsAppClientPortError> {
        validate_provider_query(&query).map_err(contract_error)?;
        let account_id = provider_query_account_id(&query).to_owned();
        let response = match query {
            WhatsAppProviderQuery::Account { .. } => WhatsAppProviderQueryResponse::Account(
                durable
                    .account(&account_id)
                    .await
                    .map_err(|error| WhatsAppClientPortError::Protocol(format!("{error:?}")))?,
            ),
            WhatsAppProviderQuery::RuntimeStatus { .. } => {
                if self.runtime.persistence().account(&account_id).is_none() {
                    if let Some(account) = durable
                        .account(&account_id)
                        .await
                        .map_err(|error| WhatsAppClientPortError::Protocol(format!("{error:?}")))?
                    {
                        self.runtime.persistence_mut().put_account(account);
                    }
                }
                WhatsAppProviderQueryResponse::RuntimeStatus(
                    self.runtime.runtime_status(&account_id),
                )
            }
            WhatsAppProviderQuery::CachedMessages {
                provider_chat_id,
                limit,
                ..
            } => WhatsAppProviderQueryResponse::Messages(
                durable
                    .messages(&account_id, provider_chat_id.as_deref(), None, i64::from(limit))
                    .await
                    .map_err(|error| WhatsAppClientPortError::Protocol(format!("{error:?}")))?,
            ),
            WhatsAppProviderQuery::SearchMessages {
                provider_chat_id,
                query,
                limit,
                ..
            } => WhatsAppProviderQueryResponse::Messages(
                durable
                    .messages(
                        &account_id,
                        provider_chat_id.as_deref(),
                        Some(&query),
                        i64::from(limit),
                    )
                    .await
                    .map_err(|error| WhatsAppClientPortError::Protocol(format!("{error:?}")))?,
            ),
            WhatsAppProviderQuery::Dialogs { limit, .. } => WhatsAppProviderQueryResponse::Dialogs(
                durable
                    .dialogs(&account_id, i64::from(limit))
                    .await
                    .map_err(|error| WhatsAppClientPortError::Protocol(format!("{error:?}")))?,
            ),
            WhatsAppProviderQuery::Participants {
                provider_chat_id,
                limit,
                ..
            } => WhatsAppProviderQueryResponse::Participants(
                durable
                    .participants(&account_id, &provider_chat_id, i64::from(limit))
                    .await
                    .map_err(|error| WhatsAppClientPortError::Protocol(format!("{error:?}")))?,
            ),
            WhatsAppProviderQuery::Replay {
                after_sequence,
                limit,
                ..
            } => WhatsAppProviderQueryResponse::Realtime(
                durable
                    .replay_events_after(&account_id, after_sequence, i64::from(limit))
                    .await
                    .map_err(|error| WhatsAppClientPortError::Protocol(format!("{error:?}")))?,
            ),
            WhatsAppProviderQuery::PendingCommands { limit, .. } => {
                WhatsAppProviderQueryResponse::Commands(
                    durable
                        .pending_commands(&account_id, i64::from(limit))
                        .await
                        .map_err(|error| WhatsAppClientPortError::Protocol(format!("{error:?}")))?,
                )
            }
            WhatsAppProviderQuery::Events {
                kind,
                provider_chat_id,
                limit,
                ..
            } => WhatsAppProviderQueryResponse::Events(
                durable
                    .events_by_kind(
                        &account_id,
                        kind,
                        provider_chat_id.as_deref(),
                        i64::from(limit),
                    )
                    .await
                    .map_err(|error| WhatsAppClientPortError::Protocol(format!("{error:?}")))?,
            ),
            WhatsAppProviderQuery::ClaimPendingCommands {
                host_claim_id,
                lease_seconds,
                limit,
                ..
            } => WhatsAppProviderQueryResponse::Commands(
                self.runtime
                    .claim_pending_commands_durable(
                        durable,
                        &account_id,
                        &host_claim_id,
                        i64::from(lease_seconds),
                        i64::from(limit),
                    )
                    .await
                    .map_err(|error| WhatsAppClientPortError::Protocol(format!("{error:?}")))?,
            ),
        };
        Ok(WhatsAppClientResponse::Query(response))
    }

    pub async fn replay_durable(
        &self,
        durable: &WhatsAppDurablePersistence,
        account_id: &str,
        after_sequence: u64,
        limit: u32,
    ) -> Result<WhatsAppClientResponse, WhatsAppClientPortError> {
        if limit == 0 || limit > 500 {
            return Err(WhatsAppClientPortError::Protocol(
                "WhatsApp replay limit is invalid".to_owned(),
            ));
        }
        durable
            .replay_events_after(account_id, after_sequence, i64::from(limit))
            .await
            .map(|frames| {
                WhatsAppClientResponse::Query(WhatsAppProviderQueryResponse::Realtime(frames))
            })
            .map_err(|error| match error {
                WhatsAppDurablePersistenceError::Codec => {
                    WhatsAppClientPortError::Codec("WhatsApp durable replay payload is invalid".to_owned())
                }
                WhatsAppDurablePersistenceError::Database => WhatsAppClientPortError::Protocol(
                    "WhatsApp durable replay is unavailable".to_owned(),
                ),
                WhatsAppDurablePersistenceError::InvalidRow => WhatsAppClientPortError::Protocol(
                    "WhatsApp durable replay row is invalid".to_owned(),
                ),
            })
    }
}

fn lifecycle_account_id(
    request: &hermes_whatsapp_api::WhatsAppLifecycleRequest,
) -> Option<&str> {
    match request {
        hermes_whatsapp_api::WhatsAppLifecycleRequest::Provision(setup) => {
            Some(&setup.account_id)
        }
        hermes_whatsapp_api::WhatsAppLifecycleRequest::Start { account_id }
        | hermes_whatsapp_api::WhatsAppLifecycleRequest::Stop { account_id }
        | hermes_whatsapp_api::WhatsAppLifecycleRequest::Revoke { account_id }
        | hermes_whatsapp_api::WhatsAppLifecycleRequest::Relink { account_id }
        | hermes_whatsapp_api::WhatsAppLifecycleRequest::Remove { account_id } => Some(account_id),
    }
}

pub fn encode_module_request(
    request_id: u64,
    request: &WhatsAppClientRequest,
) -> Result<Vec<u8>, WhatsAppClientPortError> {
    if request_id == 0 {
        return Err(WhatsAppClientPortError::Protocol(
            "WhatsApp request id must be non-zero".to_owned(),
        ));
    }
    let payload = match request {
        WhatsAppClientRequest::Lifecycle(request) => encode_lifecycle_request(request),
        WhatsAppClientRequest::HostObservation(envelope) => encode_host_bridge_payload(envelope)
            .map_err(|error| WhatsAppClientPortError::Codec(format!("{error:?}")))?,
        WhatsAppClientRequest::HostCommandFailed {
            operation_id,
            host_claim_id,
            reason,
        } => hermes_whatsapp_api::wire::WhatsAppHostCommandFailureV1 {
            operation_id: operation_id.clone(),
            host_claim_id: host_claim_id.clone(),
            reason: reason.clone(),
        }
        .encode_to_vec(),
        WhatsAppClientRequest::RetryCommand { operation_id } => hermes_whatsapp_api::wire::WhatsAppRetryCommandV1 {
            operation_id: operation_id.clone(),
        }
        .encode_to_vec(),
        WhatsAppClientRequest::DeadLetterCommand { operation_id, reason } => hermes_whatsapp_api::wire::WhatsAppDeadLetterCommandV1 {
            operation_id: operation_id.clone(),
            reason: reason.clone(),
        }
        .encode_to_vec(),
        WhatsAppClientRequest::Command(command) => encode_command(command),
        WhatsAppClientRequest::Query(query) => encode_query(query),
    };
    Ok(ModuleClientRequestV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        module_id: WHATSAPP_MODULE_ID.to_owned(),
        owner_id: WHATSAPP_OWNER_ID.to_owned(),
        contract: Some(contract()),
        request_id,
        request_payload: payload,
    }
    .encode_to_vec())
}

pub fn decode_module_request(
    bytes: &[u8],
) -> Result<(u64, WhatsAppClientRequest), WhatsAppClientPortError> {
    let envelope = ModuleClientRequestV1::decode(bytes)
        .map_err(|error| WhatsAppClientPortError::Codec(error.to_string()))?;
    if envelope.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR
        || envelope.module_id != WHATSAPP_MODULE_ID
        || envelope.owner_id != WHATSAPP_OWNER_ID
        || envelope.request_id == 0
    {
        return Err(WhatsAppClientPortError::Protocol(
            "WhatsApp client routing metadata is not admitted".to_owned(),
        ));
    }
    let actual = envelope.contract.as_ref().ok_or_else(|| {
        WhatsAppClientPortError::Protocol("WhatsApp client contract is missing".to_owned())
    })?;
    if actual != &contract() {
        return Err(WhatsAppClientPortError::Protocol(
            "WhatsApp client contract reference is not admitted".to_owned(),
        ));
    }
    if envelope.request_payload.is_empty() || envelope.request_payload.len() > MAX_TEXT_BYTES {
        return Err(WhatsAppClientPortError::Protocol(
            "WhatsApp client payload size is invalid".to_owned(),
        ));
    }
    let request = if let Ok(lifecycle) = decode_lifecycle_request(&envelope.request_payload) {
        WhatsAppClientRequest::Lifecycle(lifecycle)
    } else if let Ok(observation) = decode_host_bridge_payload(&envelope.request_payload) {
        WhatsAppClientRequest::HostObservation(observation)
    } else if let Ok(failure) = hermes_whatsapp_api::wire::WhatsAppHostCommandFailureV1::decode(
        envelope.request_payload.as_slice(),
    ) {
        if failure.operation_id.is_empty() || failure.host_claim_id.is_empty() || failure.reason.is_empty() {
            return Err(WhatsAppClientPortError::Codec("WhatsApp host command failure payload is invalid".to_owned()));
        }
        WhatsAppClientRequest::HostCommandFailed {
            operation_id: failure.operation_id,
            host_claim_id: failure.host_claim_id,
            reason: failure.reason,
        }
    } else if let Ok(retry) = decode_retry_request(&envelope.request_payload) {
        WhatsAppClientRequest::RetryCommand { operation_id: retry }
    } else if let Ok((operation_id, reason)) = decode_dead_letter_request(&envelope.request_payload) {
        WhatsAppClientRequest::DeadLetterCommand { operation_id, reason }
    } else if let Ok(command) = decode_command(&envelope.request_payload) {
        WhatsAppClientRequest::Command(command)
    } else if let Ok(query) = decode_query(&envelope.request_payload) {
        WhatsAppClientRequest::Query(query)
    } else {
        return Err(WhatsAppClientPortError::Codec(
            "WhatsApp client payload is not a generated provider request".to_owned(),
        ));
    };
    Ok((envelope.request_id, request))
}

fn decode_retry_request(bytes: &[u8]) -> Result<String, WhatsAppClientPortError> {
    let request = hermes_whatsapp_api::wire::WhatsAppRetryCommandV1::decode(bytes)
        .map_err(|_| WhatsAppClientPortError::Codec("retry payload is invalid".to_owned()))?;
    if request.operation_id.trim().is_empty() {
        return Err(WhatsAppClientPortError::Codec("retry operation id is empty".to_owned()));
    }
    Ok(request.operation_id)
}

fn decode_dead_letter_request(bytes: &[u8]) -> Result<(String, String), WhatsAppClientPortError> {
    let request = hermes_whatsapp_api::wire::WhatsAppDeadLetterCommandV1::decode(bytes)
        .map_err(|_| WhatsAppClientPortError::Codec("dead-letter payload is invalid".to_owned()))?;
    if request.operation_id.trim().is_empty() || request.reason.trim().is_empty() {
        return Err(WhatsAppClientPortError::Codec("dead-letter payload is invalid".to_owned()));
    }
    Ok((request.operation_id, request.reason))
}

pub fn encode_module_response(
    request_id: u64,
    response: &WhatsAppClientResponse,
) -> Result<Vec<u8>, WhatsAppClientPortError> {
    if request_id == 0 {
        return Err(WhatsAppClientPortError::Protocol(
            "WhatsApp response id must be non-zero".to_owned(),
        ));
    }
    use hermes_whatsapp_api::wire::whats_app_client_response_v1::Response;
    let response = match response {
        WhatsAppClientResponse::Account(account) => {
            Response::Account(account_response_to_wire(account))
        }
        WhatsAppClientResponse::ObservationAccepted { provider_event_id } => {
            Response::ObservationAccepted(hermes_whatsapp_api::wire::WhatsAppObservationAcceptedV1 {
                provider_event_id: provider_event_id.clone(),
            })
        }
        WhatsAppClientResponse::Accepted { operation_id } => {
            Response::Accepted(hermes_whatsapp_api::wire::WhatsAppOperationAcceptedV1 {
                operation_id: operation_id.clone(),
            })
        }
        WhatsAppClientResponse::HostCommandFailureRecorded { operation_id } => {
            Response::HostCommandFailureRecorded(hermes_whatsapp_api::wire::WhatsAppHostCommandFailureRecordedV1 {
                operation_id: operation_id.clone(),
            })
        }
        WhatsAppClientResponse::CommandLifecycleUpdated { operation_id, state } => {
            Response::CommandLifecycleUpdated(hermes_whatsapp_api::wire::WhatsAppCommandLifecycleResponseV1 {
                operation_id: operation_id.clone(),
                state: state.clone(),
            })
        }
        WhatsAppClientResponse::Query(query_response) => {
            let encoded = encode_query_response(query_response).ok_or_else(|| {
                WhatsAppClientPortError::Codec(
                    "WhatsApp query response has no generated wire representation".to_owned(),
                )
            })?;
            let query = hermes_whatsapp_api::wire::WhatsAppQueryResponseV1::decode(encoded.as_slice())
                .map_err(|_| WhatsAppClientPortError::Codec("WhatsApp query response is invalid".to_owned()))?;
            Response::Query(query)
        }
    };
    let payload = hermes_whatsapp_api::wire::WhatsAppClientResponseV1 {
        response: Some(response),
    }.encode_to_vec();
    Ok(ModuleClientResponseV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        request_id,
        response_payload: payload,
        error_code: String::new(),
    }
    .encode_to_vec())
}

pub fn decode_module_response(
    bytes: &[u8],
) -> Result<(u64, WhatsAppClientResponse), WhatsAppClientPortError> {
    let envelope = ModuleClientResponseV1::decode(bytes)
        .map_err(|error| WhatsAppClientPortError::Codec(error.to_string()))?;
    if envelope.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR || envelope.request_id == 0 {
        return Err(WhatsAppClientPortError::Protocol(
            "WhatsApp response routing metadata is invalid".to_owned(),
        ));
    }
    if !envelope.error_code.is_empty() || envelope.response_payload.is_empty() {
        return Err(WhatsAppClientPortError::Protocol(
            "WhatsApp response payload is invalid".to_owned(),
        ));
    }
    let payload = hermes_whatsapp_api::wire::WhatsAppClientResponseV1::decode(envelope.response_payload.as_slice())
        .map_err(|_| WhatsAppClientPortError::Codec("WhatsApp response payload is invalid".to_owned()))?;
    let response = match payload.response.ok_or_else(|| WhatsAppClientPortError::Codec("WhatsApp response variant is missing".to_owned()))? {
        hermes_whatsapp_api::wire::whats_app_client_response_v1::Response::Account(value) => {
            let wrapped = hermes_whatsapp_api::wire::WhatsAppQueryResponseV1 {
                response: Some(hermes_whatsapp_api::wire::whats_app_query_response_v1::Response::Account(
                    hermes_whatsapp_api::wire::WhatsAppAccountList { account: vec![value] },
                )),
            };
            match hermes_whatsapp_api::client_wire::decode_query_response(&wrapped.encode_to_vec())
                .map_err(|_| WhatsAppClientPortError::Codec("WhatsApp account response is invalid".to_owned()))?
            {
                hermes_whatsapp_api::WhatsAppProviderQueryResponse::Account(Some(account)) => WhatsAppClientResponse::Account(account),
                _ => return Err(WhatsAppClientPortError::Codec("WhatsApp account response is empty".to_owned())),
            }
        }
        hermes_whatsapp_api::wire::whats_app_client_response_v1::Response::ObservationAccepted(value) => WhatsAppClientResponse::ObservationAccepted { provider_event_id: value.provider_event_id },
        hermes_whatsapp_api::wire::whats_app_client_response_v1::Response::Accepted(value) => WhatsAppClientResponse::Accepted { operation_id: value.operation_id },
        hermes_whatsapp_api::wire::whats_app_client_response_v1::Response::HostCommandFailureRecorded(value) => WhatsAppClientResponse::HostCommandFailureRecorded { operation_id: value.operation_id },
        hermes_whatsapp_api::wire::whats_app_client_response_v1::Response::CommandLifecycleUpdated(value) => WhatsAppClientResponse::CommandLifecycleUpdated { operation_id: value.operation_id, state: value.state },
        hermes_whatsapp_api::wire::whats_app_client_response_v1::Response::Query(value) => WhatsAppClientResponse::Query(
            hermes_whatsapp_api::client_wire::decode_query_response(&value.encode_to_vec())
                .map_err(|_| WhatsAppClientPortError::Codec("WhatsApp query response is invalid".to_owned()))?,
        ),
    };
    Ok((envelope.request_id, response))
}

fn encode_lifecycle_request(
    request: &hermes_whatsapp_api::WhatsAppLifecycleRequest,
) -> Vec<u8> {
    use hermes_whatsapp_api::wire::whats_app_lifecycle_request_v1::Request;
    let request = match request {
        hermes_whatsapp_api::WhatsAppLifecycleRequest::Provision(setup) => {
            Request::Provision(hermes_whatsapp_api::wire::ProvisionAccountRequest {
                account_id: setup.account_id.clone(),
                display_name: setup.display_name.clone(),
                external_account_id: setup.external_account_id.clone(),
                provider_shape: setup.provider_shape.as_str().to_owned(),
                runtime_kind: setup.runtime_kind.as_str().to_owned(),
                credential: setup.credentials.iter().map(|binding| hermes_whatsapp_api::wire::WhatsAppCredentialBindingV1 {
                    purpose: binding.purpose.as_str().to_owned(),
                    secret_ref: binding.secret_ref.clone(),
                    revision: binding.revision,
                }).collect(),
            })
        }
        hermes_whatsapp_api::WhatsAppLifecycleRequest::Start { account_id } => {
            Request::Start(hermes_whatsapp_api::wire::AccountIdRequest {
                account_id: account_id.clone(),
            })
        }
        hermes_whatsapp_api::WhatsAppLifecycleRequest::Stop { account_id } => {
            Request::Stop(hermes_whatsapp_api::wire::AccountIdRequest {
                account_id: account_id.clone(),
            })
        }
        hermes_whatsapp_api::WhatsAppLifecycleRequest::Revoke { account_id } => {
            Request::Revoke(hermes_whatsapp_api::wire::AccountIdRequest {
                account_id: account_id.clone(),
            })
        }
        hermes_whatsapp_api::WhatsAppLifecycleRequest::Relink { account_id } => {
            Request::Relink(hermes_whatsapp_api::wire::AccountIdRequest {
                account_id: account_id.clone(),
            })
        }
        hermes_whatsapp_api::WhatsAppLifecycleRequest::Remove { account_id } => {
            Request::Remove(hermes_whatsapp_api::wire::AccountIdRequest {
                account_id: account_id.clone(),
            })
        }
    };
    hermes_whatsapp_api::wire::WhatsAppLifecycleRequestV1 {
        request: Some(request),
    }
    .encode_to_vec()
}

fn decode_lifecycle_request(
    bytes: &[u8],
) -> Result<hermes_whatsapp_api::WhatsAppLifecycleRequest, WhatsAppClientPortError> {
    use hermes_whatsapp_api::wire::whats_app_lifecycle_request_v1::Request;
    let payload = hermes_whatsapp_api::wire::WhatsAppLifecycleRequestV1::decode(bytes)
        .map_err(|_| WhatsAppClientPortError::Codec("WhatsApp lifecycle payload is invalid".to_owned()))?;
    match payload.request.ok_or_else(|| WhatsAppClientPortError::Codec("WhatsApp lifecycle request is missing".to_owned()))? {
        Request::Provision(value) => {
            if value.provider_shape != hermes_whatsapp_api::WhatsAppProviderShape::WebCompanion.as_str()
                || value.runtime_kind != hermes_whatsapp_api::WhatsAppRuntimeKind::HiddenWebView.as_str()
            {
                return Err(WhatsAppClientPortError::Protocol(
                    "WhatsApp lifecycle provider shape is not admitted".to_owned(),
                ));
            }
            Ok(hermes_whatsapp_api::WhatsAppLifecycleRequest::Provision(
                hermes_whatsapp_api::WhatsAppAccountSetup {
                    account_id: value.account_id,
                    display_name: value.display_name,
                    external_account_id: value.external_account_id,
                    provider_shape: hermes_whatsapp_api::WhatsAppProviderShape::WebCompanion,
                    runtime_kind: hermes_whatsapp_api::WhatsAppRuntimeKind::HiddenWebView,
                    credentials: value.credential.into_iter().map(|binding| {
                        if binding.purpose != hermes_whatsapp_api::WhatsAppCredentialPurpose::WebSessionKey.as_str() {
                            return Err(WhatsAppClientPortError::Protocol("WhatsApp credential purpose is not admitted".to_owned()));
                        }
                        Ok(hermes_whatsapp_api::WhatsAppCredentialBinding {
                            purpose: hermes_whatsapp_api::WhatsAppCredentialPurpose::WebSessionKey,
                            secret_ref: binding.secret_ref,
                            revision: binding.revision,
                        })
                    }).collect::<Result<Vec<_>, _>>()?,
                },
            ))
        }
        Request::Start(value) => Ok(hermes_whatsapp_api::WhatsAppLifecycleRequest::Start { account_id: value.account_id }),
        Request::Stop(value) => Ok(hermes_whatsapp_api::WhatsAppLifecycleRequest::Stop { account_id: value.account_id }),
        Request::Revoke(value) => Ok(hermes_whatsapp_api::WhatsAppLifecycleRequest::Revoke { account_id: value.account_id }),
        Request::Relink(value) => Ok(hermes_whatsapp_api::WhatsAppLifecycleRequest::Relink { account_id: value.account_id }),
        Request::Remove(value) => Ok(hermes_whatsapp_api::WhatsAppLifecycleRequest::Remove { account_id: value.account_id }),
    }
}

fn account_response_to_wire(
    account: &hermes_whatsapp_api::WhatsAppAccount,
) -> hermes_whatsapp_api::wire::WhatsAppAccountResponseV1 {
    hermes_whatsapp_api::wire::WhatsAppAccountResponseV1 {
        account_id: account.account_id.clone(),
        display_name: account.display_name.clone(),
        external_account_id: account.external_account_id.clone(),
        provider_shape: account.provider_shape.as_str().to_owned(),
        runtime_kind: account.runtime_kind.as_str().to_owned(),
        account_state: account.account_state.as_str().to_owned(),
        runtime_state: format!("{:?}", account.runtime_state).to_lowercase(),
    }
}

fn contract_error(error: WhatsAppContractError) -> WhatsAppClientPortError {
    WhatsAppClientPortError::Protocol(format!("WhatsApp provider query rejected: {error:?}"))
}
