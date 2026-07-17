#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalRuntimeAttestation {
    registration_id: String,
    runtime_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
    distribution_sha256: [u8; 32],
}

impl ExternalRuntimeAttestation {
    #[must_use]
    pub fn new(
        registration_id: impl Into<String>,
        runtime_id: impl Into<String>,
        runtime_generation: u64,
        grant_epoch: u64,
        distribution_sha256: [u8; 32],
    ) -> Self {
        Self {
            registration_id: registration_id.into(),
            runtime_id: runtime_id.into(),
            runtime_generation,
            grant_epoch,
            distribution_sha256,
        }
    }
    #[must_use]
    pub fn registration_id(&self) -> &str {
        &self.registration_id
    }
    #[must_use]
    pub fn runtime_id(&self) -> &str {
        &self.runtime_id
    }
    #[must_use]
    pub fn runtime_generation(&self) -> u64 {
        self.runtime_generation
    }
    #[must_use]
    pub fn grant_epoch(&self) -> u64 {
        self.grant_epoch
    }
    #[must_use]
    pub fn distribution_sha256(&self) -> &[u8; 32] {
        &self.distribution_sha256
    }
}
