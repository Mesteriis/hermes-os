//! WhatsApp-owned PostgreSQL projections and event journal.

use hermes_whatsapp_api::{
    WhatsAppAccount, WhatsAppDialog, WhatsAppMedia, WhatsAppMessage, WhatsAppParticipant,
    WhatsAppProviderCommand, WhatsAppProviderEvent, WhatsAppRealtimeFrame,
};
use hermes_whatsapp_core::{
    WhatsAppOperation, WhatsAppOperationState, operation_dead_lettered, operation_failed,
    operation_host_claimed, operation_retry_scheduled,
};
use serde_json::Value;
use sqlx::{PgConnectOptions, PgPool, Row};
use hermes_storage_protocol::StorageBindingV1;

pub const WHATSAPP_SCHEMA_V1: &str = r#"
CREATE TABLE IF NOT EXISTS whatsapp_accounts (
    account_id TEXT PRIMARY KEY,
    projection_payload JSONB NOT NULL
);
CREATE TABLE IF NOT EXISTS whatsapp_operations (
    operation_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    state TEXT NOT NULL,
    host_claim_id TEXT,
    host_claimed_until_unix_seconds BIGINT,
    projection_payload JSONB NOT NULL
);
CREATE TABLE IF NOT EXISTS whatsapp_commands (
    operation_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    command_payload JSONB NOT NULL
);
CREATE TABLE IF NOT EXISTS whatsapp_event_journal (
    account_id TEXT NOT NULL,
    sequence BIGINT NOT NULL,
    event_payload JSONB NOT NULL,
    PRIMARY KEY (account_id, sequence)
);
CREATE TABLE IF NOT EXISTS whatsapp_message_projections (
    message_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    provider_chat_id TEXT NOT NULL,
    projection_payload JSONB NOT NULL
);
CREATE TABLE IF NOT EXISTS whatsapp_dialog_projections (
    account_id TEXT NOT NULL,
    provider_chat_id TEXT NOT NULL,
    projection_payload JSONB NOT NULL,
    PRIMARY KEY (account_id, provider_chat_id)
);
CREATE TABLE IF NOT EXISTS whatsapp_participant_projections (
    account_id TEXT NOT NULL,
    provider_chat_id TEXT NOT NULL,
    provider_identity_id TEXT NOT NULL,
    projection_payload JSONB NOT NULL,
    PRIMARY KEY (account_id, provider_chat_id, provider_identity_id)
);
CREATE TABLE IF NOT EXISTS whatsapp_media_projections (
    media_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    projection_payload JSONB NOT NULL
);
"#;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhatsAppDurablePersistenceError {
    Database,
    Codec,
    InvalidRow,
}

pub struct WhatsAppDurablePersistence {
    pool: PgPool,
}

