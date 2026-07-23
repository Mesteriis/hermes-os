//! Typed client port for Telegram operational commands, queries and replay.

use hermes_runtime_protocol::v1::{
    ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1,
};
use hermes_telegram_api::{
    MAX_PAGE_SIZE, TelegramClientRequest, TelegramClientResponse,
};
use hermes_telegram_core::project_message;
use hermes_telegram_persistence::{TelegramDurablePersistence, TelegramDurablePersistenceError};
use hermes_telegram_tdlib::{TdlibError, TdlibTransport};
use prost::Message;

use crate::TelegramRuntime;

#[derive(Debug)]
pub enum TelegramClientPortError {
    Provider(TdlibError),
    Persistence(TelegramDurablePersistenceError),
    Protocol(String),
    Codec(String),
}

const MODULE_CLIENT_PROTOCOL_MAJOR: u32 = 1;
const TELEGRAM_MODULE_ID: &str = "hermes-telegram-runtime";
const TELEGRAM_OWNER_ID: &str = "telegram";
const TELEGRAM_CLIENT_CONTRACT_NAME: &str = "telegram.client";
const TELEGRAM_CLIENT_CONTRACT_MAJOR: u32 = 1;
const TELEGRAM_CLIENT_CONTRACT_REVISION: u32 = 1;

fn telegram_client_contract() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: TELEGRAM_OWNER_ID.to_owned(),
        name: TELEGRAM_CLIENT_CONTRACT_NAME.to_owned(),
        major: TELEGRAM_CLIENT_CONTRACT_MAJOR,
        revision: TELEGRAM_CLIENT_CONTRACT_REVISION,
        schema_sha256: Vec::new(),
    }
}

fn validate_contract(contract: &ContractReferenceV1) -> Result<(), TelegramClientPortError> {
    let expected = telegram_client_contract();
    if contract != &expected {
        return Err(TelegramClientPortError::Protocol(
            "Telegram client contract reference is not admitted".to_owned(),
        ));
    }
    Ok(())
}

fn lifecycle_wire_request(
    request: &TelegramClientRequest,
) -> Option<hermes_telegram_api::client_wire::TelegramLifecycleRequest> {
    use hermes_telegram_api::client_wire::TelegramLifecycleRequest;
    match request {
        TelegramClientRequest::ProvisionAccount { setup } => {
            Some(TelegramLifecycleRequest::Provision(setup.clone()))
        }
        TelegramClientRequest::RetryCommand {
            operation_id,
            now_unix_seconds,
            next_attempt_at_unix_seconds,
        } => Some(TelegramLifecycleRequest::Retry {
            operation_id: operation_id.clone(),
            now_unix_seconds: *now_unix_seconds,
            next_attempt_at_unix_seconds: *next_attempt_at_unix_seconds,
        }),
        TelegramClientRequest::ListAccounts => Some(TelegramLifecycleRequest::ListAccounts),
        TelegramClientRequest::GetAccount { account_id } => {
            Some(TelegramLifecycleRequest::GetAccount {
                account_id: account_id.clone(),
            })
        }
        TelegramClientRequest::RetireAccount { account_id } => {
            Some(TelegramLifecycleRequest::RetireAccount {
                account_id: account_id.clone(),
            })
        }
        TelegramClientRequest::StartAccount {
            account_id,
            topology,
            holder,
            expires_at_unix_seconds,
            now_unix_seconds,
        } => Some(TelegramLifecycleRequest::StartAccount {
            account_id: account_id.clone(),
            topology: topology.clone(),
            holder: holder.clone(),
            expires_at_unix_seconds: *expires_at_unix_seconds,
            now_unix_seconds: *now_unix_seconds,
        }),
        TelegramClientRequest::StopAccount { account_id } => {
            Some(TelegramLifecycleRequest::StopAccount {
                account_id: account_id.clone(),
            })
        }
        TelegramClientRequest::Replay {
            account_id,
            after_sequence,
            limit,
        } => Some(TelegramLifecycleRequest::Replay {
            account_id: account_id.clone(),
            after_sequence: *after_sequence,
            limit: *limit,
        }),
        _ => None,
    }
}

