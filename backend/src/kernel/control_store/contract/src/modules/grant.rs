#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrantSet {
    registration_id: String,
    grant_epoch: u64,
    capability_ids: Vec<String>,
}

impl GrantSet {
    #[must_use]
    pub fn new(
        registration_id: impl Into<String>,
        grant_epoch: u64,
        capability_ids: Vec<String>,
    ) -> Self {
        Self {
            registration_id: registration_id.into(),
            grant_epoch,
            capability_ids,
        }
    }

    #[must_use]
    pub fn registration_id(&self) -> &str {
        &self.registration_id
    }
    #[must_use]
    pub fn grant_epoch(&self) -> u64 {
        self.grant_epoch
    }
    #[must_use]
    pub fn capability_ids(&self) -> &[String] {
        &self.capability_ids
    }
}
