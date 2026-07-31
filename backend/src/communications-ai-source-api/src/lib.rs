#![forbid(unsafe_code)]

mod content;
mod envelope;

pub use content::{
    CommunicationReplySourceContentErrorV1, CommunicationSummarySourceContentErrorV1,
    decode_communication_reply_source_content_v1, decode_communication_summary_source_content_v1,
    encode_communication_reply_source_content_v1, encode_communication_summary_source_content_v1,
    validate_communication_reply_source_content_v1,
    validate_communication_summary_source_content_v1,
};
pub use envelope::{
    CommunicationReplySourceEnvelopeBuildErrorV1, CommunicationReplySourceEnvelopeContextV1,
    CommunicationSummarySourceEnvelopeBuildErrorV1, CommunicationSummarySourceEnvelopeContextV1,
    build_communication_reply_source_prepare_outbox_record_v1,
    build_communication_reply_source_prepared_outbox_record_v1,
    build_communication_reply_source_rejected_outbox_record_v1,
    build_communication_summary_source_prepare_outbox_record_v1,
    build_communication_summary_source_prepared_outbox_record_v1,
    build_communication_summary_source_rejected_outbox_record_v1,
};
use hermes_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

pub const PACKAGE: &str = "hermes-communications-ai-source-api";
pub const COMMUNICATIONS_AI_SOURCE_OWNER_V1: &str = "communications";
pub const COMMUNICATION_REPLY_SOURCE_PREPARE_CONTRACT_NAME_V1: &str =
    "communication_reply_source_prepare";
pub const COMMUNICATION_REPLY_SOURCE_PREPARED_CONTRACT_NAME_V1: &str =
    "communication_reply_source_prepared";
pub const COMMUNICATION_REPLY_SOURCE_REJECTED_CONTRACT_NAME_V1: &str =
    "communication_reply_source_rejected";
pub const COMMUNICATION_SUMMARY_SOURCE_PREPARE_CONTRACT_NAME_V1: &str =
    "communication_summary_source_prepare";
pub const COMMUNICATION_SUMMARY_SOURCE_PREPARED_CONTRACT_NAME_V1: &str =
    "communication_summary_source_prepared";
pub const COMMUNICATION_SUMMARY_SOURCE_REJECTED_CONTRACT_NAME_V1: &str =
    "communication_summary_source_rejected";
pub const COMMUNICATIONS_AI_SOURCE_CONTRACT_MAJOR_V1: u32 = 1;
pub const COMMUNICATIONS_AI_SOURCE_CONTRACT_REVISION_V1: u32 = 2;
pub const COMMUNICATION_REPLY_SOURCE_MAX_BYTES_V1: u64 = 256 * 1024;
pub const COMMUNICATION_SUMMARY_SOURCE_MAX_BYTES_V1: u64 = 256 * 1024;
pub const COMMUNICATION_REPLY_SOURCE_MAX_PROOF_BYTES_V1: usize = 2_048;
pub const COMMUNICATION_SUMMARY_SOURCE_MAX_PROOF_BYTES_V1: usize = 2_048;
pub const COMMUNICATION_REPLY_SOURCE_MAX_IN_FLIGHT_V1: u32 = 32;
pub const COMMUNICATIONS_AI_SOURCE_CAPABILITY_ID_V1: &str = "communications.ai-reply-source.v1";
pub const COMMUNICATIONS_SUMMARY_SOURCE_CAPABILITY_ID_V1: &str =
    "communications.ai-summary-source.v1";
pub const COMMUNICATION_REPLY_SOURCE_BLOB_TARGET_OWNER_ID_V1: &str =
    "communication_reply_suggestion";
pub const COMMUNICATION_REPLY_SOURCE_BLOB_TARGET_MODULE_ID_V1: &str =
    "hermes-communication-reply-suggestion-runtime";
pub const COMMUNICATION_REPLY_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1: &str =
    "communication_reply_suggestion.source.blob.v1";
pub const COMMUNICATION_SUMMARY_SOURCE_BLOB_TARGET_OWNER_ID_V1: &str = "communication_summary";
pub const COMMUNICATION_SUMMARY_SOURCE_BLOB_TARGET_MODULE_ID_V1: &str =
    "hermes-communication-summary-runtime";
pub const COMMUNICATION_SUMMARY_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1: &str =
    "communication_summary.source.blob.v1";

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/hermes.communications.ai_source.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/communications_ai_source_schema.rs"
));

pub const COMMUNICATIONS_AI_SOURCE_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/communications-ai-source-v1.bin"));

#[must_use]
pub fn communication_reply_source_prepare_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(COMMUNICATION_REPLY_SOURCE_PREPARE_CONTRACT_NAME_V1)
}

#[must_use]
pub fn communication_reply_source_prepared_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(COMMUNICATION_REPLY_SOURCE_PREPARED_CONTRACT_NAME_V1)
}

#[must_use]
pub fn communication_reply_source_rejected_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(COMMUNICATION_REPLY_SOURCE_REJECTED_CONTRACT_NAME_V1)
}

