use hermes_mail_api::client_contract::{
    MAIL_CLIENT_CONTRACT_MAJOR, MAIL_CLIENT_CONTRACT_REVISION, MAIL_CLIENT_DESCRIPTOR_SET_V1,
    MAIL_MODULE_ID, MAIL_OWNER_ID, MailClientContractV1,
};
use hermes_mail_api::{MailClientRequestV1, MailClientResponseV1, client_wire};
use hermes_runtime_protocol::v1::{
    ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::managed::MailAdmittedRuntime;

const MODULE_CLIENT_PROTOCOL_MAJOR: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailClientPortErrorV1 {
    Protocol,
    Runtime,
}

fn mail_client_contract(contract: MailClientContractV1) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: MAIL_OWNER_ID.to_owned(),
        name: contract.contract_name().to_owned(),
        major: MAIL_CLIENT_CONTRACT_MAJOR,
        revision: MAIL_CLIENT_CONTRACT_REVISION,
        schema_sha256: Sha256::digest(MAIL_CLIENT_DESCRIPTOR_SET_V1).to_vec(),
    }
}

fn validate_contract(
    reference: &ContractReferenceV1,
) -> Result<MailClientContractV1, MailClientPortErrorV1> {
    let contract = MailClientContractV1::from_contract_name(&reference.name)
        .ok_or(MailClientPortErrorV1::Protocol)?;
    if reference != &mail_client_contract(contract) {
        return Err(MailClientPortErrorV1::Protocol);
    }
    Ok(contract)
}

fn request_contract(request: &MailClientRequestV1) -> MailClientContractV1 {
    match request {
        MailClientRequestV1::SyncInbox(_) => MailClientContractV1::Sync,
        MailClientRequestV1::SendMail(_) => MailClientContractV1::Delivery,
    }
}

fn encode_request_payload(request: &MailClientRequestV1) -> Vec<u8> {
    match request {
        MailClientRequestV1::SyncInbox(value) => client_wire::encode_sync_request(value),
        MailClientRequestV1::SendMail(value) => client_wire::encode_delivery_request(value),
    }
}

fn decode_request_payload(
    contract: MailClientContractV1,
    bytes: &[u8],
) -> Result<MailClientRequestV1, MailClientPortErrorV1> {
    match contract {
        MailClientContractV1::Sync => client_wire::decode_sync_request(bytes)
            .map(MailClientRequestV1::SyncInbox)
            .map_err(|_| MailClientPortErrorV1::Protocol),
        MailClientContractV1::Delivery => client_wire::decode_delivery_request(bytes)
            .map(MailClientRequestV1::SendMail)
            .map_err(|_| MailClientPortErrorV1::Protocol),
    }
}

pub fn encode_module_request(
    request_id: u64,
    request: &MailClientRequestV1,
) -> Result<Vec<u8>, MailClientPortErrorV1> {
    if request_id == 0 {
        return Err(MailClientPortErrorV1::Protocol);
    }
    let contract = request_contract(request);
    Ok(ModuleClientRequestV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        module_id: MAIL_MODULE_ID.to_owned(),
        owner_id: MAIL_OWNER_ID.to_owned(),
        contract: Some(mail_client_contract(contract)),
        request_id,
        request_payload: encode_request_payload(request),
    }
    .encode_to_vec())
}

pub fn decode_module_request(
    bytes: &[u8],
) -> Result<(u64, MailClientContractV1, MailClientRequestV1), MailClientPortErrorV1> {
    let envelope =
        ModuleClientRequestV1::decode(bytes).map_err(|_| MailClientPortErrorV1::Protocol)?;
    if envelope.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR
        || envelope.module_id != MAIL_MODULE_ID
        || envelope.owner_id != MAIL_OWNER_ID
        || envelope.request_id == 0
        || envelope.request_payload.is_empty()
    {
        return Err(MailClientPortErrorV1::Protocol);
    }
    let contract = validate_contract(
        envelope
            .contract
            .as_ref()
            .ok_or(MailClientPortErrorV1::Protocol)?,
    )?;
    let request = decode_request_payload(contract, &envelope.request_payload)?;
    Ok((envelope.request_id, contract, request))
}

