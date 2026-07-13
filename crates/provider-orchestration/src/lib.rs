//! Provider-neutral application workflows.
//!
//! This crate turns provider contracts into domain-port calls. It intentionally
//! contains no provider client, storage implementation, vault implementation,
//! scheduler, or runtime process management.

use chrono::{DateTime, Utc};
use hermes_communications_api::commands::{
    CommunicationProviderCommand, ProviderCommandQueuePort, ProviderCommandQueuePortError,
};
use hermes_communications_api::evidence::{
    CommunicationEvidencePort, CommunicationEvidencePortError, NewRawCommunicationRecord,
    StoredRawCommunicationRecord,
};
use hermes_provider_api::ProviderObservationEnvelope;
use serde_json::Value;
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

pub async fn reconcile_provider_command_observation(
    command_queue: &dyn ProviderCommandQueuePort,
    account_id: &str,
    channel_kind: &str,
    provider_message_id: &str,
    command_kinds: &[&str],
    observed_at: DateTime<Utc>,
    provider_state: Value,
) -> Result<Vec<CommunicationProviderCommand>, ProviderCommandObservationReconciliationError> {
    command_queue
        .mark_observed_by_provider_message(
            account_id,
            channel_kind,
            provider_message_id,
            command_kinds,
            observed_at,
            provider_state,
        )
        .await
        .map_err(ProviderCommandObservationReconciliationError::CommandQueue)
}

#[derive(Debug, Error)]
pub enum ProviderCommandObservationReconciliationError {
    #[error("provider command observation reconciliation failed: {0}")]
    CommandQueue(ProviderCommandQueuePortError),
}
