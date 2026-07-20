//! Exact Scheduler JobKind contract request retained from one module descriptor.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleSchedulerJobRequestV1 {
    registration_id: String,
    capability_id: String,
    owner: String,
    name: String,
    major: u32,
    revision: u32,
    schema_sha256: [u8; 32],
}

impl ModuleSchedulerJobRequestV1 {
    #[must_use]
    pub fn new(
        registration_id: impl Into<String>,
        capability_id: impl Into<String>,
        owner: impl Into<String>,
        name: impl Into<String>,
        major: u32,
        revision: u32,
        schema_sha256: [u8; 32],
    ) -> Self {
        Self {
            registration_id: registration_id.into(),
            capability_id: capability_id.into(),
            owner: owner.into(),
            name: name.into(),
            major,
            revision,
            schema_sha256,
        }
    }

    #[must_use]
    pub fn registration_id(&self) -> &str {
        &self.registration_id
    }

    #[must_use]
    pub fn capability_id(&self) -> &str {
        &self.capability_id
    }

    #[must_use]
    pub fn owner(&self) -> &str {
        &self.owner
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn major(&self) -> u32 {
        self.major
    }

    #[must_use]
    pub const fn revision(&self) -> u32 {
        self.revision
    }

    #[must_use]
    pub const fn schema_sha256(&self) -> &[u8; 32] {
        &self.schema_sha256
    }
}