fn client_request_from_lifecycle(
    request: hermes_telegram_api::client_wire::TelegramLifecycleRequest,
) -> TelegramClientRequest {
    use hermes_telegram_api::client_wire::TelegramLifecycleRequest;
    match request {
        TelegramLifecycleRequest::Provision(setup) => {
            TelegramClientRequest::ProvisionAccount { setup }
        }
        TelegramLifecycleRequest::Retry {
            operation_id,
            now_unix_seconds,
            next_attempt_at_unix_seconds,
        } => TelegramClientRequest::RetryCommand {
            operation_id,
            now_unix_seconds,
            next_attempt_at_unix_seconds,
        },
        TelegramLifecycleRequest::ListAccounts => TelegramClientRequest::ListAccounts,
        TelegramLifecycleRequest::GetAccount { account_id } => {
            TelegramClientRequest::GetAccount { account_id }
        }
        TelegramLifecycleRequest::RetireAccount { account_id } => {
            TelegramClientRequest::RetireAccount { account_id }
        }
        TelegramLifecycleRequest::StartAccount {
            account_id,
            topology,
            holder,
            expires_at_unix_seconds,
            now_unix_seconds,
        } => TelegramClientRequest::StartAccount {
            account_id,
            topology,
            holder,
            expires_at_unix_seconds,
            now_unix_seconds,
        },
        TelegramLifecycleRequest::StopAccount { account_id } => {
            TelegramClientRequest::StopAccount { account_id }
        }
        TelegramLifecycleRequest::Replay {
            account_id,
            after_sequence,
            limit,
        } => TelegramClientRequest::Replay {
            account_id,
            after_sequence,
            limit,
        },
    }
}

pub fn encode_module_request(
    request_id: u64,
    request: &TelegramClientRequest,
) -> Result<Vec<u8>, TelegramClientPortError> {
    if request_id == 0 {
        return Err(TelegramClientPortError::Protocol(
            "Telegram client request id must be non-zero".to_owned(),
        ));
    }
    let request_payload = if let Some(lifecycle) = lifecycle_wire_request(request) {
        hermes_telegram_api::client_wire::encode_lifecycle_request(&lifecycle)
    } else if let TelegramClientRequest::Command(command) = request {
        hermes_telegram_api::client_wire::encode_command(command)
    } else if let TelegramClientRequest::Query(query) = request {
        hermes_telegram_api::client_wire::encode_query(query)
    } else if let TelegramClientRequest::AuthorizationStatus = request {
        hermes_telegram_api::client_wire::encode_request(
            &hermes_telegram_api::client_wire::TelegramAuthorizationRequest::Status,
        )
    } else if let TelegramClientRequest::SubmitAuthorizationPassword { password } = request {
        hermes_telegram_api::client_wire::encode_request(
            &hermes_telegram_api::client_wire::TelegramAuthorizationRequest::SubmitPassword(
                password.clone(),
            ),
        )
    } else {
        return Err(TelegramClientPortError::Protocol(
            "Telegram client request variant has no generated wire mapping".to_owned(),
        ));
    };
    let envelope = ModuleClientRequestV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        module_id: TELEGRAM_MODULE_ID.to_owned(),
        owner_id: TELEGRAM_OWNER_ID.to_owned(),
        contract: Some(telegram_client_contract()),
        request_id,
        request_payload,
    };
    Ok(envelope.encode_to_vec())
}

pub fn decode_module_request(
    bytes: &[u8],
) -> Result<(u64, TelegramClientRequest), TelegramClientPortError> {
    let (request_id, request_payload) = decode_module_request_payload(bytes)?;
    let request = if let Ok(lifecycle) =
        hermes_telegram_api::client_wire::decode_lifecycle_request(&request_payload)
    {
        client_request_from_lifecycle(lifecycle)
    } else if let Ok(command) = hermes_telegram_api::client_wire::decode_command(&request_payload) {
        TelegramClientRequest::Command(command)
    } else if let Ok(query) = hermes_telegram_api::client_wire::decode_query(&request_payload) {
        TelegramClientRequest::Query(query)
    } else if let Ok(auth) = hermes_telegram_api::client_wire::decode_request(&request_payload) {
        match auth {
            hermes_telegram_api::client_wire::TelegramAuthorizationRequest::Status => {
                TelegramClientRequest::AuthorizationStatus
            }
            hermes_telegram_api::client_wire::TelegramAuthorizationRequest::SubmitPassword(
                password,
            ) => TelegramClientRequest::SubmitAuthorizationPassword { password },
        }
    } else {
        return Err(TelegramClientPortError::Protocol(
            "Telegram client request payload has no generated wire mapping".to_owned(),
        ));
    };
    Ok((request_id, request))
}

