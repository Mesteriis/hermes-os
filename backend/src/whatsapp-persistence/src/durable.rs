use hermes_events_protocol::delivery::OutboxRecordV1;
use hermes_storage_protocol::StorageBindingV1;
use sqlx::{PgPool, Row, postgres::PgConnectOptions};

pub const WHATSAPP_SCHEMA_V1: &str = r#"
CREATE TABLE IF NOT EXISTS whatsapp_communications_outbox (
    message_id BYTEA PRIMARY KEY,
    envelope_sha256 BYTEA NOT NULL,
    exact_envelope_bytes BYTEA NOT NULL,
    created_at_unix_seconds BIGINT NOT NULL,
    published_at_unix_seconds BIGINT,
    CHECK (octet_length(message_id) = 16),
    CHECK (octet_length(envelope_sha256) = 32),
    CHECK (octet_length(exact_envelope_bytes) > 0)
);
CREATE INDEX IF NOT EXISTS whatsapp_communications_outbox_pending_idx
    ON whatsapp_communications_outbox (created_at_unix_seconds, message_id)
    WHERE published_at_unix_seconds IS NULL;
CREATE TABLE IF NOT EXISTS whatsapp_host_observations (
    account_id TEXT NOT NULL,
    provider_event_id TEXT NOT NULL,
    evidence_kind SMALLINT NOT NULL,
    observed_at_unix_seconds BIGINT NOT NULL,
    PRIMARY KEY (account_id, provider_event_id),
    CHECK (char_length(account_id) BETWEEN 1 AND 256),
    CHECK (char_length(provider_event_id) BETWEEN 1 AND 256),
    CHECK (evidence_kind BETWEEN 1 AND 11)
);
CREATE TABLE IF NOT EXISTS whatsapp_provider_commands (
    operation_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    exact_command_bytes BYTEA NOT NULL,
    state SMALLINT NOT NULL,
    host_claim_id TEXT,
    lease_expires_at_unix_seconds BIGINT,
    requested_at_unix_seconds BIGINT NOT NULL,
    completed_at_unix_seconds BIGINT,
    CHECK (char_length(operation_id) BETWEEN 1 AND 256),
    CHECK (char_length(account_id) BETWEEN 1 AND 256),
    CHECK (octet_length(exact_command_bytes) BETWEEN 1 AND 524288),
    CHECK (state BETWEEN 1 AND 4),
    CHECK ((state = 1 AND host_claim_id IS NULL AND lease_expires_at_unix_seconds IS NULL AND completed_at_unix_seconds IS NULL)
        OR (state = 2 AND host_claim_id IS NOT NULL AND lease_expires_at_unix_seconds IS NOT NULL AND completed_at_unix_seconds IS NULL)
        OR (state IN (3, 4) AND host_claim_id IS NOT NULL AND lease_expires_at_unix_seconds IS NOT NULL AND completed_at_unix_seconds IS NOT NULL))
);
CREATE INDEX IF NOT EXISTS whatsapp_provider_commands_claimable_idx
    ON whatsapp_provider_commands (account_id, requested_at_unix_seconds, operation_id)
    WHERE state IN (1, 2);
"#;

