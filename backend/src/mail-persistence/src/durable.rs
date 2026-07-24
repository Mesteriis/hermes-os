//! Mail-owned durable storage. It never reads or mutates Communications state.

use hermes_events_protocol::delivery::OutboxRecordV1;
use hermes_storage_protocol::StorageBindingV1;
use sqlx::{PgPool, Row, postgres::PgConnectOptions};

pub const MAIL_SCHEMA_V1: &str = r#"
CREATE TABLE IF NOT EXISTS mail_communications_outbox (
    message_id BYTEA PRIMARY KEY,
    envelope_sha256 BYTEA NOT NULL,
    exact_envelope_bytes BYTEA NOT NULL,
    created_at_unix_seconds BIGINT NOT NULL,
    published_at_unix_seconds BIGINT,
    CHECK (octet_length(message_id) = 16),
    CHECK (octet_length(envelope_sha256) = 32),
    CHECK (octet_length(exact_envelope_bytes) > 0)
);
CREATE INDEX IF NOT EXISTS mail_communications_outbox_pending_idx
    ON mail_communications_outbox (created_at_unix_seconds, message_id)
    WHERE published_at_unix_seconds IS NULL;
CREATE TABLE IF NOT EXISTS mail_delivery_attempts (
    operation_id TEXT PRIMARY KEY,
    connection_id TEXT NOT NULL,
    rfc822_sha256 BYTEA NOT NULL,
    state SMALLINT NOT NULL,
    attempted_at_unix_seconds BIGINT NOT NULL,
    completed_at_unix_seconds BIGINT,
    response_code SMALLINT,
    CHECK (operation_id <> ''),
    CHECK (connection_id <> ''),
    CHECK (octet_length(rfc822_sha256) = 32),
    CHECK (state IN (1, 2, 3)),
    CHECK ((state = 1 AND completed_at_unix_seconds IS NULL AND response_code IS NULL)
        OR (state = 2 AND completed_at_unix_seconds IS NOT NULL AND response_code BETWEEN 200 AND 299)
        OR (state = 3 AND completed_at_unix_seconds IS NOT NULL AND response_code IS NULL))
);
CREATE INDEX IF NOT EXISTS mail_delivery_attempts_unresolved_idx
    ON mail_delivery_attempts (attempted_at_unix_seconds, operation_id)
    WHERE state = 1;
CREATE TABLE IF NOT EXISTS mail_gmail_sync_cursors (
    connection_id TEXT PRIMARY KEY,
    next_page_token TEXT NOT NULL,
    observed_history_id TEXT,
    updated_at_unix_seconds BIGINT NOT NULL,
    CHECK (connection_id <> ''),
    CHECK (next_page_token <> ''),
    CHECK (observed_history_id IS NULL OR observed_history_id <> ''),
    CHECK (updated_at_unix_seconds > 0)
);
CREATE TABLE IF NOT EXISTS mail_gmail_history_checkpoints (
    connection_id TEXT PRIMARY KEY,
    start_history_id TEXT NOT NULL,
    next_page_token TEXT,
    updated_at_unix_seconds BIGINT NOT NULL,
    CHECK (connection_id <> ''),
    CHECK (start_history_id <> ''),
    CHECK (next_page_token IS NULL OR next_page_token <> ''),
    CHECK (updated_at_unix_seconds > 0)
);
CREATE TABLE IF NOT EXISTS mail_gmail_oauth_credential_bindings (
    connection_id TEXT PRIMARY KEY,
    access_token_record_id BYTEA NOT NULL,
    access_token_revision BIGINT NOT NULL,
    refresh_credential_record_id BYTEA NOT NULL,
    refresh_credential_revision BIGINT NOT NULL,
    updated_at_unix_seconds BIGINT NOT NULL,
    CHECK (connection_id <> ''),
    CHECK (octet_length(access_token_record_id) = 16),
    CHECK (access_token_revision > 0),
    CHECK (octet_length(refresh_credential_record_id) = 16),
    CHECK (refresh_credential_revision > 0),
    CHECK (updated_at_unix_seconds > 0)
);
"#;

