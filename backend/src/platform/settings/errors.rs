use thiserror::Error;

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("unsupported setting value kind: {0}")]
    UnsupportedValueKind(String),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("setting key has invalid format")]
    InvalidSettingKey,

    #[error("setting key must not refer to secrets or credentials")]
    SecretLikeSettingKey,

    #[error("invalid setting value: {0}")]
    InvalidValue(&'static str),

    #[error("application setting was not found: {setting_key}")]
    SettingNotFound { setting_key: String },

    #[error("application setting is read-only: {setting_key}")]
    ReadOnlySetting { setting_key: String },
}

impl SettingsError {
    pub fn is_invalid_request(&self) -> bool {
        matches!(
            self,
            Self::UnsupportedValueKind(_)
                | Self::EmptyField(_)
                | Self::InvalidSettingKey
                | Self::SecretLikeSettingKey
                | Self::InvalidValue(_)
                | Self::ReadOnlySetting { .. }
        )
    }
}
