use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewCommunicationProviderCommand {
    pub command_id: String,
    pub account_id: String,
    pub channel_kind: String,
    pub command_kind: String,
    pub idempotency_key: String,
    pub provider_conversation_id: Option<String>,
    pub provider_message_id: Option<String>,
    pub target_ref: Value,
    pub payload: Value,
    pub capability_state: String,
    pub action_class: String,
    pub confirmation_decision: String,
    pub actor_id: String,
    pub max_retries: i32,
}

impl NewCommunicationProviderCommand {
    pub fn new(
        command_id: impl Into<String>,
        account_id: impl Into<String>,
        channel_kind: impl Into<String>,
        command_kind: impl Into<String>,
        idempotency_key: impl Into<String>,
        actor_id: impl Into<String>,
    ) -> Self {
        Self {
            command_id: command_id.into(),
            account_id: account_id.into(),
            channel_kind: channel_kind.into(),
            command_kind: command_kind.into(),
            idempotency_key: idempotency_key.into(),
            provider_conversation_id: None,
            provider_message_id: None,
            target_ref: serde_json::json!({}),
            payload: serde_json::json!({}),
            capability_state: "available".to_owned(),
            action_class: "provider_write".to_owned(),
            confirmation_decision: "not_required".to_owned(),
            actor_id: actor_id.into(),
            max_retries: 3,
        }
    }

    pub fn provider_conversation_id(mut self, provider_conversation_id: impl Into<String>) -> Self {
        self.provider_conversation_id = Some(provider_conversation_id.into());
        self
    }

    pub fn provider_message_id(mut self, provider_message_id: impl Into<String>) -> Self {
        self.provider_message_id = Some(provider_message_id.into());
        self
    }

    pub fn target_ref(mut self, target_ref: Value) -> Self {
        self.target_ref = target_ref;
        self
    }

    pub fn payload(mut self, payload: Value) -> Self {
        self.payload = payload;
        self
    }

