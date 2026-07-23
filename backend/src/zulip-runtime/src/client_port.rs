//! Typed local client port for Zulip operational commands and operation status.

use hermes_runtime_protocol::v1::{ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1};
use hermes_zulip_api::{ZulipClientRequestV1, ZulipClientResponseV1, client_wire};
use prost::Message;

use crate::managed::ZulipAdmittedRuntimeV1;

const MODULE_CLIENT_PROTOCOL_MAJOR: u32 = 1;
const MODULE_ID: &str = "hermes-zulip-runtime";
const OWNER_ID: &str = "zulip";
const CONTRACT_NAME: &str = "zulip.client";
const CONTRACT_MAJOR: u32 = 1;
const CONTRACT_REVISION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZulipClientPortErrorV1 {
    Protocol,
    Runtime,
}

pub fn encode_module_request(
    request_id: u64,
    request: &ZulipClientRequestV1,
) -> Result<Vec<u8>, ZulipClientPortErrorV1> {
    if request_id == 0 {
        return Err(ZulipClientPortErrorV1::Protocol);
    }
    Ok(ModuleClientRequestV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        module_id: MODULE_ID.to_owned(),
        owner_id: OWNER_ID.to_owned(),
        contract: Some(client_contract()),
        request_id,
        request_payload: client_wire::encode_request(request),
    }
    .encode_to_vec())
}

pub fn decode_module_request(
    bytes: &[u8],
) -> Result<(u64, ZulipClientRequestV1), ZulipClientPortErrorV1> {
    let envelope = ModuleClientRequestV1::decode(bytes).map_err(|_| ZulipClientPortErrorV1::Protocol)?;
    if envelope.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR
        || envelope.module_id != MODULE_ID
        || envelope.owner_id != OWNER_ID
        || envelope.request_id == 0
        || envelope.contract.as_ref() != Some(&client_contract())
        || envelope.request_payload.is_empty()
    {
        return Err(ZulipClientPortErrorV1::Protocol);
    }
    let request = client_wire::decode_request(&envelope.request_payload)
        .map_err(|_| ZulipClientPortErrorV1::Protocol)?;
    Ok((envelope.request_id, request))
}

pub async fn handle_client_request(
    runtime: &ZulipAdmittedRuntimeV1,
    bytes: &[u8],
    requested_at_unix_seconds: i64,
) -> Result<Vec<u8>, ZulipClientPortErrorV1> {
    if requested_at_unix_seconds <= 0 {
        return Err(ZulipClientPortErrorV1::Protocol);
    }
    let (request_id, request) = decode_module_request(bytes)?;
    let response = match request {
        ZulipClientRequestV1::Command(command) => runtime
            .submit_command(&command, requested_at_unix_seconds)
            .await
            .map(ZulipClientResponseV1::CommandReceipt)
            .map_err(|_| ZulipClientPortErrorV1::Runtime)?,
        ZulipClientRequestV1::OperationStatus { operation_id } => runtime
            .command_operation_status(&operation_id)
            .await
            .map(ZulipClientResponseV1::OperationStatus)
            .map_err(|_| ZulipClientPortErrorV1::Runtime)?,
    };
    Ok(ModuleClientResponseV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        request_id,
        response_payload: client_wire::encode_response(&response),
        error_code: String::new(),
    }
    .encode_to_vec())
}

pub fn decode_module_response(
    bytes: &[u8],
) -> Result<(u64, ZulipClientResponseV1), ZulipClientPortErrorV1> {
    let envelope = ModuleClientResponseV1::decode(bytes).map_err(|_| ZulipClientPortErrorV1::Protocol)?;
    if envelope.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR
        || envelope.request_id == 0
        || !envelope.error_code.is_empty()
        || envelope.response_payload.is_empty()
    {
        return Err(ZulipClientPortErrorV1::Protocol);
    }
    let response = client_wire::decode_response(&envelope.response_payload)
        .map_err(|_| ZulipClientPortErrorV1::Protocol)?;
    Ok((envelope.request_id, response))
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
    use hermes_zulip_api::{ZulipClientRequestV1, ZulipCommandV1};

    use super::{decode_module_request, encode_module_request};

    #[test]
    fn accepts_only_the_exact_zulip_contract() {
        let request = ZulipClientRequestV1::Command(ZulipCommandV1::SendStream {
            operation_id: "operation".into(), account_id: "account".into(), stream: "stream".into(),
            topic: "topic".into(), content: "content".into(),
        });
        let bytes = encode_module_request(1, &request).expect("request");
        assert_eq!(decode_module_request(&bytes), Ok((1, request)));
    }
}
