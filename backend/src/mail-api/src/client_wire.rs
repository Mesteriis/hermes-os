use prost::Message;

use crate::{
    MailClientRequestV1, MailClientResponseV1, MailSendMailRequestV1, MailSyncInboxRequestV1, wire,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailClientWireErrorV1 {
    InvalidPayload,
    MissingVariant,
}

pub fn encode_request(request: &MailClientRequestV1) -> Vec<u8> {
    use wire::mail_client_request_v1::Request;
    let request = match request {
        MailClientRequestV1::SyncInbox(value) => Request::SyncInbox(wire::SyncInboxRequestV1 {
            operation_id: value.operation_id.clone(),
        }),
        MailClientRequestV1::SendMail(value) => Request::SendMail(wire::SendMailRequestV1 {
            operation_id: value.operation_id.clone(),
            provider_conversation_id: value.provider_conversation_id.clone(),
            recipient: value.recipients.clone(),
            subject: value.subject.clone(),
            text_body: value.text_body.clone(),
        }),
    };
    wire::MailClientRequestV1 {
        request: Some(request),
    }
    .encode_to_vec()
}

pub fn decode_request(bytes: &[u8]) -> Result<MailClientRequestV1, MailClientWireErrorV1> {
    use wire::mail_client_request_v1::Request;
    let request = wire::MailClientRequestV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?
        .request
        .ok_or(MailClientWireErrorV1::MissingVariant)?;
    match request {
        Request::SyncInbox(value) => Ok(MailClientRequestV1::SyncInbox(MailSyncInboxRequestV1 {
            operation_id: value.operation_id,
        })),
        Request::SendMail(value) => Ok(MailClientRequestV1::SendMail(MailSendMailRequestV1 {
            operation_id: value.operation_id,
            provider_conversation_id: value.provider_conversation_id,
            recipients: value.recipient,
            subject: value.subject,
            text_body: value.text_body,
        })),
    }
}

pub fn encode_response(response: &MailClientResponseV1) -> Vec<u8> {
    use wire::mail_client_response_v1::Response;
    let response = match response {
        MailClientResponseV1::SyncInboxCompleted {
            operation_id,
            observed_messages,
        } => Response::SyncInboxCompleted(wire::SyncInboxCompletedV1 {
            operation_id: operation_id.clone(),
            observed_messages: *observed_messages,
        }),
        MailClientResponseV1::MailAccepted {
            operation_id,
            response_code,
        } => Response::MailAccepted(wire::MailAcceptedV1 {
            operation_id: operation_id.clone(),
            response_code: u32::from(*response_code),
        }),
    };
    wire::MailClientResponseV1 {
        response: Some(response),
    }
    .encode_to_vec()
}

pub fn decode_response(bytes: &[u8]) -> Result<MailClientResponseV1, MailClientWireErrorV1> {
    use wire::mail_client_response_v1::Response;
    let response = wire::MailClientResponseV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?
        .response
        .ok_or(MailClientWireErrorV1::MissingVariant)?;
    match response {
        Response::SyncInboxCompleted(value) => Ok(MailClientResponseV1::SyncInboxCompleted {
            operation_id: value.operation_id,
            observed_messages: value.observed_messages,
        }),
        Response::MailAccepted(value) => Ok(MailClientResponseV1::MailAccepted {
            operation_id: value.operation_id,
            response_code: u16::try_from(value.response_code)
                .map_err(|_| MailClientWireErrorV1::InvalidPayload)?,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::{decode_request, decode_response, encode_request, encode_response};
    use crate::{
        MailClientRequestV1, MailClientResponseV1, MailSendMailRequestV1, MailSyncInboxRequestV1,
    };
    #[test]
    fn preserves_sync_request_and_terminal_response() {
        let request = MailClientRequestV1::SyncInbox(MailSyncInboxRequestV1 {
            operation_id: "operation".into(),
        });
        assert_eq!(decode_request(&encode_request(&request)), Ok(request));
        let smtp = MailClientRequestV1::SendMail(MailSendMailRequestV1 {
            operation_id: "operation".into(),
            provider_conversation_id: "conversation".into(),
            recipients: vec!["owner@example.test".into()],
            subject: "Subject".into(),
            text_body: "Body".into(),
        });
        assert_eq!(decode_request(&encode_request(&smtp)), Ok(smtp));
        let response = MailClientResponseV1::SyncInboxCompleted {
            operation_id: "operation".into(),
            observed_messages: 7,
        };
        assert_eq!(decode_response(&encode_response(&response)), Ok(response));
    }
}
