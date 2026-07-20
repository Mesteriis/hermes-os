//! Exact Blob quota request retained from one registered module descriptor.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleBlobQuotaRequestV1 {
    registration_id: String,
    capability_id: String,
    owner_id: String,
    max_bytes: u64,
}

impl ModuleBlobQuotaRequestV1 {
    #[must_use]
    pub fn new(
        registration_id: impl Into<String>,
        capability_id: impl Into<String>,
        owner_id: impl Into<String>,
        max_bytes: u64,
    ) -> Self {
        Self {
            registration_id: registration_id.into(),
            capability_id: capability_id.into(),
            owner_id: owner_id.into(),
            max_bytes,
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
    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }

    #[must_use]
    pub const fn max_bytes(&self) -> u64 {
        self.max_bytes
    }
}
