//! Structural validation for managed publication of client-safe realtime events.

use crate::v1::{
    ContractReferenceV1, ManagedRuntimeClientRealtimePublishRequestV1,
    ManagedRuntimeClientRealtimePublishResponseV1,
};

const MAX_IDENTIFIER_BYTES: usize = 128;
const MAX_NAME_BYTES: usize = 256;
const MAX_CURSOR_BYTES: usize = 512;
const MAX_PAYLOAD_BYTES: usize = 64 * 1024;
const SCHEMA_SHA256_BYTES: usize = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManagedClientRealtimeValidationErrorV1 {
    InvalidRequest,
    InvalidResponse,
}

pub fn validate_managed_client_realtime_publish_request_v1(
    request: &ManagedRuntimeClientRealtimePublishRequestV1,
) -> Result<(), ManagedClientRealtimeValidationErrorV1> {
    if !request.contract.as_ref().is_some_and(valid_contract)
        || !valid_text(&request.logical_owner_id, MAX_IDENTIFIER_BYTES)
        || !valid_bytes(&request.event_id, MAX_IDENTIFIER_BYTES)
        || !valid_cursor(&request.cursor)
        || !valid_text(&request.event_kind, MAX_NAME_BYTES)
        || request.occurred_at_unix_millis == 0
        || !optional_text(&request.causation_id)
        || !optional_text(&request.correlation_id)
        || !optional_text(&request.trace_id)
        || request.payload.len() > MAX_PAYLOAD_BYTES
    {
        return Err(ManagedClientRealtimeValidationErrorV1::InvalidRequest);
    }
    Ok(())
}

pub fn validate_managed_client_realtime_publish_response_v1(
    response: &ManagedRuntimeClientRealtimePublishResponseV1,
) -> Result<(), ManagedClientRealtimeValidationErrorV1> {
    valid_cursor(&response.accepted_cursor)
        .then_some(())
        .ok_or(ManagedClientRealtimeValidationErrorV1::InvalidResponse)
}

fn valid_contract(contract: &ContractReferenceV1) -> bool {
    valid_text(&contract.owner, MAX_IDENTIFIER_BYTES)
        && valid_text(&contract.name, MAX_IDENTIFIER_BYTES)
        && contract.major > 0
        && contract.revision > 0
        && contract.schema_sha256.len() == SCHEMA_SHA256_BYTES
}

fn valid_cursor(value: &str) -> bool {
    valid_text(value, MAX_CURSOR_BYTES)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_graphic() && byte != b'?' && byte != b'#')
}

fn valid_bytes(value: &[u8], maximum: usize) -> bool {
    !value.is_empty() && value.len() <= maximum
}

fn valid_text(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum
        && value.chars().all(|character| !character.is_control())
}

fn optional_text(value: &str) -> bool {
    value.is_empty() || valid_text(value, MAX_IDENTIFIER_BYTES)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> ManagedRuntimeClientRealtimePublishRequestV1 {
        ManagedRuntimeClientRealtimePublishRequestV1 {
            contract: Some(ContractReferenceV1 {
                owner: "communication_delivery_intent".to_owned(),
                name: "communication_delivery_intent.status_changed".to_owned(),
                major: 1,
                revision: 1,
                schema_sha256: vec![7; 32],
            }),
            logical_owner_id: "owner-1".to_owned(),
            event_id: vec![1; 16],
            cursor: "communication-delivery-intent/42".to_owned(),
            event_kind: "status_changed".to_owned(),
            occurred_at_unix_millis: 1,
            causation_id: String::new(),
            correlation_id: "operation-1".to_owned(),
            trace_id: String::new(),
            payload: vec![2; 64],
        }
    }

    #[test]
    fn accepts_an_exact_bounded_client_safe_publication() {
        assert_eq!(
            validate_managed_client_realtime_publish_request_v1(&request()),
            Ok(())
        );
    }

    #[test]
    fn rejects_an_unbounded_payload_or_url_like_cursor() {
        let mut invalid = request();
        invalid.payload = vec![0; MAX_PAYLOAD_BYTES + 1];
        assert_eq!(
            validate_managed_client_realtime_publish_request_v1(&invalid),
            Err(ManagedClientRealtimeValidationErrorV1::InvalidRequest)
        );
        invalid = request();
        invalid.cursor = "cursor?secret".to_owned();
        assert_eq!(
            validate_managed_client_realtime_publish_request_v1(&invalid),
            Err(ManagedClientRealtimeValidationErrorV1::InvalidRequest)
        );
    }
}
