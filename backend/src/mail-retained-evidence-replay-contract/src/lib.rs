#![forbid(unsafe_code)]

use hermes_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/hermes.mail.replay.v1.rs"));
}

include!(concat!(env!("OUT_DIR"), "/mail_replay_schema.rs"));

pub const MAIL_REPLAY_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/mail-replay-v1.bin"));

pub const PACKAGE: &str = "hermes-mail-retained-evidence-replay-contract";
pub const MAIL_REPLAY_OWNER_ID_V1: &str = "mail";
pub const MAIL_REPLAY_TARGET_MODULE_ID_V1: &str = "hermes-mail-runtime";
pub const MAIL_REPLAY_CAPABILITY_ID_V1: &str = "mail.retained-evidence-replay.v1";
pub const MAIL_REPLAY_COMMAND_CONTRACT_NAME_V1: &str = "mail_retained_evidence_replay_command";
pub const MAIL_REPLAY_RESULT_CONTRACT_NAME_V1: &str = "mail_retained_evidence_replay_result";
pub const MAIL_REPLAY_CONTRACT_MAJOR_V1: u32 = 1;
pub const MAIL_REPLAY_CONTRACT_REVISION_V1: u32 = 1;
pub const MAIL_REPLAY_MAX_IN_FLIGHT_V1: u32 = 8;

#[must_use]
pub fn mail_replay_command_contract_reference_v1() -> ContractReferenceV1 {
    contract(MAIL_REPLAY_COMMAND_CONTRACT_NAME_V1)
}

#[must_use]
pub fn mail_replay_result_contract_reference_v1() -> ContractReferenceV1 {
    contract(MAIL_REPLAY_RESULT_CONTRACT_NAME_V1)
}

#[must_use]
pub fn mail_replay_command_publish_request_v1() -> CapabilityRequestV1 {
    route(
        DurableEnvelopeKindV1::Command,
        mail_replay_command_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn mail_replay_command_consume_request_v1() -> CapabilityRequestV1 {
    route(
        DurableEnvelopeKindV1::Command,
        mail_replay_command_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn mail_replay_result_publish_request_v1() -> CapabilityRequestV1 {
    route(
        DurableEnvelopeKindV1::Result,
        mail_replay_result_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn mail_replay_result_consume_request_v1() -> CapabilityRequestV1 {
    route(
        DurableEnvelopeKindV1::Result,
        mail_replay_result_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

fn contract(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: MAIL_REPLAY_OWNER_ID_V1.to_owned(),
        name: name.to_owned(),
        major: MAIL_REPLAY_CONTRACT_MAJOR_V1,
        revision: MAIL_REPLAY_CONTRACT_REVISION_V1,
        schema_sha256: MAIL_REPLAY_SCHEMA_SHA256_V1.to_vec(),
    }
}

fn route(
    kind: DurableEnvelopeKindV1,
    contract: ContractReferenceV1,
    direction: EventRouteDirectionV1,
    requirement: EventSubscriptionRequirementV1,
) -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: kind as i32,
            contract: Some(contract),
            direction: direction as i32,
            max_in_flight: MAIL_REPLAY_MAX_IN_FLIGHT_V1,
            subscription_requirement: requirement as i32,
            max_deliver: u32::from(direction == EventRouteDirectionV1::Consume) * 10,
            ack_wait_millis: u32::from(direction == EventRouteDirectionV1::Consume) * 30_000,
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_and_result_are_exact_mail_routes() {
        let Some(Request::EventRoute(command)) = mail_replay_command_consume_request_v1().request
        else {
            panic!("command route");
        };
        let Some(Request::EventRoute(result)) = mail_replay_result_publish_request_v1().request
        else {
            panic!("result route");
        };
        assert_eq!(command.envelope_kind, DurableEnvelopeKindV1::Command as i32);
        assert_eq!(result.envelope_kind, DurableEnvelopeKindV1::Result as i32);
        assert_eq!(command.contract.expect("contract").owner, "mail");
        assert_eq!(result.contract.expect("contract").owner, "mail");
    }
}