pub fn decode_module_request_payload(
    bytes: &[u8],
) -> Result<(u64, Vec<u8>), TelegramClientPortError> {
    let envelope = ModuleClientRequestV1::decode(bytes)
        .map_err(|error| TelegramClientPortError::Codec(error.to_string()))?;
    if envelope.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR
        || envelope.module_id != TELEGRAM_MODULE_ID
        || envelope.owner_id != TELEGRAM_OWNER_ID
        || envelope.request_id == 0
    {
        return Err(TelegramClientPortError::Protocol(
            "Telegram client request routing metadata is not admitted".to_owned(),
        ));
    }
    validate_contract(envelope.contract.as_ref().ok_or_else(|| {
        TelegramClientPortError::Protocol("Telegram client contract is missing".to_owned())
    })?)?;
    if envelope.request_payload.is_empty() {
        return Err(TelegramClientPortError::Protocol(
            "Telegram client request payload is empty".to_owned(),
        ));
    }
    Ok((envelope.request_id, envelope.request_payload))
}

pub fn encode_module_response_payload(
    request_id: u64,
    response_payload: Vec<u8>,
) -> Result<Vec<u8>, TelegramClientPortError> {
    if request_id == 0 || response_payload.is_empty() {
        return Err(TelegramClientPortError::Protocol(
            "Telegram client response payload is invalid".to_owned(),
        ));
    }
    Ok(ModuleClientResponseV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        request_id,
        response_payload,
        error_code: String::new(),
    }
    .encode_to_vec())
}

pub fn encode_module_response(
    request_id: u64,
    response: &TelegramClientResponse,
) -> Result<Vec<u8>, TelegramClientPortError> {
    if request_id == 0 {
        return Err(TelegramClientPortError::Protocol(
            "Telegram client response id must be non-zero".to_owned(),
        ));
    }
    let response_payload = if let Some(payload) =
        hermes_telegram_api::client_wire::encode_lifecycle_response(response)
    {
        payload
    } else if let TelegramClientResponse::Query(query_response) = response {
        if let Some(payload) =
            hermes_telegram_api::client_wire::encode_query_response(query_response)
        {
            payload
        } else {
            serde_json::to_vec(response)
                .map_err(|error| TelegramClientPortError::Codec(error.to_string()))?
        }
    } else if let TelegramClientResponse::Realtime(frames) = response {
        hermes_telegram_api::client_wire::encode_realtime_response(frames)
    } else if let TelegramClientResponse::AuthorizationStatus(status) = response {
        hermes_telegram_api::client_wire::encode_response(
            &hermes_telegram_api::client_wire::TelegramAuthorizationResponse::Status(
                status.clone(),
            ),
        )
    } else if let TelegramClientResponse::AuthorizationPasswordAccepted = response {
        hermes_telegram_api::client_wire::encode_response(
            &hermes_telegram_api::client_wire::TelegramAuthorizationResponse::PasswordAccepted,
        )
    } else {
        return Err(TelegramClientPortError::Protocol(
            "Telegram client response variant has no generated wire mapping".to_owned(),
        ));
    };
    Ok(ModuleClientResponseV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        request_id,
        response_payload,
        error_code: String::new(),
    }
    .encode_to_vec())
}

