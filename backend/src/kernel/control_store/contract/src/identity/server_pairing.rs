//! Typed pairing record persisted by the private Kernel Control Store.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerBootstrapPairing {
    token_sha256: [u8; 32],
    certificate_sha256: [u8; 32],
    challenge: [u8; 32],
    expires_at_unix_ms: u64,
}

impl ServerBootstrapPairing {
    #[must_use]
    pub fn new(
        token_sha256: [u8; 32],
        certificate_sha256: [u8; 32],
        challenge: [u8; 32],
        expires_at_unix_ms: u64,
    ) -> Self {
        Self {
            token_sha256,
            certificate_sha256,
            challenge,
            expires_at_unix_ms,
        }
    }

    #[must_use]
    pub fn token_sha256(&self) -> &[u8; 32] {
        &self.token_sha256
    }

    #[must_use]
    pub fn certificate_sha256(&self) -> &[u8; 32] {
        &self.certificate_sha256
    }

    #[must_use]
    pub fn challenge(&self) -> &[u8; 32] {
        &self.challenge
    }

    #[must_use]
    pub fn expires_at_unix_ms(&self) -> u64 {
        self.expires_at_unix_ms
    }
}
