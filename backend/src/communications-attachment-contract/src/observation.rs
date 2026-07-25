use hermes_events_protocol::{
    delivery::{OutboxRecordError, OutboxRecordV1},
    v1::{
        ActorKindV1, ActorRefV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1,
        ObservationMetadataV1, SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::validate_envelope_v1,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use crate::{
    COMMUNICATION_ATTACHMENT_BLOB_ADMISSION_OBSERVATION_SCHEMA_SHA256,
    blob_admission_v1::AttachmentBlobAdmissionObservationV1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentObservationEnvelopeContextV1 {
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub module_id: String,
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentBlobExpectedStateV1 {
    DescriptorOnly,
    BlobPending,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentBlobAdmissionTransitionV1 {
    Requested,
    Admitted,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentBlobAdmissionFactV1 {
    pub attachment_anchor_id: [u8; 16],
    pub source_observation_id: [u8; 16],
    pub correlation_id: [u8; 16],
    pub media_cursor_sha256: [u8; 32],
    pub expected_state: AttachmentBlobExpectedStateV1,
    pub transition: AttachmentBlobAdmissionTransitionV1,
    pub observed_at_unix_seconds: i64,
    pub blob_reference_binding_sha256: Option<[u8; 32]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentObservationEnvelopeBuildErrorV1 {
    InvalidContext,
    InvalidEnvelope,
    OutboxRejected,
}

pub fn build_attachment_blob_admission_outbox_record_v1(
    fact: &AttachmentBlobAdmissionFactV1,
    context: &AttachmentObservationEnvelopeContextV1,
) -> Result<OutboxRecordV1, AttachmentObservationEnvelopeBuildErrorV1> {
    validate_context(context)?;
    if !valid_identifier(&fact.attachment_anchor_id)
        || !valid_identifier(&fact.source_observation_id)
        || !valid_identifier(&fact.correlation_id)
        || !valid_sha256(&fact.media_cursor_sha256)
        || !valid_timestamp(fact.observed_at_unix_seconds, 0)
        || !valid_blob_admission_fact(fact)
    {
        return Err(AttachmentObservationEnvelopeBuildErrorV1::InvalidEnvelope);
    }
    let message_id = attachment_blob_admission_message_id(fact);
    let timestamp = Timestamp {
        seconds: context.recorded_at_unix_seconds,
        nanos: context.recorded_at_nanos,
    };
    let payload = AttachmentBlobAdmissionObservationV1 {
        attachment_anchor_id: fact.attachment_anchor_id.to_vec(),
        expected_state: attachment_blob_expected_state_value(fact.expected_state),
        transition: attachment_blob_transition_value(fact.transition),
        evidence_id: fact.source_observation_id.to_vec(),
        observed_at_unix_seconds: fact.observed_at_unix_seconds,
        blob_reference_binding_sha256: fact
            .blob_reference_binding_sha256
            .map_or_else(Vec::new, |value| value.to_vec()),
    }
    .encode_to_vec();
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: message_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: "communications".to_owned(),
            name: "communication_attachment_blob_admission_observed".to_owned(),
            major: 1,
            revision: 1,
            schema_sha256: COMMUNICATION_ATTACHMENT_BLOB_ADMISSION_OBSERVATION_SCHEMA_SHA256
                .to_vec(),
        }),
        source: Some(SourceRefV1 {
            module_id: context.module_id.clone(),
            runtime_instance_id: runtime_source_reference(&context.runtime_instance_id).to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(timestamp),
        partition_key: fact.attachment_anchor_id.to_vec(),
        causation_message_id: fact.source_observation_id.to_vec(),
        correlation_id: fact.correlation_id.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: context.module_id.as_bytes().to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: context.module_id.as_bytes().to_vec(),
            epoch: context.runtime_generation,
        }),
        semantics: Some(Semantics::Observation(ObservationMetadataV1 {
            observation_id: message_id.to_vec(),
            observed_at: Some(timestamp),
            occurred_at: Some(Timestamp {
                seconds: fact.observed_at_unix_seconds,
                nanos: 0,
            }),
            source_cursor_sha256: fact.media_cursor_sha256.to_vec(),
            source_sequence: None,
        })),
        payload,
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| AttachmentObservationEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn validate_context(
    context: &AttachmentObservationEnvelopeContextV1,
) -> Result<(), AttachmentObservationEnvelopeBuildErrorV1> {
    if context.runtime_generation == 0
        || !valid_module_id(&context.module_id)
        || !valid_runtime_instance_id(&context.runtime_instance_id)
        || !valid_timestamp(context.recorded_at_unix_seconds, context.recorded_at_nanos)
    {
        return Err(AttachmentObservationEnvelopeBuildErrorV1::InvalidContext);
    }
    Ok(())
}

fn valid_blob_admission_fact(fact: &AttachmentBlobAdmissionFactV1) -> bool {
    match (
        fact.expected_state,
        fact.transition,
        fact.blob_reference_binding_sha256,
    ) {
        (
            AttachmentBlobExpectedStateV1::DescriptorOnly,
            AttachmentBlobAdmissionTransitionV1::Requested,
            None,
        )
        | (
            AttachmentBlobExpectedStateV1::DescriptorOnly,
            AttachmentBlobAdmissionTransitionV1::Rejected,
            None,
        )
        | (
            AttachmentBlobExpectedStateV1::BlobPending,
            AttachmentBlobAdmissionTransitionV1::Rejected,
            None,
        ) => true,
        (
            AttachmentBlobExpectedStateV1::BlobPending,
            AttachmentBlobAdmissionTransitionV1::Admitted,
            Some(binding),
        ) => valid_sha256(&binding),
        _ => false,
    }
}

fn attachment_blob_admission_message_id(fact: &AttachmentBlobAdmissionFactV1) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"hermes.communications.attachment-blob-admission-observation.v1\0");
    hasher.update(fact.attachment_anchor_id);
    hasher.update(fact.source_observation_id);
    hasher.update([attachment_blob_expected_state_value(fact.expected_state) as u8]);
    hasher.update([attachment_blob_transition_value(fact.transition) as u8]);
    let digest = hasher.finalize();
    digest[..16].try_into().expect("SHA-256 prefix length")
}

const fn attachment_blob_expected_state_value(value: AttachmentBlobExpectedStateV1) -> i32 {
    match value {
        AttachmentBlobExpectedStateV1::DescriptorOnly => 1,
        AttachmentBlobExpectedStateV1::BlobPending => 2,
    }
}

const fn attachment_blob_transition_value(value: AttachmentBlobAdmissionTransitionV1) -> i32 {
    match value {
        AttachmentBlobAdmissionTransitionV1::Requested => 1,
        AttachmentBlobAdmissionTransitionV1::Admitted => 2,
        AttachmentBlobAdmissionTransitionV1::Rejected => 3,
    }
}

fn runtime_source_reference(runtime_instance_id: &str) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"hermes.runtime.source-reference.v1\0");
    hasher.update(runtime_instance_id.as_bytes());
    let digest: [u8; 32] = hasher.finalize().into();
    digest[..16]
        .try_into()
        .expect("fixed SHA-256 prefix length")
}

fn valid_identifier(value: &[u8; 16]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

fn valid_sha256(value: &[u8; 32]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

fn valid_module_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}

fn valid_runtime_instance_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}

fn valid_timestamp(seconds: i64, nanos: i32) -> bool {
    (-62_135_596_800..=253_402_300_799).contains(&seconds) && (0..1_000_000_000).contains(&nanos)
}

fn outbox_error(_: OutboxRecordError) -> AttachmentObservationEnvelopeBuildErrorV1 {
    AttachmentObservationEnvelopeBuildErrorV1::OutboxRejected
}

#[cfg(test)]
mod tests {
    use super::*;
    use hermes_events_protocol::validation::envelope::decode_envelope_v1;

    #[test]
    fn blob_admission_is_anchor_partitioned_and_schema_bound() {
        let record = build_attachment_blob_admission_outbox_record_v1(
            &AttachmentBlobAdmissionFactV1 {
                attachment_anchor_id: [1; 16],
                source_observation_id: [2; 16],
                correlation_id: [5; 16],
                media_cursor_sha256: [3; 32],
                expected_state: AttachmentBlobExpectedStateV1::BlobPending,
                transition: AttachmentBlobAdmissionTransitionV1::Admitted,
                observed_at_unix_seconds: 1_700_000_000,
                blob_reference_binding_sha256: Some([4; 32]),
            },
            &AttachmentObservationEnvelopeContextV1 {
                runtime_instance_id: "mail-runtime-test".to_owned(),
                runtime_generation: 2,
                module_id: "mail-runtime".to_owned(),
                recorded_at_unix_seconds: 1_700_000_001,
                recorded_at_nanos: 0,
            },
        )
        .expect("blob admission record");
        let envelope = decode_envelope_v1(record.exact_bytes()).expect("envelope");
        let payload = AttachmentBlobAdmissionObservationV1::decode(envelope.payload.as_slice())
            .expect("payload");

        assert_eq!(
            envelope.contract.expect("contract").name,
            "communication_attachment_blob_admission_observed"
        );
        assert_eq!(envelope.partition_key, [1; 16]);
        assert_eq!(envelope.correlation_id, [5; 16]);
        assert_eq!(payload.expected_state, 2);
        assert_eq!(payload.transition, 2);
        assert_eq!(payload.evidence_id, [2; 16]);
    }

    #[test]
    fn rejects_missing_blob_integrity_binding() {
        let error = build_attachment_blob_admission_outbox_record_v1(
            &AttachmentBlobAdmissionFactV1 {
                attachment_anchor_id: [1; 16],
                source_observation_id: [2; 16],
                correlation_id: [5; 16],
                media_cursor_sha256: [3; 32],
                expected_state: AttachmentBlobExpectedStateV1::BlobPending,
                transition: AttachmentBlobAdmissionTransitionV1::Admitted,
                observed_at_unix_seconds: 1_700_000_000,
                blob_reference_binding_sha256: None,
            },
            &AttachmentObservationEnvelopeContextV1 {
                runtime_instance_id: "mail-runtime-test".to_owned(),
                runtime_generation: 2,
                module_id: "mail-runtime".to_owned(),
                recorded_at_unix_seconds: 1_700_000_001,
                recorded_at_nanos: 0,
            },
        )
        .expect_err("missing binding");
        assert_eq!(
            error,
            AttachmentObservationEnvelopeBuildErrorV1::InvalidEnvelope
        );
    }
}
