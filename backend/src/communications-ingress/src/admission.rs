//! Descriptor metadata for the exact provider-neutral observation ingress.

use hermes_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

use crate::{
    COMMUNICATION_ATTACHMENT_ANCHOR_RECORDED_SCHEMA_SHA256,
    COMMUNICATION_ATTACHMENT_BLOB_ADMISSION_OBSERVATION_SCHEMA_SHA256,
    COMMUNICATION_ATTACHMENT_SAFETY_VERDICT_OBSERVATION_SCHEMA_SHA256,
    COMMUNICATION_OBSERVATION_SCHEMA_SHA256,
};

pub const COMMUNICATION_OBSERVED_CONTRACT_OWNER: &str = "communications";
pub const COMMUNICATION_OBSERVED_CONTRACT_NAME: &str = "communication_observed";
pub const COMMUNICATION_OBSERVED_CONTRACT_MAJOR: u32 = 1;
pub const COMMUNICATION_OBSERVED_CONTRACT_REVISION: u32 = 1;
pub const COMMUNICATION_OBSERVED_MAX_IN_FLIGHT: u32 = 64;
pub const COMMUNICATION_ATTACHMENT_BLOB_ADMISSION_OBSERVED_CONTRACT_NAME: &str =
    "communication_attachment_blob_admission_observed";
pub const COMMUNICATION_ATTACHMENT_SAFETY_VERDICT_OBSERVED_CONTRACT_NAME: &str =
    "communication_attachment_safety_verdict_observed";
pub const COMMUNICATION_ATTACHMENT_ANCHOR_RECORDED_CONTRACT_NAME: &str =
    "communication_attachment_anchor_recorded";

#[must_use]
pub fn communication_observed_contract_reference_v1() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATION_OBSERVED_CONTRACT_OWNER.to_owned(),
        name: COMMUNICATION_OBSERVED_CONTRACT_NAME.to_owned(),
        major: COMMUNICATION_OBSERVED_CONTRACT_MAJOR,
        revision: COMMUNICATION_OBSERVED_CONTRACT_REVISION,
        schema_sha256: COMMUNICATION_OBSERVATION_SCHEMA_SHA256.to_vec(),
    }
}

#[must_use]
pub fn communication_attachment_blob_admission_observed_contract_reference_v1()
-> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATION_OBSERVED_CONTRACT_OWNER.to_owned(),
        name: COMMUNICATION_ATTACHMENT_BLOB_ADMISSION_OBSERVED_CONTRACT_NAME.to_owned(),
        major: COMMUNICATION_OBSERVED_CONTRACT_MAJOR,
        revision: COMMUNICATION_OBSERVED_CONTRACT_REVISION,
        schema_sha256: COMMUNICATION_ATTACHMENT_BLOB_ADMISSION_OBSERVATION_SCHEMA_SHA256.to_vec(),
    }
}

#[must_use]
pub fn communication_attachment_safety_verdict_observed_contract_reference_v1()
-> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATION_OBSERVED_CONTRACT_OWNER.to_owned(),
        name: COMMUNICATION_ATTACHMENT_SAFETY_VERDICT_OBSERVED_CONTRACT_NAME.to_owned(),
        major: COMMUNICATION_OBSERVED_CONTRACT_MAJOR,
        revision: COMMUNICATION_OBSERVED_CONTRACT_REVISION,
        schema_sha256: COMMUNICATION_ATTACHMENT_SAFETY_VERDICT_OBSERVATION_SCHEMA_SHA256.to_vec(),
    }
}

#[must_use]
pub fn communication_attachment_anchor_recorded_contract_reference_v1() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATION_OBSERVED_CONTRACT_OWNER.to_owned(),
        name: COMMUNICATION_ATTACHMENT_ANCHOR_RECORDED_CONTRACT_NAME.to_owned(),
        major: COMMUNICATION_OBSERVED_CONTRACT_MAJOR,
        revision: COMMUNICATION_OBSERVED_CONTRACT_REVISION,
        schema_sha256: COMMUNICATION_ATTACHMENT_ANCHOR_RECORDED_SCHEMA_SHA256.to_vec(),
    }
}

