//! Desired non-secret identity for the managed Events credential authority.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformEventsAuthorityConfigurationV1 {
    revision: u64,
    account_public_key: String,
    signer_credential_revision: u64,
}

impl PlatformEventsAuthorityConfigurationV1 {
    #[must_use]
    pub fn new(
        revision: u64,
        account_public_key: impl Into<String>,
        signer_credential_revision: u64,
    ) -> Self {
        Self {
            revision,
            account_public_key: account_public_key.into(),
            signer_credential_revision,
        }
    }

    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    #[must_use]
    pub fn account_public_key(&self) -> &str {
        &self.account_public_key
    }

    #[must_use]
    pub const fn signer_credential_revision(&self) -> u64 {
        self.signer_credential_revision
    }
}
