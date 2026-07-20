//! Durable, broker-neutral resource budgets for the Kernel Event Hub.

use crate::ModuleEventEnvelopeKindV1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformEventStreamBudgetV1 {
    envelope_kind: ModuleEventEnvelopeKindV1,
    max_bytes: u64,
    max_age_millis: u64,
    replicas: u8,
}

impl PlatformEventStreamBudgetV1 {
    #[must_use]
    pub const fn new(
        envelope_kind: ModuleEventEnvelopeKindV1,
        max_bytes: u64,
        max_age_millis: u64,
        replicas: u8,
    ) -> Self {
        Self {
            envelope_kind,
            max_bytes,
            max_age_millis,
            replicas,
        }
    }

    #[must_use]
    pub const fn envelope_kind(self) -> ModuleEventEnvelopeKindV1 {
        self.envelope_kind
    }

    #[must_use]
    pub const fn max_bytes(self) -> u64 {
        self.max_bytes
    }

    #[must_use]
    pub const fn max_age_millis(self) -> u64 {
        self.max_age_millis
    }

    #[must_use]
    pub const fn replicas(self) -> u8 {
        self.replicas
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformEventHubTopologyV1 {
    revision: u64,
    nats_endpoint: String,
    nats_username: String,
    credential_revision: u64,
    stream_budgets: Vec<PlatformEventStreamBudgetV1>,
}

impl PlatformEventHubTopologyV1 {
    #[must_use]
    pub fn new(
        revision: u64,
        nats_endpoint: impl Into<String>,
        nats_username: impl Into<String>,
        credential_revision: u64,
        stream_budgets: Vec<PlatformEventStreamBudgetV1>,
    ) -> Self {
        Self {
            revision,
            nats_endpoint: nats_endpoint.into(),
            nats_username: nats_username.into(),
            credential_revision,
            stream_budgets,
        }
    }

    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    #[must_use]
    pub fn nats_endpoint(&self) -> &str {
        &self.nats_endpoint
    }

    #[must_use]
    pub fn nats_username(&self) -> &str {
        &self.nats_username
    }

    #[must_use]
    pub const fn credential_revision(&self) -> u64 {
        self.credential_revision
    }

    #[must_use]
    pub fn stream_budgets(&self) -> &[PlatformEventStreamBudgetV1] {
        &self.stream_budgets
    }

    #[must_use]
    pub fn budget_for(
        &self,
        envelope_kind: ModuleEventEnvelopeKindV1,
    ) -> Option<PlatformEventStreamBudgetV1> {
        self.stream_budgets
            .iter()
            .copied()
            .find(|budget| budget.envelope_kind() == envelope_kind)
    }
}