    fn validate(&self) -> Result<(), CommunicationProviderCommandError> {
        validate_non_empty("command_id", &self.command_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("channel_kind", &self.channel_kind)?;
        validate_non_empty("command_kind", &self.command_kind)?;
        validate_non_empty("idempotency_key", &self.idempotency_key)?;
        validate_non_empty("capability_state", &self.capability_state)?;
        validate_non_empty("action_class", &self.action_class)?;
        validate_non_empty("confirmation_decision", &self.confirmation_decision)?;
        validate_non_empty("actor_id", &self.actor_id)?;
        if self.max_retries <= 0 {
            return Err(CommunicationProviderCommandError::Invalid(
                "max_retries must be greater than zero".to_owned(),
            ));
        }
        if !self.target_ref.is_object() {
            return Err(CommunicationProviderCommandError::Invalid(
                "target_ref must be a JSON object".to_owned(),
            ));
        }
        if !self.payload.is_object() {
            return Err(CommunicationProviderCommandError::Invalid(
                "payload must be a JSON object".to_owned(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationProviderCommand {
    pub command_id: String,
    pub account_id: String,
    pub channel_kind: String,
    pub command_kind: String,
    pub idempotency_key: String,
    pub provider_conversation_id: Option<String>,
    pub provider_message_id: Option<String>,
    pub target_ref: Value,
    pub payload: Value,
    pub capability_state: String,
    pub action_class: String,
    pub confirmation_decision: String,
    pub status: String,
    pub retry_count: i32,
    pub max_retries: i32,
    pub last_error: Option<String>,
    pub result_payload: Value,
    pub audit_metadata: Value,
    pub provider_state: Value,
    pub reconciliation_status: String,
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub provider_observed_at: Option<DateTime<Utc>>,
    pub reconciled_at: Option<DateTime<Utc>>,
    pub dead_lettered_at: Option<DateTime<Utc>>,
    pub actor_id: String,
    pub happened_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct CommunicationProviderCommandStore {
    pool: PgPool,
}

impl CommunicationProviderCommandStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn enqueue(
        &self,
        command: &NewCommunicationProviderCommand,
    ) -> Result<CommunicationProviderCommand, CommunicationProviderCommandError> {
        command.validate()?;
        ensure_canonical_communication_account(&self.pool, &command.account_id).await?;
        let row = sqlx::query(&provider_command_returning_sql(
            r#"
            INSERT INTO communication_provider_commands (
                command_id, account_id, channel_kind, command_kind, idempotency_key,
                provider_conversation_id, provider_message_id, target_ref, payload,
                capability_state, action_class, confirmation_decision, status,
                retry_count, max_retries, last_error, result_payload, audit_metadata,
                actor_id, happened_at, completed_at, created_at, updated_at
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9,
                $10, $11, $12, 'queued',
                0, $13, NULL, '{}'::jsonb, '{}'::jsonb,
                $14, now(), NULL, now(), now()
            )
            ON CONFLICT (account_id, idempotency_key)
            DO UPDATE SET updated_at = communication_provider_commands.updated_at
            "#,
        ))
        .bind(command.command_id.trim())
        .bind(command.account_id.trim())
        .bind(command.channel_kind.trim())
        .bind(command.command_kind.trim())
        .bind(command.idempotency_key.trim())
        .bind(command.provider_conversation_id.as_deref())
        .bind(command.provider_message_id.as_deref())
        .bind(&command.target_ref)
        .bind(&command.payload)
        .bind(command.capability_state.trim())
        .bind(command.action_class.trim())
        .bind(command.confirmation_decision.trim())
        .bind(command.max_retries)
        .bind(command.actor_id.trim())
        .fetch_one(&self.pool)
        .await?;

        row_to_provider_command(row)
    }

    pub async fn claim_due(
        &self,
        account_id: &str,
        channel_kind: &str,
        now: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<CommunicationProviderCommand>, CommunicationProviderCommandError> {
        validate_non_empty("account_id", account_id)?;
        validate_non_empty("channel_kind", channel_kind)?;
        let limit = limit.clamp(1, 100);
        let rows = sqlx::query(&provider_command_returning_sql_with_alias(
            r#"
            UPDATE communication_provider_commands command
            SET status = 'executing',
                retry_count = retry_count + 1,
                last_error = NULL,
                updated_at = $3
            FROM (
                SELECT command_id
                FROM communication_provider_commands
                WHERE account_id = $1
                  AND channel_kind = $2
                  AND status IN ('queued', 'retrying')
                  AND retry_count < max_retries
                ORDER BY updated_at ASC, command_id ASC
                LIMIT $4
                FOR UPDATE SKIP LOCKED
            ) due
            WHERE command.command_id = due.command_id
            "#,
            "command",
        ))
        .bind(account_id.trim())
        .bind(channel_kind.trim())
        .bind(now)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_provider_command).collect()
    }

    pub async fn mark_completed(
        &self,
        command_id: &str,
        channel_kind: &str,
        now: DateTime<Utc>,
        result_payload: Value,
    ) -> Result<Option<CommunicationProviderCommand>, CommunicationProviderCommandError> {
        validate_non_empty("command_id", command_id)?;
        validate_non_empty("channel_kind", channel_kind)?;
        if !result_payload.is_object() {
            return Err(CommunicationProviderCommandError::Invalid(
                "result_payload must be a JSON object".to_owned(),
            ));
        }
        let row = sqlx::query(&provider_command_returning_sql(
            r#"
            UPDATE communication_provider_commands
            SET status = 'completed',
                provider_message_id = COALESCE(
                    provider_message_id,
                    NULLIF($4 #>> '{provider_message_id}', ''),
                    NULLIF($4 #>> '{message_id}', '')
                ),
                completed_at = $3,
                result_payload = $4,
                reconciliation_status = CASE
                    WHEN COALESCE(
                        provider_message_id,
                        NULLIF($4 #>> '{provider_message_id}', ''),
                        NULLIF($4 #>> '{message_id}', '')
                    ) IS NULL THEN reconciliation_status
                    ELSE 'awaiting_provider'
                END,
                last_error = NULL,
                updated_at = $3
            WHERE command_id = $1
              AND channel_kind = $2
            "#,
        ))
        .bind(command_id.trim())
        .bind(channel_kind.trim())
        .bind(now)
        .bind(result_payload)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_provider_command).transpose()
    }

    pub async fn mark_observed_by_provider_message(
        &self,
        account_id: &str,
        channel_kind: &str,
        provider_message_id: &str,
        command_kinds: &[&str],
        observed_at: DateTime<Utc>,
        provider_state: Value,
    ) -> Result<Vec<CommunicationProviderCommand>, CommunicationProviderCommandError> {
        validate_non_empty("account_id", account_id)?;
        validate_non_empty("channel_kind", channel_kind)?;
        validate_non_empty("provider_message_id", provider_message_id)?;
        if command_kinds.is_empty() {
            return Err(CommunicationProviderCommandError::Invalid(
                "command_kinds must not be empty".to_owned(),
            ));
        }
        for command_kind in command_kinds {
            validate_non_empty("command_kind", command_kind)?;
        }
        if !provider_state.is_object() {
            return Err(CommunicationProviderCommandError::Invalid(
                "provider_state must be a JSON object".to_owned(),
            ));
        }
        let command_kinds = command_kinds
            .iter()
            .map(|command_kind| command_kind.trim().to_owned())
            .collect::<Vec<_>>();
        let rows = sqlx::query(&provider_command_returning_sql(
            r#"
            UPDATE communication_provider_commands
            SET status = 'completed',
                provider_message_id = COALESCE(provider_message_id, $3),
                result_payload = result_payload || jsonb_build_object(
                    'provider_observed', true,
                    'provider_message_id', $3
                ),
                provider_state = provider_state || $6,
                reconciliation_status = 'observed',
                provider_observed_at = $5,
                reconciled_at = $5,
                completed_at = COALESCE(completed_at, $5),
                last_error = NULL,
                updated_at = $5
            WHERE account_id = $1
              AND channel_kind = $2
              AND command_kind = ANY($4::text[])
              AND status IN ('completed', 'executing')
              AND reconciliation_status <> 'observed'
              AND (
                    provider_message_id = $3
                    OR result_payload #>> '{provider_message_id}' = $3
                    OR result_payload #>> '{message_id}' = $3
              )
            "#,
        ))
        .bind(account_id.trim())
        .bind(channel_kind.trim())
        .bind(provider_message_id.trim())
        .bind(command_kinds)
        .bind(observed_at)
        .bind(provider_state)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_provider_command).collect()
    }

    pub async fn mark_failed(
        &self,
        command_id: &str,
        channel_kind: &str,
        now: DateTime<Utc>,
        error: &str,
        result_payload: Value,
    ) -> Result<Option<CommunicationProviderCommand>, CommunicationProviderCommandError> {
        validate_non_empty("command_id", command_id)?;
        validate_non_empty("channel_kind", channel_kind)?;
        validate_non_empty("error", error)?;
        if !result_payload.is_object() {
            return Err(CommunicationProviderCommandError::Invalid(
                "result_payload must be a JSON object".to_owned(),
            ));
        }
        let row = sqlx::query(&provider_command_returning_sql(
            r#"
            UPDATE communication_provider_commands
            SET status = CASE
                    WHEN retry_count >= max_retries THEN 'dead_letter'
                    ELSE 'retrying'
                END,
                last_error = $4,
                result_payload = $5,
                updated_at = $3
            WHERE command_id = $1
              AND channel_kind = $2
            "#,
        ))
        .bind(command_id.trim())
        .bind(channel_kind.trim())
        .bind(now)
        .bind(error.trim())
        .bind(result_payload)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_provider_command).transpose()
    }

    pub async fn manual_retry(
        &self,
        command_id: &str,
        channel_kind: &str,
        now: DateTime<Utc>,
    ) -> Result<Option<CommunicationProviderCommand>, CommunicationProviderCommandError> {
        validate_non_empty("command_id", command_id)?;
        validate_non_empty("channel_kind", channel_kind)?;
        let row = sqlx::query(&provider_command_returning_sql(
            r#"
            UPDATE communication_provider_commands
            SET status = 'retrying',
                retry_count = 0,
                completed_at = NULL,
                last_error = NULL,
                reconciliation_status = 'not_observed',
                provider_observed_at = NULL,
                reconciled_at = NULL,
                updated_at = $3
            WHERE command_id = $1
              AND channel_kind = $2
              AND status IN ('failed', 'dead_letter')
            "#,
        ))
        .bind(command_id.trim())
        .bind(channel_kind.trim())
        .bind(now)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_provider_command).transpose()
    }

    pub async fn list(
        &self,
        account_id: &str,
        channel_kind: &str,
        limit: i64,
    ) -> Result<Vec<CommunicationProviderCommand>, CommunicationProviderCommandError> {
        validate_non_empty("account_id", account_id)?;
        validate_non_empty("channel_kind", channel_kind)?;
        let rows = sqlx::query(
            r#"
            SELECT
                command_id, account_id, channel_kind, command_kind, idempotency_key,
                provider_conversation_id, provider_message_id, target_ref, payload,
                capability_state, action_class, confirmation_decision, status,
                retry_count, max_retries, last_error, result_payload, audit_metadata,
                provider_state, reconciliation_status, next_attempt_at, last_attempt_at,
                provider_observed_at, reconciled_at, dead_lettered_at,
                actor_id, happened_at, completed_at, created_at, updated_at
            FROM communication_provider_commands
            WHERE account_id = $1
              AND channel_kind = $2
            ORDER BY updated_at DESC, command_id ASC
            LIMIT $3
            "#,
        )
        .bind(account_id.trim())
        .bind(channel_kind.trim())
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_provider_command).collect()
    }
}

fn provider_command_returning_sql(prefix: &str) -> String {
    provider_command_returning_sql_with_alias(prefix, "")
}

fn provider_command_returning_sql_with_alias(prefix: &str, alias: &str) -> String {
    let qualifier = if alias.trim().is_empty() {
        String::new()
    } else {
        format!("{}.", alias.trim())
    };
    format!(
        r#"
        {prefix}
        RETURNING
            {qualifier}command_id,
            {qualifier}account_id,
            {qualifier}channel_kind,
            {qualifier}command_kind,
            {qualifier}idempotency_key,
            {qualifier}provider_conversation_id,
            {qualifier}provider_message_id,
            {qualifier}target_ref,
            {qualifier}payload,
            {qualifier}capability_state,
            {qualifier}action_class,
            {qualifier}confirmation_decision,
            {qualifier}status,
            {qualifier}retry_count,
            {qualifier}max_retries,
            {qualifier}last_error,
            {qualifier}result_payload,
            {qualifier}audit_metadata,
            {qualifier}provider_state,
            {qualifier}reconciliation_status,
            {qualifier}next_attempt_at,
            {qualifier}last_attempt_at,
            {qualifier}provider_observed_at,
            {qualifier}reconciled_at,
            {qualifier}dead_lettered_at,
            {qualifier}actor_id,
            {qualifier}happened_at,
            {qualifier}completed_at,
            {qualifier}created_at,
            {qualifier}updated_at
        "#
    )
}

fn row_to_provider_command(
    row: PgRow,
) -> Result<CommunicationProviderCommand, CommunicationProviderCommandError> {
    Ok(CommunicationProviderCommand {
        command_id: row.try_get("command_id")?,
        account_id: row.try_get("account_id")?,
        channel_kind: row.try_get("channel_kind")?,
        command_kind: row.try_get("command_kind")?,
        idempotency_key: row.try_get("idempotency_key")?,
        provider_conversation_id: row.try_get("provider_conversation_id")?,
        provider_message_id: row.try_get("provider_message_id")?,
        target_ref: row.try_get("target_ref")?,
        payload: row.try_get("payload")?,
        capability_state: row.try_get("capability_state")?,
        action_class: row.try_get("action_class")?,
        confirmation_decision: row.try_get("confirmation_decision")?,
        status: row.try_get("status")?,
        retry_count: row.try_get("retry_count")?,
        max_retries: row.try_get("max_retries")?,
        last_error: row.try_get("last_error")?,
        result_payload: row.try_get("result_payload")?,
        audit_metadata: row.try_get("audit_metadata")?,
        provider_state: row.try_get("provider_state")?,
        reconciliation_status: row.try_get("reconciliation_status")?,
        next_attempt_at: row.try_get("next_attempt_at")?,
        last_attempt_at: row.try_get("last_attempt_at")?,
        provider_observed_at: row.try_get("provider_observed_at")?,
        reconciled_at: row.try_get("reconciled_at")?,
        dead_lettered_at: row.try_get("dead_lettered_at")?,
        actor_id: row.try_get("actor_id")?,
        happened_at: row.try_get("happened_at")?,
        completed_at: row.try_get("completed_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

async fn ensure_canonical_communication_account(
    pool: &PgPool,
    account_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO communication_accounts (
            account_id, provider_kind, display_name, external_account_id,
            config, metadata, created_at, updated_at
        )
        SELECT
            account_id,
            provider_kind,
            display_name,
            external_account_id,
            config,
            jsonb_build_object('source_table', 'communication_provider_accounts'),
            created_at,
            updated_at
        FROM communication_provider_accounts
        WHERE account_id = $1
        ON CONFLICT (account_id)
        DO UPDATE SET
            provider_kind = EXCLUDED.provider_kind,
            display_name = EXCLUDED.display_name,
            external_account_id = EXCLUDED.external_account_id,
            config = EXCLUDED.config,
            updated_at = EXCLUDED.updated_at
        "#,
    )
    .bind(account_id.trim())
    .execute(pool)
    .await?;
    Ok(())
}

fn validate_non_empty(
    field: &'static str,
    value: &str,
) -> Result<(), CommunicationProviderCommandError> {
    if value.trim().is_empty() {
        return Err(CommunicationProviderCommandError::Invalid(format!(
            "{field} must not be empty"
        )));
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum CommunicationProviderCommandError {
    #[error("invalid communication provider command: {0}")]
    Invalid(String),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
