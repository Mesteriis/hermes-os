use super::super::DEFAULT_MAIL_SYNC_BATCH_SIZE;
use super::super::errors::MailSyncError;
use super::super::models::MailSyncStatus;
use super::super::rows::row_to_status;
use super::MailSyncStore;

impl MailSyncStore {
    pub async fn sync_statuses(&self) -> Result<Vec<MailSyncStatus>, MailSyncError> {
        let rows = sqlx::query(
            r#"
            WITH latest AS (
                SELECT DISTINCT ON (account_id)
                    account_id,
                    status,
                    phase,
                    progress_mode,
                    progress_percent,
                    processed_messages,
                    estimated_total_messages,
                    current_batch_size,
                    started_at,
                    updated_at,
                    completed_at,
                    next_run_at,
                    error_code,
                    error_message,
                    fetched_messages,
                    projected_messages,
                    upserted_personas,
                    upserted_organizations
                FROM communication_mail_sync_runs
                ORDER BY account_id, started_at DESC
            )
            SELECT
                a.account_id,
                CASE
                    WHEN latest.status = 'failed'
                         AND COALESCE(failures.consecutive_failures, 0)
                             >= COALESCE(settings.failure_threshold, 3)
                        THEN 'degraded'
                    WHEN latest.status = 'failed' THEN 'warning'
                    ELSE COALESCE(latest.status, 'idle')
                END AS status,
                COALESCE(latest.phase, 'idle') AS phase,
                COALESCE(latest.progress_mode, 'none') AS progress_mode,
                latest.progress_percent,
                COALESCE(latest.processed_messages, 0) AS processed_messages,
                latest.estimated_total_messages,
                COALESCE(latest.current_batch_size, COALESCE(settings.batch_size, $1)) AS current_batch_size,
                COALESCE(settings.failure_threshold, 3) AS failure_threshold,
                latest.started_at AS last_started_at,
                latest.updated_at AS last_updated_at,
                latest.completed_at AS last_completed_at,
                COALESCE(
                    latest.next_run_at,
                    CASE
                        WHEN COALESCE(settings.sync_enabled, true) THEN now()
                        ELSE NULL
                    END
                ) AS next_run_at,
                latest.error_code AS last_error_code,
                latest.error_message AS last_error_message,
                COALESCE(failures.consecutive_failures, 0) AS consecutive_failures,
                COALESCE(latest.fetched_messages, 0) AS last_fetched_messages,
                COALESCE(latest.projected_messages, 0) AS last_projected_messages,
                COALESCE(latest.upserted_personas, 0) AS last_upserted_personas,
                COALESCE(latest.upserted_organizations, 0) AS last_upserted_organizations
            FROM communication_provider_accounts a
            LEFT JOIN communication_account_sync_settings settings ON settings.account_id = a.account_id
            LEFT JOIN latest ON latest.account_id = a.account_id
            LEFT JOIN LATERAL (
                SELECT count(*)::BIGINT AS consecutive_failures
                FROM (
                    SELECT
                        status,
                        error_code,
                        count(*) FILTER (
                            WHERE status <> 'failed'
                               OR error_code <> 'provider_network_error'
                        ) OVER (
                            ORDER BY started_at DESC
                            ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW
                        ) AS terminal_run_seen
                    FROM communication_mail_sync_runs
                    WHERE account_id = a.account_id
                    ORDER BY started_at DESC
                ) runs
                WHERE runs.status = 'failed'
                  AND runs.error_code = 'provider_network_error'
                  AND runs.terminal_run_seen = 0
            ) failures ON true
            WHERE a.provider_kind IN ('gmail', 'icloud', 'imap')
              AND COALESCE(a.config->>'auth_state', '') <> 'deleted'
              AND NOT (a.config ? 'deleted_at')
            ORDER BY a.display_name ASC, a.account_id ASC
            "#,
        )
        .bind(DEFAULT_MAIL_SYNC_BATCH_SIZE)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_status).collect()
    }
}
