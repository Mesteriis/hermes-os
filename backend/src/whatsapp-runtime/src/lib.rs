//! WhatsApp runtime boundary for typed host observations.
//!
//! This crate owns no browser API, credential material, provider command
//! execution, or Communications persistence. It converts an admitted host
//! observation into an exact provider-neutral Communications outbox record.

pub mod client_port;
pub mod client_transport;
mod communications_outbox;
pub mod managed;

use hermes_communications_ingress::{
    CommunicationObservationDraft, ObservationEnvelopeBuildErrorV1, ObservationEnvelopeContextV1,
    build_observation_outbox_record_v1,
};
use hermes_whatsapp_api::host_bridge::WhatsAppHostBridgeEnvelopeV1;
use hermes_whatsapp_api::{
    WhatsAppProviderCommand, client_wire, provider_command_account_id,
    provider_command_operation_id, validate_provider_command,
};
use hermes_whatsapp_core::{
    WhatsAppCoreError, communication_observation_draft, project_host_observation,
};
use hermes_whatsapp_persistence::{
    WhatsAppClaimedCommandV1, WhatsAppDurablePersistence, WhatsAppDurablePersistenceError,
    WhatsAppHostObservationRecordV1,
};

pub use communications_outbox::{
    WhatsAppCommunicationsOutboxRelayError, relay_communications_outbox_once,
};

pub const PACKAGE: &str = "hermes-whatsapp-runtime";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatsAppRuntimeIdentity {
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
}

