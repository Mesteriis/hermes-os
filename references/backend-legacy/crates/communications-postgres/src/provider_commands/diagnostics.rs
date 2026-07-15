use std::collections::HashMap;

use serde_json::Value;
use sqlx::Row;

use super::{
    CommunicationProviderCommandDiagnostic, CommunicationProviderCommandDiagnostics,
    CommunicationProviderCommandError, CommunicationProviderCommandStatusCount,
    CommunicationProviderCommandStore,
};

impl CommunicationProviderCommandStore {
    pub async fn read_sync_statuses(
        &self,
        message_ids: &[String],
    ) -> Result<HashMap<String, String>, CommunicationProviderCommandError> {
        if message_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT ON (target_ref->>'message_id')
                target_ref->>'message_id' AS message_id,
                status,
                reconciliation_status,
                result_payload
            FROM communication_provider_commands
            WHERE channel_kind = 'mail'
              AND command_kind IN ('mark_read', 'mark_unread')
              AND target_ref->>'message_id' = ANY($1)
            ORDER BY target_ref->>'message_id', created_at DESC, command_id DESC
            "#,
        )
        .bind(message_ids)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                let message_id: String = row.try_get("message_id")?;
                let status: String = row.try_get("status")?;
                let reconciliation_status: String = row.try_get("reconciliation_status")?;
                let result_payload: Value = row.try_get("result_payload")?;
                Ok((
                    message_id,
                    read_sync_status(&status, &reconciliation_status, &result_payload).to_owned(),
                ))
            })
            .collect::<Result<HashMap<_, _>, sqlx::Error>>()
            .map_err(CommunicationProviderCommandError::Sqlx)
    }

    pub async fn diagnostics(
        &self,
        account_id: Option<&str>,
        status: Option<&str>,
        limit: i64,
    ) -> Result<CommunicationProviderCommandDiagnostics, CommunicationProviderCommandError> {
        let account_id = account_id.map(str::trim).filter(|value| !value.is_empty());
        let status = status.map(str::trim).filter(|value| !value.is_empty());
        if let Some(status) = status
            && !matches!(
                status,
                "queued" | "executing" | "retrying" | "completed" | "dead_letter"
            )
        {
            return Err(CommunicationProviderCommandError::Invalid(
                "unsupported provider command diagnostic status".to_owned(),
            ));
        }
        if !(1..=100).contains(&limit) {
            return Err(CommunicationProviderCommandError::Invalid(
                "provider command diagnostic limit must be between 1 and 100".to_owned(),
            ));
        }

        let rows = sqlx::query(
            r#"
            SELECT
                command_id,
                account_id,
                command_kind,
                target_ref->>'message_id' AS message_id,
                status,
                retry_count,
                max_retries,
                reconciliation_status,
                next_attempt_at,
                last_attempt_at,
                dead_lettered_at,
                last_error,
                created_at,
                updated_at
            FROM communication_provider_commands
            WHERE channel_kind = 'mail'
              AND ($1::text IS NULL OR account_id = $1)
              AND ($2::text IS NULL OR status = $2)
            ORDER BY updated_at DESC, command_id ASC
            LIMIT $3
            "#,
        )
        .bind(account_id)
        .bind(status)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        let items = rows
            .into_iter()
            .map(|row| {
                Ok(CommunicationProviderCommandDiagnostic {
                    command_id: row.try_get("command_id")?,
                    account_id: row.try_get("account_id")?,
                    command_kind: row.try_get("command_kind")?,
                    message_id: row.try_get("message_id")?,
                    status: row.try_get("status")?,
                    retry_count: row.try_get("retry_count")?,
                    max_retries: row.try_get("max_retries")?,
                    reconciliation_status: row.try_get("reconciliation_status")?,
                    next_attempt_at: row.try_get("next_attempt_at")?,
                    last_attempt_at: row.try_get("last_attempt_at")?,
                    dead_lettered_at: row.try_get("dead_lettered_at")?,
                    last_error: row
                        .try_get::<Option<String>, _>("last_error")?
                        .map(sanitize_diagnostic_error),
                    created_at: row.try_get("created_at")?,
                    updated_at: row.try_get("updated_at")?,
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()?;

        let count_rows = sqlx::query(
            r#"
            SELECT status, COUNT(*)::bigint AS count
            FROM communication_provider_commands
            WHERE channel_kind = 'mail'
              AND ($1::text IS NULL OR account_id = $1)
            GROUP BY status
            ORDER BY status
            "#,
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await?;
        let counts = count_rows
            .into_iter()
            .map(|row| {
                Ok(CommunicationProviderCommandStatusCount {
                    status: row.try_get("status")?,
                    count: row.try_get("count")?,
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()?;

        Ok(CommunicationProviderCommandDiagnostics { items, counts })
    }
}

fn sanitize_diagnostic_error(error: String) -> String {
    let normalized = error
        .replace("\\r", " ")
        .replace("\\n", " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    normalized.chars().take(240).collect()
}

fn read_sync_status<'a>(
    status: &'a str,
    reconciliation_status: &str,
    result_payload: &Value,
) -> &'a str {
    if result_payload
        .get("superseded")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return "superseded";
    }
    match status {
        "queued" => "queued",
        "executing" => "syncing",
        "retrying" => "retrying",
        "dead_letter" => "failed",
        "completed" if reconciliation_status == "observed" => "synced",
        "completed" => "awaiting_provider",
        other => other,
    }
}
