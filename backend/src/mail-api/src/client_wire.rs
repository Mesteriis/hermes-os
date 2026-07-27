use prost::Message;

use crate::{
    MAX_DELIVERY_ATTACHMENTS, MailClientResponseV1, MailDeliveryOperationStatusV1,
    MailDeliveryOutcomeV1, MailDeliveryStatusRequestV1, MailSendMailRequestV1,
    MailSyncInboxRequestV1, wire,
};

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
        attachment_anchor_id: request
            .attachment_anchor_ids
            .iter()
            .map(|anchor_id| anchor_id.to_vec())
            .collect(),
        cc_recipient: request.cc_recipients.clone(),
        bcc_recipient: request.bcc_recipients.clone(),
    }
    .encode_to_vec()
}

pub fn decode_delivery_request(
    bytes: &[u8],
) -> Result<MailSendMailRequestV1, MailClientWireErrorV1> {
    let request = wire::SendMailRequestV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    let attachment_anchor_ids = request
        .attachment_anchor_id
        .iter()
        .map(|anchor_id| {
            let anchor_id: [u8; 16] = anchor_id
                .as_slice()
                .try_into()
                .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
            (!anchor_id.iter().all(|byte| *byte == 0))
                .then_some(anchor_id)
                .ok_or(MailClientWireErrorV1::InvalidPayload)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let request = MailSendMailRequestV1 {
        operation_id: request.operation_id,
        provider_conversation_id: request.provider_conversation_id,
        recipients: request.recipient,
        cc_recipients: request.cc_recipient,
        bcc_recipients: request.bcc_recipient,
        subject: request.subject,
        text_body: request.text_body,
        attachment_anchor_ids,
    };
    if request.operation_id.trim().is_empty()
        || request.recipients.is_empty()
        || request
            .recipients
            .iter()
            .any(|recipient| recipient.trim().is_empty())
        || request
            .cc_recipients
            .iter()
            .chain(&request.bcc_recipients)
            .any(|recipient| recipient.trim().is_empty())
        || request.attachment_anchor_ids.len() > MAX_DELIVERY_ATTACHMENTS
        || request
            .attachment_anchor_ids
            .iter()
            .enumerate()
            .any(|(index, anchor_id)| request.attachment_anchor_ids[..index].contains(anchor_id))
        || encode_delivery_request(&request) != bytes
    {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(request)
}

#[must_use]
pub fn encode_delivery_status_request(request: &MailDeliveryStatusRequestV1) -> Vec<u8> {
    wire::GetMailDeliveryStatusRequestV1 {
        operation_id: request.operation_id.clone(),
    }
    .encode_to_vec()
}

pub fn decode_delivery_status_request(
    bytes: &[u8],
) -> Result<MailDeliveryStatusRequestV1, MailClientWireErrorV1> {
    let request = wire::GetMailDeliveryStatusRequestV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    let request = MailDeliveryStatusRequestV1 {
        operation_id: request.operation_id,
    };
    if request.operation_id.trim().is_empty() || encode_delivery_status_request(&request) != bytes {
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
pub fn encode_delivery_response(operation_id: &str) -> Vec<u8> {
    wire::MailAcceptedV1 {
        operation_id: operation_id.to_owned(),
    }
    .encode_to_vec()
}

pub fn decode_delivery_response(
    bytes: &[u8],
) -> Result<MailClientResponseV1, MailClientWireErrorV1> {
    let response =
        wire::MailAcceptedV1::decode(bytes).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    if response.operation_id.trim().is_empty()
        || encode_delivery_response(&response.operation_id) != bytes
    {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(MailClientResponseV1::MailAccepted {
        operation_id: response.operation_id,
    })
}

#[must_use]
pub fn encode_delivery_status_response(status: Option<&MailDeliveryOperationStatusV1>) -> Vec<u8> {
    wire::GetMailDeliveryStatusResponseV1 {
        status: status.map(|status| wire::MailDeliveryOperationStatusV1 {
            operation_id: status.operation_id.clone(),
            connection_id: status.connection_id.clone(),
            outcome: match status.outcome {
                MailDeliveryOutcomeV1::Pending => {
                    wire::MailDeliveryOutcomeV1::MailDeliveryOutcomePending as i32
                }
                MailDeliveryOutcomeV1::Accepted => {
                    wire::MailDeliveryOutcomeV1::MailDeliveryOutcomeAccepted as i32
                }
                MailDeliveryOutcomeV1::Rejected => {
                    wire::MailDeliveryOutcomeV1::MailDeliveryOutcomeRejected as i32
                }
                MailDeliveryOutcomeV1::OutcomeUnknown => {
                    wire::MailDeliveryOutcomeV1::MailDeliveryOutcomeUnknown as i32
                }
            },
            requested_at_unix_seconds: status.requested_at_unix_seconds,
            completed_at_unix_seconds: status.completed_at_unix_seconds,
            response_code: status.response_code.map(u32::from),
        }),
    }
    .encode_to_vec()
}

pub fn decode_delivery_status_response(
    bytes: &[u8],
) -> Result<MailClientResponseV1, MailClientWireErrorV1> {
    let response = wire::GetMailDeliveryStatusResponseV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    let status = response
        .status
        .map(|status| {
            let outcome = match wire::MailDeliveryOutcomeV1::try_from(status.outcome)
                .map_err(|_| MailClientWireErrorV1::InvalidPayload)?
            {
                wire::MailDeliveryOutcomeV1::MailDeliveryOutcomePending => {
                    MailDeliveryOutcomeV1::Pending
                }
                wire::MailDeliveryOutcomeV1::MailDeliveryOutcomeAccepted => {
                    MailDeliveryOutcomeV1::Accepted
                }
                wire::MailDeliveryOutcomeV1::MailDeliveryOutcomeRejected => {
                    MailDeliveryOutcomeV1::Rejected
                }
                wire::MailDeliveryOutcomeV1::MailDeliveryOutcomeUnknown => {
                    MailDeliveryOutcomeV1::OutcomeUnknown
                }
                wire::MailDeliveryOutcomeV1::MailDeliveryOutcomeUnspecified => {
                    return Err(MailClientWireErrorV1::InvalidPayload);
                }
            };
            let response_code = status
                .response_code
                .map(u16::try_from)
                .transpose()
                .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
            let status = MailDeliveryOperationStatusV1 {
                operation_id: status.operation_id,
                connection_id: status.connection_id,
                outcome,
                requested_at_unix_seconds: status.requested_at_unix_seconds,
                completed_at_unix_seconds: status.completed_at_unix_seconds,
                response_code,
            };
            valid_delivery_status(&status)
                .then_some(status)
                .ok_or(MailClientWireErrorV1::InvalidPayload)
        })
        .transpose()?;
    if encode_delivery_status_response(status.as_ref()) != bytes {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(MailClientResponseV1::DeliveryStatus(status))
}

fn valid_delivery_status(status: &MailDeliveryOperationStatusV1) -> bool {
    if status.operation_id.trim().is_empty()
        || status.connection_id.trim().is_empty()
        || status.requested_at_unix_seconds <= 0
    {
        return false;
    }
    match status.outcome {
        MailDeliveryOutcomeV1::Pending | MailDeliveryOutcomeV1::OutcomeUnknown => {
            status.completed_at_unix_seconds.is_none() && status.response_code.is_none()
        }
        MailDeliveryOutcomeV1::Accepted => {
            status.completed_at_unix_seconds.is_some()
                && status
                    .response_code
                    .is_some_and(|code| (200..300).contains(&code))
        }
        MailDeliveryOutcomeV1::Rejected => {
            status.completed_at_unix_seconds.is_some() && status.response_code.is_none()
        }
    }
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
            cc_recipients: Vec::new(),
            bcc_recipients: Vec::new(),
            subject: "subject".to_owned(),
            text_body: "body".to_owned(),
            attachment_anchor_ids: Vec::new(),
        };
        assert_eq!(
            decode_delivery_request(&encode_delivery_request(&delivery)),
            Ok(delivery.clone())
        );
        let query = MailDeliveryStatusRequestV1 {
            operation_id: delivery.operation_id,
        };
        assert_eq!(
            decode_delivery_status_request(&encode_delivery_status_request(&query)),
            Ok(query)
        );
        let status = MailDeliveryOperationStatusV1 {
            operation_id: "delivery-operation".to_owned(),
            connection_id: "mail-account".to_owned(),
            outcome: MailDeliveryOutcomeV1::Accepted,
            requested_at_unix_seconds: 1_783_110_000,
            completed_at_unix_seconds: Some(1_783_110_001),
            response_code: Some(250),
        };
        assert_eq!(
            decode_delivery_status_response(&encode_delivery_status_response(Some(&status))),
            Ok(MailClientResponseV1::DeliveryStatus(Some(status)))
        );
    }

    #[test]
    fn rejects_an_exact_delivery_payload_as_sync() {
        let delivery = MailSendMailRequestV1 {
            operation_id: "delivery-operation".to_owned(),
            provider_conversation_id: "conversation".to_owned(),
            recipients: vec!["recipient@example.com".to_owned()],
            cc_recipients: Vec::new(),
            bcc_recipients: Vec::new(),
            subject: "subject".to_owned(),
            text_body: "body".to_owned(),
            attachment_anchor_ids: Vec::new(),
        };

        assert_eq!(
            decode_sync_request(&encode_delivery_request(&delivery)),
            Err(MailClientWireErrorV1::InvalidPayload)
        );
    }

    #[test]
    fn rejects_noncanonical_or_unbounded_attachment_anchor_ids() {
        let request = |attachment_anchor_id: Vec<Vec<u8>>| {
            wire::SendMailRequestV1 {
                operation_id: "delivery-operation".to_owned(),
                provider_conversation_id: "conversation".to_owned(),
                recipient: vec!["recipient@example.com".to_owned()],
                subject: "subject".to_owned(),
                text_body: "body".to_owned(),
                attachment_anchor_id,
                cc_recipient: Vec::new(),
                bcc_recipient: Vec::new(),
            }
            .encode_to_vec()
        };

        for invalid in [
            vec![vec![0; 16]],
            vec![vec![1; 15]],
            vec![vec![1; 16], vec![1; 16]],
            (1_u8..=17).map(|value| vec![value; 16]).collect(),
        ] {
            assert_eq!(
                decode_delivery_request(&request(invalid)),
                Err(MailClientWireErrorV1::InvalidPayload)
            );
        }
    }
}
