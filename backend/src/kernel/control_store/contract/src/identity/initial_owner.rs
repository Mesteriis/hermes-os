#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitialOwnerIdentity {
    owner_id: String,
    device_id: String,
    public_key_sec1: [u8; 65],
}

impl InitialOwnerIdentity {
    #[must_use]
    pub fn new(
        owner_id: impl Into<String>,
        device_id: impl Into<String>,
        public_key_sec1: [u8; 65],
    ) -> Self {
        Self {
            owner_id: owner_id.into(),
            device_id: device_id.into(),
            public_key_sec1,
        }
    }

    #[must_use]
    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }

    #[must_use]
    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    #[must_use]
    pub fn public_key_sec1(&self) -> &[u8; 65] {
        &self.public_key_sec1
    }
}
