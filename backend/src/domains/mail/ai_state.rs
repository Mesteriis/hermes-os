use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};
use thiserror::Error;

use crate::platform::events::{EventStore, NewEventEnvelope};
use crate::platform::observations::ObservationStoreError;

const EVENT_TYPE_CHANGED: &str = "mail.ai_state.changed";
use super::evidence::link_mail_entity_in_transaction;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MailAiState {
    New,
    Processing,
    Processed,
    ReviewRequired,
    Failed,
    Archived,
}

impl MailAiState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::New => "NEW",
            Self::Processing => "PROCESSING",
            Self::Processed => "PROCESSED",
            Self::ReviewRequired => "REVIEW_REQUIRED",
            Self::Failed => "FAILED",
            Self::Archived => "ARCHIVED",
        }
    }
}

impl TryFrom<&str> for MailAiState {
    type Error = MailAiStateError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "NEW" => Ok(Self::New),
            "PROCESSING" => Ok(Self::Processing),
            "PROCESSED" => Ok(Self::Processed),
            "REVIEW_REQUIRED" => Ok(Self::ReviewRequired),
            "FAILED" => Ok(Self::Failed),
            "ARCHIVED" => Ok(Self::Archived),
            _ => Err(MailAiStateError::Invalid("ai_state")),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct MailAiStateRecord {
    pub message_id: String,
    pub ai_state: MailAiState,
    pub review_reason: Option<String>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MailAiStateTransitionRequest {
    pub ai_state: MailAiState,
    pub review_reason: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Clone)]
pub struct MailAiStateStore {
    pool: PgPool,
}

impl MailAiStateStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn current(
        &self,
        message_id: &str,
    ) -> Result<Option<MailAiStateRecord>, MailAiStateError> {
        let message_id = normalize_required("message_id", message_id)?;
        let row = sqlx::query(
            r#"
            SELECT
                m.message_id,
                COALESCE(s.ai_state, 'NEW') AS ai_state,
                s.review_reason,
                s.last_error,
                COALESCE(s.created_at, m.projected_at) AS created_at,
                COALESCE(s.updated_at, m.projected_at) AS updated_at
            FROM communication_messages m
            LEFT JOIN mail_ai_states s ON s.message_id = m.message_id
            WHERE m.message_id = $1
            "#,
        )
        .bind(&message_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_ai_state).transpose()
    }

    pub async fn transition(
        &self,
        message_id: &str,
        request: MailAiStateTransitionRequest,
    ) -> Result<Option<MailAiStateRecord>, MailAiStateError> {
        self.transition_with_observation(message_id, request, None, "ai_state_transition", None)
            .await
    }

    pub async fn transition_with_observation(
        &self,
        message_id: &str,
        request: MailAiStateTransitionRequest,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<Option<MailAiStateRecord>, MailAiStateError> {
        let message_id = normalize_required("message_id", message_id)?;
        let update = NormalizedMailAiStateTransition::from_request(request)?;
        let mut transaction = self.pool.begin().await?;

        let Some(previous) = select_current_ai_state(&mut transaction, &message_id).await? else {
            transaction.rollback().await?;
            return Ok(None);
        };

        let row = sqlx::query(
            r#"
            INSERT INTO mail_ai_states (message_id, ai_state, review_reason, last_error)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (message_id)
            DO UPDATE SET
                ai_state = EXCLUDED.ai_state,
                review_reason = EXCLUDED.review_reason,
                last_error = EXCLUDED.last_error,
                updated_at = now()
            RETURNING message_id, ai_state, review_reason, last_error, created_at, updated_at
            "#,
        )
        .bind(&message_id)
        .bind(update.ai_state.as_str())
        .bind(update.review_reason.as_deref())
        .bind(update.last_error.as_deref())
        .fetch_one(&mut *transaction)
        .await?;
        let record = row_to_ai_state(row)?;
        let event = ai_state_changed_event(&record, previous.ai_state)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_mail_entity_in_transaction(
                &mut transaction,
                observation_id,
                "communication_message",
                record.message_id.clone(),
                relationship_kind,
                json!({
                    "previous_ai_state": previous.ai_state.as_str(),
                    "ai_state": record.ai_state.as_str(),
                }),
                metadata,
            )
            .await?;
        }
        transaction.commit().await?;

        Ok(Some(record))
    }
}

