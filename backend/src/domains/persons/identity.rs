// This file exceeds 700 lines because it groups the person identity store
// with multi-channel identity resolution, candidate review, and identity
// merge logic. These are tightly coupled through the identity resolution
// algorithm and shared SQL queries.

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, Postgres};
use sqlx::{Row, Transaction};
use thiserror::Error;

use crate::platform::events::{
    EventEnvelope, EventEnvelopeError, EventStore, EventStoreError, NewEventEnvelope,
};

const PERSON_IDENTITY_REVIEW_EVENT_TYPE: &str = "person_identity.review_state_changed";
const PERSON_IDENTITY_REVIEW_SOURCE_KIND: &str = "person_identity_review";
const PERSON_IDENTITY_REVIEW_SOURCE_PROVIDER: &str = "local_api";
const PERSON_IDENTITY_REVIEW_PREFIX: &str = "person_identity_review:";
const PERSON_IDENTITY_ID_PREFIX: &str = "identity_candidate:v1:";
const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 100;
const MIN_LIMIT: i64 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum PersonIdentityCandidateKind {
    MergePersons,
    AttachEmailAddress,
    SplitPerson,
}

impl PersonIdentityCandidateKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MergePersons => "merge_persons",
            Self::AttachEmailAddress => "attach_email_address",
            Self::SplitPerson => "split_person",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum PersonIdentityReviewState {
    Suggested,
    UserConfirmed,
    UserRejected,
}

