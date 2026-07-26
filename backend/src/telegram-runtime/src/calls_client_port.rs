use hermes_runtime_protocol::v1::{ContractReferenceV1, ModuleClientRequestV1};
use hermes_telegram_calls_api::{
    TELEGRAM_CALLS_CONTRACT_MAJOR, TELEGRAM_CALLS_CONTRACT_REVISION,
    TELEGRAM_CALLS_DESCRIPTOR_SET_V1, TELEGRAM_CALLS_MODULE_ID, TELEGRAM_CALLS_OWNER_ID,
    TelegramCallsContractV1,
    wire::{
        CallDirectionV1, CallDiscardReasonV1, CallFailureCategoryV1, CallFrameV1, CallListV1,
        CallStateV1, CallsFailureV1, CallsQueryRequestV1, CallsQueryResponseV1,
        CallsReplayRequestV1, CallsReplayResponseV1, EmptyV1, TelegramCallV1,
        calls_failure_v1::Code as CallsFailureCodeV1, calls_query_request_v1,
        calls_query_response_v1,
    },
};
use hermes_telegram_calls_core::{
    TelegramCallDirection, TelegramCallDiscardReason, TelegramCallFailureCategory,
    TelegramCallSession, TelegramProviderCallState,
};
use hermes_telegram_calls_persistence::{
    TelegramCallFrame, TelegramCallsPersistence, TelegramCallsPersistenceError,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::client_port::{
    MODULE_CLIENT_PROTOCOL_MAJOR, TelegramClientPortError, encode_module_response_payload,
};

const MAX_LIST_LIMIT: u32 = 200;
const MAX_ID_BYTES: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TelegramCallsRoute {
    Query,
    Realtime,
}

pub fn calls_route(bytes: &[u8]) -> Result<Option<TelegramCallsRoute>, TelegramClientPortError> {
    let envelope = ModuleClientRequestV1::decode(bytes)
        .map_err(|error| TelegramClientPortError::Codec(error.to_string()))?;
    let Some(contract) = envelope.contract.as_ref() else {
        return Err(TelegramClientPortError::Protocol(
            "Telegram client contract is missing".to_owned(),
        ));
    };
    let Some(contract_kind) = TelegramCallsContractV1::from_contract_name(&contract.name) else {
        return Ok(None);
    };
    let route = match contract_kind {
        TelegramCallsContractV1::Query => TelegramCallsRoute::Query,
        TelegramCallsContractV1::Realtime => TelegramCallsRoute::Realtime,
        TelegramCallsContractV1::Command => {
            return Err(TelegramClientPortError::Protocol(
                "Telegram Calls command route is not admitted".to_owned(),
            ));
        }
    };
    validate_calls_envelope(&envelope, contract, contract_kind)?;
    Ok(Some(route))
}

pub async fn handle_calls_module_request(
    bytes: &[u8],
    persistence: &TelegramCallsPersistence,
) -> Result<Vec<u8>, TelegramClientPortError> {
    let envelope = ModuleClientRequestV1::decode(bytes)
        .map_err(|error| TelegramClientPortError::Codec(error.to_string()))?;
    let contract = envelope.contract.as_ref().ok_or_else(|| {
        TelegramClientPortError::Protocol("Telegram client contract is missing".to_owned())
    })?;
    let contract_kind =
        TelegramCallsContractV1::from_contract_name(&contract.name).ok_or_else(|| {
            TelegramClientPortError::Protocol("Telegram Calls route is not admitted".to_owned())
        })?;
    let response_payload = match contract_kind {
        TelegramCallsContractV1::Query => {
            validate_calls_envelope(&envelope, contract, contract_kind)?;
            handle_query(&envelope.request_payload, persistence).await?
        }
        TelegramCallsContractV1::Realtime => {
            validate_calls_envelope(&envelope, contract, contract_kind)?;
            handle_replay(&envelope.request_payload, persistence).await?
        }
        TelegramCallsContractV1::Command => {
            return Err(TelegramClientPortError::Protocol(
                "Telegram Calls command route is not admitted".to_owned(),
            ));
        }
    };
    encode_module_response_payload(envelope.request_id, response_payload)
}

fn validate_calls_envelope(
    envelope: &ModuleClientRequestV1,
    contract: &ContractReferenceV1,
    route: TelegramCallsContractV1,
) -> Result<(), TelegramClientPortError> {
    if envelope.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR
        || envelope.module_id != TELEGRAM_CALLS_MODULE_ID
        || envelope.owner_id != TELEGRAM_CALLS_OWNER_ID
        || envelope.request_id == 0
        || envelope.request_payload.is_empty()
        || contract.owner != TELEGRAM_CALLS_OWNER_ID
        || contract.name != route.contract_name()
        || contract.major != TELEGRAM_CALLS_CONTRACT_MAJOR
        || contract.revision != TELEGRAM_CALLS_CONTRACT_REVISION
        || contract.schema_sha256 != Sha256::digest(TELEGRAM_CALLS_DESCRIPTOR_SET_V1).as_slice()
    {
        return Err(TelegramClientPortError::Protocol(
            "Telegram Calls client routing metadata is not admitted".to_owned(),
        ));
    }
    Ok(())
}

async fn handle_query(
    payload: &[u8],
    persistence: &TelegramCallsPersistence,
) -> Result<Vec<u8>, TelegramClientPortError> {
    let request = CallsQueryRequestV1::decode(payload)
        .map_err(|error| TelegramClientPortError::Codec(error.to_string()))?;
    let response = match request.request {
        Some(calls_query_request_v1::Request::ListCalls(query)) => {
            if invalid_id(&query.account_id) || invalid_cursor(&query.after_call_session_id) {
                query_failure(CallsFailureCodeV1::InvalidRequest, "account_id")
            } else if let Some(limit) = validated_limit(query.limit) {
                match persistence
                    .list_calls(&query.account_id, &query.after_call_session_id, limit)
                    .await
                {
                    Ok(calls) => {
                        let next_call_session_id = if calls.len() == limit as usize {
                            calls
                                .last()
                                .map(|call| call.call_session_id.clone())
                                .unwrap_or_default()
                        } else {
                            String::new()
                        };
                        query_response(calls_query_response_v1::Response::CallList(CallListV1 {
                            calls: calls.iter().map(call_wire).collect(),
                            next_call_session_id,
                        }))
                    }
                    Err(error) => query_persistence_failure(error),
                }
            } else {
                query_failure(CallsFailureCodeV1::InvalidRequest, "limit")
            }
        }
        Some(calls_query_request_v1::Request::GetCall(query)) => {
            if invalid_id(&query.account_id) || invalid_id(&query.call_session_id) {
                query_failure(CallsFailureCodeV1::InvalidRequest, "call_session_id")
            } else {
                match persistence
                    .call(&query.account_id, &query.call_session_id)
                    .await
                {
                    Ok(Some(call)) => {
                        query_response(calls_query_response_v1::Response::Call(call_wire(&call)))
                    }
                    Ok(None) => query_failure(CallsFailureCodeV1::NotFound, "call_session_id"),
                    Err(error) => query_persistence_failure(error),
                }
            }
        }
        Some(calls_query_request_v1::Request::GetActiveCall(query)) => {
            if invalid_id(&query.account_id) {
                query_failure(CallsFailureCodeV1::InvalidRequest, "account_id")
            } else {
                match persistence.active_call(&query.account_id).await {
                    Ok(Some(call)) => {
                        query_response(calls_query_response_v1::Response::Call(call_wire(&call)))
                    }
                    Ok(None) => {
                        query_response(calls_query_response_v1::Response::NoActiveCall(EmptyV1 {}))
                    }
                    Err(error) => query_persistence_failure(error),
                }
            }
        }
        Some(calls_query_request_v1::Request::ListCallOperations(_))
        | Some(calls_query_request_v1::Request::GetCallOperation(_)) => {
            query_failure(CallsFailureCodeV1::Unavailable, "call_operations")
        }
        None => query_failure(CallsFailureCodeV1::InvalidRequest, "request"),
    };
    Ok(response.encode_to_vec())
}

async fn handle_replay(
    payload: &[u8],
    persistence: &TelegramCallsPersistence,
) -> Result<Vec<u8>, TelegramClientPortError> {
    let request = CallsReplayRequestV1::decode(payload)
        .map_err(|error| TelegramClientPortError::Codec(error.to_string()))?;
    let response = if invalid_id(&request.account_id) {
        replay_failure(CallsFailureCodeV1::InvalidRequest, "account_id")
    } else if let Some(limit) = validated_limit(request.limit) {
        match persistence
            .realtime_after(&request.account_id, request.after_sequence, limit)
            .await
        {
            Ok(frames) => {
                let next_sequence = frames
                    .last()
                    .map(|frame| frame.sequence)
                    .unwrap_or(request.after_sequence);
                CallsReplayResponseV1 {
                    earliest_available_sequence: frames.first().map(|frame| frame.sequence),
                    latest_available_sequence: frames.last().map(|frame| frame.sequence),
                    frames: frames.iter().map(frame_wire).collect(),
                    next_sequence,
                    reset_required: false,
                    failure: None,
                }
            }
            Err(error) => replay_persistence_failure(error),
        }
    } else {
        replay_failure(CallsFailureCodeV1::InvalidRequest, "limit")
    };
    Ok(response.encode_to_vec())
}

fn call_wire(call: &TelegramCallSession) -> TelegramCallV1 {
    TelegramCallV1 {
        call_session_id: call.call_session_id.clone(),
        account_id: call.account_id.clone(),
        provider_call_unique_id: call.provider_call_unique_id,
        provider_user_id: call.provider_user_id.clone(),
        direction: match call.direction {
            TelegramCallDirection::Incoming => CallDirectionV1::Incoming as i32,
            TelegramCallDirection::Outgoing => CallDirectionV1::Outgoing as i32,
        },
        state: match call.state {
            TelegramProviderCallState::Pending => CallStateV1::Pending as i32,
            TelegramProviderCallState::ExchangingKeys => CallStateV1::ExchangingKeys as i32,
            TelegramProviderCallState::MediaReady => CallStateV1::MediaReady as i32,
            TelegramProviderCallState::HangingUp => CallStateV1::HangingUp as i32,
            TelegramProviderCallState::Discarded => CallStateV1::Ended as i32,
            TelegramProviderCallState::Error => CallStateV1::Failed as i32,
        },
        pending_created: call.pending_created,
        pending_received: call.pending_received,
        discard_reason: call.discard_reason.map(|reason| match reason {
            TelegramCallDiscardReason::Empty => CallDiscardReasonV1::Empty as i32,
            TelegramCallDiscardReason::Missed => CallDiscardReasonV1::Missed as i32,
            TelegramCallDiscardReason::Declined => CallDiscardReasonV1::Declined as i32,
            TelegramCallDiscardReason::Disconnected => CallDiscardReasonV1::Disconnected as i32,
            TelegramCallDiscardReason::HungUp => CallDiscardReasonV1::HungUp as i32,
        }),
        failure_category: call.failure_category.map(|category| match category {
            TelegramCallFailureCategory::Network => CallFailureCategoryV1::Network as i32,
            TelegramCallFailureCategory::NotAvailable => CallFailureCategoryV1::NotAvailable as i32,
            TelegramCallFailureCategory::Permission => CallFailureCategoryV1::Permission as i32,
            TelegramCallFailureCategory::Protocol => CallFailureCategoryV1::Protocol as i32,
            TelegramCallFailureCategory::Unknown => CallFailureCategoryV1::Unknown as i32,
        }),
        revision: call.revision,
        created_at_unix_seconds: call.created_at_unix_seconds,
        updated_at_unix_seconds: call.updated_at_unix_seconds,
        ended_at_unix_seconds: call.ended_at_unix_seconds,
    }
}

fn frame_wire(frame: &TelegramCallFrame) -> CallFrameV1 {
    CallFrameV1 {
        sequence: frame.sequence,
        call: Some(call_wire(&frame.session)),
    }
}

fn query_response(response: calls_query_response_v1::Response) -> CallsQueryResponseV1 {
    CallsQueryResponseV1 {
        response: Some(response),
    }
}

fn query_failure(code: CallsFailureCodeV1, field: &str) -> CallsQueryResponseV1 {
    query_response(calls_query_response_v1::Response::Failure(failure(
        code, field,
    )))
}

fn query_persistence_failure(error: TelegramCallsPersistenceError) -> CallsQueryResponseV1 {
    let (code, field) = persistence_failure(error);
    query_failure(code, field)
}

fn replay_failure(code: CallsFailureCodeV1, field: &str) -> CallsReplayResponseV1 {
    CallsReplayResponseV1 {
        frames: Vec::new(),
        next_sequence: 0,
        reset_required: false,
        earliest_available_sequence: None,
        latest_available_sequence: None,
        failure: Some(failure(code, field)),
    }
}

fn replay_persistence_failure(error: TelegramCallsPersistenceError) -> CallsReplayResponseV1 {
    let (code, field) = persistence_failure(error);
    replay_failure(code, field)
}

fn persistence_failure(error: TelegramCallsPersistenceError) -> (CallsFailureCodeV1, &'static str) {
    match error {
        TelegramCallsPersistenceError::InvalidRequest(field) => {
            (CallsFailureCodeV1::InvalidRequest, field)
        }
        TelegramCallsPersistenceError::IdentityConflict
        | TelegramCallsPersistenceError::StateRegression
        | TelegramCallsPersistenceError::TerminalConflict => {
            (CallsFailureCodeV1::Conflict, "call_state")
        }
        TelegramCallsPersistenceError::Database | TelegramCallsPersistenceError::InvalidRow => {
            (CallsFailureCodeV1::Unavailable, "persistence")
        }
    }
}

fn failure(code: CallsFailureCodeV1, field: &str) -> CallsFailureV1 {
    CallsFailureV1 {
        code: code as i32,
        field: field.to_owned(),
    }
}

fn validated_limit(limit: u32) -> Option<u32> {
    (1..=MAX_LIST_LIMIT).contains(&limit).then_some(limit)
}

fn invalid_id(value: &str) -> bool {
    value.trim().is_empty() || value.len() > MAX_ID_BYTES
}

fn invalid_cursor(value: &str) -> bool {
    value.len() > MAX_ID_BYTES
}

#[cfg(test)]
mod tests {
    use hermes_runtime_protocol::v1::{ContractReferenceV1, ModuleClientRequestV1};

    use super::*;

    fn request_envelope(contract: TelegramCallsContractV1, payload: Vec<u8>) -> Vec<u8> {
        ModuleClientRequestV1 {
            protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
            module_id: TELEGRAM_CALLS_MODULE_ID.to_owned(),
            owner_id: TELEGRAM_CALLS_OWNER_ID.to_owned(),
            contract: Some(ContractReferenceV1 {
                owner: TELEGRAM_CALLS_OWNER_ID.to_owned(),
                name: contract.contract_name().to_owned(),
                major: TELEGRAM_CALLS_CONTRACT_MAJOR,
                revision: TELEGRAM_CALLS_CONTRACT_REVISION,
                schema_sha256: Sha256::digest(TELEGRAM_CALLS_DESCRIPTOR_SET_V1).to_vec(),
            }),
            request_id: 1,
            request_payload: payload,
        }
        .encode_to_vec()
    }

    #[test]
    fn history_routes_are_exact_and_command_remains_closed() {
        let query = request_envelope(
            TelegramCallsContractV1::Query,
            CallsQueryRequestV1 {
                request: Some(calls_query_request_v1::Request::ListCalls(
                    hermes_telegram_calls_api::wire::ListCallsRequestV1 {
                        account_id: "account-1".to_owned(),
                        after_call_session_id: String::new(),
                        limit: 10,
                    },
                )),
            }
            .encode_to_vec(),
        );
        let realtime = request_envelope(
            TelegramCallsContractV1::Realtime,
            CallsReplayRequestV1 {
                account_id: "account-1".to_owned(),
                after_sequence: 0,
                limit: 10,
            }
            .encode_to_vec(),
        );
        let command = request_envelope(TelegramCallsContractV1::Command, vec![1]);

        assert!(matches!(
            calls_route(&query),
            Ok(Some(TelegramCallsRoute::Query))
        ));
        assert!(matches!(
            calls_route(&realtime),
            Ok(Some(TelegramCallsRoute::Realtime))
        ));
        assert!(calls_route(&command).is_err());
    }

    #[test]
    fn call_wire_excludes_runtime_scoped_tdlib_identity() {
        let call = TelegramCallSession {
            call_session_id: "call-1".to_owned(),
            account_id: "account-1".to_owned(),
            runtime_generation: 9,
            tdlib_call_id: 77,
            provider_call_unique_id: Some(101),
            provider_user_id: "user-2".to_owned(),
            direction: TelegramCallDirection::Incoming,
            state: TelegramProviderCallState::Pending,
            pending_created: true,
            pending_received: false,
            discard_reason: None,
            failure_category: None,
            revision: 1,
            created_at_unix_seconds: 10,
            updated_at_unix_seconds: 10,
            ended_at_unix_seconds: None,
        };

        let bytes = call_wire(&call).encode_to_vec();
        assert!(!bytes.is_empty());
        assert!(!String::from_utf8_lossy(&bytes).contains("runtime_generation"));
        assert!(!String::from_utf8_lossy(&bytes).contains("tdlib_call_id"));
    }
}
