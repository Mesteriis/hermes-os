use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::MailSyncError;
use super::models::{MailSyncDueAccount, MailSyncRun, MailSyncSettings, MailSyncStatus};

pub(super) fn row_to_settings(row: PgRow) -> Result<MailSyncSettings, MailSyncError> {
    Ok(MailSyncSettings {
        account_id: row.try_get("account_id")?,
        sync_enabled: row.try_get("sync_enabled")?,
        batch_size: row.try_get("batch_size")?,
        poll_interval_seconds: row.try_get("poll_interval_seconds")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_to_status(row: PgRow) -> Result<MailSyncStatus, MailSyncError> {
    Ok(MailSyncStatus {
        account_id: row.try_get("account_id")?,
        status: row.try_get("status")?,
        phase: row.try_get("phase")?,
        progress_mode: row.try_get("progress_mode")?,
        progress_percent: row.try_get("progress_percent")?,
        processed_messages: row.try_get("processed_messages")?,
        estimated_total_messages: row.try_get("estimated_total_messages")?,
        current_batch_size: row.try_get("current_batch_size")?,
        last_started_at: row.try_get("last_started_at")?,
        last_updated_at: row.try_get("last_updated_at")?,
        last_completed_at: row.try_get("last_completed_at")?,
        next_run_at: row.try_get("next_run_at")?,
        last_error_code: row.try_get("last_error_code")?,
        last_error_message: row.try_get("last_error_message")?,
        consecutive_failures: row.try_get("consecutive_failures")?,
        last_fetched_messages: row.try_get("last_fetched_messages")?,
        last_projected_messages: row.try_get("last_projected_messages")?,
        last_upserted_personas: row.try_get("last_upserted_personas")?,
        last_upserted_organizations: row.try_get("last_upserted_organizations")?,
    })
}

pub(super) fn row_to_due_account(row: PgRow) -> Result<MailSyncDueAccount, MailSyncError> {
    Ok(MailSyncDueAccount {
        account_id: row.try_get("account_id")?,
        batch_size: row.try_get("batch_size")?,
        poll_interval_seconds: row.try_get("poll_interval_seconds")?,
    })
}

pub(super) fn row_to_run(row: PgRow) -> Result<MailSyncRun, MailSyncError> {
    Ok(MailSyncRun {
        run_id: row.try_get("run_id")?,
        account_id: row.try_get("account_id")?,
        trigger: row.try_get("trigger")?,
        status: row.try_get("status")?,
        phase: row.try_get("phase")?,
        progress_mode: row.try_get("progress_mode")?,
        progress_percent: row.try_get("progress_percent")?,
        processed_messages: row.try_get("processed_messages")?,
        estimated_total_messages: row.try_get("estimated_total_messages")?,
        current_batch_size: row.try_get("current_batch_size")?,
        fetched_messages: row.try_get("fetched_messages")?,
        projected_messages: row.try_get("projected_messages")?,
        upserted_personas: row.try_get("upserted_personas")?,
        upserted_organizations: row.try_get("upserted_organizations")?,
        checkpoint_before: row.try_get("checkpoint_before")?,
        checkpoint_after: row.try_get("checkpoint_after")?,
        checkpoint_saved: row.try_get("checkpoint_saved")?,
        error_code: row.try_get("error_code")?,
        error_message: row.try_get("error_message")?,
        started_at: row.try_get("started_at")?,
        completed_at: row.try_get("completed_at")?,
        next_run_at: row.try_get("next_run_at")?,
    })
}