pub struct WhatsAppDurablePersistence {
    pool: PgPool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhatsAppDurablePersistenceError {
    Database,
    InvalidRow,
    ObservationConflict,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatsAppHostObservationRecordV1 {
    pub account_id: String,
    pub provider_event_id: String,
    pub evidence_kind: i16,
    pub observed_at_unix_seconds: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhatsAppProviderCommandStateV1 {
    Pending = 1,
    Claimed = 2,
    Succeeded = 3,
    Failed = 4,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatsAppClaimedCommandV1 {
    pub operation_id: String,
    pub account_id: String,
    pub exact_command_bytes: Vec<u8>,
}

impl WhatsAppDurablePersistence {
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
        Ok(Self { pool })
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

    pub async fn enqueue_communications_outbox(
        &self,
        record: &OutboxRecordV1,
        created_at_unix_seconds: i64,
    ) -> Result<(), WhatsAppDurablePersistenceError> {
        sqlx::query("INSERT INTO whatsapp_communications_outbox (message_id, envelope_sha256, exact_envelope_bytes, created_at_unix_seconds) VALUES ($1, $2, $3, $4) ON CONFLICT (message_id) DO NOTHING")
            .bind(record.message_id().as_slice())
            .bind(record.envelope_sha256().as_slice())
            .bind(record.exact_bytes())
            .bind(created_at_unix_seconds)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| WhatsAppDurablePersistenceError::Database)
    }

    pub async fn enqueue_provider_command(
        &self,
        operation_id: &str,
        account_id: &str,
        exact_command_bytes: &[u8],
        requested_at_unix_seconds: i64,
    ) -> Result<bool, WhatsAppDurablePersistenceError> {
        if operation_id.trim().is_empty()
            || account_id.trim().is_empty()
            || exact_command_bytes.is_empty()
            || exact_command_bytes.len() > 512 * 1024
            || requested_at_unix_seconds <= 0
        {
            return Err(WhatsAppDurablePersistenceError::InvalidRow);
        }
        sqlx::query("INSERT INTO whatsapp_provider_commands (operation_id, account_id, exact_command_bytes, state, requested_at_unix_seconds) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (operation_id) DO NOTHING")
            .bind(operation_id)
            .bind(account_id)
            .bind(exact_command_bytes)
            .bind(WhatsAppProviderCommandStateV1::Pending as i16)
            .bind(requested_at_unix_seconds)
            .execute(&self.pool)
            .await
            .map(|result| result.rows_affected() == 1)
            .map_err(|_| WhatsAppDurablePersistenceError::Database)
    }

    pub async fn claim_provider_commands(
        &self,
        account_id: &str,
        host_claim_id: &str,
        now_unix_seconds: i64,
        lease_seconds: i64,
        limit: i64,
    ) -> Result<Vec<WhatsAppClaimedCommandV1>, WhatsAppDurablePersistenceError> {
        if account_id.trim().is_empty()
            || host_claim_id.trim().is_empty()
            || now_unix_seconds <= 0
            || !(1..=300).contains(&lease_seconds)
        {
            return Err(WhatsAppDurablePersistenceError::InvalidRow);
        }
        let lease_expires_at_unix_seconds = now_unix_seconds
            .checked_add(lease_seconds)
            .ok_or(WhatsAppDurablePersistenceError::InvalidRow)?;
        let rows = sqlx::query(
            "WITH candidates AS (SELECT operation_id FROM whatsapp_provider_commands WHERE account_id = $1 AND (state = $2 OR (state = $3 AND lease_expires_at_unix_seconds < $4)) ORDER BY requested_at_unix_seconds ASC, operation_id ASC LIMIT $5 FOR UPDATE SKIP LOCKED) UPDATE whatsapp_provider_commands AS command SET state = $3, host_claim_id = $6, lease_expires_at_unix_seconds = $7 FROM candidates WHERE command.operation_id = candidates.operation_id RETURNING command.operation_id, command.account_id, command.exact_command_bytes",
        )
        .bind(account_id)
        .bind(WhatsAppProviderCommandStateV1::Pending as i16)
        .bind(WhatsAppProviderCommandStateV1::Claimed as i16)
        .bind(now_unix_seconds)
        .bind(limit.clamp(1, 64))
        .bind(host_claim_id)
        .bind(lease_expires_at_unix_seconds)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        rows.into_iter().map(|row| Ok(WhatsAppClaimedCommandV1 {
            operation_id: row.try_get("operation_id").map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?,
            account_id: row.try_get("account_id").map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?,
            exact_command_bytes: row.try_get("exact_command_bytes").map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?,
        })).collect()
    }

    pub async fn complete_provider_command(
        &self,
        operation_id: &str,
        account_id: &str,
        host_claim_id: &str,
        succeeded: bool,
        completed_at_unix_seconds: i64,
    ) -> Result<bool, WhatsAppDurablePersistenceError> {
        if operation_id.trim().is_empty()
            || account_id.trim().is_empty()
            || host_claim_id.trim().is_empty()
            || completed_at_unix_seconds <= 0
        {
            return Err(WhatsAppDurablePersistenceError::InvalidRow);
        }
        sqlx::query("UPDATE whatsapp_provider_commands SET state = $4, completed_at_unix_seconds = $5 WHERE operation_id = $1 AND account_id = $2 AND host_claim_id = $3 AND state = $6 AND lease_expires_at_unix_seconds >= $5")
            .bind(operation_id)
            .bind(account_id)
            .bind(host_claim_id)
            .bind(if succeeded { WhatsAppProviderCommandStateV1::Succeeded as i16 } else { WhatsAppProviderCommandStateV1::Failed as i16 })
            .bind(completed_at_unix_seconds)
            .bind(WhatsAppProviderCommandStateV1::Claimed as i16)
            .execute(&self.pool)
            .await
            .map(|result| result.rows_affected() == 1)
            .map_err(|_| WhatsAppDurablePersistenceError::Database)
    }

    pub async fn complete_provider_command_and_enqueue_observation(
        &self,
        operation_id: &str,
        account_id: &str,
        host_claim_id: &str,
        succeeded: bool,
        observation: &WhatsAppHostObservationRecordV1,
        record: &OutboxRecordV1,
        completed_at_unix_seconds: i64,
    ) -> Result<bool, WhatsAppDurablePersistenceError> {
        if operation_id.trim().is_empty()
            || account_id.trim().is_empty()
            || host_claim_id.trim().is_empty()
            || observation.account_id != account_id
            || completed_at_unix_seconds <= 0
        {
            return Err(WhatsAppDurablePersistenceError::InvalidRow);
        }
        let mut transaction = self.pool.begin().await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        let completed = sqlx::query("UPDATE whatsapp_provider_commands SET state = $4, completed_at_unix_seconds = $5 WHERE operation_id = $1 AND account_id = $2 AND host_claim_id = $3 AND state = $6 AND lease_expires_at_unix_seconds >= $5")
            .bind(operation_id)
            .bind(account_id)
            .bind(host_claim_id)
            .bind(if succeeded { WhatsAppProviderCommandStateV1::Succeeded as i16 } else { WhatsAppProviderCommandStateV1::Failed as i16 })
            .bind(completed_at_unix_seconds)
            .bind(WhatsAppProviderCommandStateV1::Claimed as i16)
            .execute(&mut *transaction)
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        if completed.rows_affected() != 1 {
            transaction.rollback().await.map_err(|_| WhatsAppDurablePersistenceError::Database)?;
            return Ok(false);
        }
        let inserted = sqlx::query("INSERT INTO whatsapp_host_observations (account_id, provider_event_id, evidence_kind, observed_at_unix_seconds) VALUES ($1, $2, $3, $4) ON CONFLICT (account_id, provider_event_id) DO NOTHING RETURNING account_id")
            .bind(&observation.account_id)
            .bind(&observation.provider_event_id)
            .bind(observation.evidence_kind)
            .bind(observation.observed_at_unix_seconds)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        if inserted.is_none() {
            return Err(WhatsAppDurablePersistenceError::ObservationConflict);
        }
        sqlx::query("INSERT INTO whatsapp_communications_outbox (message_id, envelope_sha256, exact_envelope_bytes, created_at_unix_seconds) VALUES ($1, $2, $3, $4) ON CONFLICT (message_id) DO NOTHING")
            .bind(record.message_id().as_slice())
            .bind(record.envelope_sha256().as_slice())
            .bind(record.exact_bytes())
            .bind(completed_at_unix_seconds)
            .execute(&mut *transaction)
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        transaction.commit().await.map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        Ok(true)
    }

    pub async fn record_host_observation_and_enqueue(
        &self,
        observation: &WhatsAppHostObservationRecordV1,
        record: &OutboxRecordV1,
        created_at_unix_seconds: i64,
    ) -> Result<bool, WhatsAppDurablePersistenceError> {
        if observation.account_id.trim().is_empty()
            || observation.provider_event_id.trim().is_empty()
            || !(1..=11).contains(&observation.evidence_kind)
        {
            return Err(WhatsAppDurablePersistenceError::InvalidRow);
        }
        let mut transaction = self.pool.begin().await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        let inserted = sqlx::query(
            "INSERT INTO whatsapp_host_observations (account_id, provider_event_id, evidence_kind, observed_at_unix_seconds) VALUES ($1, $2, $3, $4) ON CONFLICT (account_id, provider_event_id) DO NOTHING RETURNING account_id",
        )
        .bind(&observation.account_id)
        .bind(&observation.provider_event_id)
        .bind(observation.evidence_kind)
        .bind(observation.observed_at_unix_seconds)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        if inserted.is_none() {
            let row = sqlx::query(
                "SELECT evidence_kind, observed_at_unix_seconds FROM whatsapp_host_observations WHERE account_id = $1 AND provider_event_id = $2",
            )
            .bind(&observation.account_id)
            .bind(&observation.provider_event_id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
            let evidence_kind: i16 = row.try_get("evidence_kind")
                .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
            let observed_at_unix_seconds: i64 = row.try_get("observed_at_unix_seconds")
                .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
            if evidence_kind != observation.evidence_kind
                || observed_at_unix_seconds != observation.observed_at_unix_seconds
            {
                return Err(WhatsAppDurablePersistenceError::ObservationConflict);
            }
            transaction.commit().await
                .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
            return Ok(false);
        }
        sqlx::query("INSERT INTO whatsapp_communications_outbox (message_id, envelope_sha256, exact_envelope_bytes, created_at_unix_seconds) VALUES ($1, $2, $3, $4) ON CONFLICT (message_id) DO NOTHING")
            .bind(record.message_id().as_slice())
            .bind(record.envelope_sha256().as_slice())
            .bind(record.exact_bytes())
            .bind(created_at_unix_seconds)
            .execute(&mut *transaction)
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        transaction.commit().await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        Ok(true)
    }

    pub async fn pending_communications_outbox(
        &self,
        limit: i64,
    ) -> Result<Vec<OutboxRecordV1>, WhatsAppDurablePersistenceError> {
        let rows = sqlx::query("SELECT exact_envelope_bytes FROM whatsapp_communications_outbox WHERE published_at_unix_seconds IS NULL ORDER BY created_at_unix_seconds ASC, message_id ASC LIMIT $1")
            .bind(limit.clamp(1, 256))
            .fetch_all(&self.pool)
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        rows.into_iter()
            .map(|row| {
                let bytes: Vec<u8> = row
                    .try_get("exact_envelope_bytes")
                    .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
                OutboxRecordV1::accept(bytes)
                    .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)
            })
            .collect()
    }

    pub async fn mark_communications_outbox_published(
        &self,
        message_id: &[u8; 16],
        published_at_unix_seconds: i64,
    ) -> Result<bool, WhatsAppDurablePersistenceError> {
        sqlx::query("UPDATE whatsapp_communications_outbox SET published_at_unix_seconds = $2 WHERE message_id = $1 AND published_at_unix_seconds IS NULL")
            .bind(message_id.as_slice())
            .bind(published_at_unix_seconds)
            .execute(&self.pool)
            .await
            .map(|result| result.rows_affected() == 1)
            .map_err(|_| WhatsAppDurablePersistenceError::Database)
    }
}