pub fn decode_module_response(
    bytes: &[u8],
) -> Result<(u64, TelegramClientResponse), TelegramClientPortError> {
    let envelope = ModuleClientResponseV1::decode(bytes)
        .map_err(|error| TelegramClientPortError::Codec(error.to_string()))?;
    if envelope.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR || envelope.request_id == 0 {
        return Err(TelegramClientPortError::Protocol(
            "Telegram client response routing metadata is invalid".to_owned(),
        ));
    }
    if !envelope.error_code.is_empty() {
        return Err(TelegramClientPortError::Protocol(envelope.error_code));
    }
    if envelope.response_payload.is_empty() {
        return Err(TelegramClientPortError::Protocol(
            "Telegram client response payload is empty".to_owned(),
        ));
    }
    let response = if let Ok(response) =
        hermes_telegram_api::client_wire::decode_lifecycle_response(&envelope.response_payload)
    {
        response
    } else if let Ok(response) =
        hermes_telegram_api::client_wire::decode_query_response(&envelope.response_payload)
    {
        TelegramClientResponse::Query(response)
    } else if let Ok(response) =
        hermes_telegram_api::client_wire::decode_realtime_response(&envelope.response_payload)
    {
        TelegramClientResponse::Realtime(response)
    } else if let Ok(response) =
        hermes_telegram_api::client_wire::decode_response(&envelope.response_payload)
    {
        match response {
            hermes_telegram_api::client_wire::TelegramAuthorizationResponse::Status(status) => {
                TelegramClientResponse::AuthorizationStatus(status)
            }
            hermes_telegram_api::client_wire::TelegramAuthorizationResponse::PasswordAccepted => {
                TelegramClientResponse::AuthorizationPasswordAccepted
            }
        }
    } else {
        return Err(TelegramClientPortError::Protocol(
            "Telegram client response payload has no generated wire mapping".to_owned(),
        ));
    };
    Ok((envelope.request_id, response))
}

pub struct TelegramClientPort<'a, T> {
    runtime: &'a mut TelegramRuntime<T>,
}

impl<'a, T: TdlibTransport> TelegramClientPort<'a, T> {
    pub fn new(runtime: &'a mut TelegramRuntime<T>) -> Self {
        Self { runtime }
    }

