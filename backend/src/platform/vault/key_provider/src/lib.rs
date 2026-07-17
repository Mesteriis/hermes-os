//! Private Vault port for the platform wrapping-key boundary.

pub struct WrappingKey([u8; 32]);

impl WrappingKey {
    #[must_use]
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

pub trait WrappingKeyProvider {
    type Error;

    fn load_or_create(&self) -> Result<WrappingKey, Self::Error>;
}
