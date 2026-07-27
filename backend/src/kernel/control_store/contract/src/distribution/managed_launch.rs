//! Durable authority records for a managed child from a signed bundled release.

use crate::{ModuleRegistration, OperationIdV1};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundledManagedArtifactProposalInputV1 {
    operation_id: OperationIdV1,
    request_digest: [u8; 32],
    distribution_id: String,
    distribution_generation: u64,
    artifact_id: String,
}

impl BundledManagedArtifactProposalInputV1 {
    #[must_use]
    pub fn new(
        operation_id: OperationIdV1,
        request_digest: [u8; 32],
        distribution_id: impl Into<String>,
        distribution_generation: u64,
        artifact_id: impl Into<String>,
    ) -> Self {
        Self {
            operation_id,
            request_digest,
            distribution_id: distribution_id.into(),
            distribution_generation,
            artifact_id: artifact_id.into(),
        }
    }

    #[must_use]
    pub const fn operation_id(&self) -> OperationIdV1 {
        self.operation_id
    }
    #[must_use]
    pub const fn request_digest(&self) -> &[u8; 32] {
        &self.request_digest
    }
    #[must_use]
    pub fn distribution_id(&self) -> &str {
        &self.distribution_id
    }
    #[must_use]
    pub const fn distribution_generation(&self) -> u64 {
        self.distribution_generation
    }
    #[must_use]
    pub fn artifact_id(&self) -> &str {
        &self.artifact_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundledManagedArtifactProposalReceiptV1 {
    registration: ModuleRegistration,
    replayed: bool,
}

impl BundledManagedArtifactProposalReceiptV1 {
    #[must_use]
    pub const fn new(registration: ModuleRegistration, replayed: bool) -> Self {
        Self {
            registration,
            replayed,
        }
    }

    #[must_use]
    pub const fn registration(&self) -> &ModuleRegistration {
        &self.registration
    }
    #[must_use]
    pub const fn replayed(&self) -> bool {
        self.replayed
    }
}

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
    runtime_instance_id: String,
    binding_revision: u64,
    kernel_generation: u64,
    runtime_generation: u64,
    grant_epoch: u64,
}

impl ManagedLaunchRecord {
    #[must_use]
    pub fn new(
        registration_id: impl Into<String>,
        runtime_instance_id: impl Into<String>,
        binding_revision: u64,
        kernel_generation: u64,
        runtime_generation: u64,
        grant_epoch: u64,
    ) -> Self {
        Self {
            registration_id: registration_id.into(),
            runtime_instance_id: runtime_instance_id.into(),
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
    pub fn runtime_instance_id(&self) -> &str {
        &self.runtime_instance_id
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
