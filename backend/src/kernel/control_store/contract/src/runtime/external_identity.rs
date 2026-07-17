//! Owner-pinned public identity for one external module runtime registration.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalRuntimeIdentity {
    registration_id: String,
    public_key_sec1: [u8; 65],
}

impl ExternalRuntimeIdentity {
    #[must_use]
    pub fn new(registration_id: impl Into<String>, public_key_sec1: [u8; 65]) -> Self {
        Self {
            registration_id: registration_id.into(),
            public_key_sec1,
        }
    }

    #[must_use]
    pub fn registration_id(&self) -> &str {
        &self.registration_id
    }

    #[must_use]
    pub fn public_key_sec1(&self) -> &[u8; 65] {
        &self.public_key_sec1
    }
}