pub async fn handle_client_request(
    runtime: &mut MailAdmittedRuntime,
    bytes: &[u8],
) -> Result<Vec<u8>, MailClientPortErrorV1> {
    let (request_id, contract, request) = decode_module_request(bytes)?;
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
    encode_module_response(request_id, contract, &response)
}

fn encode_module_response(
    request_id: u64,
    contract: MailClientContractV1,
    response: &MailClientResponseV1,
) -> Result<Vec<u8>, MailClientPortErrorV1> {
    if request_id == 0 {
        return Err(MailClientPortErrorV1::Protocol);
    }
    let response_payload = match (contract, response) {
        (
            MailClientContractV1::Sync,
            MailClientResponseV1::SyncInboxCompleted {
                operation_id,
                observed_messages,
            },
        ) => client_wire::encode_sync_response(operation_id, *observed_messages),
        (
            MailClientContractV1::Delivery,
            MailClientResponseV1::MailAccepted {
                operation_id,
                response_code,
            },
        ) => client_wire::encode_delivery_response(operation_id, *response_code),
        _ => return Err(MailClientPortErrorV1::Protocol),
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
    contract: MailClientContractV1,
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
    let response = match contract {
        MailClientContractV1::Sync => client_wire::decode_sync_response(&envelope.response_payload),
        MailClientContractV1::Delivery => {
            client_wire::decode_delivery_response(&envelope.response_payload)
        }
    }
    .map_err(|_| MailClientPortErrorV1::Protocol)?;
    Ok((envelope.request_id, response))
}

#[cfg(test)]
mod tests {
    use hermes_mail_api::{MailSendMailRequestV1, MailSyncInboxRequestV1};

    use super::*;

    fn sync_request() -> MailClientRequestV1 {
        MailClientRequestV1::SyncInbox(MailSyncInboxRequestV1 {
            operation_id: "sync-operation".to_owned(),
        })
    }

    fn delivery_request() -> MailClientRequestV1 {
        MailClientRequestV1::SendMail(MailSendMailRequestV1 {
            operation_id: "delivery-operation".to_owned(),
            provider_conversation_id: "conversation".to_owned(),
            recipients: vec!["recipient@example.com".to_owned()],
            subject: "subject".to_owned(),
            text_body: "body".to_owned(),
        })
    }

    #[test]
    fn sync_request_uses_only_the_exact_sync_contract() {
        let encoded = encode_module_request(1, &sync_request()).expect("sync request");
        let (request_id, contract, decoded) =
            decode_module_request(&encoded).expect("decode sync request");

        assert_eq!(request_id, 1);
        assert_eq!(contract, MailClientContractV1::Sync);
        assert_eq!(decoded, sync_request());
    }

    #[test]
    fn delivery_payload_is_rejected_under_sync_contract() {
        let encoded = encode_module_request(1, &delivery_request()).expect("delivery request");
        let mut envelope = ModuleClientRequestV1::decode(encoded.as_slice()).expect("envelope");
        envelope.contract = Some(mail_client_contract(MailClientContractV1::Sync));

        assert_eq!(
            decode_module_request(&envelope.encode_to_vec()),
            Err(MailClientPortErrorV1::Protocol)
        );
    }

    #[test]
    fn umbrella_client_contract_is_not_admitted() {
        let encoded = encode_module_request(1, &sync_request()).expect("sync request");
        let mut envelope = ModuleClientRequestV1::decode(encoded.as_slice()).expect("envelope");
        envelope.contract.as_mut().expect("contract").name = "mail.client".to_owned();

        assert_eq!(
            decode_module_request(&envelope.encode_to_vec()),
            Err(MailClientPortErrorV1::Protocol)
        );
    }
}
