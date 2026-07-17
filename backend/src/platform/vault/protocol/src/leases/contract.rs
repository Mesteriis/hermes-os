//! Typed, secret-free credential lease bindings.

use crate::{
    MAX_LEASE_TTL_SECONDS, VaultProtocolError, VaultPurposeRequestV1, validate_logical_owner_id,
    validate_runtime_identifier,
};

const LEASE_ID_HEX_BYTES: usize = 32;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeaseIdV1(String);

impl LeaseIdV1 {
    pub fn new(value: String) -> Result<Self, VaultProtocolError> {
        if value.len() != LEASE_ID_HEX_BYTES
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
        {
            return Err(VaultProtocolError::InvalidLeaseId);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeaseAudienceV1 {
    module_registration_id: String,
    runtime_instance_id: String,
    grant_epoch: u64,
}

impl LeaseAudienceV1 {
    pub fn new(
        module_registration_id: String,
        runtime_instance_id: String,
        grant_epoch: u64,
    ) -> Result<Self, VaultProtocolError> {
        validate_runtime_identifier(&module_registration_id)?;
        validate_runtime_identifier(&runtime_instance_id)?;
        if grant_epoch == 0 {
            return Err(VaultProtocolError::InvalidGrantEpoch);
        }
        Ok(Self {
            module_registration_id,
            runtime_instance_id,
            grant_epoch,
        })
    }

    #[must_use]
    pub fn module_registration_id(&self) -> &str {
        &self.module_registration_id
    }

    #[must_use]
    pub fn runtime_instance_id(&self) -> &str {
        &self.runtime_instance_id
    }

    #[must_use]
    pub const fn grant_epoch(&self) -> u64 {
        self.grant_epoch
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaultLeaseIssueRequestV1 {
    vault_instance_id: String,
    vault_runtime_generation: u64,
    secret_revision: u64,
    logical_owner_id: String,
    purpose: VaultPurposeRequestV1,
    audience: LeaseAudienceV1,
}

impl VaultLeaseIssueRequestV1 {
    pub fn new(
        vault_instance_id: String,
        vault_runtime_generation: u64,
        secret_revision: u64,
        logical_owner_id: String,
        purpose: VaultPurposeRequestV1,
        audience: LeaseAudienceV1,
    ) -> Result<Self, VaultProtocolError> {
        validate_runtime_identifier(&vault_instance_id)?;
        validate_logical_owner_id(&logical_owner_id)?;
        if vault_runtime_generation == 0 {
            return Err(VaultProtocolError::InvalidRuntimeGeneration);
        }
        if secret_revision == 0 {
            return Err(VaultProtocolError::InvalidSecretRevision);
        }
        Ok(Self {
            vault_instance_id,
            vault_runtime_generation,
            secret_revision,
            logical_owner_id,
            purpose,
            audience,
        })
    }

    #[must_use]
    pub fn vault_instance_id(&self) -> &str {
        &self.vault_instance_id
    }

    #[must_use]
    pub const fn vault_runtime_generation(&self) -> u64 {
        self.vault_runtime_generation
    }

    #[must_use]
    pub const fn secret_revision(&self) -> u64 {
        self.secret_revision
    }

    #[must_use]
    pub fn logical_owner_id(&self) -> &str {
        &self.logical_owner_id
    }

    #[must_use]
    pub fn purpose(&self) -> &VaultPurposeRequestV1 {
        &self.purpose
    }

    #[must_use]
    pub fn audience(&self) -> &LeaseAudienceV1 {
        &self.audience
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CredentialLeaseV1 {
    lease_id: LeaseIdV1,
    request: VaultLeaseIssueRequestV1,
    issued_at_unix_seconds: u64,
    expires_at_unix_seconds: u64,
    single_resolve: bool,
}

impl CredentialLeaseV1 {
    pub fn new(
        lease_id: LeaseIdV1,
        request: VaultLeaseIssueRequestV1,
        issued_at_unix_seconds: u64,
        requested_lease_ttl_seconds: u32,
        single_resolve: bool,
    ) -> Result<Self, VaultProtocolError> {
        if !(1..=MAX_LEASE_TTL_SECONDS).contains(&requested_lease_ttl_seconds) {
            return Err(VaultProtocolError::InvalidLeaseTtl);
        }
        let expires_at_unix_seconds = issued_at_unix_seconds
            .checked_add(u64::from(requested_lease_ttl_seconds))
            .ok_or(VaultProtocolError::InvalidLeaseTtl)?;
        Ok(Self {
            lease_id,
            request,
            issued_at_unix_seconds,
            expires_at_unix_seconds,
            single_resolve,
        })
    }

    #[must_use]
    pub fn lease_id(&self) -> &LeaseIdV1 {
        &self.lease_id
    }

    #[must_use]
    pub fn request(&self) -> &VaultLeaseIssueRequestV1 {
        &self.request
    }

    #[must_use]
    pub const fn issued_at_unix_seconds(&self) -> u64 {
        self.issued_at_unix_seconds
    }

    #[must_use]
    pub const fn expires_at_unix_seconds(&self) -> u64 {
        self.expires_at_unix_seconds
    }

    #[must_use]
    pub const fn single_resolve(&self) -> bool {
        self.single_resolve
    }
}