impl PersonIdentityReviewState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Suggested => "suggested",
            Self::UserConfirmed => "user_confirmed",
            Self::UserRejected => "user_rejected",
        }
    }

    fn parse(value: impl AsRef<str>) -> Result<Self, PersonIdentityError> {
        match value.as_ref() {
            "suggested" => Ok(Self::Suggested),
            "user_confirmed" => Ok(Self::UserConfirmed),
            "user_rejected" => Ok(Self::UserRejected),
            _ => Err(PersonIdentityError::InvalidReviewState(
                value.as_ref().to_owned(),
            )),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersonIdentityReviewCommand {
    pub command_id: String,
    pub identity_candidate_id: String,
    pub review_state: PersonIdentityReviewState,
    pub actor_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersonIdentityReviewCommandResult {
    pub identity_candidate_id: String,
    pub review_state: PersonIdentityReviewState,
    pub event_id: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct PersonIdentityCandidate {
    pub identity_candidate_id: String,
    pub candidate_kind: String,
    pub left_person_id: String,
    pub right_person_id: Option<String>,
    pub email_address: Option<String>,
    pub evidence_summary: String,
    pub confidence: f64,
    pub review_state: String,
    pub generated_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PersonIdentityDetail {
    pub items: Vec<PersonIdentityCandidate>,
}

#[derive(Clone)]
pub struct PersonIdentityStore {
    pool: PgPool,
}

impl PersonIdentityStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn refresh_candidates(&self, limit: i64) -> Result<usize, PersonIdentityError> {
        let limit = validate_limit(limit)?;
        let rows = sqlx::query(
            r#"
            SELECT
                c1.person_id AS left_person_id,
                c2.person_id AS right_person_id,
                lower(trim(c1.display_name)) AS normalized_display_name
            FROM persons c1
            JOIN persons c2
                ON c1.person_id < c2.person_id
               AND lower(trim(c1.display_name)) = lower(trim(c2.display_name))
            WHERE position('@' in lower(trim(c1.display_name))) = 0
              AND position('@' in lower(trim(c2.display_name))) = 0
            ORDER BY
                lower(trim(c1.display_name)),
                c1.person_id,
                c2.person_id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut count = 0usize;
        for row in rows {
            let left = row.try_get::<String, _>("left_person_id")?;
            let right = row.try_get::<String, _>("right_person_id")?;
            let candidate = PersonIdentityCandidatePayload {
                candidate_kind: PersonIdentityCandidateKind::MergePersons,
                left_person_id: left,
                right_person_id: Some(right),
                email_address: None,
                evidence_summary: format!(
                    "Same normalized display name: {}",
                    row.try_get::<String, _>("normalized_display_name")?
                ),
                confidence: 0.72,
            };
            upsert_candidate(
                &self.pool,
                &candidate,
                candidate.identity_candidate_id(),
                PersonIdentityReviewState::Suggested,
            )
            .await?;
            count += 1;
        }

        let rows = sqlx::query(
            r#"
            SELECT
                merge.left_person_id,
                merge.right_person_id
            FROM person_identity_candidates merge
            WHERE merge.candidate_kind = 'merge_persons'
              AND merge.review_state = 'user_confirmed'
              AND merge.right_person_id IS NOT NULL
              AND NOT EXISTS (
                  SELECT 1
                  FROM person_identity_candidates split
                  WHERE split.candidate_kind = 'split_person'
                    AND split.left_person_id = merge.left_person_id
                    AND split.right_person_id = merge.right_person_id
              )
            ORDER BY merge.updated_at DESC, merge.identity_candidate_id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        for row in rows {
            let left = row.try_get::<String, _>("left_person_id")?;
            let right = row.try_get::<String, _>("right_person_id")?;
            let candidate = PersonIdentityCandidatePayload {
                candidate_kind: PersonIdentityCandidateKind::SplitPerson,
                left_person_id: left.clone(),
                right_person_id: Some(right.clone()),
                email_address: None,
                evidence_summary: format!(
                    "Previously confirmed merge can be split: {left} and {right}"
                ),
                confidence: 1.0,
            };
            upsert_candidate(
                &self.pool,
                &candidate,
                candidate.identity_candidate_id(),
                PersonIdentityReviewState::Suggested,
            )
            .await?;
            count += 1;
        }

        Ok(count)
    }

    pub async fn set_review_state(
        &self,
        command: &PersonIdentityReviewCommand,
    ) -> Result<PersonIdentityReviewCommandResult, PersonIdentityError> {
        let command_id = validate_non_empty("command_id", &command.command_id)?;
        let identity_candidate_id =
            validate_non_empty("identity_candidate_id", &command.identity_candidate_id)?;
        let actor_id = validate_non_empty("actor_id", &command.actor_id)?;

        let mut transaction = self.pool.begin().await?;
        self.ensure_candidate_exists(&mut transaction, &identity_candidate_id)
            .await?;

        let event_id = format!("{PERSON_IDENTITY_REVIEW_PREFIX}{command_id}");
        let event = ReviewCommandEvent {
            command_id,
            identity_candidate_id: identity_candidate_id.clone(),
            review_state: command.review_state,
            actor_id: actor_id.clone(),
            event_id: event_id.clone(),
            occurred_at: Utc::now(),
        }
        .to_event()?;

        EventStore::append_in_transaction(&mut transaction, &event).await?;
        self.apply_review_state_in_transaction(
            &mut transaction,
            &identity_candidate_id,
            command.review_state,
            &event_id,
            &actor_id,
            event.occurred_at,
        )
        .await?;
        self.materialize_split_candidate_for_confirmed_merge_in_transaction(
            &mut transaction,
            &identity_candidate_id,
            command.review_state,
        )
        .await?;

        transaction.commit().await?;

        Ok(PersonIdentityReviewCommandResult {
            identity_candidate_id,
            review_state: command.review_state,
            event_id,
        })
    }

    pub async fn apply_review_event(
        &self,
        event: &EventEnvelope,
    ) -> Result<(), PersonIdentityError> {
        if event.event_type != PERSON_IDENTITY_REVIEW_EVENT_TYPE {
            return Err(PersonIdentityError::InvalidEventType);
        }

        let parsed = ReviewEvent::from_payload(&event.payload)?;
        let actor_id = event
            .actor
            .as_ref()
            .and_then(|value| value.get("actor_id"))
            .and_then(Value::as_str)
            .ok_or(PersonIdentityError::MissingActorId)?;
        let actor_id = validate_non_empty("actor_id", actor_id)?;
        let mut transaction = self.pool.begin().await?;
        self.ensure_candidate_exists(&mut transaction, &parsed.identity_candidate_id)
            .await?;
        self.apply_review_state_in_transaction(
            &mut transaction,
            &parsed.identity_candidate_id,
            parsed.review_state,
            &event.event_id,
            &actor_id,
            event.occurred_at,
        )
        .await?;
        self.materialize_split_candidate_for_confirmed_merge_in_transaction(
            &mut transaction,
            &parsed.identity_candidate_id,
            parsed.review_state,
        )
        .await?;

        transaction.commit().await?;
        Ok(())
    }

    pub async fn list_candidates(
        &self,
        limit: Option<i64>,
    ) -> Result<Vec<PersonIdentityCandidate>, PersonIdentityError> {
        let limit = validate_optional_limit(limit)?;

        let rows = sqlx::query(
            r#"
            SELECT
                identity_candidate_id,
                candidate_kind,
                left_person_id,
                right_person_id,
                email_address,
                evidence_summary,
                confidence,
                review_state,
                generated_at,
                reviewed_at,
                updated_at
            FROM person_identity_candidates
            ORDER BY updated_at DESC, identity_candidate_id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_person_identity_candidate)
            .collect()
    }

    pub async fn person_identity(
        &self,
        person_id: &str,
    ) -> Result<PersonIdentityDetail, PersonIdentityError> {
        let person_id = validate_non_empty("person_id", person_id)?;

        let rows = sqlx::query(
            r#"
            SELECT
                identity_candidate_id,
                candidate_kind,
                left_person_id,
                right_person_id,
                email_address,
                evidence_summary,
                confidence,
                review_state,
                generated_at,
                reviewed_at,
                updated_at
            FROM person_identity_candidates merge
            WHERE (merge.left_person_id = $1 OR merge.right_person_id = $1)
              AND merge.candidate_kind = 'merge_persons'
              AND merge.review_state = 'user_confirmed'
              AND NOT EXISTS (
                  SELECT 1
                  FROM person_identity_candidates split
                  WHERE split.candidate_kind = 'split_person'
                    AND split.review_state = 'user_confirmed'
                    AND LEAST(split.left_person_id, split.right_person_id) =
                        LEAST(merge.left_person_id, merge.right_person_id)
                    AND GREATEST(split.left_person_id, split.right_person_id) =
                        GREATEST(merge.left_person_id, merge.right_person_id)
              )
            ORDER BY updated_at DESC, identity_candidate_id
            "#,
        )
        .bind(&person_id)
        .fetch_all(&self.pool)
        .await?;

        let items = rows
            .into_iter()
            .map(row_to_person_identity_candidate)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(PersonIdentityDetail { items })
    }

    async fn apply_review_state_in_transaction(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        identity_candidate_id: &str,
        review_state: PersonIdentityReviewState,
        event_id: &str,
        actor_id: &str,
        reviewed_at: DateTime<Utc>,
    ) -> Result<(), PersonIdentityError> {
        match review_state {
            PersonIdentityReviewState::Suggested => {
                sqlx::query(
                    r#"
                    UPDATE person_identity_candidates
                    SET
                        review_state = $1,
                        event_id = NULL,
                        actor_id = NULL,
                        reviewed_at = NULL,
                        updated_at = now()
                    WHERE identity_candidate_id = $2
                    "#,
                )
                .bind(review_state.as_str())
                .bind(identity_candidate_id)
                .execute(&mut **transaction)
                .await?;
            }
            PersonIdentityReviewState::UserConfirmed | PersonIdentityReviewState::UserRejected => {
                sqlx::query(
                    r#"
                    UPDATE person_identity_candidates
                    SET
                        review_state = $1,
                        event_id = $2,
                        actor_id = $3,
                        reviewed_at = $4,
                        updated_at = now()
                    WHERE identity_candidate_id = $5
                    "#,
                )
                .bind(review_state.as_str())
                .bind(event_id)
                .bind(actor_id)
                .bind(reviewed_at)
                .bind(identity_candidate_id)
                .execute(&mut **transaction)
                .await?;
            }
        }

        Ok(())
    }

    async fn materialize_split_candidate_for_confirmed_merge_in_transaction(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        identity_candidate_id: &str,
        review_state: PersonIdentityReviewState,
    ) -> Result<(), PersonIdentityError> {
        if review_state != PersonIdentityReviewState::UserConfirmed {
            return Ok(());
        }

        let row = sqlx::query(
            r#"
            SELECT candidate_kind, left_person_id, right_person_id
            FROM person_identity_candidates
            WHERE identity_candidate_id = $1
            "#,
        )
        .bind(identity_candidate_id)
        .fetch_one(&mut **transaction)
        .await?;

        let candidate_kind = row.try_get::<String, _>("candidate_kind")?;
        if candidate_kind != PersonIdentityCandidateKind::MergePersons.as_str() {
            return Ok(());
        }

        let left = row.try_get::<String, _>("left_person_id")?;
        let Some(right) = row.try_get::<Option<String>, _>("right_person_id")? else {
            return Ok(());
        };
        let candidate = PersonIdentityCandidatePayload {
            candidate_kind: PersonIdentityCandidateKind::SplitPerson,
            left_person_id: left.clone(),
            right_person_id: Some(right.clone()),
            email_address: None,
            evidence_summary: format!(
                "Previously confirmed merge can be split: {left} and {right}"
            ),
            confidence: 1.0,
        };
        upsert_candidate_in_transaction(
            transaction,
            &candidate,
            candidate.identity_candidate_id(),
            PersonIdentityReviewState::Suggested,
        )
        .await
    }

    async fn ensure_candidate_exists(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        identity_candidate_id: &str,
    ) -> Result<(), PersonIdentityError> {
        let exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM person_identity_candidates
                WHERE identity_candidate_id = $1
            )
            "#,
        )
        .bind(identity_candidate_id)
        .fetch_one(&mut **transaction)
        .await?;

        if !exists {
            return Err(PersonIdentityError::IdentityCandidateNotFound);
        }

        Ok(())
    }
}

#[derive(Debug)]
struct PersonIdentityCandidatePayload {
    candidate_kind: PersonIdentityCandidateKind,
    left_person_id: String,
    right_person_id: Option<String>,
    email_address: Option<String>,
    evidence_summary: String,
    confidence: f64,
}

impl PersonIdentityCandidatePayload {
    fn identity_candidate_id(&self) -> String {
        let left = self.left_person_id.clone();
        let right = self
            .right_person_id
            .clone()
            .unwrap_or_else(|| String::from("single"));

        match self.candidate_kind {
            PersonIdentityCandidateKind::MergePersons => {
                format!("{PERSON_IDENTITY_ID_PREFIX}merge_persons:{left}:{right}")
            }
            PersonIdentityCandidateKind::AttachEmailAddress => {
                format!("{PERSON_IDENTITY_ID_PREFIX}attach_email_address:{left}:{right}")
            }
            PersonIdentityCandidateKind::SplitPerson => {
                format!("{PERSON_IDENTITY_ID_PREFIX}split_person:{left}:{right}")
            }
        }
    }
}

struct ReviewCommandEvent {
    command_id: String,
    identity_candidate_id: String,
    review_state: PersonIdentityReviewState,
    actor_id: String,
    event_id: String,
    occurred_at: DateTime<Utc>,
}

impl ReviewCommandEvent {
    fn to_event(&self) -> Result<NewEventEnvelope, PersonIdentityError> {
        Ok(NewEventEnvelope::builder(
            self.event_id.clone(),
            PERSON_IDENTITY_REVIEW_EVENT_TYPE,
            self.occurred_at,
            json!({
                "kind": PERSON_IDENTITY_REVIEW_SOURCE_KIND,
                "provider": PERSON_IDENTITY_REVIEW_SOURCE_PROVIDER,
                "source_id": self.command_id.clone(),
            }),
            json!({
                "kind": "person_identity_review",
            }),
        )
        .actor(json!({ "actor_id": self.actor_id.clone() }))
        .payload(self.review_payload())
        .build()?)
    }

    fn review_payload(&self) -> Value {
        json!({
            "identity_candidate_id": self.identity_candidate_id,
            "review_state": self.review_state.as_str(),
        })
    }
}

#[derive(Debug)]
struct ReviewEvent {
    identity_candidate_id: String,
    review_state: PersonIdentityReviewState,
}

impl ReviewEvent {
    fn from_payload(payload: &Value) -> Result<Self, PersonIdentityError> {
        let payload = as_object(payload)?;
        Ok(Self {
            identity_candidate_id: required_payload_string(payload, "identity_candidate_id")?,
            review_state: PersonIdentityReviewState::parse(required_payload_string(
                payload,
                "review_state",
            )?)?,
        })
    }
}

async fn upsert_candidate(
    pool: &PgPool,
    payload: &PersonIdentityCandidatePayload,
    identity_candidate_id: String,
    review_state: PersonIdentityReviewState,
) -> Result<(), PersonIdentityError> {
    sqlx::query(
        r#"
        INSERT INTO person_identity_candidates (
            identity_candidate_id,
            candidate_kind,
            left_person_id,
            right_person_id,
            email_address,
            evidence_summary,
            confidence,
            review_state,
            event_id,
            actor_id,
            reviewed_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NULL, NULL, NULL)
        ON CONFLICT (identity_candidate_id)
        DO UPDATE SET
            candidate_kind = EXCLUDED.candidate_kind,
            left_person_id = EXCLUDED.left_person_id,
            right_person_id = EXCLUDED.right_person_id,
            email_address = EXCLUDED.email_address,
            evidence_summary = EXCLUDED.evidence_summary,
            confidence = EXCLUDED.confidence,
            review_state = CASE
                WHEN person_identity_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN person_identity_candidates.review_state
                ELSE EXCLUDED.review_state
            END,
            event_id = CASE
                WHEN person_identity_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN person_identity_candidates.event_id
                ELSE NULL
            END,
            actor_id = CASE
                WHEN person_identity_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN person_identity_candidates.actor_id
                ELSE NULL
            END,
            reviewed_at = CASE
                WHEN person_identity_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN person_identity_candidates.reviewed_at
                ELSE NULL
            END,
            updated_at = now()
        "#,
    )
    .bind(identity_candidate_id)
    .bind(payload.candidate_kind.as_str())
    .bind(&payload.left_person_id)
    .bind(&payload.right_person_id)
    .bind(&payload.email_address)
    .bind(&payload.evidence_summary)
    .bind(payload.confidence)
    .bind(review_state.as_str())
    .execute(pool)
    .await?;

    Ok(())
}

async fn upsert_candidate_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    payload: &PersonIdentityCandidatePayload,
    identity_candidate_id: String,
    review_state: PersonIdentityReviewState,
) -> Result<(), PersonIdentityError> {
    sqlx::query(
        r#"
        INSERT INTO person_identity_candidates (
            identity_candidate_id,
            candidate_kind,
            left_person_id,
            right_person_id,
            email_address,
            evidence_summary,
            confidence,
            review_state,
            event_id,
            actor_id,
            reviewed_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NULL, NULL, NULL)
        ON CONFLICT (identity_candidate_id)
        DO UPDATE SET
            candidate_kind = EXCLUDED.candidate_kind,
            left_person_id = EXCLUDED.left_person_id,
            right_person_id = EXCLUDED.right_person_id,
            email_address = EXCLUDED.email_address,
            evidence_summary = EXCLUDED.evidence_summary,
            confidence = EXCLUDED.confidence,
            review_state = CASE
                WHEN person_identity_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN person_identity_candidates.review_state
                ELSE EXCLUDED.review_state
            END,
            event_id = CASE
                WHEN person_identity_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN person_identity_candidates.event_id
                ELSE NULL
            END,
            actor_id = CASE
                WHEN person_identity_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN person_identity_candidates.actor_id
                ELSE NULL
            END,
            reviewed_at = CASE
                WHEN person_identity_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN person_identity_candidates.reviewed_at
                ELSE NULL
            END,
            updated_at = now()
        "#,
    )
    .bind(identity_candidate_id)
    .bind(payload.candidate_kind.as_str())
    .bind(&payload.left_person_id)
    .bind(&payload.right_person_id)
    .bind(&payload.email_address)
    .bind(&payload.evidence_summary)
    .bind(payload.confidence)
    .bind(review_state.as_str())
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

fn row_to_person_identity_candidate(
    row: sqlx::postgres::PgRow,
) -> Result<PersonIdentityCandidate, PersonIdentityError> {
    Ok(PersonIdentityCandidate {
        identity_candidate_id: row.try_get("identity_candidate_id")?,
        candidate_kind: row.try_get("candidate_kind")?,
        left_person_id: row.try_get("left_person_id")?,
        right_person_id: row.try_get("right_person_id")?,
        email_address: row.try_get("email_address")?,
        evidence_summary: row.try_get("evidence_summary")?,
        confidence: row.try_get("confidence")?,
        review_state: row.try_get("review_state")?,
        generated_at: row.try_get("generated_at")?,
        reviewed_at: row.try_get("reviewed_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn as_object(value: &Value) -> Result<&serde_json::Map<String, Value>, PersonIdentityError> {
    value
        .as_object()
        .ok_or_else(|| PersonIdentityError::InvalidPayload("payload".to_owned()))
}

fn required_payload_string(
    payload: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<String, PersonIdentityError> {
    let raw = payload
        .get(field)
        .ok_or_else(|| PersonIdentityError::MissingPayloadField(field.to_owned()))?;
    let value = raw
        .as_str()
        .ok_or_else(|| PersonIdentityError::InvalidPayload(field.to_owned()))?;
    validate_non_empty(field, value)
}

fn validate_non_empty(field: &str, value: &str) -> Result<String, PersonIdentityError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(PersonIdentityError::EmptyField(field.to_owned()));
    }

    Ok(normalized.to_owned())
}

fn validate_limit(limit: i64) -> Result<i64, PersonIdentityError> {
    if !(MIN_LIMIT..=MAX_LIMIT).contains(&limit) {
        return Err(PersonIdentityError::InvalidLimit);
    }

    Ok(limit)
}

fn validate_optional_limit(limit: Option<i64>) -> Result<i64, PersonIdentityError> {
    validate_limit(limit.unwrap_or(DEFAULT_LIMIT))
}

#[derive(Debug, Error)]
pub enum PersonIdentityError {
    #[error("limit must be between 1 and 100")]
    InvalidLimit,

    #[error("field must not be empty: {0}")]
    EmptyField(String),

    #[error("candidate kind is not supported: {0}")]
    InvalidCandidateKind(String),

    #[error("review_state must be suggested, user_confirmed, or user_rejected")]
    InvalidReviewState(String),

    #[error("candidate was not found")]
    IdentityCandidateNotFound,

    #[error("payload must be an object")]
    InvalidPayload(String),

    #[error("payload field was missing: {0}")]
    MissingPayloadField(String),

    #[error("actor_id is missing from event")]
    MissingActorId,

    #[error("invalid review event type")]
    InvalidEventType,

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    EventEnvelope(#[from] EventEnvelopeError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
