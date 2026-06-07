use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::migrate::Migrator;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};
use thiserror::Error;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[derive(Clone)]
pub struct EventStore {
    pool: PgPool,
}

impl EventStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn append(&self, event: &NewEventEnvelope) -> Result<i64, EventStoreError> {
        let mut transaction = self.pool.begin().await?;
        let position = Self::append_in_transaction(&mut transaction, event).await?;
        transaction.commit().await?;

        Ok(position)
    }

    pub async fn append_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        event: &NewEventEnvelope,
    ) -> Result<i64, EventStoreError> {
        let position = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO event_log (
                event_id,
                event_type,
                schema_version,
                occurred_at,
                source,
                actor,
                subject,
                payload,
                provenance,
                causation_id,
                correlation_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING position
            "#,
        )
        .bind(&event.event_id)
        .bind(&event.event_type)
        .bind(event.schema_version)
        .bind(event.occurred_at)
        .bind(&event.source)
        .bind(&event.actor)
        .bind(&event.subject)
        .bind(&event.payload)
        .bind(&event.provenance)
        .bind(&event.causation_id)
        .bind(&event.correlation_id)
        .fetch_one(&mut **transaction)
        .await?;

        Ok(position)
    }

    pub async fn get_by_id(
        &self,
        event_id: &str,
    ) -> Result<Option<EventEnvelope>, EventStoreError> {
        let row = sqlx::query(
            r#"
            SELECT
                event_id,
                event_type,
                schema_version,
                occurred_at,
                recorded_at,
                source,
                actor,
                subject,
                payload,
                provenance,
                causation_id,
                correlation_id
            FROM event_log
            WHERE event_id = $1
            "#,
        )
        .bind(event_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_event).transpose()
    }

    pub async fn list_after_position(
        &self,
        after_position: i64,
        limit: u32,
    ) -> Result<Vec<StoredEventEnvelope>, EventStoreError> {
        if after_position < 0 {
            return Err(EventStoreError::InvalidReplayPosition(after_position));
        }

        let limit = i64::from(limit.clamp(1, 1000));
        let rows = sqlx::query(
            r#"
            SELECT
                position,
                event_id,
                event_type,
                schema_version,
                occurred_at,
                recorded_at,
                source,
                actor,
                subject,
                payload,
                provenance,
                causation_id,
                correlation_id
            FROM event_log
            WHERE position > $1
            ORDER BY position ASC
            LIMIT $2
            "#,
        )
        .bind(after_position)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_stored_event).collect()
    }
}

#[derive(Clone)]
pub struct ProjectionCursorStore {
    pool: PgPool,
}