    pub async fn handle_module_request_durable(
        &mut self,
        bytes: &[u8],
        durable: &TelegramDurablePersistence,
    ) -> Result<Vec<u8>, TelegramClientPortError> {
        let (request_id, request) = decode_module_request(bytes)?;
        let response = match request {
            TelegramClientRequest::ProvisionAccount { setup } => self
                .runtime
                .provision_account_durable(durable, setup)
                .await
                .map(TelegramClientResponse::Account)
                .map_err(|error| TelegramClientPortError::Protocol(format!("{error:?}")))?,
            TelegramClientRequest::ListAccounts => TelegramClientResponse::Accounts(
                durable
                    .accounts()
                    .await
                    .map_err(TelegramClientPortError::Persistence)?,
            ),
            TelegramClientRequest::GetAccount { account_id } => TelegramClientResponse::Account(
                durable
                    .account(&account_id)
                    .await
                    .map_err(TelegramClientPortError::Persistence)?
                    .map(|(account, _credentials)| account)
                    .ok_or_else(|| {
                        TelegramClientPortError::Protocol("Telegram account is unknown".to_owned())
                    })?,
            ),
            TelegramClientRequest::RetireAccount { account_id } => self
                .runtime
                .retire_account_durable(durable, &account_id)
                .await
                .map(TelegramClientResponse::Account)
                .map_err(|error| TelegramClientPortError::Protocol(format!("{error:?}")))?,
            TelegramClientRequest::Command(command) => self
                .runtime
                .execute_provider_command_durable(durable, command)
                .await
                .map(|operation| TelegramClientResponse::Accepted {
                    operation_id: operation.operation_id,
                })
                .map_err(|error| TelegramClientPortError::Protocol(format!("{error:?}")))?,
            TelegramClientRequest::RetryCommand {
                operation_id,
                now_unix_seconds,
                next_attempt_at_unix_seconds,
            } => self
                .runtime
                .retry_operation_durable(
                    durable,
                    &operation_id,
                    now_unix_seconds,
                    next_attempt_at_unix_seconds,
                )
                .await
                .map(TelegramClientResponse::Operation)
                .map_err(|error| TelegramClientPortError::Protocol(format!("{error:?}")))?,
            TelegramClientRequest::StartAccount {
                account_id,
                topology,
                holder,
                expires_at_unix_seconds,
                now_unix_seconds,
            } => self
                .runtime
                .start_admitted_account_durable(
                    durable,
                    &account_id,
                    &topology,
                    &holder,
                    expires_at_unix_seconds,
                    now_unix_seconds,
                )
                .await
                .map(TelegramClientResponse::Account)
                .map_err(|error| TelegramClientPortError::Protocol(format!("{error:?}")))?,
            TelegramClientRequest::StopAccount { account_id } => self
                .runtime
                .stop_account_durable(durable, &account_id)
                .await
                .map(TelegramClientResponse::Account)
                .map_err(|error| TelegramClientPortError::Protocol(format!("{error:?}")))?,
            TelegramClientRequest::Replay {
                account_id,
                after_sequence,
                limit,
            } => {
                self.replay_durable(durable, &account_id, after_sequence, limit)
                    .await?
            }
            TelegramClientRequest::Query(
                hermes_telegram_api::TelegramProviderQuery::LoadHistory {
                    account_id,
                    provider_chat_id,
                    from_message_id,
                    mode,
                    limit,
                },
            ) => {
                let page = self
                    .runtime
                    .load_history_with_options(
                        &account_id,
                        &provider_chat_id,
                        from_message_id,
                        mode,
                        limit,
                    )
                    .map_err(TelegramClientPortError::Provider)?;
                for observation in &page.items {
                    let projection = project_message(observation).map_err(|_| {
                        TelegramClientPortError::Protocol(
                            "Telegram history projection is invalid".to_owned(),
                        )
                    })?;
                    durable
                        .upsert_message(&projection)
                        .await
                        .map_err(TelegramClientPortError::Persistence)?;
                }
                TelegramClientResponse::Query(
                    hermes_telegram_api::TelegramProviderQueryResponse::HistoryPage(page),
                )
            }
            TelegramClientRequest::Query(
                hermes_telegram_api::TelegramProviderQuery::Operations { account_id, limit },
            ) => TelegramClientResponse::Query(
                hermes_telegram_api::TelegramProviderQueryResponse::Operations(
                    durable
                        .operations_for_account(&account_id, i64::from(limit))
                        .await
                        .map_err(TelegramClientPortError::Persistence)?,
                ),
            ),
            TelegramClientRequest::Query(
                hermes_telegram_api::TelegramProviderQuery::ChatAvatar {
                    account_id,
                    provider_chat_id,
                },
            ) => TelegramClientResponse::Query(
                hermes_telegram_api::TelegramProviderQueryResponse::ChatAvatar(
                    durable
                        .chat_avatar(&account_id, &provider_chat_id)
                        .await
                        .map_err(TelegramClientPortError::Persistence)?,
                ),
            ),
            TelegramClientRequest::Query(
                hermes_telegram_api::TelegramProviderQuery::Attachment { attachment_id, .. },
            ) => TelegramClientResponse::Query(
                hermes_telegram_api::TelegramProviderQueryResponse::Attachment(
                    durable
                        .attachment(&attachment_id)
                        .await
                        .map_err(TelegramClientPortError::Persistence)?,
                ),
            ),
            TelegramClientRequest::Query(
                hermes_telegram_api::TelegramProviderQuery::MessageReferences {
                    message_id, ..
                },
            ) => TelegramClientResponse::Query(
                hermes_telegram_api::TelegramProviderQueryResponse::MessageReferences(
                    durable
                        .message_references(&message_id)
                        .await
                        .map_err(TelegramClientPortError::Persistence)?,
                ),
            ),
            TelegramClientRequest::Query(
                hermes_telegram_api::TelegramProviderQuery::ReplyChain {
                    account_id,
                    provider_chat_id,
                    provider_message_id,
                    limit,
                },
            ) => TelegramClientResponse::Query(
                hermes_telegram_api::TelegramProviderQueryResponse::ReplyChain(
                    durable
                        .reply_chain(
                            &account_id,
                            &provider_chat_id,
                            &provider_message_id,
                            i64::from(limit),
                        )
                        .await
                        .map_err(TelegramClientPortError::Persistence)?,
                ),
            ),
            TelegramClientRequest::Query(
                hermes_telegram_api::TelegramProviderQuery::ForwardChain {
                    account_id,
                    provider_chat_id,
                    provider_message_id,
                    limit,
                },
            ) => TelegramClientResponse::Query(
                hermes_telegram_api::TelegramProviderQueryResponse::ForwardChain(
                    durable
                        .forward_chain(
                            &account_id,
                            &provider_chat_id,
                            &provider_message_id,
                            i64::from(limit),
                        )
                        .await
                        .map_err(TelegramClientPortError::Persistence)?,
                ),
            ),
            TelegramClientRequest::Query(
                hermes_telegram_api::TelegramProviderQuery::AttachmentForMessage {
                    account_id,
                    provider_chat_id,
                    provider_message_id,
                },
            ) => TelegramClientResponse::Query(
                hermes_telegram_api::TelegramProviderQueryResponse::Attachment(
                    durable
                        .attachment_for_message(
                            &account_id,
                            &provider_chat_id,
                            &provider_message_id,
                        )
                        .await
                        .map_err(TelegramClientPortError::Persistence)?,
                ),
            ),
            TelegramClientRequest::Query(hermes_telegram_api::TelegramProviderQuery::File {
                account_id,
                provider_file_id,
            }) => TelegramClientResponse::Query(
                hermes_telegram_api::TelegramProviderQueryResponse::File(
                    durable
                        .file(&account_id, &provider_file_id)
                        .await
                        .map_err(TelegramClientPortError::Persistence)?,
                ),
            ),
            TelegramClientRequest::Query(
                hermes_telegram_api::TelegramProviderQuery::Commands {
                    account_id,
                    provider_chat_id,
                    provider_message_id,
                    command_kinds,
                    limit,
                },
            ) => TelegramClientResponse::Query(
                hermes_telegram_api::TelegramProviderQueryResponse::Commands(
                    durable
                        .command_records_for_account(
                            &account_id,
                            provider_chat_id.as_deref(),
                            provider_message_id.as_deref(),
                            &command_kinds,
                            i64::from(limit),
                        )
                        .await
                        .map_err(TelegramClientPortError::Persistence)?,
                ),
            ),
            TelegramClientRequest::Query(
                hermes_telegram_api::TelegramProviderQuery::ReactionSummary {
                    account_id,
                    provider_chat_id,
                    provider_message_id,
                },
            ) => TelegramClientResponse::Query(
                hermes_telegram_api::TelegramProviderQueryResponse::ReactionSummary(
                    durable
                        .reaction_summary(&format!(
                            "telegram:{account_id}:{provider_chat_id}:{provider_message_id}"
                        ))
                        .await
                        .map_err(TelegramClientPortError::Persistence)?,
                ),
            ),
            TelegramClientRequest::Query(
                hermes_telegram_api::TelegramProviderQuery::TopicMessageIds {
                    account_id,
                    provider_chat_id,
                    provider_topic_id,
                    limit,
                },
            ) => TelegramClientResponse::Query(
                hermes_telegram_api::TelegramProviderQueryResponse::TopicMessageIds(
                    durable
                        .message_ids_for_topic(
                            &account_id,
                            &provider_chat_id,
                            &provider_topic_id,
                            i64::from(limit),
                        )
                        .await
                        .map_err(TelegramClientPortError::Persistence)?,
                ),
            ),
            TelegramClientRequest::Query(query) => self
                .runtime
                .execute_provider_query(query)
                .map(TelegramClientResponse::Query)
                .map_err(TelegramClientPortError::Provider)?,
            TelegramClientRequest::AuthorizationStatus
            | TelegramClientRequest::SubmitAuthorizationPassword { .. } => {
                return Err(TelegramClientPortError::Protocol(
                    "Telegram authorization requests require the authorization port".to_owned(),
                ));
            }
        };
        encode_module_response(request_id, &response)
    }

    pub async fn replay_durable(
        &self,
        durable: &TelegramDurablePersistence,
        account_id: &str,
        after_sequence: u64,
        limit: u32,
    ) -> Result<TelegramClientResponse, TelegramClientPortError> {
        if limit == 0 || limit > MAX_PAGE_SIZE {
            return Err(TelegramClientPortError::Provider(TdlibError::Protocol(
                "Telegram realtime replay limit is invalid".to_owned(),
            )));
        }
        durable
            .replay_provider_events_after(account_id, after_sequence, i64::from(limit))
            .await
            .map(TelegramClientResponse::Realtime)
            .map_err(TelegramClientPortError::Persistence)
    }
}