#[derive(Debug)]
struct NormalizedMailAiStateTransition {
    ai_state: MailAiState,
    review_reason: Option<String>,
    last_error: Option<String>,
}

impl NormalizedMailAiStateTransition {
    fn from_request(request: MailAiStateTransitionRequest) -> Result<Self, MailAiStateError> {
        let review_reason = normalize_optional(request.review_reason)?;
        let last_error = normalize_optional(request.last_error)?;

        match request.ai_state {
            MailAiState::ReviewRequired if review_reason.is_none() => {
                return Err(MailAiStateError::Invalid("review_reason"));
            }
            MailAiState::Failed if last_error.is_none() => {
                return Err(MailAiStateError::Invalid("last_error"));
            }
            _ => {}
        }

        Ok(Self {
            ai_state: request.ai_state,
            review_reason: if request.ai_state == MailAiState::ReviewRequired {
                review_reason
            } else {
                None
            },
            last_error: if request.ai_state == MailAiState::Failed {
                last_error
            } else {
                None
            },
        })
    }
}

async fn select_current_ai_state(
    transaction: &mut Transaction<'_, Postgres>,
    message_id: &str,
) -> Result<Option<MailAiStateRecord>, MailAiStateError> {
    let row = sqlx::query(
        r#"
        SELECT
            m.message_id,
            COALESCE(s.ai_state, 'NEW') AS ai_state,
            s.review_reason,
            s.last_error,
            COALESCE(s.created_at, m.projected_at) AS created_at,
            COALESCE(s.updated_at, m.projected_at) AS updated_at
        FROM communication_messages m
        LEFT JOIN mail_ai_states s ON s.message_id = m.message_id
        WHERE m.message_id = $1
        "#,
    )
    .bind(message_id)
    .fetch_optional(&mut **transaction)
    .await?;

    row.map(row_to_ai_state).transpose()
}

fn row_to_ai_state(row: PgRow) -> Result<MailAiStateRecord, MailAiStateError> {
    let ai_state: String = row.try_get("ai_state")?;
    Ok(MailAiStateRecord {
        message_id: row.try_get("message_id")?,
        ai_state: MailAiState::try_from(ai_state.as_str())?,
        review_reason: row.try_get("review_reason")?,
        last_error: row.try_get("last_error")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn ai_state_changed_event(
    record: &MailAiStateRecord,
    previous_ai_state: MailAiState,
) -> Result<NewEventEnvelope, MailAiStateError> {
    Ok(NewEventEnvelope::builder(
        format!(
            "mail_ai_state_event:{}:{:x}",
            record.message_id,
            system_time_nanos()
        ),
        EVENT_TYPE_CHANGED,
        Utc::now(),
        json!({ "kind": "mail_ai_state_api" }),
        json!({
            "kind": "mail_ai_state",
            "id": record.message_id,
            "message_id": record.message_id,
        }),
    )
    .actor(json!({ "actor_id": "hermes-frontend" }))
    .payload(json!({
        "message_id": record.message_id,
        "ai_state": record.ai_state.as_str(),
        "previous_ai_state": previous_ai_state.as_str(),
        "review_required": record.review_reason.is_some(),
        "failed": record.last_error.is_some(),
    }))
    .provenance(json!({
        "source_kind": "local_api",
        "source_id": record.message_id,
    }))
    .correlation_id(record.message_id.clone())
    .build()?)
}

fn normalize_required(field: &'static str, value: &str) -> Result<String, MailAiStateError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(MailAiStateError::Invalid(field));
    }
    Ok(value.to_owned())
}

fn normalize_optional(value: Option<String>) -> Result<Option<String>, MailAiStateError> {
    match value {
        Some(value) => {
            let value = value.trim();
            if value.is_empty() {
                Ok(None)
            } else {
                Ok(Some(value.to_owned()))
            }
        }
        None => Ok(None),
    }
}

fn system_time_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}

#[derive(Debug, Error)]
pub enum MailAiStateError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),
    #[error(transparent)]
    EventStore(#[from] crate::platform::events::EventStoreError),
    #[error(transparent)]
    EventEnvelope(#[from] crate::platform::events::EventEnvelopeError),
    #[error("invalid mail AI state field: {0}")]
    Invalid(&'static str),
}
