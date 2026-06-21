use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::super::errors::TelegramError;
use super::super::validation::validate_non_empty;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramQrLoginStartRequest {
    pub account_id: String,
    pub display_name: String,
    pub external_account_id: String,
    pub api_id: Option<i64>,
    pub api_hash: Option<String>,
    pub session_encryption_key: Option<String>,
    pub tdlib_data_path: Option<String>,
    #[serde(default)]
    pub transcription_enabled: bool,
}

impl TelegramQrLoginStartRequest {
    pub(crate) fn with_app_credentials(
        mut self,
        api_id: Option<i64>,
        api_hash: Option<String>,
    ) -> Self {
        if self.api_id.is_none() {
            self.api_id = api_id;
        }
        if self
            .api_hash
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .is_none()
        {
            self.api_hash = api_hash;
        }
        self
    }

    pub(crate) fn required_api_id(&self) -> Result<i64, TelegramError> {
        let api_id = self
            .api_id
            .ok_or_else(|| TelegramError::InvalidRequest("api_id must not be empty".to_owned()))?;
        if api_id <= 0 {
            return Err(TelegramError::InvalidRequest(
                "api_id must be greater than zero".to_owned(),
            ));
        }
        Ok(api_id)
    }

    pub(crate) fn required_api_hash(&self) -> Result<&str, TelegramError> {
        self.api_hash
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| TelegramError::InvalidRequest("api_hash must not be empty".to_owned()))
    }

    pub(crate) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("external_account_id", &self.external_account_id)?;
        self.required_api_id()?;
        self.required_api_hash()?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramQrLoginPasswordRequest {
    pub password: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramQrLoginStatus {
    WaitingQrScan,
    WaitingPassword,
    Ready,
    Expired,
    Failed,
    RuntimeUnavailable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramQrLoginStatusResponse {
    pub setup_id: String,
    pub account_id: String,
    pub status: TelegramQrLoginStatus,
    pub qr_link: Option<String>,
    pub qr_svg: Option<String>,
    pub telegram_user_id: Option<String>,
    pub telegram_username: Option<String>,
    pub suggested_account_id: Option<String>,
    pub suggested_display_name: Option<String>,
    pub suggested_external_account_id: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub poll_after_ms: u64,
    pub message: Option<String>,
}
