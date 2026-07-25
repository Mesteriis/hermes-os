use prost::Message;

use crate::{MailClientResponseV1, MailSendMailRequestV1, MailSyncInboxRequestV1, wire};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailClientWireErrorV1 {
    InvalidPayload,
}

#[must_use]
pub fn encode_sync_request(request: &MailSyncInboxRequestV1) -> Vec<u8> {
    wire::SyncInboxRequestV1 {
        operation_id: request.operation_id.clone(),
    }
    .encode_to_vec()
}

pub fn decode_sync_request(bytes: &[u8]) -> Result<MailSyncInboxRequestV1, MailClientWireErrorV1> {
    let request = wire::SyncInboxRequestV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    let request = MailSyncInboxRequestV1 {
        operation_id: request.operation_id,
    };
    if request.operation_id.trim().is_empty() || encode_sync_request(&request) != bytes {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(request)
}

#[must_use]
pub fn encode_delivery_request(request: &MailSendMailRequestV1) -> Vec<u8> {
    wire::SendMailRequestV1 {
        operation_id: request.operation_id.clone(),
        provider_conversation_id: request.provider_conversation_id.clone(),
        recipient: request.recipients.clone(),
        subject: request.subject.clone(),
        text_body: request.text_body.clone(),
    }
    .encode_to_vec()
}

pub fn decode_delivery_request(
    bytes: &[u8],
) -> Result<MailSendMailRequestV1, MailClientWireErrorV1> {
    let request = wire::SendMailRequestV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    let request = MailSendMailRequestV1 {
        operation_id: request.operation_id,
        provider_conversation_id: request.provider_conversation_id,
        recipients: request.recipient,
        subject: request.subject,
        text_body: request.text_body,
    };
    if request.operation_id.trim().is_empty()
        || request.recipients.is_empty()
        || request
            .recipients
            .iter()
            .any(|recipient| recipient.trim().is_empty())
        || encode_delivery_request(&request) != bytes
    {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(request)
}

#[must_use]
pub fn encode_sync_response(operation_id: &str, observed_messages: u32) -> Vec<u8> {
    wire::SyncInboxCompletedV1 {
        operation_id: operation_id.to_owned(),
        observed_messages,
    }
    .encode_to_vec()
}

pub fn decode_sync_response(bytes: &[u8]) -> Result<MailClientResponseV1, MailClientWireErrorV1> {
    let response = wire::SyncInboxCompletedV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    if response.operation_id.trim().is_empty()
        || encode_sync_response(&response.operation_id, response.observed_messages) != bytes
    {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(MailClientResponseV1::SyncInboxCompleted {
        operation_id: response.operation_id,
        observed_messages: response.observed_messages,
    })
}

#[must_use]
pub fn encode_delivery_response(operation_id: &str, response_code: u16) -> Vec<u8> {
    wire::MailAcceptedV1 {
        operation_id: operation_id.to_owned(),
        response_code: u32::from(response_code),
    }
    .encode_to_vec()
}

pub fn decode_delivery_response(
    bytes: &[u8],
) -> Result<MailClientResponseV1, MailClientWireErrorV1> {
    let response =
        wire::MailAcceptedV1::decode(bytes).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    let response_code =
        u16::try_from(response.response_code).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    if response.operation_id.trim().is_empty()
        || !(200..600).contains(&response_code)
        || encode_delivery_response(&response.operation_id, response_code) != bytes
    {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(MailClientResponseV1::MailAccepted {
        operation_id: response.operation_id,
        response_code,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_each_exact_route_payload() {
        let sync = MailSyncInboxRequestV1 {
            operation_id: "sync-operation".to_owned(),
        };
        assert_eq!(decode_sync_request(&encode_sync_request(&sync)), Ok(sync));

        let delivery = MailSendMailRequestV1 {
            operation_id: "delivery-operation".to_owned(),
            provider_conversation_id: "conversation".to_owned(),
            recipients: vec!["recipient@example.com".to_owned()],
            subject: "subject".to_owned(),
            text_body: "body".to_owned(),
        };
        assert_eq!(
            decode_delivery_request(&encode_delivery_request(&delivery)),
            Ok(delivery)
        );
    }

    #[test]
    fn rejects_an_exact_delivery_payload_as_sync() {
        let delivery = MailSendMailRequestV1 {
            operation_id: "delivery-operation".to_owned(),
            provider_conversation_id: "conversation".to_owned(),
            recipients: vec!["recipient@example.com".to_owned()],
            subject: "subject".to_owned(),
            text_body: "body".to_owned(),
        };

        assert_eq!(
            decode_sync_request(&encode_delivery_request(&delivery)),
            Err(MailClientWireErrorV1::InvalidPayload)
        );
    }
}
