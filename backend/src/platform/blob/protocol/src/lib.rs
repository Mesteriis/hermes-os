//! Opaque Blob references and bounded range contracts.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobBackupClassV1 {
    Required,
    Rebuildable,
    Excluded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobRefV1 {
    reference_id: [u8; 16],
    owner_id: String,
    declared_size: u64,
    expires_at_unix_ms: Option<u64>,
    backup_class: BlobBackupClassV1,
}

impl BlobRefV1 {
    pub fn new(
        reference_id: [u8; 16],
        owner_id: impl Into<String>,
        declared_size: u64,
        expires_at_unix_ms: Option<u64>,
        backup_class: BlobBackupClassV1,
    ) -> Result<Self, BlobContractError> {
        let owner_id = owner_id.into();
        (reference_id.iter().any(|byte| *byte != 0)
            && valid_owner_id(&owner_id)
            && declared_size > 0
            && expires_at_unix_ms.is_none_or(|value| value > 0))
        .then_some(Self {
            reference_id,
            owner_id,
            declared_size,
            expires_at_unix_ms,
            backup_class,
        })
        .ok_or(BlobContractError::InvalidReference)
    }

    #[must_use]
    pub const fn reference_id(&self) -> &[u8; 16] {
        &self.reference_id
    }
    #[must_use]
    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }
    #[must_use]
    pub const fn declared_size(&self) -> u64 {
        self.declared_size
    }
    #[must_use]
    pub const fn expires_at_unix_ms(&self) -> Option<u64> {
        self.expires_at_unix_ms
    }
    #[must_use]
    pub const fn backup_class(&self) -> BlobBackupClassV1 {
        self.backup_class
    }

    #[must_use]
    pub const fn is_expired_at(&self, now_unix_ms: u64) -> bool {
        match self.expires_at_unix_ms {
            Some(expires_at) => expires_at <= now_unix_ms,
            None => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobRangeV1 {
    start: u64,
    end_exclusive: u64,
}

/// Runtime and grant identity that scopes one Blob operation.
///
/// The key material that permits the operation is deliberately not part of
/// this contract. Blob receives it only through the Vault route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobAccessFenceV1 {
    owner_id: String,
    module_registration_id: String,
    capability_id: String,
    runtime_instance_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
}

impl BlobAccessFenceV1 {
    pub fn new(
        owner_id: impl Into<String>,
        module_registration_id: impl Into<String>,
        capability_id: impl Into<String>,
        runtime_instance_id: impl Into<String>,
        runtime_generation: u64,
        grant_epoch: u64,
    ) -> Result<Self, BlobContractError> {
        let owner_id = owner_id.into();
        let module_registration_id = module_registration_id.into();
        let capability_id = capability_id.into();
        let runtime_instance_id = runtime_instance_id.into();
        (valid_owner_id(&owner_id)
            && valid_opaque_id(&module_registration_id)
            && valid_opaque_id(&capability_id)
            && valid_opaque_id(&runtime_instance_id)
            && runtime_generation > 0
            && grant_epoch > 0)
            .then_some(Self {
                owner_id,
                module_registration_id,
                capability_id,
                runtime_instance_id,
                runtime_generation,
                grant_epoch,
            })
            .ok_or(BlobContractError::InvalidFence)
    }

    #[must_use]
    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }
    #[must_use]
    pub fn module_registration_id(&self) -> &str {
        &self.module_registration_id
    }
    #[must_use]
    pub fn capability_id(&self) -> &str {
        &self.capability_id
    }
    #[must_use]
    pub fn runtime_instance_id(&self) -> &str {
        &self.runtime_instance_id
    }
    #[must_use]
    pub const fn runtime_generation(&self) -> u64 {
        self.runtime_generation
    }
    #[must_use]
    pub const fn grant_epoch(&self) -> u64 {
        self.grant_epoch
    }
}

/// Kernel-authorized aggregate byte budget for one Blob capability grant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobQuotaGrantV1 {
    owner_id: String,
    module_registration_id: String,
    capability_id: String,
    grant_epoch: u64,
    max_bytes: u64,
}

impl BlobQuotaGrantV1 {
    pub fn new(
        owner_id: impl Into<String>,
        module_registration_id: impl Into<String>,
        capability_id: impl Into<String>,
        grant_epoch: u64,
        max_bytes: u64,
    ) -> Result<Self, BlobContractError> {
        let owner_id = owner_id.into();
        let module_registration_id = module_registration_id.into();
        let capability_id = capability_id.into();
        (valid_owner_id(&owner_id)
            && valid_opaque_id(&module_registration_id)
            && valid_opaque_id(&capability_id)
            && grant_epoch > 0
            && (1..=MAX_QUOTA_BYTES).contains(&max_bytes))
        .then_some(Self {
            owner_id,
            module_registration_id,
            capability_id,
            grant_epoch,
            max_bytes,
        })
        .ok_or(BlobContractError::InvalidQuota)
    }

    #[must_use]
    pub fn matches(&self, fence: &BlobAccessFenceV1) -> bool {
        self.owner_id == fence.owner_id
            && self.module_registration_id == fence.module_registration_id
            && self.capability_id == fence.capability_id
            && self.grant_epoch == fence.grant_epoch
    }

    #[must_use]
    pub const fn max_bytes(&self) -> u64 {
        self.max_bytes
    }
}

impl BlobRangeV1 {
    pub fn new(
        start: u64,
        end_exclusive: u64,
        declared_size: u64,
    ) -> Result<Self, BlobContractError> {
        (start < end_exclusive && end_exclusive <= declared_size)
            .then_some(Self {
                start,
                end_exclusive,
            })
            .ok_or(BlobContractError::InvalidRange)
    }

    #[must_use]
    pub const fn start(&self) -> u64 {
        self.start
    }
    #[must_use]
    pub const fn end_exclusive(&self) -> u64 {
        self.end_exclusive
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobContractError {
    InvalidReference,
    InvalidRange,
    InvalidFence,
    InvalidQuota,
}

const MAX_QUOTA_BYTES: u64 = 1 << 40;

fn valid_owner_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}

fn valid_opaque_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}
