//! Typed descriptor-declared Vault purpose bound to one approved capability.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleVaultPurposeRequestV1 {
    registration_id: String,
    capability_id: String,
    purpose_id: String,
    requested_lease_ttl_seconds: u16,
    secret_class: u8,
    action: u8,
    target_scope: u8,
    key_schema_revision: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModuleVaultPurposePolicyV1 {
    pub secret_class: u8,
    pub action: u8,
    pub target_scope: u8,
    pub key_schema_revision: u32,
}

impl ModuleVaultPurposeRequestV1 {
    #[must_use]
    pub fn new(
        registration_id: impl Into<String>,
        capability_id: impl Into<String>,
        purpose_id: impl Into<String>,
        requested_lease_ttl_seconds: u16,
        secret_class: u8,
        action: u8,
        target_scope: u8,
    ) -> Self {
        Self {
            registration_id: registration_id.into(),
            capability_id: capability_id.into(),
            purpose_id: purpose_id.into(),
            requested_lease_ttl_seconds,
            secret_class,
            action,
            target_scope,
            key_schema_revision: 0,
        }
    }

    #[must_use]
    pub fn new_with_key_schema_revision(
        registration_id: impl Into<String>,
        capability_id: impl Into<String>,
        purpose_id: impl Into<String>,
        requested_lease_ttl_seconds: u16,
        policy: ModuleVaultPurposePolicyV1,
    ) -> Self {
        Self {
            registration_id: registration_id.into(),
            capability_id: capability_id.into(),
            purpose_id: purpose_id.into(),
            requested_lease_ttl_seconds,
            secret_class: policy.secret_class,
            action: policy.action,
            target_scope: policy.target_scope,
            key_schema_revision: policy.key_schema_revision,
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
    pub fn purpose_id(&self) -> &str {
        &self.purpose_id
    }
    #[must_use]
    pub const fn requested_lease_ttl_seconds(&self) -> u16 {
        self.requested_lease_ttl_seconds
    }
    #[must_use]
    pub const fn secret_class(&self) -> u8 {
        self.secret_class
    }
    #[must_use]
    pub const fn action(&self) -> u8 {
        self.action
    }
    #[must_use]
    pub const fn target_scope(&self) -> u8 {
        self.target_scope
    }
    #[must_use]
    pub const fn key_schema_revision(&self) -> u32 {
        self.key_schema_revision
    }
}