#[derive(Clone)]
pub struct WhatsAppRuntimeAdmission {
    pub logical_owner_id: String,
    pub module_registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

#[derive(Debug)]
pub enum WhatsAppHostIngressError {
    Core(WhatsAppCoreError),
    Envelope(ObservationEnvelopeBuildErrorV1),
    Persistence(WhatsAppDurablePersistenceError),
}

#[derive(Debug)]
pub enum WhatsAppCommandQueueError {
    InvalidCommand,
    Persistence(WhatsAppDurablePersistenceError),
    Wire,
}

impl WhatsAppRuntimeIdentity {
    pub fn observation_context(
        &self,
        recorded_at_unix_seconds: i64,
        recorded_at_nanos: i32,
    ) -> ObservationEnvelopeContextV1 {
        ObservationEnvelopeContextV1 {
            runtime_instance_id: self.runtime_instance_id.clone(),
            runtime_generation: self.runtime_generation,
            module_id: "whatsapp-runtime".to_owned(),
            recorded_at_unix_seconds,
            recorded_at_nanos,
        }
    }
}

pub fn draft_host_observation(
    envelope: &WhatsAppHostBridgeEnvelopeV1,
) -> Result<CommunicationObservationDraft, WhatsAppHostIngressError> {
    communication_observation_draft(
        &project_host_observation(envelope).map_err(WhatsAppHostIngressError::Core)?,
    )
    .map_err(WhatsAppHostIngressError::Core)
}

pub async fn accept_host_observation(
    durable: &WhatsAppDurablePersistence,
    identity: &WhatsAppRuntimeIdentity,
    envelope: &WhatsAppHostBridgeEnvelopeV1,
    recorded_at_unix_seconds: i64,
    recorded_at_nanos: i32,
) -> Result<(), WhatsAppHostIngressError> {
    let projection = project_host_observation(envelope).map_err(WhatsAppHostIngressError::Core)?;
    let draft =
        communication_observation_draft(&projection).map_err(WhatsAppHostIngressError::Core)?;
    let record = build_observation_outbox_record_v1(
        &draft,
        &identity.observation_context(recorded_at_unix_seconds, recorded_at_nanos),
    )
    .map_err(WhatsAppHostIngressError::Envelope)?;
    let observation = WhatsAppHostObservationRecordV1 {
        account_id: projection.account_id,
        provider_event_id: projection.provider_event_id,
        evidence_kind: evidence_kind_value(projection.evidence_kind),
        observed_at_unix_seconds: projection.observed_at_unix_seconds,
    };
    match &envelope.observation {
        hermes_whatsapp_api::host_bridge::WhatsAppHostObservationV1::CommandResult {
            operation_id,
            host_claim_id,
            succeeded,
            ..
        } => durable
            .complete_provider_command_and_enqueue_observation(
                hermes_whatsapp_persistence::WhatsAppProviderCommandCompletionV1 {
                    operation_id,
                    account_id: &observation.account_id,
                    host_claim_id,
                    succeeded: *succeeded,
                    observation: &observation,
                    record: &record,
                    completed_at_unix_seconds: recorded_at_unix_seconds,
                },
            )
            .await
            .map_err(WhatsAppHostIngressError::Persistence)
            .and_then(|completed| {
                completed
                    .then_some(())
                    .ok_or(WhatsAppHostIngressError::Persistence(
                        WhatsAppDurablePersistenceError::ObservationConflict,
                    ))
            }),
        _ => durable
            .record_host_observation_and_enqueue(&observation, &record, recorded_at_unix_seconds)
            .await
            .map_err(WhatsAppHostIngressError::Persistence)
            .map(|_| ()),
    }
}

pub async fn enqueue_provider_command(
    durable: &WhatsAppDurablePersistence,
    command: &WhatsAppProviderCommand,
    requested_at_unix_seconds: i64,
) -> Result<bool, WhatsAppCommandQueueError> {
    validate_provider_command(command).map_err(|_| WhatsAppCommandQueueError::InvalidCommand)?;
    durable
        .enqueue_provider_command(
            provider_command_operation_id(command),
            provider_command_account_id(command),
            &client_wire::encode_command(command),
            requested_at_unix_seconds,
        )
        .await
        .map_err(WhatsAppCommandQueueError::Persistence)
}

pub async fn claim_provider_commands(
    durable: &WhatsAppDurablePersistence,
    account_id: &str,
    host_claim_id: &str,
    now_unix_seconds: i64,
    lease_seconds: i64,
    limit: i64,
) -> Result<Vec<WhatsAppProviderCommand>, WhatsAppCommandQueueError> {
    let claimed = durable
        .claim_provider_commands(
            account_id,
            host_claim_id,
            now_unix_seconds,
            lease_seconds,
            limit,
        )
        .await
        .map_err(WhatsAppCommandQueueError::Persistence)?;
    decode_claimed_commands(claimed)
}

fn decode_claimed_commands(
    claimed: Vec<WhatsAppClaimedCommandV1>,
) -> Result<Vec<WhatsAppProviderCommand>, WhatsAppCommandQueueError> {
    claimed
        .into_iter()
        .map(|record| {
            let command = client_wire::decode_command(&record.exact_command_bytes)
                .map_err(|_| WhatsAppCommandQueueError::Wire)?;
            (provider_command_operation_id(&command) == record.operation_id
                && provider_command_account_id(&command) == record.account_id)
                .then_some(command)
                .ok_or(WhatsAppCommandQueueError::Wire)
        })
        .collect()
}

const fn evidence_kind_value(
    value: hermes_communications_ingress::CommunicationEvidenceKindV1,
) -> i16 {
    match value {
        hermes_communications_ingress::CommunicationEvidenceKindV1::EmailMessage => 1,
        hermes_communications_ingress::CommunicationEvidenceKindV1::ChatMessage => 2,
        hermes_communications_ingress::CommunicationEvidenceKindV1::MessageEdited => 3,
        hermes_communications_ingress::CommunicationEvidenceKindV1::MessageDeleted => 4,
        hermes_communications_ingress::CommunicationEvidenceKindV1::ReactionChanged => 5,
        hermes_communications_ingress::CommunicationEvidenceKindV1::DeliveryStateChanged => 6,
        hermes_communications_ingress::CommunicationEvidenceKindV1::ConversationStateChanged => 7,
        hermes_communications_ingress::CommunicationEvidenceKindV1::ParticipantChanged => 8,
        hermes_communications_ingress::CommunicationEvidenceKindV1::MediaChanged => 9,
        hermes_communications_ingress::CommunicationEvidenceKindV1::TopicChanged => 10,
        hermes_communications_ingress::CommunicationEvidenceKindV1::TypingChanged => 11,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hermes_whatsapp_api::host_bridge::{
        HOST_BRIDGE_PROTOCOL_MAJOR, HOST_BRIDGE_PROTOCOL_REVISION, WhatsAppHostObservationV1,
    };

    #[test]
    fn message_identity_becomes_metadata_only_chat_evidence() {
        let draft = draft_host_observation(&WhatsAppHostBridgeEnvelopeV1 {
            protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
            protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
            account_id: "wa-1".to_owned(),
            provider_event_id: "event-1".to_owned(),
            observed_at_unix_seconds: 1_782_504_000,
            observation: WhatsAppHostObservationV1::MessageIdentity {
                provider_chat_id: "chat-1".to_owned(),
                provider_message_id: "message-1".to_owned(),
                sender_id: "sender-1".to_owned(),
            },
        })
        .expect("draft");

        assert_eq!(
            draft.source.provider,
            hermes_communications_ingress::ProviderProvenanceV1::WhatsAppWeb
        );
        assert_eq!(
            draft.kind,
            hermes_communications_ingress::CommunicationEvidenceKindV1::ChatMessage
        );
        assert_eq!(
            draft.body,
            hermes_communications_ingress::BodyAvailabilityV1::MetadataOnly
        );
    }

    #[test]
    fn session_material_is_not_a_communications_observation() {
        let result = draft_host_observation(&WhatsAppHostBridgeEnvelopeV1 {
            protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
            protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
            account_id: "wa-1".to_owned(),
            provider_event_id: "event-1".to_owned(),
            observed_at_unix_seconds: 1_782_504_000,
            observation: WhatsAppHostObservationV1::SessionLinked {
                secret_ref: "secret-ref".to_owned(),
                revision: 1,
            },
        });

        assert!(matches!(
            result,
            Err(WhatsAppHostIngressError::Core(
                WhatsAppCoreError::UnsupportedObservation
            ))
        ));
    }
}