pub struct MailDurablePersistence {
    pool: PgPool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailDurablePersistenceError {
    Database,
    InvalidRow,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailOAuthCredentialBindingV1 {
    pub access_token_record_id: [u8; 16],
    pub access_token_revision: u64,
    pub refresh_credential_record_id: [u8; 16],
    pub refresh_credential_revision: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i16)]
pub enum MailSmtpDeliveryAttemptStateV1 {
    OutcomeUnknown = 1,
    Accepted = 2,
    Rejected = 3,
}

impl MailDurablePersistence {
    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, MailDurablePersistenceError> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || binding.access().runtime_principal().is_empty()
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
        {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let port =
            u16::try_from(pgbouncer_port).map_err(|_| MailDurablePersistenceError::InvalidRow)?;
        let options = PgConnectOptions::new()
            .host(pgbouncer_host)
            .port(port)
            .username(binding.access().runtime_principal())
            .password(password)
            .database(database_id);
        let pool = PgPool::connect_with(options)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        Ok(Self { pool })
    }

    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn initialize(&self) -> Result<(), MailDurablePersistenceError> {
        sqlx::raw_sql(MAIL_SCHEMA_V1)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| MailDurablePersistenceError::Database)
    }

    pub async fn enqueue_communications_outbox(
        &self,
        record: &OutboxRecordV1,
        created_at_unix_seconds: i64,
    ) -> Result<(), MailDurablePersistenceError> {
        sqlx::query("INSERT INTO mail_communications_outbox (message_id, envelope_sha256, exact_envelope_bytes, created_at_unix_seconds) VALUES ($1, $2, $3, $4) ON CONFLICT (message_id) DO NOTHING")
            .bind(record.message_id().as_slice())
            .bind(record.envelope_sha256().as_slice())
            .bind(record.exact_bytes())
            .bind(created_at_unix_seconds)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| MailDurablePersistenceError::Database)
    }

    pub async fn gmail_sync_progress(
        &self,
        connection_id: &str,
    ) -> Result<Option<(String, Option<String>)>, MailDurablePersistenceError> {
        if connection_id.trim().is_empty() {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        sqlx::query("SELECT next_page_token, observed_history_id FROM mail_gmail_sync_cursors WHERE connection_id = $1")
            .bind(connection_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?
            .map(|row| Ok((
                row.try_get("next_page_token").map_err(|_| MailDurablePersistenceError::InvalidRow)?,
                row.try_get("observed_history_id").map_err(|_| MailDurablePersistenceError::InvalidRow)?,
            )))
            .transpose()
    }

    pub async fn gmail_history_checkpoint(
        &self,
        connection_id: &str,
    ) -> Result<Option<(String, Option<String>)>, MailDurablePersistenceError> {
        if connection_id.trim().is_empty() {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        sqlx::query("SELECT start_history_id, next_page_token FROM mail_gmail_history_checkpoints WHERE connection_id = $1")
            .bind(connection_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?
            .map(|row| Ok((
                row.try_get("start_history_id").map_err(|_| MailDurablePersistenceError::InvalidRow)?,
                row.try_get("next_page_token").map_err(|_| MailDurablePersistenceError::InvalidRow)?,
            )))
            .transpose()
    }

    pub async fn gmail_oauth_credential_binding(
        &self,
        connection_id: &str,
    ) -> Result<Option<GmailOAuthCredentialBindingV1>, MailDurablePersistenceError> {
        if connection_id.trim().is_empty() {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        sqlx::query("SELECT access_token_record_id, access_token_revision, refresh_credential_record_id, refresh_credential_revision FROM mail_gmail_oauth_credential_bindings WHERE connection_id = $1")
            .bind(connection_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?
            .map(|row| {
                let access_token_record_id: Vec<u8> = row.try_get("access_token_record_id").map_err(|_| MailDurablePersistenceError::InvalidRow)?;
                let refresh_credential_record_id: Vec<u8> = row.try_get("refresh_credential_record_id").map_err(|_| MailDurablePersistenceError::InvalidRow)?;
                Ok(GmailOAuthCredentialBindingV1 {
                    access_token_record_id: access_token_record_id.as_slice().try_into().map_err(|_| MailDurablePersistenceError::InvalidRow)?,
                    access_token_revision: u64::try_from(row.try_get::<i64, _>("access_token_revision").map_err(|_| MailDurablePersistenceError::InvalidRow)?).map_err(|_| MailDurablePersistenceError::InvalidRow)?,
                    refresh_credential_record_id: refresh_credential_record_id.as_slice().try_into().map_err(|_| MailDurablePersistenceError::InvalidRow)?,
                    refresh_credential_revision: u64::try_from(row.try_get::<i64, _>("refresh_credential_revision").map_err(|_| MailDurablePersistenceError::InvalidRow)?).map_err(|_| MailDurablePersistenceError::InvalidRow)?,
                })
            })
            .transpose()
    }

    pub async fn store_gmail_oauth_credential_binding(
        &self,
        connection_id: &str,
        binding: &GmailOAuthCredentialBindingV1,
        updated_at_unix_seconds: i64,
    ) -> Result<(), MailDurablePersistenceError> {
        if connection_id.trim().is_empty()
            || binding.access_token_record_id.iter().all(|byte| *byte == 0)
            || binding.access_token_revision == 0
            || binding
                .refresh_credential_record_id
                .iter()
                .all(|byte| *byte == 0)
            || binding.refresh_credential_revision == 0
            || updated_at_unix_seconds <= 0
        {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        sqlx::query("INSERT INTO mail_gmail_oauth_credential_bindings (connection_id, access_token_record_id, access_token_revision, refresh_credential_record_id, refresh_credential_revision, updated_at_unix_seconds) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (connection_id) DO UPDATE SET access_token_record_id = EXCLUDED.access_token_record_id, access_token_revision = EXCLUDED.access_token_revision, refresh_credential_record_id = EXCLUDED.refresh_credential_record_id, refresh_credential_revision = EXCLUDED.refresh_credential_revision, updated_at_unix_seconds = EXCLUDED.updated_at_unix_seconds")
            .bind(connection_id)
            .bind(binding.access_token_record_id.as_slice())
            .bind(i64::try_from(binding.access_token_revision).map_err(|_| MailDurablePersistenceError::InvalidRow)?)
            .bind(binding.refresh_credential_record_id.as_slice())
            .bind(i64::try_from(binding.refresh_credential_revision).map_err(|_| MailDurablePersistenceError::InvalidRow)?)
            .bind(updated_at_unix_seconds)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| MailDurablePersistenceError::Database)
    }

    pub async fn enqueue_communications_outbox_and_store_gmail_sync_progress(
        &self,
        records: &[OutboxRecordV1],
        connection_id: &str,
        next_page_token: Option<&str>,
        observed_history_id: Option<&str>,
        updated_at_unix_seconds: i64,
    ) -> Result<(), MailDurablePersistenceError> {
        if connection_id.trim().is_empty()
            || updated_at_unix_seconds <= 0
            || next_page_token.is_some_and(|token| token.trim().is_empty())
            || observed_history_id.is_some_and(|history_id| history_id.trim().is_empty())
        {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        for record in records {
            sqlx::query("INSERT INTO mail_communications_outbox (message_id, envelope_sha256, exact_envelope_bytes, created_at_unix_seconds) VALUES ($1, $2, $3, $4) ON CONFLICT (message_id) DO NOTHING")
                .bind(record.message_id().as_slice())
                .bind(record.envelope_sha256().as_slice())
                .bind(record.exact_bytes())
                .bind(updated_at_unix_seconds)
                .execute(&mut *transaction)
                .await
                .map_err(|_| MailDurablePersistenceError::Database)?;
        }
        if let Some(next_page_token) = next_page_token {
            sqlx::query("INSERT INTO mail_gmail_sync_cursors (connection_id, next_page_token, observed_history_id, updated_at_unix_seconds) VALUES ($1, $2, $3, $4) ON CONFLICT (connection_id) DO UPDATE SET next_page_token = EXCLUDED.next_page_token, observed_history_id = EXCLUDED.observed_history_id, updated_at_unix_seconds = EXCLUDED.updated_at_unix_seconds")
                .bind(connection_id)
                .bind(next_page_token)
                .bind(observed_history_id)
                .bind(updated_at_unix_seconds)
                .execute(&mut *transaction)
                .await
                .map_err(|_| MailDurablePersistenceError::Database)?;
        } else if let Some(observed_history_id) = observed_history_id {
            sqlx::query("DELETE FROM mail_gmail_sync_cursors WHERE connection_id = $1")
                .bind(connection_id)
                .execute(&mut *transaction)
                .await
                .map_err(|_| MailDurablePersistenceError::Database)?;
            sqlx::query("INSERT INTO mail_gmail_history_checkpoints (connection_id, start_history_id, next_page_token, updated_at_unix_seconds) VALUES ($1, $2, NULL, $3) ON CONFLICT (connection_id) DO UPDATE SET start_history_id = EXCLUDED.start_history_id, next_page_token = NULL, updated_at_unix_seconds = EXCLUDED.updated_at_unix_seconds")
                .bind(connection_id)
                .bind(observed_history_id)
                .bind(updated_at_unix_seconds)
                .execute(&mut *transaction)
                .await
                .map_err(|_| MailDurablePersistenceError::Database)?;
        } else {
            sqlx::query("DELETE FROM mail_gmail_sync_cursors WHERE connection_id = $1")
                .bind(connection_id)
                .execute(&mut *transaction)
                .await
                .map_err(|_| MailDurablePersistenceError::Database)?;
        }
        transaction
            .commit()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)
    }

    pub async fn enqueue_communications_outbox_and_store_gmail_history_checkpoint(
        &self,
        records: &[OutboxRecordV1],
        connection_id: &str,
        start_history_id: &str,
        next_page_token: Option<&str>,
        updated_at_unix_seconds: i64,
    ) -> Result<(), MailDurablePersistenceError> {
        if connection_id.trim().is_empty()
            || start_history_id.trim().is_empty()
            || updated_at_unix_seconds <= 0
            || next_page_token.is_some_and(|token| token.trim().is_empty())
        {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        for record in records {
            sqlx::query("INSERT INTO mail_communications_outbox (message_id, envelope_sha256, exact_envelope_bytes, created_at_unix_seconds) VALUES ($1, $2, $3, $4) ON CONFLICT (message_id) DO NOTHING")
                .bind(record.message_id().as_slice())
                .bind(record.envelope_sha256().as_slice())
                .bind(record.exact_bytes())
                .bind(updated_at_unix_seconds)
                .execute(&mut *transaction)
                .await
                .map_err(|_| MailDurablePersistenceError::Database)?;
        }
        sqlx::query("INSERT INTO mail_gmail_history_checkpoints (connection_id, start_history_id, next_page_token, updated_at_unix_seconds) VALUES ($1, $2, $3, $4) ON CONFLICT (connection_id) DO UPDATE SET start_history_id = EXCLUDED.start_history_id, next_page_token = EXCLUDED.next_page_token, updated_at_unix_seconds = EXCLUDED.updated_at_unix_seconds")
            .bind(connection_id)
            .bind(start_history_id)
            .bind(next_page_token)
            .bind(updated_at_unix_seconds)
            .execute(&mut *transaction)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        transaction
            .commit()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)
    }

    pub async fn clear_gmail_history_checkpoint(
        &self,
        connection_id: &str,
    ) -> Result<(), MailDurablePersistenceError> {
        if connection_id.trim().is_empty() {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        sqlx::query("DELETE FROM mail_gmail_history_checkpoints WHERE connection_id = $1")
            .bind(connection_id)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| MailDurablePersistenceError::Database)
    }

    pub async fn begin_delivery_attempt(
        &self,
        operation_id: &str,
        connection_id: &str,
        rfc822_sha256: &[u8; 32],
        attempted_at_unix_seconds: i64,
    ) -> Result<bool, MailDurablePersistenceError> {
        if operation_id.trim().is_empty()
            || connection_id.trim().is_empty()
            || attempted_at_unix_seconds <= 0
        {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        sqlx::query("INSERT INTO mail_delivery_attempts (operation_id, connection_id, rfc822_sha256, state, attempted_at_unix_seconds) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (operation_id) DO NOTHING")
            .bind(operation_id)
            .bind(connection_id)
            .bind(rfc822_sha256.as_slice())
            .bind(MailSmtpDeliveryAttemptStateV1::OutcomeUnknown as i16)
            .bind(attempted_at_unix_seconds)
            .execute(&self.pool)
            .await
            .map(|result| result.rows_affected() == 1)
            .map_err(|_| MailDurablePersistenceError::Database)
    }

    pub async fn complete_delivery_accepted(
        &self,
        operation_id: &str,
        rfc822_sha256: &[u8; 32],
        response_code: u16,
        record: &OutboxRecordV1,
        completed_at_unix_seconds: i64,
    ) -> Result<(), MailDurablePersistenceError> {
        if operation_id.trim().is_empty()
            || !(200..300).contains(&response_code)
            || completed_at_unix_seconds <= 0
        {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        let updated = sqlx::query("UPDATE mail_delivery_attempts SET state = $3, completed_at_unix_seconds = $4, response_code = $5 WHERE operation_id = $1 AND rfc822_sha256 = $2 AND state = $6")
            .bind(operation_id)
            .bind(rfc822_sha256.as_slice())
            .bind(MailSmtpDeliveryAttemptStateV1::Accepted as i16)
            .bind(completed_at_unix_seconds)
            .bind(i16::try_from(response_code).map_err(|_| MailDurablePersistenceError::InvalidRow)?)
            .bind(MailSmtpDeliveryAttemptStateV1::OutcomeUnknown as i16)
            .execute(&mut *transaction)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        if updated.rows_affected() != 1 {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        sqlx::query("INSERT INTO mail_communications_outbox (message_id, envelope_sha256, exact_envelope_bytes, created_at_unix_seconds) VALUES ($1, $2, $3, $4) ON CONFLICT (message_id) DO NOTHING")
            .bind(record.message_id().as_slice())
            .bind(record.envelope_sha256().as_slice())
            .bind(record.exact_bytes())
            .bind(completed_at_unix_seconds)
            .execute(&mut *transaction)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        transaction
            .commit()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)
    }

    pub async fn complete_delivery_rejected(
        &self,
        operation_id: &str,
        rfc822_sha256: &[u8; 32],
        completed_at_unix_seconds: i64,
    ) -> Result<(), MailDurablePersistenceError> {
        if operation_id.trim().is_empty() || completed_at_unix_seconds <= 0 {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        sqlx::query("UPDATE mail_delivery_attempts SET state = $3, completed_at_unix_seconds = $4 WHERE operation_id = $1 AND rfc822_sha256 = $2 AND state = $5")
            .bind(operation_id)
            .bind(rfc822_sha256.as_slice())
            .bind(MailSmtpDeliveryAttemptStateV1::Rejected as i16)
            .bind(completed_at_unix_seconds)
            .bind(MailSmtpDeliveryAttemptStateV1::OutcomeUnknown as i16)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| MailDurablePersistenceError::Database)
    }

    pub async fn pending_communications_outbox(
        &self,
        limit: i64,
    ) -> Result<Vec<OutboxRecordV1>, MailDurablePersistenceError> {
        let rows = sqlx::query("SELECT exact_envelope_bytes FROM mail_communications_outbox WHERE published_at_unix_seconds IS NULL ORDER BY created_at_unix_seconds ASC, message_id ASC LIMIT $1")
            .bind(limit.clamp(1, 256))
            .fetch_all(&self.pool)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        rows.into_iter()
            .map(|row| {
                let bytes: Vec<u8> = row
                    .try_get("exact_envelope_bytes")
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
                OutboxRecordV1::accept(bytes).map_err(|_| MailDurablePersistenceError::InvalidRow)
            })
            .collect()
    }

    pub async fn mark_communications_outbox_published(
        &self,
        message_id: &[u8; 16],
        published_at_unix_seconds: i64,
    ) -> Result<bool, MailDurablePersistenceError> {
        sqlx::query("UPDATE mail_communications_outbox SET published_at_unix_seconds = $2 WHERE message_id = $1 AND published_at_unix_seconds IS NULL")
            .bind(message_id.as_slice())
            .bind(published_at_unix_seconds)
            .execute(&self.pool)
            .await
            .map(|result| result.rows_affected() == 1)
            .map_err(|_| MailDurablePersistenceError::Database)
    }
}
