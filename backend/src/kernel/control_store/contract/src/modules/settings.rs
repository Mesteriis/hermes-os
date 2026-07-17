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
            (
                Self::PendingValidation,
                Self::PendingApply | Self::BlockedConfig
            ) | (
                Self::PendingApply,
                Self::Applying | Self::AwaitingExternalRestart | Self::BlockedConfig
            ) | (Self::Applying, Self::Current | Self::BlockedConfig)
                | (
                    Self::AwaitingExternalRestart,
                    Self::Current | Self::BlockedConfig
                )
                | (Self::BlockedConfig, Self::PendingValidation)
        )
    }
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

impl SettingsSchemaBinding {
    #[must_use]
    pub fn new(
        registration_id: impl Into<String>,
        schema_major: u32,
        schema_revision: u32,
        schema_sha256: [u8; 32],
        desired_revision: u64,
        effective_revision: u64,
        apply_state: SettingsApplyState,
        sanitized_reason_code: Option<String>,
    ) -> Self {
        Self {
            registration_id: registration_id.into(),
            schema_major,
            schema_revision,
            schema_sha256,
            desired_revision,
            effective_revision,
            apply_state,
            sanitized_reason_code,
        }
    }
    #[must_use]
    pub fn registration_id(&self) -> &str {
        &self.registration_id
    }
    #[must_use]
    pub fn schema_major(&self) -> u32 {
        self.schema_major
    }
    #[must_use]
    pub fn schema_revision(&self) -> u32 {
        self.schema_revision
    }
    #[must_use]
    pub fn schema_sha256(&self) -> &[u8; 32] {
        &self.schema_sha256
    }
    #[must_use]
    pub fn desired_revision(&self) -> u64 {
        self.desired_revision
    }
    #[must_use]
    pub fn effective_revision(&self) -> u64 {
        self.effective_revision
    }
    #[must_use]
    pub fn apply_state(&self) -> SettingsApplyState {
        self.apply_state
    }
    #[must_use]
    pub fn sanitized_reason_code(&self) -> Option<&str> {
        self.sanitized_reason_code.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsDesiredSnapshot {
    pub registration_id: String,
    pub expected_revision: u64,
    pub snapshot_bytes: Vec<u8>,
}
