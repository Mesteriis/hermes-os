//! Typed, persistence-agnostic Kernel Control Store boundary.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreHealth {
    Trustworthy,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitialOwnerIdentity {
    owner_id: String,
    device_id: String,
    public_key_sec1: [u8; 65],
}

impl InitialOwnerIdentity {
    #[must_use] pub fn new(owner_id: impl Into<String>, device_id: impl Into<String>, public_key_sec1: [u8; 65]) -> Self { Self { owner_id: owner_id.into(), device_id: device_id.into(), public_key_sec1 } }
    #[must_use] pub fn owner_id(&self) -> &str { &self.owner_id }
    #[must_use] pub fn device_id(&self) -> &str { &self.device_id }
    #[must_use] pub fn public_key_sec1(&self) -> &[u8; 65] { &self.public_key_sec1 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleRegistrationState {
    Pending,
    Approved,
    Suspended,
    Revoked,
    BlockedIncompatible,
}

impl ModuleRegistrationState {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Suspended => "suspended",
            Self::Revoked => "revoked",
            Self::BlockedIncompatible => "blocked_incompatible",
        }
    }

    #[must_use]
    pub fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Pending, Self::Approved | Self::Revoked | Self::BlockedIncompatible)
                | (Self::Approved, Self::Suspended | Self::Revoked | Self::BlockedIncompatible)
                | (Self::Suspended, Self::Approved | Self::Revoked | Self::BlockedIncompatible)
                | (Self::BlockedIncompatible, Self::Revoked)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleRegistration {
    registration_id: String,
    module_id: String,
    owner_id: String,
    descriptor_sha256: [u8; 32],
    state: ModuleRegistrationState,
    grant_epoch: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrantSet {
    registration_id: String,
    grant_epoch: u64,
    capability_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalRuntimeAttestation {
    registration_id: String,
    runtime_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
    distribution_sha256: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsSchemaBinding {
    registration_id: String,
    schema_major: u32,
    schema_revision: u32,
    schema_sha256: [u8; 32],
    desired_revision: u64,
    effective_revision: u64,
    apply_state: SettingsApplyState,
    sanitized_reason_code: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsApplyState {
    Current,
    PendingValidation,
    PendingApply,
    Applying,
    AwaitingExternalRestart,
    BlockedConfig,
}

impl SettingsApplyState {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::PendingValidation => "pending_validation",
            Self::PendingApply => "pending_apply",
            Self::Applying => "applying",
            Self::AwaitingExternalRestart => "awaiting_external_restart",
            Self::BlockedConfig => "blocked_config",
        }
    }

    #[must_use]
    pub fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::PendingValidation, Self::PendingApply | Self::BlockedConfig)
                | (Self::PendingApply, Self::Applying | Self::AwaitingExternalRestart | Self::BlockedConfig)
                | (Self::Applying, Self::Current | Self::BlockedConfig)
                | (Self::AwaitingExternalRestart, Self::Current | Self::BlockedConfig)
                | (Self::BlockedConfig, Self::PendingValidation)
        )
    }
}

