//! Public typed contracts for scoped Vault operations.

mod leases;
mod operations;
mod transport;

pub use leases::{CredentialLeaseV1, LeaseAudienceV1, LeaseIdV1, VaultLeaseIssueRequestV1};
pub use operations::{VaultTransportCommandError, VaultTransportCommandV1};
pub use transport::{
    VaultCiphertextFrameV1, VaultTransportBindingV1, VaultTransportDirectionV1,
    VaultTransportError, VaultTransportPublicKey, VaultTransportSessionV1, seal,
};

pub const MAX_PURPOSE_ID_BYTES: usize = 128;
pub const MAX_CONFIGURATION_INSTANCE_ID_BYTES: usize = 128;
pub const MAX_LOGICAL_OWNER_ID_BYTES: usize = 128;
pub const MAX_LEASE_TTL_SECONDS: u32 = 3_600;
pub const DEFAULT_LEASE_TTL_SECONDS: u32 = 600;
pub const MAX_CREDENTIAL_BYTES: usize = 65_536;
pub const MAX_SESSION_CREDENTIAL_BYTES: usize = 4 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SecretClassV1 {
    ProviderCredential,
    OAuthRefreshCredential,
    SessionCredentialBlob,
    PlatformCredential,
    SessionStoreKey,
}

impl SecretClassV1 {
    #[must_use]
    pub const fn code(self) -> i64 {
        match self {
            Self::ProviderCredential => 1,
            Self::OAuthRefreshCredential => 2,
            Self::SessionCredentialBlob => 3,
            Self::PlatformCredential => 4,
            Self::SessionStoreKey => 5,
        }
    }

    pub const fn from_code(value: i64) -> Option<Self> {
        match value {
            1 => Some(Self::ProviderCredential),
            2 => Some(Self::OAuthRefreshCredential),
            3 => Some(Self::SessionCredentialBlob),
            4 => Some(Self::PlatformCredential),
            5 => Some(Self::SessionStoreKey),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum VaultActionV1 {
    Resolve,
    Create,
    ReplaceCas,
    Retire,
    Delete,
    IssueSessionStoreKey,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaultPurposeRequestV1 {
    purpose_id: String,
    configuration_instance_id: String,
    allowed_secret_classes: Vec<SecretClassV1>,
    actions: Vec<VaultActionV1>,
    requested_lease_ttl_seconds: u32,
}

impl VaultPurposeRequestV1 {
    pub fn new(
        purpose_id: String,
        configuration_instance_id: String,
        allowed_secret_classes: Vec<SecretClassV1>,
        actions: Vec<VaultActionV1>,
        requested_lease_ttl_seconds: u32,
    ) -> Result<Self, VaultProtocolError> {
        validate_identifier(
            &purpose_id,
            MAX_PURPOSE_ID_BYTES,
            VaultProtocolError::InvalidPurpose,
        )?;
        validate_identifier(
            &configuration_instance_id,
            MAX_CONFIGURATION_INSTANCE_ID_BYTES,
            VaultProtocolError::InvalidConfigurationInstance,
        )?;
        validate_secret_classes(&allowed_secret_classes)?;
        validate_actions(&actions)?;
        if !(1..=MAX_LEASE_TTL_SECONDS).contains(&requested_lease_ttl_seconds) {
            return Err(VaultProtocolError::InvalidLeaseTtl);
        }
        Ok(Self {
            purpose_id,
            configuration_instance_id,
            allowed_secret_classes,
            actions,
            requested_lease_ttl_seconds,
        })
    }

    #[must_use]
    pub fn purpose_id(&self) -> &str {
        &self.purpose_id
    }

    #[must_use]
    pub fn configuration_instance_id(&self) -> &str {
        &self.configuration_instance_id
    }

    #[must_use]
    pub fn allowed_secret_classes(&self) -> &[SecretClassV1] {
        &self.allowed_secret_classes
    }

    #[must_use]
    pub fn actions(&self) -> &[VaultActionV1] {
        &self.actions
    }

    #[must_use]
    pub fn requested_lease_ttl_seconds(&self) -> u32 {
        self.requested_lease_ttl_seconds
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VaultProtocolError {
    InvalidLogicalOwner,
    InvalidRuntimeIdentifier,
    InvalidPurpose,
    InvalidConfigurationInstance,
    InvalidSecretClasses,
    InvalidActions,
    InvalidLeaseTtl,
    InvalidLeaseId,
    InvalidGrantEpoch,
    InvalidRuntimeGeneration,
    InvalidSecretRevision,
}

pub fn validate_logical_owner_id(value: &str) -> Result<(), VaultProtocolError> {
    validate_identifier(
        value,
        MAX_LOGICAL_OWNER_ID_BYTES,
        VaultProtocolError::InvalidLogicalOwner,
    )
}

pub fn validate_vault_instance_id(value: &str) -> Result<(), VaultProtocolError> {
    validate_runtime_identifier(value)
}

pub(crate) fn validate_runtime_identifier(value: &str) -> Result<(), VaultProtocolError> {
    validate_identifier(
        value,
        MAX_LOGICAL_OWNER_ID_BYTES,
        VaultProtocolError::InvalidRuntimeIdentifier,
    )
}

fn validate_identifier(
    value: &str,
    maximum: usize,
    error: VaultProtocolError,
) -> Result<(), VaultProtocolError> {
    if value.is_empty() || value.len() > maximum || !value.bytes().all(is_identifier_byte) {
        return Err(error);
    }
    Ok(())
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':')
}

fn validate_secret_classes(values: &[SecretClassV1]) -> Result<(), VaultProtocolError> {
    if values.is_empty() || values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(VaultProtocolError::InvalidSecretClasses);
    }
    Ok(())
}

fn validate_actions(values: &[VaultActionV1]) -> Result<(), VaultProtocolError> {
    if values.is_empty() || values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(VaultProtocolError::InvalidActions);
    }
    Ok(())
}
