use chrono::{DateTime, TimeDelta, Utc};

use crate::vault::{HostVault, HostVaultError, VaultMode};

use super::errors::MailSyncError;
use super::models::MailSyncSettings;
use super::{MAX_BATCH_SIZE, MAX_POLL_INTERVAL_SECONDS, MIN_POLL_INTERVAL_SECONDS};

pub(super) fn require_unlocked_vault(vault: &HostVault) -> Result<(), HostVaultError> {
    match vault.status()?.state {
        VaultMode::Unlocked => Ok(()),
        VaultMode::Locked => Err(HostVaultError::Locked),
        VaultMode::Uninitialized => Err(HostVaultError::Uninitialized),
    }
}

pub(super) fn validate_account_id(account_id: &str) -> Result<(), MailSyncError> {
    if account_id.trim().is_empty() {
        return Err(MailSyncError::InvalidSetting {
            field: "account_id",
            message: "must not be empty",
        });
    }
    Ok(())
}

pub(super) fn validate_settings(
    batch_size: i32,
    poll_interval_seconds: i32,
    failure_threshold: i32,
) -> Result<(), MailSyncError> {
    if !(1..=MAX_BATCH_SIZE).contains(&batch_size) {
        return Err(MailSyncError::InvalidSetting {
            field: "batch_size",
            message: "must be between 1 and 500",
        });
    }
    if !(MIN_POLL_INTERVAL_SECONDS..=MAX_POLL_INTERVAL_SECONDS).contains(&poll_interval_seconds) {
        return Err(MailSyncError::InvalidSetting {
            field: "poll_interval_seconds",
            message: "must be between 60 and 86400",
        });
    }
    if !(1..=10).contains(&failure_threshold) {
        return Err(MailSyncError::InvalidSetting {
            field: "failure_threshold",
            message: "must be between 1 and 10",
        });
    }
    Ok(())
}

pub(super) fn next_run_at(settings: &MailSyncSettings) -> Option<DateTime<Utc>> {
    if settings.sync_enabled {
        Some(Utc::now() + TimeDelta::seconds(i64::from(settings.poll_interval_seconds)))
    } else {
        None
    }
}

pub(super) fn mail_sync_run_id(account_id: &str) -> String {
    format!(
        "mail-sync-run:v1:{}:{}",
        account_id.trim(),
        Utc::now().timestamp_micros()
    )
}