impl WhatsAppDurablePersistence {
    pub async fn connect(database_url: &str) -> Result<Self, WhatsAppDurablePersistenceError> {
        let pool = PgPool::connect(database_url)
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        Ok(Self::new(pool))
    }

    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, WhatsAppDurablePersistenceError> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || binding.access().runtime_principal().is_empty()
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
        {
            return Err(WhatsAppDurablePersistenceError::InvalidRow);
        }
        let port = u16::try_from(pgbouncer_port)
            .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
        let options = PgConnectOptions::new()
            .host(pgbouncer_host)
            .port(port)
            .username(binding.access().runtime_principal())
            .password(password)
            .database(database_id);
        let pool = PgPool::connect_with(options)
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        Ok(Self::new(pool))
    }

    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn initialize(&self) -> Result<(), WhatsAppDurablePersistenceError> {
        sqlx::raw_sql(WHATSAPP_SCHEMA_V1)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| WhatsAppDurablePersistenceError::Database)
    }

    pub async fn upsert_account(
        &self,
        account: &WhatsAppAccount,
    ) -> Result<(), WhatsAppDurablePersistenceError> {
        let payload = serde_json::to_value(account).map_err(|_| WhatsAppDurablePersistenceError::Codec)?;
        sqlx::query(
            "INSERT INTO whatsapp_accounts (account_id, projection_payload) VALUES ($1, $2) ON CONFLICT (account_id) DO UPDATE SET projection_payload = EXCLUDED.projection_payload",
        )
        .bind(&account.account_id)
        .bind(payload)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|_| WhatsAppDurablePersistenceError::Database)
    }

    pub async fn account(
        &self,
        account_id: &str,
    ) -> Result<Option<WhatsAppAccount>, WhatsAppDurablePersistenceError> {
        let row = sqlx::query(
            "SELECT projection_payload FROM whatsapp_accounts WHERE account_id = $1",
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        let Some(row) = row else { return Ok(None); };
        let payload: Value = row
            .try_get("projection_payload")
            .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
        serde_json::from_value(payload)
            .map(Some)
            .map_err(|_| WhatsAppDurablePersistenceError::Codec)
    }

    pub async fn messages(
        &self,
        account_id: &str,
        provider_chat_id: Option<&str>,
        query: Option<&str>,
        limit: i64,
    ) -> Result<Vec<WhatsAppMessage>, WhatsAppDurablePersistenceError> {
        let rows = sqlx::query(
            "SELECT projection_payload FROM whatsapp_message_projections WHERE account_id = $1 ORDER BY message_id DESC",
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        let query = query.map(str::to_lowercase);
        let mut messages = Vec::new();
        for row in rows {
            let payload: Value = row
                .try_get("projection_payload")
                .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
            let message: WhatsAppMessage = serde_json::from_value(payload)
                .map_err(|_| WhatsAppDurablePersistenceError::Codec)?;
            if provider_chat_id.is_some_and(|chat_id| message.provider_chat_id != chat_id)
                || query.as_ref().is_some_and(|text| {
                    !message
                        .text
                        .as_deref()
                        .unwrap_or_default()
                        .to_lowercase()
                        .contains(text)
                })
            {
                continue;
            }
            messages.push(message);
            if messages.len() >= usize::try_from(limit).unwrap_or_default() {
                break;
            }
        }
        Ok(messages)
    }

    pub async fn dialogs(
        &self,
        account_id: &str,
        limit: i64,
    ) -> Result<Vec<WhatsAppDialog>, WhatsAppDurablePersistenceError> {
        let rows = sqlx::query(
            "SELECT projection_payload FROM whatsapp_dialog_projections WHERE account_id = $1 ORDER BY provider_chat_id ASC LIMIT $2",
        )
        .bind(account_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        rows.into_iter()
            .map(|row| {
                let payload: Value = row
                    .try_get("projection_payload")
                    .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
                serde_json::from_value(payload)
                    .map_err(|_| WhatsAppDurablePersistenceError::Codec)
            })
            .collect()
    }

    pub async fn participants(
        &self,
        account_id: &str,
        provider_chat_id: &str,
        limit: i64,
    ) -> Result<Vec<WhatsAppParticipant>, WhatsAppDurablePersistenceError> {
        let rows = sqlx::query(
            "SELECT projection_payload FROM whatsapp_participant_projections WHERE account_id = $1 AND provider_chat_id = $2 ORDER BY provider_identity_id ASC LIMIT $3",
        )
        .bind(account_id)
        .bind(provider_chat_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        rows.into_iter()
            .map(|row| {
                let payload: Value = row
                    .try_get("projection_payload")
                    .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
                serde_json::from_value(payload)
                    .map_err(|_| WhatsAppDurablePersistenceError::Codec)
            })
            .collect()
    }

    pub async fn pending_commands(
        &self,
        account_id: &str,
        limit: i64,
    ) -> Result<Vec<WhatsAppProviderCommand>, WhatsAppDurablePersistenceError> {
        let rows = sqlx::query(
            "SELECT command_payload FROM whatsapp_commands WHERE account_id = $1 AND operation_id IN (SELECT operation_id FROM whatsapp_operations WHERE account_id = $1 AND state IN ('awaiting_provider', 'retry_scheduled', 'host_claimed')) ORDER BY operation_id ASC LIMIT $2",
        )
        .bind(account_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        rows.into_iter()
            .map(|row| {
                let payload: Value = row
                    .try_get("command_payload")
                    .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
                serde_json::from_value(payload)
                    .map_err(|_| WhatsAppDurablePersistenceError::Codec)
            })
            .collect()
    }

    pub async fn save_operation(
        &self,
        operation: &WhatsAppOperation,
    ) -> Result<(), WhatsAppDurablePersistenceError> {
        let payload = serde_json::to_value(operation).map_err(|_| WhatsAppDurablePersistenceError::Codec)?;
        sqlx::query(
            "INSERT INTO whatsapp_operations (operation_id, account_id, state, host_claim_id, host_claimed_until_unix_seconds, projection_payload) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (operation_id) DO UPDATE SET state = EXCLUDED.state, host_claim_id = EXCLUDED.host_claim_id, host_claimed_until_unix_seconds = EXCLUDED.host_claimed_until_unix_seconds, projection_payload = EXCLUDED.projection_payload",
        )
        .bind(&operation.operation_id)
        .bind(&operation.account_id)
        .bind(operation_state_label(operation.state))
        .bind(&operation.host_claim_id)
        .bind(operation.host_claimed_until_unix_seconds)
        .bind(payload)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|_| WhatsAppDurablePersistenceError::Database)
    }

    pub async fn claim_pending_commands(
        &self,
        account_id: &str,
        claim_id: &str,
        now_unix_seconds: i64,
        lease_seconds: i64,
        limit: i64,
    ) -> Result<Vec<WhatsAppProviderCommand>, WhatsAppDurablePersistenceError> {
        let until = now_unix_seconds
            .checked_add(lease_seconds)
            .ok_or(WhatsAppDurablePersistenceError::InvalidRow)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        let rows = sqlx::query(
            "SELECT operation_id, projection_payload FROM whatsapp_operations WHERE account_id = $1 AND state IN ('awaiting_provider', 'retry_scheduled') AND (host_claimed_until_unix_seconds IS NULL OR host_claimed_until_unix_seconds <= $2) ORDER BY operation_id LIMIT $3 FOR UPDATE SKIP LOCKED",
        )
        .bind(account_id)
        .bind(now_unix_seconds)
        .bind(limit)
        .fetch_all(&mut *transaction)
        .await
        .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        let mut commands = Vec::with_capacity(rows.len());
        for row in rows {
            let operation_id = row
                .try_get::<String, _>("operation_id")
                .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
            let operation_payload: Value = row
                .try_get("projection_payload")
                .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
            let operation: WhatsAppOperation = serde_json::from_value(operation_payload)
                .map_err(|_| WhatsAppDurablePersistenceError::Codec)?;
            let command_row = sqlx::query(
                "SELECT command_payload FROM whatsapp_commands WHERE operation_id = $1",
            )
            .bind(&operation_id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
            let command_payload: Value = command_row
                .try_get("command_payload")
                .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
            let command: WhatsAppProviderCommand = serde_json::from_value(command_payload)
                .map_err(|_| WhatsAppDurablePersistenceError::Codec)?;
            let claimed = operation_host_claimed(&operation, claim_id, until);
            let claimed_payload = serde_json::to_value(&claimed)
                .map_err(|_| WhatsAppDurablePersistenceError::Codec)?;
            sqlx::query(
                "UPDATE whatsapp_operations SET state = $2, host_claim_id = $3, host_claimed_until_unix_seconds = $4, projection_payload = $5 WHERE operation_id = $1",
            )
            .bind(&operation_id)
            .bind(operation_state_label(claimed.state))
            .bind(&claimed.host_claim_id)
            .bind(claimed.host_claimed_until_unix_seconds)
            .bind(claimed_payload)
            .execute(&mut *transaction)
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
            commands.push(command);
        }
        transaction
            .commit()
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        Ok(commands)
    }

    pub async fn fail_claimed_command(
        &self,
        operation_id: &str,
        claim_id: &str,
        reason: impl Into<String>,
    ) -> Result<bool, WhatsAppDurablePersistenceError> {
        let row = sqlx::query(
            "SELECT projection_payload FROM whatsapp_operations WHERE operation_id = $1 AND state = 'host_claimed' AND host_claim_id = $2",
        )
        .bind(operation_id)
        .bind(claim_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        let Some(row) = row else { return Ok(false); };
        let payload: Value = row
            .try_get("projection_payload")
            .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
        let operation: WhatsAppOperation = serde_json::from_value(payload)
            .map_err(|_| WhatsAppDurablePersistenceError::Codec)?;
        let failed = operation_failed(&operation, reason);
        self.save_operation(&failed).await?;
        Ok(true)
    }

    pub async fn retry_command(
        &self,
        operation_id: &str,
    ) -> Result<bool, WhatsAppDurablePersistenceError> {
        let Some(operation) = self.load_operation(operation_id).await? else { return Ok(false); };
        if !matches!(operation.state, WhatsAppOperationState::Failed | WhatsAppOperationState::RetryScheduled) {
            return Ok(false);
        }
        self.save_operation(&operation_retry_scheduled(&operation)).await?;
        Ok(true)
    }

    pub async fn operation(
        &self,
        operation_id: &str,
    ) -> Result<Option<WhatsAppOperation>, WhatsAppDurablePersistenceError> {
        self.load_operation(operation_id).await
    }

    pub async fn dead_letter_command(
        &self,
        operation_id: &str,
        reason: impl Into<String>,
    ) -> Result<bool, WhatsAppDurablePersistenceError> {
        let Some(operation) = self.load_operation(operation_id).await? else { return Ok(false); };
        if operation.state == WhatsAppOperationState::Completed || operation.state == WhatsAppOperationState::DeadLettered {
            return Ok(false);
        }
        self.save_operation(&operation_dead_lettered(&operation, reason)).await?;
        Ok(true)
    }

    async fn load_operation(
        &self,
        operation_id: &str,
    ) -> Result<Option<WhatsAppOperation>, WhatsAppDurablePersistenceError> {
        let row = sqlx::query("SELECT projection_payload FROM whatsapp_operations WHERE operation_id = $1")
            .bind(operation_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        row.map(|row| {
            let payload: Value = row.try_get("projection_payload").map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
            serde_json::from_value(payload).map_err(|_| WhatsAppDurablePersistenceError::Codec)
        }).transpose()
    }

    pub async fn save_command(
        &self,
        command: &WhatsAppProviderCommand,
    ) -> Result<(), WhatsAppDurablePersistenceError> {
        let operation_id = hermes_whatsapp_api::provider_command_operation_id(command);
        let payload = serde_json::to_value(command).map_err(|_| WhatsAppDurablePersistenceError::Codec)?;
        sqlx::query(
            "INSERT INTO whatsapp_commands (operation_id, account_id, command_payload) VALUES ($1, $2, $3) ON CONFLICT (operation_id) DO UPDATE SET command_payload = EXCLUDED.command_payload",
        )
        .bind(operation_id)
        .bind(hermes_whatsapp_api::provider_command_account_id(command))
        .bind(payload)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|_| WhatsAppDurablePersistenceError::Database)
    }

    pub async fn append_event(
        &self,
        frame: &WhatsAppRealtimeFrame,
    ) -> Result<(), WhatsAppDurablePersistenceError> {
        let sequence = i64::try_from(frame.sequence).map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
        let payload = serde_json::to_value(&frame.event).map_err(|_| WhatsAppDurablePersistenceError::Codec)?;
        sqlx::query(
            "INSERT INTO whatsapp_event_journal (account_id, sequence, event_payload) VALUES ($1, $2, $3) ON CONFLICT (account_id, sequence) DO NOTHING",
        )
        .bind(&frame.account_id)
        .bind(sequence)
        .bind(payload)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|_| WhatsAppDurablePersistenceError::Database)
    }

    pub async fn upsert_message(&self, message: &WhatsAppMessage) -> Result<(), WhatsAppDurablePersistenceError> {
        self.upsert_projection(
            "whatsapp_message_projections",
            "message_id",
            &message.provider_message_id,
            &[&message.account_id, &message.provider_chat_id],
            message,
        )
        .await
    }

    pub async fn upsert_dialog(&self, dialog: &WhatsAppDialog) -> Result<(), WhatsAppDurablePersistenceError> {
        let payload = serde_json::to_value(dialog).map_err(|_| WhatsAppDurablePersistenceError::Codec)?;
        sqlx::query("INSERT INTO whatsapp_dialog_projections (account_id, provider_chat_id, projection_payload) VALUES ($1, $2, $3) ON CONFLICT (account_id, provider_chat_id) DO UPDATE SET projection_payload = EXCLUDED.projection_payload")
            .bind(&dialog.account_id).bind(&dialog.provider_chat_id).bind(payload).execute(&self.pool).await.map(|_| ()).map_err(|_| WhatsAppDurablePersistenceError::Database)
    }

    pub async fn upsert_participant(&self, participant: &WhatsAppParticipant) -> Result<(), WhatsAppDurablePersistenceError> {
        let payload = serde_json::to_value(participant).map_err(|_| WhatsAppDurablePersistenceError::Codec)?;
        sqlx::query("INSERT INTO whatsapp_participant_projections (account_id, provider_chat_id, provider_identity_id, projection_payload) VALUES ($1, $2, $3, $4) ON CONFLICT (account_id, provider_chat_id, provider_identity_id) DO UPDATE SET projection_payload = EXCLUDED.projection_payload")
            .bind(&participant.account_id).bind(&participant.provider_chat_id).bind(&participant.provider_identity_id).bind(payload).execute(&self.pool).await.map(|_| ()).map_err(|_| WhatsAppDurablePersistenceError::Database)
    }

    pub async fn upsert_media(&self, media: &WhatsAppMedia) -> Result<(), WhatsAppDurablePersistenceError> {
        let payload = serde_json::to_value(media).map_err(|_| WhatsAppDurablePersistenceError::Codec)?;
        sqlx::query("INSERT INTO whatsapp_media_projections (media_id, account_id, projection_payload) VALUES ($1, $2, $3) ON CONFLICT (media_id) DO UPDATE SET projection_payload = EXCLUDED.projection_payload")
            .bind(&media.provider_media_id).bind(&media.account_id).bind(payload).execute(&self.pool).await.map(|_| ()).map_err(|_| WhatsAppDurablePersistenceError::Database)
    }

    async fn upsert_projection<T: serde::Serialize>(
        &self,
        table: &str,
        key_column: &str,
        key: &str,
        _scope: &[&str],
        value: &T,
    ) -> Result<(), WhatsAppDurablePersistenceError> {
        let payload = serde_json::to_value(value).map_err(|_| WhatsAppDurablePersistenceError::Codec)?;
        let query = format!("INSERT INTO {table} ({key_column}, account_id, provider_chat_id, projection_payload) VALUES ($1, $2, $3, $4) ON CONFLICT ({key_column}) DO UPDATE SET projection_payload = EXCLUDED.projection_payload");
        let message = serde_json::from_value::<WhatsAppMessage>(payload.clone()).map_err(|_| WhatsAppDurablePersistenceError::Codec)?;
        sqlx::query(&query).bind(key).bind(&message.account_id).bind(&message.provider_chat_id).bind(payload).execute(&self.pool).await.map(|_| ()).map_err(|_| WhatsAppDurablePersistenceError::Database)
    }

    pub async fn events_by_kind(
        &self,
        account_id: &str,
        kind: hermes_whatsapp_api::WhatsAppProviderEventKind,
        provider_chat_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<WhatsAppProviderEvent>, WhatsAppDurablePersistenceError> {
        let rows = sqlx::query(
            "SELECT event_payload FROM whatsapp_event_journal WHERE account_id = $1 ORDER BY sequence ASC",
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        let mut events = Vec::new();
        for row in rows {
            let payload: Value = row
                .try_get("event_payload")
                .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
            let event: WhatsAppProviderEvent = serde_json::from_value(payload)
                .map_err(|_| WhatsAppDurablePersistenceError::Codec)?;
            if hermes_whatsapp_api::provider_event_kind(&event) != kind
                || provider_chat_id.is_some_and(|chat_id| {
                    hermes_whatsapp_api::provider_event_chat_id(&event) != Some(chat_id)
                })
            {
                continue;
            }
            events.push(event);
            if events.len() >= usize::try_from(limit).unwrap_or_default() {
                break;
            }
        }
        Ok(events)
    }

    pub async fn replay_events_after(&self, account_id: &str, after_sequence: u64, limit: i64) -> Result<Vec<WhatsAppRealtimeFrame>, WhatsAppDurablePersistenceError> {
        let rows = sqlx::query("SELECT sequence, event_payload FROM whatsapp_event_journal WHERE account_id = $1 AND sequence > $2 ORDER BY sequence ASC LIMIT $3")
            .bind(account_id).bind(i64::try_from(after_sequence).map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?).bind(limit).fetch_all(&self.pool).await.map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        rows.into_iter().map(|row| {
            let sequence = row.try_get::<i64, _>("sequence").map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
            let payload: Value = row.try_get("event_payload").map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
            Ok(WhatsAppRealtimeFrame { account_id: account_id.to_owned(), sequence: u64::try_from(sequence).map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?, event: serde_json::from_value(payload).map_err(|_| WhatsAppDurablePersistenceError::Codec)? })
        }).collect()
    }
}

fn operation_state_label(state: WhatsAppOperationState) -> &'static str {
    match state {
        WhatsAppOperationState::Accepted => "accepted",
        WhatsAppOperationState::Running => "running",
        WhatsAppOperationState::AwaitingProvider => "awaiting_provider",
        WhatsAppOperationState::HostClaimed => "host_claimed",
        WhatsAppOperationState::Completed => "completed",
        WhatsAppOperationState::Failed => "failed",
        WhatsAppOperationState::RetryScheduled => "retry_scheduled",
        WhatsAppOperationState::DeadLettered => "dead_lettered",
    }
}