impl ProjectionCursorStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn last_processed_position(
        &self,
        projection_name: &str,
    ) -> Result<i64, EventStoreError> {
        validate_non_empty("projection_name", projection_name)?;

        let position = sqlx::query_scalar::<_, Option<i64>>(
            r#"
            SELECT last_processed_position
            FROM projection_cursors
            WHERE projection_name = $1
            "#,
        )
        .bind(projection_name.trim())
        .fetch_optional(&self.pool)
        .await?;

        Ok(position.flatten().unwrap_or(0))
    }

    pub async fn save_position(
        &self,
        projection_name: &str,
        position: i64,
    ) -> Result<i64, EventStoreError> {
        validate_non_empty("projection_name", projection_name)?;
        if position < 0 {
            return Err(EventStoreError::InvalidReplayPosition(position));
        }

        let saved_position = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO projection_cursors (
                projection_name,
                last_processed_position,
                updated_at
            )
            VALUES ($1, $2, now())
            ON CONFLICT (projection_name)
            DO UPDATE SET
                last_processed_position = GREATEST(
                    projection_cursors.last_processed_position,
                    EXCLUDED.last_processed_position
                ),
                updated_at = now()
            RETURNING last_processed_position
            "#,
        )
        .bind(projection_name.trim())
        .bind(position)
        .fetch_one(&self.pool)
        .await?;

        Ok(saved_position)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EventEnvelope {
    pub event_id: String,
    pub event_type: String,
    pub schema_version: i32,
    pub occurred_at: DateTime<Utc>,
    pub recorded_at: DateTime<Utc>,
    pub source: Value,
    pub actor: Option<Value>,
    pub subject: Value,
    pub payload: Value,
    pub provenance: Value,
    pub causation_id: Option<String>,
    pub correlation_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StoredEventEnvelope {
    pub position: i64,
    pub event: EventEnvelope,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewEventEnvelope {
    pub event_id: String,
    pub event_type: String,
    pub schema_version: i32,
    pub occurred_at: DateTime<Utc>,
    pub source: Value,
    pub actor: Option<Value>,
    pub subject: Value,
    pub payload: Value,
    pub provenance: Value,
    pub causation_id: Option<String>,
    pub correlation_id: Option<String>,
}

impl NewEventEnvelope {
    pub fn builder(
        event_id: impl Into<String>,
        event_type: impl Into<String>,
        occurred_at: DateTime<Utc>,
        source: Value,
        subject: Value,
    ) -> NewEventEnvelopeBuilder {
        NewEventEnvelopeBuilder {
            event_id: event_id.into(),
            event_type: event_type.into(),
            schema_version: 1,
            occurred_at,
            source,
            actor: None,
            subject,
            payload: json!({}),
            provenance: json!({}),
            causation_id: None,
            correlation_id: None,
        }
    }
}

pub struct NewEventEnvelopeBuilder {
    event_id: String,
    event_type: String,
    schema_version: i32,
    occurred_at: DateTime<Utc>,
    source: Value,
    actor: Option<Value>,
    subject: Value,
    payload: Value,
    provenance: Value,
    causation_id: Option<String>,
    correlation_id: Option<String>,
}

impl NewEventEnvelopeBuilder {
    pub fn schema_version(mut self, schema_version: i32) -> Self {
        self.schema_version = schema_version;
        self
    }

    pub fn actor(mut self, actor: Value) -> Self {
        self.actor = Some(actor);
        self
    }

    pub fn payload(mut self, payload: Value) -> Self {
        self.payload = payload;
        self
    }

    pub fn provenance(mut self, provenance: Value) -> Self {
        self.provenance = provenance;
        self
    }

    pub fn correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    pub fn causation_id(mut self, causation_id: impl Into<String>) -> Self {
        self.causation_id = Some(causation_id.into());
        self
    }

    pub fn build(self) -> Result<NewEventEnvelope, EventEnvelopeError> {
        validate_non_empty("event_id", &self.event_id)?;
        validate_non_empty("event_type", &self.event_type)?;

        if self.schema_version <= 0 {
            return Err(EventEnvelopeError::InvalidSchemaVersion);
        }

        validate_object("source", &self.source)?;
        validate_object("subject", &self.subject)?;
        validate_object("payload", &self.payload)?;
        validate_object("provenance", &self.provenance)?;

        if let Some(actor) = &self.actor {
            validate_object("actor", actor)?;
        }

        Ok(NewEventEnvelope {
            event_id: self.event_id.trim().to_owned(),
            event_type: self.event_type.trim().to_owned(),
            schema_version: self.schema_version,
            occurred_at: self.occurred_at,
            source: self.source,
            actor: self.actor,
            subject: self.subject,
            payload: self.payload,
            provenance: self.provenance,
            causation_id: self.causation_id,
            correlation_id: self.correlation_id,
        })
    }
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), EventStoreError> {
    MIGRATOR.run(pool).await?;
    Ok(())
}

pub fn expected_migration_summary() -> MigrationSummary {
    let mut count = 0;
    let mut latest_version = 0;

    for migration in MIGRATOR.iter() {
        count += 1;
        latest_version = latest_version.max(migration.version);
    }

    MigrationSummary {
        count,
        latest_version,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MigrationSummary {
    pub count: i64,
    pub latest_version: i64,
}

fn row_to_event(row: PgRow) -> Result<EventEnvelope, EventStoreError> {
    Ok(EventEnvelope {
        event_id: row.try_get("event_id")?,
        event_type: row.try_get("event_type")?,
        schema_version: row.try_get("schema_version")?,
        occurred_at: row.try_get("occurred_at")?,
        recorded_at: row.try_get("recorded_at")?,
        source: row.try_get("source")?,
        actor: row.try_get("actor")?,
        subject: row.try_get("subject")?,
        payload: row.try_get("payload")?,
        provenance: row.try_get("provenance")?,
        causation_id: row.try_get("causation_id")?,
        correlation_id: row.try_get("correlation_id")?,
    })
}

fn row_to_stored_event(row: PgRow) -> Result<StoredEventEnvelope, EventStoreError> {
    Ok(StoredEventEnvelope {
        position: row.try_get("position")?,
        event: row_to_event(row)?,
    })
}

fn validate_non_empty(field_name: &'static str, value: &str) -> Result<(), EventEnvelopeError> {
    if value.trim().is_empty() {
        return Err(EventEnvelopeError::EmptyField(field_name));
    }

    Ok(())
}

fn validate_object(field_name: &'static str, value: &Value) -> Result<(), EventEnvelopeError> {
    if !value.is_object() {
        return Err(EventEnvelopeError::NonObjectJson(field_name));
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum EventEnvelopeError {
    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("schema_version must be positive")]
    InvalidSchemaVersion,

    #[error("{0} must be a JSON object")]
    NonObjectJson(&'static str),
}

#[derive(Debug, Error)]
pub enum EventStoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error(transparent)]
    Envelope(#[from] EventEnvelopeError),

    #[error("replay position must be non-negative, got {0}")]
    InvalidReplayPosition(i64),
}

impl EventStoreError {
    pub fn is_unique_violation(&self) -> bool {
        match self {
            Self::Sqlx(sqlx::Error::Database(error)) => error.code().as_deref() == Some("23505"),
            _ => false,
        }
    }
}
