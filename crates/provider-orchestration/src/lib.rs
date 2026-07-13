//! Provider-neutral application workflows.
//!
//! This crate turns provider contracts into domain-port calls. It intentionally
//! contains no provider client, storage implementation, vault implementation,
//! scheduler, or runtime process management.

use hermes_communications_api::evidence::{
    CommunicationEvidencePort, CommunicationEvidencePortError, NewRawCommunicationRecord,
    StoredRawCommunicationRecord,
};
use hermes_provider_api::ProviderObservationEnvelope;
use thiserror::Error;

/// Adapts a provider-neutral observation into Communications canonical raw evidence.
pub fn observation_to_raw_communication_record(
    observation: ProviderObservationEnvelope,
) -> NewRawCommunicationRecord {
    NewRawCommunicationRecord::new(
        observation.observation_id,
        observation.account_id,
        observation.record_kind,
        observation.provider_record_id,
        observation.source_fingerprint,
        observation.import_batch_id,
        observation.payload,
    )
    .occurred_at(observation.occurred_at)
    .provenance(observation.provenance)
}

/// Persists a provider observation through the Communications evidence boundary.
pub async fn record_provider_observation(
    evidence: &dyn CommunicationEvidencePort,
    observation: ProviderObservationEnvelope,
) -> Result<StoredRawCommunicationRecord, ProviderObservationOrchestrationError> {
    let record = observation_to_raw_communication_record(observation);
    evidence
        .record_raw_source(&record)
        .await
        .map_err(ProviderObservationOrchestrationError::Evidence)
}

#[derive(Debug, Error)]
pub enum ProviderObservationOrchestrationError {
    #[error("provider observation evidence persistence failed: {0}")]
    Evidence(CommunicationEvidencePortError),
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use hermes_provider_api::{ProviderId, ProviderObservationEnvelope, ProviderObservationInput};
    use serde_json::json;

    use super::observation_to_raw_communication_record;

    #[test]
    fn observation_conversion_preserves_canonical_evidence_identity_and_provenance() {
        let observation = ProviderObservationEnvelope::try_from(ProviderObservationInput::new(
            "obs-1",
            ProviderId::parse("zulip").expect("provider id"),
            "account-1",
            "message",
            "message-1",
            "sha256:record",
            "batch-1",
            Utc.timestamp_opt(100, 0).single().expect("timestamp"),
            Utc.timestamp_opt(90, 0).single().expect("timestamp"),
            "queue:1",
            json!({"message": "metadata only"}),
            json!({"provider": "zulip"}),
        ))
        .expect("observation");

        let record = observation_to_raw_communication_record(observation);
        assert_eq!(record.raw_record_id, "obs-1");
        assert_eq!(record.account_id, "account-1");
        assert_eq!(record.occurred_at, Utc.timestamp_opt(90, 0).single());
        assert_eq!(record.provenance, json!({"provider": "zulip"}));
    }
}
