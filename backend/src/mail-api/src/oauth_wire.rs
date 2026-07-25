use prost::Message;

use crate::{
    GmailOAuthCompleteRequestV1, GmailOAuthOperationKindV1, GmailOAuthOperationStatusV1,
    GmailOAuthOutcomeV1, GmailOAuthRefreshRequestV1, GmailOAuthStartRequestV1, GmailOAuthStartedV1,
    GmailOAuthStatusRequestV1, MailClientResponseV1, client_wire::MailClientWireErrorV1, wire,
};

const MAX_OPERATION_ID_BYTES: usize = 256;
const MAX_SETUP_ID_BYTES: usize = 256;
const MAX_STATE_BYTES: usize = 1024;
const MAX_AUTHORIZATION_CODE_BYTES: usize = 8 * 1024;
const MAX_AUTHORIZATION_URL_BYTES: usize = 16 * 1024;

#[must_use]
pub fn encode_start_request(request: &GmailOAuthStartRequestV1) -> Vec<u8> {
    wire::StartGmailOAuthRequestV1 {
        operation_id: request.operation_id.clone(),
    }
    .encode_to_vec()
}

pub fn decode_start_request(
    bytes: &[u8],
) -> Result<GmailOAuthStartRequestV1, MailClientWireErrorV1> {
    let request = wire::StartGmailOAuthRequestV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    let request = GmailOAuthStartRequestV1 {
        operation_id: request.operation_id,
    };
    if !valid_identifier(&request.operation_id, MAX_OPERATION_ID_BYTES)
        || encode_start_request(&request) != bytes
    {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(request)
}

#[must_use]
pub fn encode_start_response(response: &GmailOAuthStartedV1) -> Vec<u8> {
    wire::GmailOAuthStartedV1 {
        operation_id: response.operation_id.clone(),
        setup_id: response.setup_id.clone(),
        authorization_url: response.authorization_url.clone(),
        expires_at_unix_seconds: response.expires_at_unix_seconds,
    }
    .encode_to_vec()
}

pub fn decode_start_response(bytes: &[u8]) -> Result<MailClientResponseV1, MailClientWireErrorV1> {
    let response = wire::GmailOAuthStartedV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    let response = GmailOAuthStartedV1 {
        operation_id: response.operation_id,
        setup_id: response.setup_id,
        authorization_url: response.authorization_url,
        expires_at_unix_seconds: response.expires_at_unix_seconds,
    };
    if !valid_identifier(&response.operation_id, MAX_OPERATION_ID_BYTES)
        || !valid_identifier(&response.setup_id, MAX_SETUP_ID_BYTES)
        || !valid_authorization_url(&response.authorization_url)
        || response.expires_at_unix_seconds <= 0
        || encode_start_response(&response) != bytes
    {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(MailClientResponseV1::GmailOAuthStarted(response))
}

#[must_use]
pub fn encode_complete_request(request: &GmailOAuthCompleteRequestV1) -> Vec<u8> {
    wire::CompleteGmailOAuthRequestV1 {
        operation_id: request.operation_id.clone(),
        setup_id: request.setup_id.clone(),
        state: request.state.clone(),
        authorization_code: request.authorization_code.clone(),
    }
    .encode_to_vec()
}

pub fn decode_complete_request(
    bytes: &[u8],
) -> Result<GmailOAuthCompleteRequestV1, MailClientWireErrorV1> {
    let request = wire::CompleteGmailOAuthRequestV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    let request = GmailOAuthCompleteRequestV1 {
        operation_id: request.operation_id,
        setup_id: request.setup_id,
        state: request.state,
        authorization_code: request.authorization_code,
    };
    if !valid_identifier(&request.operation_id, MAX_OPERATION_ID_BYTES)
        || !valid_identifier(&request.setup_id, MAX_SETUP_ID_BYTES)
        || !valid_secret_carrier(&request.state, MAX_STATE_BYTES)
        || !valid_secret_carrier(&request.authorization_code, MAX_AUTHORIZATION_CODE_BYTES)
        || encode_complete_request(&request) != bytes
    {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(request)
}

#[must_use]
pub fn encode_refresh_request(request: &GmailOAuthRefreshRequestV1) -> Vec<u8> {
    wire::RefreshGmailOAuthRequestV1 {
        operation_id: request.operation_id.clone(),
    }
    .encode_to_vec()
}

pub fn decode_refresh_request(
    bytes: &[u8],
) -> Result<GmailOAuthRefreshRequestV1, MailClientWireErrorV1> {
    let request = wire::RefreshGmailOAuthRequestV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    let request = GmailOAuthRefreshRequestV1 {
        operation_id: request.operation_id,
    };
    if !valid_identifier(&request.operation_id, MAX_OPERATION_ID_BYTES)
        || encode_refresh_request(&request) != bytes
    {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(request)
}

#[must_use]
pub fn encode_status_request(request: &GmailOAuthStatusRequestV1) -> Vec<u8> {
    wire::GetGmailOAuthStatusRequestV1 {
        operation_id: request.operation_id.clone(),
    }
    .encode_to_vec()
}

pub fn decode_status_request(
    bytes: &[u8],
) -> Result<GmailOAuthStatusRequestV1, MailClientWireErrorV1> {
    let request = wire::GetGmailOAuthStatusRequestV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    let request = GmailOAuthStatusRequestV1 {
        operation_id: request.operation_id,
    };
    if !valid_identifier(&request.operation_id, MAX_OPERATION_ID_BYTES)
        || encode_status_request(&request) != bytes
    {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(request)
}

#[must_use]
pub fn encode_accepted_response(operation_id: &str) -> Vec<u8> {
    wire::MailAcceptedV1 {
        operation_id: operation_id.to_owned(),
    }
    .encode_to_vec()
}

pub fn decode_accepted_response(
    bytes: &[u8],
) -> Result<MailClientResponseV1, MailClientWireErrorV1> {
    let response =
        wire::MailAcceptedV1::decode(bytes).map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    if !valid_identifier(&response.operation_id, MAX_OPERATION_ID_BYTES)
        || encode_accepted_response(&response.operation_id) != bytes
    {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(MailClientResponseV1::GmailOAuthAccepted {
        operation_id: response.operation_id,
    })
}

#[must_use]
pub fn encode_status_response(status: Option<&GmailOAuthOperationStatusV1>) -> Vec<u8> {
    wire::GetGmailOAuthStatusResponseV1 {
        status: status.map(|status| wire::GmailOAuthOperationStatusV1 {
            operation_id: status.operation_id.clone(),
            kind: match status.kind {
                GmailOAuthOperationKindV1::Complete => {
                    wire::GmailOAuthOperationKindV1::GmailOauthOperationKindComplete as i32
                }
                GmailOAuthOperationKindV1::Refresh => {
                    wire::GmailOAuthOperationKindV1::GmailOauthOperationKindRefresh as i32
                }
            },
            outcome: match status.outcome {
                GmailOAuthOutcomeV1::Pending => {
                    wire::GmailOAuthOutcomeV1::GmailOauthOutcomePending as i32
                }
                GmailOAuthOutcomeV1::Completed => {
                    wire::GmailOAuthOutcomeV1::GmailOauthOutcomeCompleted as i32
                }
                GmailOAuthOutcomeV1::Rejected => {
                    wire::GmailOAuthOutcomeV1::GmailOauthOutcomeRejected as i32
                }
                GmailOAuthOutcomeV1::OutcomeUnknown => {
                    wire::GmailOAuthOutcomeV1::GmailOauthOutcomeUnknown as i32
                }
            },
            requested_at_unix_seconds: status.requested_at_unix_seconds,
            completed_at_unix_seconds: status.completed_at_unix_seconds,
        }),
    }
    .encode_to_vec()
}

pub fn decode_status_response(bytes: &[u8]) -> Result<MailClientResponseV1, MailClientWireErrorV1> {
    let response = wire::GetGmailOAuthStatusResponseV1::decode(bytes)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?;
    let status = response.status.map(decode_status).transpose()?;
    if encode_status_response(status.as_ref()) != bytes {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(MailClientResponseV1::GmailOAuthStatus(status))
}

fn decode_status(
    status: wire::GmailOAuthOperationStatusV1,
) -> Result<GmailOAuthOperationStatusV1, MailClientWireErrorV1> {
    let kind = match wire::GmailOAuthOperationKindV1::try_from(status.kind)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?
    {
        wire::GmailOAuthOperationKindV1::GmailOauthOperationKindComplete => {
            GmailOAuthOperationKindV1::Complete
        }
        wire::GmailOAuthOperationKindV1::GmailOauthOperationKindRefresh => {
            GmailOAuthOperationKindV1::Refresh
        }
        wire::GmailOAuthOperationKindV1::GmailOauthOperationKindUnspecified => {
            return Err(MailClientWireErrorV1::InvalidPayload);
        }
    };
    let outcome = match wire::GmailOAuthOutcomeV1::try_from(status.outcome)
        .map_err(|_| MailClientWireErrorV1::InvalidPayload)?
    {
        wire::GmailOAuthOutcomeV1::GmailOauthOutcomePending => GmailOAuthOutcomeV1::Pending,
        wire::GmailOAuthOutcomeV1::GmailOauthOutcomeCompleted => GmailOAuthOutcomeV1::Completed,
        wire::GmailOAuthOutcomeV1::GmailOauthOutcomeRejected => GmailOAuthOutcomeV1::Rejected,
        wire::GmailOAuthOutcomeV1::GmailOauthOutcomeUnknown => GmailOAuthOutcomeV1::OutcomeUnknown,
        wire::GmailOAuthOutcomeV1::GmailOauthOutcomeUnspecified => {
            return Err(MailClientWireErrorV1::InvalidPayload);
        }
    };
    let status = GmailOAuthOperationStatusV1 {
        operation_id: status.operation_id,
        kind,
        outcome,
        requested_at_unix_seconds: status.requested_at_unix_seconds,
        completed_at_unix_seconds: status.completed_at_unix_seconds,
    };
    if !valid_status(&status) {
        return Err(MailClientWireErrorV1::InvalidPayload);
    }
    Ok(status)
}

fn valid_status(status: &GmailOAuthOperationStatusV1) -> bool {
    if !valid_identifier(&status.operation_id, MAX_OPERATION_ID_BYTES)
        || status.requested_at_unix_seconds <= 0
    {
        return false;
    }
    match status.outcome {
        GmailOAuthOutcomeV1::Pending | GmailOAuthOutcomeV1::OutcomeUnknown => {
            status.completed_at_unix_seconds.is_none()
        }
        GmailOAuthOutcomeV1::Completed | GmailOAuthOutcomeV1::Rejected => status
            .completed_at_unix_seconds
            .is_some_and(|value| value > 0),
    }
}

fn valid_identifier(value: &str, max_bytes: usize) -> bool {
    !value.is_empty()
        && value.len() <= max_bytes
        && value.is_ascii()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn valid_secret_carrier(value: &str, max_bytes: usize) -> bool {
    !value.is_empty()
        && value.len() <= max_bytes
        && value.is_ascii()
        && !value.contains(['\r', '\n', '\0'])
}

fn valid_authorization_url(value: &str) -> bool {
    value.starts_with("https://")
        && value.len() <= MAX_AUTHORIZATION_URL_BYTES
        && value.is_ascii()
        && !value.contains(['\r', '\n', '\0'])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oauth_wire_round_trips_without_exposing_secrets_in_results() {
        let complete = GmailOAuthCompleteRequestV1 {
            operation_id: "complete-operation".to_owned(),
            setup_id: "setup-id".to_owned(),
            state: "state-value".to_owned(),
            authorization_code: "authorization-code".to_owned(),
        };
        assert_eq!(
            decode_complete_request(&encode_complete_request(&complete)),
            Ok(complete)
        );

        let status = GmailOAuthOperationStatusV1 {
            operation_id: "complete-operation".to_owned(),
            kind: GmailOAuthOperationKindV1::Complete,
            outcome: GmailOAuthOutcomeV1::Completed,
            requested_at_unix_seconds: 1_783_110_000,
            completed_at_unix_seconds: Some(1_783_110_001),
        };
        let encoded = encode_status_response(Some(&status));
        assert_eq!(
            decode_status_response(&encoded),
            Ok(MailClientResponseV1::GmailOAuthStatus(Some(status)))
        );
        for secret in ["state-value", "authorization-code"] {
            assert!(
                !encoded
                    .windows(secret.len())
                    .any(|window| window == secret.as_bytes())
            );
        }
    }

    #[test]
    fn complete_rejects_control_bytes_and_empty_secrets() {
        let request = GmailOAuthCompleteRequestV1 {
            operation_id: "complete-operation".to_owned(),
            setup_id: "setup-id".to_owned(),
            state: String::new(),
            authorization_code: "code\r\nInjected: value".to_owned(),
        };
        assert_eq!(
            decode_complete_request(&encode_complete_request(&request)),
            Err(MailClientWireErrorV1::InvalidPayload)
        );
    }
}
