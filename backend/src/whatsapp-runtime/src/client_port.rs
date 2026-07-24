//! Typed local client port for host-submitted WhatsApp observations.

use hermes_runtime_protocol::v1::{
    ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1,
};
use hermes_whatsapp_api::{
    WhatsAppProviderQuery, WhatsAppProviderQueryResponse, client_wire,
    host_bridge::{WhatsAppHostBridgeEnvelopeV1, decode_host_bridge_payload},
    validate_provider_query,
    wire::{
        self, WhatsAppClientResponseV1, WhatsAppObservationAcceptedV1,
        whats_app_client_response_v1::Response,
    },
};
use prost::Message;

use crate::managed::WhatsAppAdmittedRuntime;

const MODULE_CLIENT_PROTOCOL_MAJOR: u32 = 1;
const MODULE_ID: &str = "hermes-whatsapp-runtime";
const OWNER_ID: &str = "whatsapp";
const CONTRACT_NAME: &str = "whatsapp.client";
const CONTRACT_MAJOR: u32 = 1;
const CONTRACT_REVISION: u32 = 1;

#[derive(Debug)]
pub enum WhatsAppClientPortError {
    Protocol,
    HostBridge,
    Ingress,
}

enum WhatsAppHostRequest {
    Observation(WhatsAppHostBridgeEnvelopeV1),
    ClaimPendingCommands {
        account_id: String,
        host_claim_id: String,
        lease_seconds: i64,
        limit: i64,
    },
}

fn decode_host_request(
    bytes: &[u8],
) -> Result<(u64, WhatsAppHostRequest), WhatsAppClientPortError> {
    let request =
        ModuleClientRequestV1::decode(bytes).map_err(|_| WhatsAppClientPortError::Protocol)?;
    if request.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR
        || request.module_id != MODULE_ID
        || request.owner_id != OWNER_ID
        || request.request_id == 0
        || request.contract.as_ref() != Some(&client_contract())
        || request.request_payload.is_empty()
    {
        return Err(WhatsAppClientPortError::Protocol);
    }
    if let Ok(query) = client_wire::decode_query(&request.request_payload) {
        validate_provider_query(&query).map_err(|_| WhatsAppClientPortError::HostBridge)?;
        let WhatsAppProviderQuery::ClaimPendingCommands {
            account_id,
            host_claim_id,
            lease_seconds,
            limit,
        } = query
        else {
            return Err(WhatsAppClientPortError::HostBridge);
        };
        return Ok((
            request.request_id,
            WhatsAppHostRequest::ClaimPendingCommands {
                account_id,
                host_claim_id,
                lease_seconds: i64::from(lease_seconds),
                limit: i64::from(limit),
            },
        ));
    }
    let envelope = decode_host_bridge_payload(&request.request_payload)
        .map_err(|_| WhatsAppClientPortError::HostBridge)?;
    Ok((
        request.request_id,
        WhatsAppHostRequest::Observation(envelope),
    ))
}

pub async fn handle_host_request(
    runtime: &WhatsAppAdmittedRuntime,
    bytes: &[u8],
    recorded_at_unix_seconds: i64,
    recorded_at_nanos: i32,
) -> Result<Vec<u8>, WhatsAppClientPortError> {
    let (request_id, request) = decode_host_request(bytes)?;
    let response_payload = match request {
        WhatsAppHostRequest::Observation(envelope) => {
            runtime
                .accept_host_observation(&envelope, recorded_at_unix_seconds, recorded_at_nanos)
                .await
                .map_err(|_| WhatsAppClientPortError::Ingress)?;
            WhatsAppClientResponseV1 {
                response: Some(Response::ObservationAccepted(
                    WhatsAppObservationAcceptedV1 {
                        provider_event_id: envelope.provider_event_id,
                    },
                )),
            }
            .encode_to_vec()
        }
        WhatsAppHostRequest::ClaimPendingCommands {
            account_id,
            host_claim_id,
            lease_seconds,
            limit,
        } => {
            let commands = runtime
                .claim_host_commands(
                    &account_id,
                    &host_claim_id,
                    recorded_at_unix_seconds,
                    lease_seconds,
                    limit,
                )
                .await
                .map_err(|_| WhatsAppClientPortError::Ingress)?;
            let query_bytes = client_wire::encode_query_response(
                &WhatsAppProviderQueryResponse::Commands(commands),
            )
            .ok_or(WhatsAppClientPortError::HostBridge)?;
            let query = wire::WhatsAppQueryResponseV1::decode(query_bytes.as_slice())
                .map_err(|_| WhatsAppClientPortError::HostBridge)?;
            WhatsAppClientResponseV1 {
                response: Some(Response::Query(query)),
            }
            .encode_to_vec()
        }
    };
    Ok(ModuleClientResponseV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        request_id,
        response_payload,
        error_code: String::new(),
    }
    .encode_to_vec())
}

fn client_contract() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: OWNER_ID.to_owned(),
        name: CONTRACT_NAME.to_owned(),
        major: CONTRACT_MAJOR,
        revision: CONTRACT_REVISION,
        schema_sha256: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hermes_whatsapp_api::host_bridge::{
        HOST_BRIDGE_PROTOCOL_MAJOR, HOST_BRIDGE_PROTOCOL_REVISION, WhatsAppHostObservationV1,
        encode_host_bridge_payload,
    };

    #[test]
    fn accepts_only_the_exact_whatsapp_client_contract() {
        let payload = encode_host_bridge_payload(&WhatsAppHostBridgeEnvelopeV1 {
            protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
            protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
            account_id: "wa-1".to_owned(),
            provider_event_id: "event-1".to_owned(),
            observed_at_unix_seconds: 1_782_504_000,
            observation: WhatsAppHostObservationV1::RuntimeState {
                state: "running".to_owned(),
            },
        })
        .expect("payload");
        let request = ModuleClientRequestV1 {
            protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
            module_id: MODULE_ID.to_owned(),
            owner_id: OWNER_ID.to_owned(),
            contract: Some(client_contract()),
            request_id: 7,
            request_payload: payload,
        };

        let (request_id, request) =
            decode_host_request(&request.encode_to_vec()).expect("decoded host observation");

        assert_eq!(request_id, 7);
        assert!(
            matches!(request, WhatsAppHostRequest::Observation(envelope) if envelope.provider_event_id == "event-1")
        );
    }
}
