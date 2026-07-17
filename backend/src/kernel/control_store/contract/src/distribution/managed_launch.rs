//! Durable authority records for a managed child from a signed bundled release.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundledManagedLaunchBinding {
    registration_id: String,
    binding_revision: u64,
    distribution_id: String,
    artifact_id: String,
    executable_sha256: [u8; 32],
    descriptor_sha256: [u8; 32],
    settings_schema_sha256: Option<[u8; 32]>,
}

impl BundledManagedLaunchBinding {
    #[must_use]
    pub fn new(
        registration_id: impl Into<String>,
        binding_revision: u64,
        distribution_id: impl Into<String>,
        artifact_id: impl Into<String>,
        executable_sha256: [u8; 32],
        descriptor_sha256: [u8; 32],
        settings_schema_sha256: Option<[u8; 32]>,
    ) -> Self {
        Self {
            registration_id: registration_id.into(),
            binding_revision,
            distribution_id: distribution_id.into(),
            artifact_id: artifact_id.into(),
            executable_sha256,
            descriptor_sha256,
            settings_schema_sha256,
        }
    }

    #[must_use]
    pub fn registration_id(&self) -> &str {
        &self.registration_id
    }
    #[must_use]
    pub fn binding_revision(&self) -> u64 {
        self.binding_revision
    }
    #[must_use]
    pub fn distribution_id(&self) -> &str {
        &self.distribution_id
    }
    #[must_use]
    pub fn artifact_id(&self) -> &str {
        &self.artifact_id
    }
    #[must_use]
    pub fn executable_sha256(&self) -> &[u8; 32] {
        &self.executable_sha256
    }
    #[must_use]
    pub fn descriptor_sha256(&self) -> &[u8; 32] {
        &self.descriptor_sha256
    }
    #[must_use]
    pub fn settings_schema_sha256(&self) -> Option<&[u8; 32]> {
        self.settings_schema_sha256.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedLaunchRecord {
    registration_id: String,
    binding_revision: u64,
    kernel_generation: u64,
    runtime_generation: u64,
    grant_epoch: u64,
}

impl ManagedLaunchRecord {
    #[must_use]
    pub fn new(
        registration_id: impl Into<String>,
        binding_revision: u64,
        kernel_generation: u64,
        runtime_generation: u64,
        grant_epoch: u64,
    ) -> Self {
        Self {
            registration_id: registration_id.into(),
            binding_revision,
            kernel_generation,
            runtime_generation,
            grant_epoch,
        }
    }

    #[must_use]
    pub fn registration_id(&self) -> &str {
        &self.registration_id
    }
    #[must_use]
    pub fn binding_revision(&self) -> u64 {
        self.binding_revision
    }
    #[must_use]
    pub fn kernel_generation(&self) -> u64 {
        self.kernel_generation
    }
    #[must_use]
    pub fn runtime_generation(&self) -> u64 {
        self.runtime_generation
    }
    #[must_use]
    pub fn grant_epoch(&self) -> u64 {
        self.grant_epoch
    }
}