#[must_use]
pub fn communication_summary_source_prepare_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(COMMUNICATION_SUMMARY_SOURCE_PREPARE_CONTRACT_NAME_V1)
}

#[must_use]
pub fn communication_summary_source_prepared_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(COMMUNICATION_SUMMARY_SOURCE_PREPARED_CONTRACT_NAME_V1)
}

#[must_use]
pub fn communication_summary_source_rejected_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(COMMUNICATION_SUMMARY_SOURCE_REJECTED_CONTRACT_NAME_V1)
}

#[must_use]
pub fn communication_reply_source_prepare_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Command,
        communication_reply_source_prepare_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn communication_reply_source_prepare_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Command,
        communication_reply_source_prepare_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn communication_reply_source_prepared_publish_request_v1() -> CapabilityRequestV1 {
    result_route(
        communication_reply_source_prepared_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn communication_reply_source_prepared_consume_request_v1() -> CapabilityRequestV1 {
    result_route(
        communication_reply_source_prepared_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn communication_reply_source_rejected_publish_request_v1() -> CapabilityRequestV1 {
    result_route(
        communication_reply_source_rejected_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn communication_reply_source_rejected_consume_request_v1() -> CapabilityRequestV1 {
    result_route(
        communication_reply_source_rejected_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn communication_summary_source_prepare_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Command,
        communication_summary_source_prepare_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn communication_summary_source_prepare_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Command,
        communication_summary_source_prepare_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn communication_summary_source_prepared_publish_request_v1() -> CapabilityRequestV1 {
    result_route(
        communication_summary_source_prepared_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn communication_summary_source_prepared_consume_request_v1() -> CapabilityRequestV1 {
    result_route(
        communication_summary_source_prepared_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn communication_summary_source_rejected_publish_request_v1() -> CapabilityRequestV1 {
    result_route(
        communication_summary_source_rejected_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn communication_summary_source_rejected_consume_request_v1() -> CapabilityRequestV1 {
    result_route(
        communication_summary_source_rejected_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

fn contract_reference(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATIONS_AI_SOURCE_OWNER_V1.to_owned(),
        name: name.to_owned(),
        major: COMMUNICATIONS_AI_SOURCE_CONTRACT_MAJOR_V1,
        revision: COMMUNICATIONS_AI_SOURCE_CONTRACT_REVISION_V1,
        schema_sha256: COMMUNICATIONS_AI_SOURCE_SCHEMA_SHA256.to_vec(),
    }
}

fn result_route(
    contract: ContractReferenceV1,
    direction: EventRouteDirectionV1,
    requirement: EventSubscriptionRequirementV1,
) -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        contract,
        direction,
        requirement,
    )
}

fn event_route(
    envelope_kind: DurableEnvelopeKindV1,
    contract: ContractReferenceV1,
    direction: EventRouteDirectionV1,
    subscription_requirement: EventSubscriptionRequirementV1,
) -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: envelope_kind as i32,
            contract: Some(contract),
            direction: direction as i32,
            max_in_flight: COMMUNICATION_REPLY_SOURCE_MAX_IN_FLIGHT_V1,
            subscription_requirement: subscription_requirement as i32,
            max_deliver: u32::from(direction == EventRouteDirectionV1::Consume) * 10,
            ack_wait_millis: u32::from(direction == EventRouteDirectionV1::Consume) * 30_000,
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_target_is_exact_reply_workflow() {
        assert_eq!(
            COMMUNICATION_REPLY_SOURCE_BLOB_TARGET_OWNER_ID_V1,
            "communication_reply_suggestion"
        );
        assert_eq!(
            COMMUNICATION_REPLY_SOURCE_BLOB_TARGET_MODULE_ID_V1,
            "hermes-communication-reply-suggestion-runtime"
        );
        assert_eq!(
            COMMUNICATION_REPLY_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1,
            "communication_reply_suggestion.source.blob.v1"
        );
    }

    #[test]
    fn summary_source_target_is_a_distinct_exact_workflow() {
        assert_eq!(
            COMMUNICATION_SUMMARY_SOURCE_BLOB_TARGET_OWNER_ID_V1,
            "communication_summary"
        );
        assert_eq!(
            COMMUNICATION_SUMMARY_SOURCE_BLOB_TARGET_MODULE_ID_V1,
            "hermes-communication-summary-runtime"
        );
        assert_eq!(
            COMMUNICATION_SUMMARY_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1,
            "communication_summary.source.blob.v1"
        );
        assert_ne!(
            communication_summary_source_prepare_contract_reference_v1(),
            communication_reply_source_prepare_contract_reference_v1()
        );
    }

    #[test]
    fn routes_are_exact_and_directional() {
        let Some(Request::EventRoute(prepare)) =
            communication_reply_source_prepare_consume_request_v1().request
        else {
            panic!("prepare route");
        };
        assert_eq!(prepare.direction, EventRouteDirectionV1::Consume as i32);
        assert_eq!(
            prepare.subscription_requirement,
            EventSubscriptionRequirementV1::Required as i32
        );
        assert_eq!(
            prepare.contract,
            Some(communication_reply_source_prepare_contract_reference_v1())
        );
    }
}