/// Exact route request used by integration descriptors. It requests publish
/// authority only; it does not expose a Communications runtime or store.
#[must_use]
pub fn communication_observed_publish_request_v1() -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: DurableEnvelopeKindV1::Observation as i32,
            contract: Some(communication_observed_contract_reference_v1()),
            direction: EventRouteDirectionV1::Publish as i32,
            max_in_flight: COMMUNICATION_OBSERVED_MAX_IN_FLIGHT,
            subscription_requirement: EventSubscriptionRequirementV1::Unspecified as i32,
            max_deliver: 0,
            ack_wait_millis: 0,
        })),
    }
}

/// Exact publish route for an integration that owns an admitted attachment
/// Blob write. It does not grant a Communications query, store, or runtime.
#[must_use]
pub fn communication_attachment_blob_admission_observed_publish_request_v1() -> CapabilityRequestV1
{
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: DurableEnvelopeKindV1::Observation as i32,
            contract: Some(
                communication_attachment_blob_admission_observed_contract_reference_v1(),
            ),
            direction: EventRouteDirectionV1::Publish as i32,
            max_in_flight: COMMUNICATION_OBSERVED_MAX_IN_FLIGHT,
            subscription_requirement: EventSubscriptionRequirementV1::Unspecified as i32,
            max_deliver: 0,
            ack_wait_millis: 0,
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integration_route_is_publish_only_and_schema_bound() {
        let request = communication_observed_publish_request_v1();
        let Some(Request::EventRoute(route)) = request.request else {
            panic!("event route");
        };

        assert_eq!(route.direction, EventRouteDirectionV1::Publish as i32);
        assert_eq!(
            route.contract.expect("contract").schema_sha256,
            COMMUNICATION_OBSERVATION_SCHEMA_SHA256,
        );
    }

    #[test]
    fn attachment_observations_are_distinct_schema_bound_contracts() {
        let blob = communication_attachment_blob_admission_observed_contract_reference_v1();
        let safety = communication_attachment_safety_verdict_observed_contract_reference_v1();
        let anchor = communication_attachment_anchor_recorded_contract_reference_v1();

        assert_eq!(blob.owner, COMMUNICATION_OBSERVED_CONTRACT_OWNER);
        assert_eq!(safety.owner, COMMUNICATION_OBSERVED_CONTRACT_OWNER);
        assert_ne!(blob.name, safety.name);
        assert_ne!(anchor.name, safety.name);
        assert_eq!(
            blob.schema_sha256,
            COMMUNICATION_ATTACHMENT_BLOB_ADMISSION_OBSERVATION_SCHEMA_SHA256,
        );
        assert_eq!(
            safety.schema_sha256,
            COMMUNICATION_ATTACHMENT_SAFETY_VERDICT_OBSERVATION_SCHEMA_SHA256,
        );
        assert_eq!(
            anchor.schema_sha256,
            COMMUNICATION_ATTACHMENT_ANCHOR_RECORDED_SCHEMA_SHA256,
        );
    }

    #[test]
    fn blob_admission_route_is_publish_only_and_schema_bound() {
        let request = communication_attachment_blob_admission_observed_publish_request_v1();
        let Some(Request::EventRoute(route)) = request.request else {
            panic!("event route");
        };

        assert_eq!(
            route.envelope_kind,
            DurableEnvelopeKindV1::Observation as i32
        );
        assert_eq!(route.direction, EventRouteDirectionV1::Publish as i32);
        assert_eq!(
            route.contract.expect("contract").name,
            COMMUNICATION_ATTACHMENT_BLOB_ADMISSION_OBSERVED_CONTRACT_NAME
        );
    }
}
