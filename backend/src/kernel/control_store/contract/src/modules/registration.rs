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
            (
                Self::Pending,
                Self::Approved | Self::Revoked | Self::BlockedIncompatible
            ) | (
                Self::Approved,
                Self::Suspended | Self::Revoked | Self::BlockedIncompatible
            ) | (
                Self::Suspended,
                Self::Approved | Self::Revoked | Self::BlockedIncompatible
            ) | (Self::BlockedIncompatible, Self::Revoked)
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

impl ModuleRegistration {
    #[must_use]
    pub fn new(
        registration_id: impl Into<String>,
        module_id: impl Into<String>,
        owner_id: impl Into<String>,
        descriptor_sha256: [u8; 32],
        state: ModuleRegistrationState,
        grant_epoch: u64,
    ) -> Self {
        Self {
            registration_id: registration_id.into(),
            module_id: module_id.into(),
            owner_id: owner_id.into(),
            descriptor_sha256,
            state,
            grant_epoch,
        }
    }
    #[must_use]
    pub fn registration_id(&self) -> &str {
        &self.registration_id
    }
    #[must_use]
    pub fn module_id(&self) -> &str {
        &self.module_id
    }
    #[must_use]
    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }
    #[must_use]
    pub fn descriptor_sha256(&self) -> &[u8; 32] {
        &self.descriptor_sha256
    }
    #[must_use]
    pub fn state(&self) -> ModuleRegistrationState {
        self.state
    }
    #[must_use]
    pub fn grant_epoch(&self) -> u64 {
        self.grant_epoch
    }
}
