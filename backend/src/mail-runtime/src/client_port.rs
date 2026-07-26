use hermes_mail_api::client_contract::{
    MAIL_CLIENT_CONTRACT_MAJOR, MAIL_CLIENT_CONTRACT_REVISION, MAIL_CLIENT_DESCRIPTOR_SET_V1,
    MAIL_MODULE_ID, MAIL_OWNER_ID, MailClientContractV1,
};
use hermes_mail_api::{MailClientRequestV1, MailClientResponseV1, client_wire, oauth_wire};
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
        MailClientRequestV1::DeliveryStatus(_) => MailClientContractV1::DeliveryQuery,
        MailClientRequestV1::GmailOAuthStart(_) => MailClientContractV1::GmailOAuthStart,
        MailClientRequestV1::GmailOAuthComplete(_) => MailClientContractV1::GmailOAuthComplete,
        MailClientRequestV1::GmailOAuthRefresh(_) => MailClientContractV1::GmailOAuthRefresh,
        MailClientRequestV1::GmailOAuthStatus(_) => MailClientContractV1::GmailOAuthQuery,
    }
}

fn encode_request_payload(request: &MailClientRequestV1) -> Vec<u8> {
    match request {
        MailClientRequestV1::SyncInbox(value) => client_wire::encode_sync_request(value),
        MailClientRequestV1::SendMail(value) => client_wire::encode_delivery_request(value),
        MailClientRequestV1::DeliveryStatus(value) => {
            client_wire::encode_delivery_status_request(value)
        }
        MailClientRequestV1::GmailOAuthStart(value) => oauth_wire::encode_start_request(value),
        MailClientRequestV1::GmailOAuthComplete(value) => {
            oauth_wire::encode_complete_request(value)
        }
        MailClientRequestV1::GmailOAuthRefresh(value) => oauth_wire::encode_refresh_request(value),
        MailClientRequestV1::GmailOAuthStatus(value) => oauth_wire::encode_status_request(value),
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
        MailClientContractV1::DeliveryQuery => client_wire::decode_delivery_status_request(bytes)
            .map(MailClientRequestV1::DeliveryStatus)
            .map_err(|_| MailClientPortErrorV1::Protocol),
        MailClientContractV1::GmailOAuthStart => oauth_wire::decode_start_request(bytes)
            .map(MailClientRequestV1::GmailOAuthStart)
            .map_err(|_| MailClientPortErrorV1::Protocol),
        MailClientContractV1::GmailOAuthComplete => oauth_wire::decode_complete_request(bytes)
            .map(MailClientRequestV1::GmailOAuthComplete)
            .map_err(|_| MailClientPortErrorV1::Protocol),
        MailClientContractV1::GmailOAuthRefresh => oauth_wire::decode_refresh_request(bytes)
            .map(MailClientRequestV1::GmailOAuthRefresh)
            .map_err(|_| MailClientPortErrorV1::Protocol),
        MailClientContractV1::GmailOAuthQuery => oauth_wire::decode_status_request(bytes)
            .map(MailClientRequestV1::GmailOAuthStatus)
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
    requested_at_unix_seconds: i64,
) -> Result<Vec<u8>, MailClientPortErrorV1> {
    if requested_at_unix_seconds <= 0 {
        return Err(MailClientPortErrorV1::Protocol);
    }
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
        MailClientRequestV1::SendMail(value) => runtime
            .submit_delivery(&value, requested_at_unix_seconds)
            .await
            .map(|operation_id| MailClientResponseV1::MailAccepted { operation_id })
            .map_err(|_| MailClientPortErrorV1::Runtime)?,
        MailClientRequestV1::DeliveryStatus(value) => runtime
            .delivery_operation_status(&value.operation_id)
            .await
            .map(MailClientResponseV1::DeliveryStatus)
            .map_err(|_| MailClientPortErrorV1::Runtime)?,
        MailClientRequestV1::GmailOAuthStart(value) => runtime
            .start_gmail_oauth(&value.operation_id, requested_at_unix_seconds)
            .await
            .map(MailClientResponseV1::GmailOAuthStarted)
            .map_err(|_| MailClientPortErrorV1::Runtime)?,
        MailClientRequestV1::GmailOAuthComplete(value) => runtime
            .submit_gmail_oauth_complete(&value, requested_at_unix_seconds)
            .await
            .map(|operation_id| MailClientResponseV1::GmailOAuthAccepted { operation_id })
            .map_err(|_| MailClientPortErrorV1::Runtime)?,
        MailClientRequestV1::GmailOAuthRefresh(value) => runtime
            .submit_gmail_oauth_refresh(&value.operation_id, requested_at_unix_seconds)
            .await
            .map(|operation_id| MailClientResponseV1::GmailOAuthAccepted { operation_id })
            .map_err(|_| MailClientPortErrorV1::Runtime)?,
        MailClientRequestV1::GmailOAuthStatus(value) => runtime
            .gmail_oauth_operation_status(&value.operation_id)
            .await
            .map(MailClientResponseV1::GmailOAuthStatus)
            .map_err(|_| MailClientPortErrorV1::Runtime)?,
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
        (MailClientContractV1::Delivery, MailClientResponseV1::MailAccepted { operation_id }) => {
            client_wire::encode_delivery_response(operation_id)
        }
        (MailClientContractV1::DeliveryQuery, MailClientResponseV1::DeliveryStatus(status)) => {
            client_wire::encode_delivery_status_response(status.as_ref())
        }
        (
            MailClientContractV1::GmailOAuthStart,
            MailClientResponseV1::GmailOAuthStarted(response),
        ) => oauth_wire::encode_start_response(response),
        (
            MailClientContractV1::GmailOAuthComplete | MailClientContractV1::GmailOAuthRefresh,
            MailClientResponseV1::GmailOAuthAccepted { operation_id },
        ) => oauth_wire::encode_accepted_response(operation_id),
        (MailClientContractV1::GmailOAuthQuery, MailClientResponseV1::GmailOAuthStatus(status)) => {
            oauth_wire::encode_status_response(status.as_ref())
        }
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
    if envelope.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR || envelope.request_id == 0 {
        return Err(MailClientPortErrorV1::Protocol);
    }
    if !envelope.error_code.is_empty() {
        return if envelope.response_payload.is_empty()
            && matches!(
                envelope.error_code.as_str(),
                "INVALID_ARGUMENT" | "REJECTED" | "RUNTIME_UNAVAILABLE"
            ) {
            Err(MailClientPortErrorV1::Runtime)
        } else {
            Err(MailClientPortErrorV1::Protocol)
        };
    }
    if envelope.response_payload.is_empty() {
        return Err(MailClientPortErrorV1::Protocol);
    }
    let response = match contract {
        MailClientContractV1::Sync => client_wire::decode_sync_response(&envelope.response_payload),
        MailClientContractV1::Delivery => {
            client_wire::decode_delivery_response(&envelope.response_payload)
        }
        MailClientContractV1::DeliveryQuery => {
            client_wire::decode_delivery_status_response(&envelope.response_payload)
        }
        MailClientContractV1::GmailOAuthStart => {
            oauth_wire::decode_start_response(&envelope.response_payload)
        }
        MailClientContractV1::GmailOAuthComplete | MailClientContractV1::GmailOAuthRefresh => {
            oauth_wire::decode_accepted_response(&envelope.response_payload)
        }
        MailClientContractV1::GmailOAuthQuery => {
            oauth_wire::decode_status_response(&envelope.response_payload)
        }
    }
    .map_err(|_| MailClientPortErrorV1::Protocol)?;
    Ok((envelope.request_id, response))
}

#[cfg(test)]
mod tests {
    use hermes_mail_api::{
        MailDeliveryStatusRequestV1, MailSendMailRequestV1, MailSyncInboxRequestV1,
    };

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
            attachment_anchor_ids: Vec::new(),
        })
    }

    fn delivery_query() -> MailClientRequestV1 {
        MailClientRequestV1::DeliveryStatus(MailDeliveryStatusRequestV1 {
            operation_id: "delivery-operation".to_owned(),
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
    fn delivery_command_and_query_use_independent_contracts() {
        let command = encode_module_request(2, &delivery_request()).expect("delivery request");
        let (_, command_contract, _) =
            decode_module_request(&command).expect("decode delivery request");
        let query = encode_module_request(3, &delivery_query()).expect("delivery query");
        let (_, query_contract, _) = decode_module_request(&query).expect("decode delivery query");

        assert_eq!(command_contract, MailClientContractV1::Delivery);
        assert_eq!(query_contract, MailClientContractV1::DeliveryQuery);
        assert_ne!(command_contract, query_contract);
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

    #[test]
    fn stable_empty_error_response_is_runtime_rejection_not_protocol_corruption() {
        let rejection = ModuleClientResponseV1 {
            protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
            request_id: 9,
            response_payload: Vec::new(),
            error_code: "REJECTED".to_owned(),
        }
        .encode_to_vec();
        assert_eq!(
            decode_module_response(MailClientContractV1::Delivery, &rejection),
            Err(MailClientPortErrorV1::Runtime)
        );

        let mut invalid =
            ModuleClientResponseV1::decode(rejection.as_slice()).expect("decode rejection");
        invalid.response_payload = vec![1];
        assert_eq!(
            decode_module_response(MailClientContractV1::Delivery, &invalid.encode_to_vec()),
            Err(MailClientPortErrorV1::Protocol)
        );
    }
}
