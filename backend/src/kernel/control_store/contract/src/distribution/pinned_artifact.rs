//! Development-only owner approval record for one local executable artifact.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnerPinnedArtifactBinding {
    registration_id: String,
    binding_revision: u64,
    canonical_artifact_path: String,
    artifact_sha256: [u8; 32],
    artifact_size: u64,
    artifact_device: u64,
    artifact_inode: u64,
    owner_signature_raw: [u8; 64],
}

impl OwnerPinnedArtifactBinding {
    #[must_use]
    pub fn new(
        registration_id: impl Into<String>,
        binding_revision: u64,
        canonical_artifact_path: impl Into<String>,
        artifact_sha256: [u8; 32],
        artifact_size: u64,
        artifact_device: u64,
        artifact_inode: u64,
        owner_signature_raw: [u8; 64],
    ) -> Self {
        Self {
            registration_id: registration_id.into(),
            binding_revision,
            canonical_artifact_path: canonical_artifact_path.into(),
            artifact_sha256,
            artifact_size,
            artifact_device,
            artifact_inode,
            owner_signature_raw,
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
    pub fn canonical_artifact_path(&self) -> &str {
        &self.canonical_artifact_path
    }
    #[must_use]
    pub fn artifact_sha256(&self) -> &[u8; 32] {
        &self.artifact_sha256
    }
    #[must_use]
    pub fn artifact_size(&self) -> u64 {
        self.artifact_size
    }
    #[must_use]
    pub fn artifact_device(&self) -> u64 {
        self.artifact_device
    }
    #[must_use]
    pub fn artifact_inode(&self) -> u64 {
        self.artifact_inode
    }
    #[must_use]
    pub fn owner_signature_raw(&self) -> &[u8; 64] {
        &self.owner_signature_raw
    }
}