impl SettingsSchemaBinding {
    #[must_use] pub fn new(registration_id: impl Into<String>, schema_major: u32, schema_revision: u32, schema_sha256: [u8; 32], desired_revision: u64, effective_revision: u64, apply_state: SettingsApplyState, sanitized_reason_code: Option<String>) -> Self { Self { registration_id: registration_id.into(), schema_major, schema_revision, schema_sha256, desired_revision, effective_revision, apply_state, sanitized_reason_code } }
    #[must_use] pub fn registration_id(&self) -> &str { &self.registration_id }
    #[must_use] pub fn schema_major(&self) -> u32 { self.schema_major }
    #[must_use] pub fn schema_revision(&self) -> u32 { self.schema_revision }
    #[must_use] pub fn schema_sha256(&self) -> &[u8; 32] { &self.schema_sha256 }
    #[must_use] pub fn desired_revision(&self) -> u64 { self.desired_revision }
    #[must_use] pub fn effective_revision(&self) -> u64 { self.effective_revision }
    #[must_use] pub fn apply_state(&self) -> SettingsApplyState { self.apply_state }
    #[must_use] pub fn sanitized_reason_code(&self) -> Option<&str> { self.sanitized_reason_code.as_deref() }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsDesiredSnapshot { pub registration_id: String, pub expected_revision: u64, pub snapshot_bytes: Vec<u8> }

impl ExternalRuntimeAttestation {
    #[must_use]
    pub fn new(
        registration_id: impl Into<String>,
        runtime_id: impl Into<String>,
        runtime_generation: u64,
        grant_epoch: u64,
        distribution_sha256: [u8; 32],
    ) -> Self {
        Self {
            registration_id: registration_id.into(),
            runtime_id: runtime_id.into(),
            runtime_generation,
            grant_epoch,
            distribution_sha256,
        }
    }
    #[must_use] pub fn registration_id(&self) -> &str { &self.registration_id }
    #[must_use] pub fn runtime_id(&self) -> &str { &self.runtime_id }
    #[must_use] pub fn runtime_generation(&self) -> u64 { self.runtime_generation }
    #[must_use] pub fn grant_epoch(&self) -> u64 { self.grant_epoch }
    #[must_use] pub fn distribution_sha256(&self) -> &[u8; 32] { &self.distribution_sha256 }
}

impl GrantSet {
    #[must_use]
    pub fn new(registration_id: impl Into<String>, grant_epoch: u64, capability_ids: Vec<String>) -> Self {
        Self { registration_id: registration_id.into(), grant_epoch, capability_ids }
    }
    #[must_use] pub fn registration_id(&self) -> &str { &self.registration_id }
    #[must_use] pub fn grant_epoch(&self) -> u64 { self.grant_epoch }
    #[must_use] pub fn capability_ids(&self) -> &[String] { &self.capability_ids }
}

impl ModuleRegistration {
    #[must_use]
    pub fn new(registration_id: impl Into<String>, module_id: impl Into<String>, owner_id: impl Into<String>, descriptor_sha256: [u8; 32], state: ModuleRegistrationState, grant_epoch: u64) -> Self {
        Self { registration_id: registration_id.into(), module_id: module_id.into(), owner_id: owner_id.into(), descriptor_sha256, state, grant_epoch }
    }
    #[must_use] pub fn registration_id(&self) -> &str { &self.registration_id }
    #[must_use] pub fn module_id(&self) -> &str { &self.module_id }
    #[must_use] pub fn owner_id(&self) -> &str { &self.owner_id }
    #[must_use] pub fn descriptor_sha256(&self) -> &[u8; 32] { &self.descriptor_sha256 }
    #[must_use] pub fn state(&self) -> ModuleRegistrationState { self.state }
    #[must_use] pub fn grant_epoch(&self) -> u64 { self.grant_epoch }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlStore {
    instance_id: String,
    generation: u64,
    identity_epoch: u64,
    grant_epoch: u64,
    health: StoreHealth,
}

impl ControlStore {
    #[must_use]
    pub fn new(instance_id: impl Into<String>, generation: u64) -> Self {
        Self {
            instance_id: instance_id.into(),
            generation,
            identity_epoch: 1,
            grant_epoch: 1,
            health: StoreHealth::Trustworthy,
        }
    }

    #[must_use]
    pub fn with_recovery_fences(
        instance_id: impl Into<String>,
        generation: u64,
        identity_epoch: u64,
        grant_epoch: u64,
    ) -> Self {
        Self {
            instance_id: instance_id.into(),
            generation,
            identity_epoch,
            grant_epoch,
            health: StoreHealth::Trustworthy,
        }
    }

    #[must_use]
    pub fn health(&self) -> StoreHealth {
        self.health
    }

    #[must_use]
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    #[must_use]
    pub fn generation(&self) -> u64 {
        self.generation
    }

    #[must_use]
    pub fn identity_epoch(&self) -> u64 {
        self.identity_epoch
    }

    #[must_use]
    pub fn grant_epoch(&self) -> u64 {
        self.grant_epoch
    }
}
