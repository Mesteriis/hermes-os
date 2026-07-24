//! Typed local client port for Mail provider operations.

use crate::managed::MailAdmittedRuntime;
use hermes_mail_api::{MailClientRequestV1, MailClientResponseV1, client_wire};
use hermes_runtime_protocol::v1::{
    ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1,
};
use prost::Message;

const MODULE_CLIENT_PROTOCOL_MAJOR: u32 = 1;
const MODULE_ID: &str = "hermes-mail-runtime";
const OWNER_ID: &str = "mail";
const CONTRACT_NAME: &str = "mail.client";
const CONTRACT_MAJOR: u32 = 1;
const CONTRACT_REVISION: u32 = 1;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailClientPortErrorV1 {
    Protocol,
    Runtime,
}

pub fn encode_module_request(
    request_id: u64,
    request: &MailClientRequestV1,
) -> Result<Vec<u8>, MailClientPortErrorV1> {
    if request_id == 0 {
        return Err(MailClientPortErrorV1::Protocol);
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
) -> Result<(u64, MailClientRequestV1), MailClientPortErrorV1> {
    let envelope =
        ModuleClientRequestV1::decode(bytes).map_err(|_| MailClientPortErrorV1::Protocol)?;
    if envelope.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR
        || envelope.module_id != MODULE_ID
        || envelope.owner_id != OWNER_ID
        || envelope.request_id == 0
        || envelope.contract.as_ref() != Some(&client_contract())
        || envelope.request_payload.is_empty()
    {
        return Err(MailClientPortErrorV1::Protocol);
    }
    client_wire::decode_request(&envelope.request_payload)
        .map(|request| (envelope.request_id, request))
        .map_err(|_| MailClientPortErrorV1::Protocol)
}
pub async fn handle_client_request(
    runtime: &mut MailAdmittedRuntime,
    bytes: &[u8],
) -> Result<Vec<u8>, MailClientPortErrorV1> {
    let (request_id, request) = decode_module_request(bytes)?;
    let response = match request {
        MailClientRequestV1::SyncInbox(value) => {
            let observed_messages = runtime
                .sync_configured_inbox(&value.operation_id)
                .await
                .map_err(|_| MailClientPortErrorV1::Runtime)?;
            MailClientResponseV1::SyncInboxCompleted {
                operation_id: value.operation_id,
                observed_messages: u32::try_from(observed_messages)
                    .map_err(|_| MailClientPortErrorV1::Runtime)?,
            }
        }
        MailClientRequestV1::SendMail(value) => {
            let operation_id = value.operation_id.clone();
            let response_code = runtime
                .send_configured_mail(&value)
                .await
                .map_err(|_| MailClientPortErrorV1::Runtime)?;
            MailClientResponseV1::MailAccepted {
                operation_id,
                response_code,
            }
        }
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
) -> Result<(u64, MailClientResponseV1), MailClientPortErrorV1> {
    let envelope =
        ModuleClientResponseV1::decode(bytes).map_err(|_| MailClientPortErrorV1::Protocol)?;
    if envelope.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR
        || envelope.request_id == 0
        || !envelope.error_code.is_empty()
        || envelope.response_payload.is_empty()
    {
        return Err(MailClientPortErrorV1::Protocol);
    }
    client_wire::decode_response(&envelope.response_payload)
        .map(|response| (envelope.request_id, response))
        .map_err(|_| MailClientPortErrorV1::Protocol)
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
    use super::{decode_module_request, encode_module_request};
    use hermes_mail_api::{MailClientRequestV1, MailSyncInboxRequestV1};
    #[test]
    fn accepts_only_the_exact_mail_contract() {
        let request = MailClientRequestV1::SyncInbox(MailSyncInboxRequestV1 {
            operation_id: "operation".into(),
        });
        assert_eq!(
            decode_module_request(&encode_module_request(1, &request).expect("request")),
            Ok((1, request))
        );
    }
}
