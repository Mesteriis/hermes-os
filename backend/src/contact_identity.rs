use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, Postgres};
use sqlx::{Row, Transaction};
use thiserror::Error;

use crate::event_log::{
    EventEnvelope, EventEnvelopeError, EventStore, EventStoreError, NewEventEnvelope,
};

const CONTACT_IDENTITY_REVIEW_EVENT_TYPE: &str = "contact_identity.review_state_changed";
const CONTACT_IDENTITY_REVIEW_SOURCE_KIND: &str = "contact_identity_review";
const CONTACT_IDENTITY_REVIEW_SOURCE_PROVIDER: &str = "local_api";
const CONTACT_IDENTITY_REVIEW_PREFIX: &str = "contact_identity_review:";
const CONTACT_IDENTITY_ID_PREFIX: &str = "identity_candidate:v1:";
const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 100;
const MIN_LIMIT: i64 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum ContactIdentityCandidateKind {
    MergeContacts,
    AttachEmailAddress,
    SplitContact,
}

impl ContactIdentityCandidateKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MergeContacts => "merge_contacts",
            Self::AttachEmailAddress => "attach_email_address",
            Self::SplitContact => "split_contact",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum ContactIdentityReviewState {
    Suggested,
    UserConfirmed,
    UserRejected,
}

impl ContactIdentityReviewState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Suggested => "suggested",
            Self::UserConfirmed => "user_confirmed",
            Self::UserRejected => "user_rejected",
        }
    }

    fn parse(value: impl AsRef<str>) -> Result<Self, ContactIdentityError> {
        match value.as_ref() {
            "suggested" => Ok(Self::Suggested),
            "user_confirmed" => Ok(Self::UserConfirmed),
            "user_rejected" => Ok(Self::UserRejected),
            _ => Err(ContactIdentityError::InvalidReviewState(
                value.as_ref().to_owned(),
            )),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContactIdentityReviewCommand {
    pub command_id: String,
    pub identity_candidate_id: String,
    pub review_state: ContactIdentityReviewState,
    pub actor_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContactIdentityReviewCommandResult {
    pub identity_candidate_id: String,
    pub review_state: ContactIdentityReviewState,
    pub event_id: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ContactIdentityCandidate {
    pub identity_candidate_id: String,
    pub candidate_kind: String,
    pub left_contact_id: String,
    pub right_contact_id: Option<String>,
    pub email_address: Option<String>,
    pub evidence_summary: String,
    pub confidence: f64,
    pub review_state: String,
    pub generated_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ContactIdentityDetail {
    pub items: Vec<ContactIdentityCandidate>,
}

#[derive(Clone)]
pub struct ContactIdentityStore {
    pool: PgPool,
}

impl ContactIdentityStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn refresh_candidates(&self, limit: i64) -> Result<usize, ContactIdentityError> {
        let limit = validate_limit(limit)?;
        let rows = sqlx::query(
            r#"
            SELECT
                c1.contact_id AS left_contact_id,
                c2.contact_id AS right_contact_id,
                lower(trim(c1.display_name)) AS normalized_display_name
            FROM contacts c1
            JOIN contacts c2
                ON c1.contact_id < c2.contact_id
               AND lower(trim(c1.display_name)) = lower(trim(c2.display_name))
            WHERE position('@' in lower(trim(c1.display_name))) = 0
              AND position('@' in lower(trim(c2.display_name))) = 0
            ORDER BY
                lower(trim(c1.display_name)),
                c1.contact_id,
                c2.contact_id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut count = 0usize;
        for row in rows {
            let left = row.try_get::<String, _>("left_contact_id")?;
            let right = row.try_get::<String, _>("right_contact_id")?;
            let candidate = ContactIdentityCandidatePayload {
                candidate_kind: ContactIdentityCandidateKind::MergeContacts,
                left_contact_id: left,
                right_contact_id: Some(right),
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
                ContactIdentityReviewState::Suggested,
            )
            .await?;
            count += 1;
        }

        Ok(count)
    }

    pub async fn set_review_state(
        &self,
        command: &ContactIdentityReviewCommand,
    ) -> Result<ContactIdentityReviewCommandResult, ContactIdentityError> {
        let command_id = validate_non_empty("command_id", &command.command_id)?;
        let identity_candidate_id =
            validate_non_empty("identity_candidate_id", &command.identity_candidate_id)?;
        let actor_id = validate_non_empty("actor_id", &command.actor_id)?;

        let mut transaction = self.pool.begin().await?;
        self.ensure_candidate_exists(&mut transaction, &identity_candidate_id)
            .await?;

        let event_id = format!("{CONTACT_IDENTITY_REVIEW_PREFIX}{command_id}");
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

        transaction.commit().await?;

        Ok(ContactIdentityReviewCommandResult {
            identity_candidate_id,
            review_state: command.review_state,
            event_id,
        })
    }

    pub async fn apply_review_event(
        &self,
        event: &EventEnvelope,
    ) -> Result<(), ContactIdentityError> {
        if event.event_type != CONTACT_IDENTITY_REVIEW_EVENT_TYPE {
            return Err(ContactIdentityError::InvalidEventType);
        }

        let parsed = ReviewEvent::from_payload(&event.payload)?;
        let actor_id = event
            .actor
            .as_ref()
            .and_then(|value| value.get("actor_id"))
            .and_then(Value::as_str)
            .ok_or(ContactIdentityError::MissingActorId)?;
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

        transaction.commit().await?;
        Ok(())
    }

    pub async fn list_candidates(
        &self,
        limit: Option<i64>,
    ) -> Result<Vec<ContactIdentityCandidate>, ContactIdentityError> {
        let limit = validate_optional_limit(limit)?;

        let rows = sqlx::query(
            r#"
            SELECT
                identity_candidate_id,
                candidate_kind,
                left_contact_id,
                right_contact_id,
                email_address,
                evidence_summary,
                confidence,
                review_state,
                generated_at,
                reviewed_at,
                updated_at
            FROM contact_identity_candidates
            ORDER BY updated_at DESC, identity_candidate_id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_contact_identity_candidate)
            .collect()
    }

    pub async fn contact_identity(
        &self,
        contact_id: &str,
    ) -> Result<ContactIdentityDetail, ContactIdentityError> {
        let contact_id = validate_non_empty("contact_id", contact_id)?;

        let rows = sqlx::query(
            r#"
            SELECT
                identity_candidate_id,
                candidate_kind,
                left_contact_id,
                right_contact_id,
                email_address,
                evidence_summary,
                confidence,
                review_state,
                generated_at,
                reviewed_at,
                updated_at
            FROM contact_identity_candidates
            WHERE (left_contact_id = $1 OR right_contact_id = $1)
              AND review_state = 'user_confirmed'
            ORDER BY updated_at DESC, identity_candidate_id
            "#,
        )
        .bind(&contact_id)
        .fetch_all(&self.pool)
        .await?;

        let items = rows
            .into_iter()
            .map(row_to_contact_identity_candidate)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ContactIdentityDetail { items })
    }

    async fn apply_review_state_in_transaction(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        identity_candidate_id: &str,
        review_state: ContactIdentityReviewState,
        event_id: &str,
        actor_id: &str,
        reviewed_at: DateTime<Utc>,
    ) -> Result<(), ContactIdentityError> {
        match review_state {
            ContactIdentityReviewState::Suggested => {
                sqlx::query(
                    r#"
                    UPDATE contact_identity_candidates
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
            ContactIdentityReviewState::UserConfirmed
            | ContactIdentityReviewState::UserRejected => {
                sqlx::query(
                    r#"
                    UPDATE contact_identity_candidates
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

    async fn ensure_candidate_exists(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        identity_candidate_id: &str,
    ) -> Result<(), ContactIdentityError> {
        let exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM contact_identity_candidates
                WHERE identity_candidate_id = $1
            )
            "#,
        )
        .bind(identity_candidate_id)
        .fetch_one(&mut **transaction)
        .await?;

        if !exists {
            return Err(ContactIdentityError::IdentityCandidateNotFound);
        }

        Ok(())
    }
}

#[derive(Debug)]
struct ContactIdentityCandidatePayload {
    candidate_kind: ContactIdentityCandidateKind,
    left_contact_id: String,
    right_contact_id: Option<String>,
    email_address: Option<String>,
    evidence_summary: String,
    confidence: f64,
}

impl ContactIdentityCandidatePayload {
    fn identity_candidate_id(&self) -> String {
        let left = self.left_contact_id.clone();
        let right = self
            .right_contact_id
            .clone()
            .unwrap_or_else(|| String::from("single"));

        match self.candidate_kind {
            ContactIdentityCandidateKind::MergeContacts => {
                format!("{CONTACT_IDENTITY_ID_PREFIX}merge_contacts:{left}:{right}")
            }
            ContactIdentityCandidateKind::AttachEmailAddress => {
                format!("{CONTACT_IDENTITY_ID_PREFIX}attach_email_address:{left}:{right}")
            }
            ContactIdentityCandidateKind::SplitContact => {
                format!("{CONTACT_IDENTITY_ID_PREFIX}split_contact:{left}:{right}")
            }
        }
    }
}

struct ReviewCommandEvent {
    command_id: String,
    identity_candidate_id: String,
    review_state: ContactIdentityReviewState,
    actor_id: String,
    event_id: String,
    occurred_at: DateTime<Utc>,
}

impl ReviewCommandEvent {
    fn to_event(&self) -> Result<NewEventEnvelope, ContactIdentityError> {
        Ok(NewEventEnvelope::builder(
            self.event_id.clone(),
            CONTACT_IDENTITY_REVIEW_EVENT_TYPE,
            self.occurred_at,
            json!({
                "kind": CONTACT_IDENTITY_REVIEW_SOURCE_KIND,
                "provider": CONTACT_IDENTITY_REVIEW_SOURCE_PROVIDER,
                "source_id": self.command_id.clone(),
            }),
            json!({
                "kind": "contact_identity_review",
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
    review_state: ContactIdentityReviewState,
}

impl ReviewEvent {
    fn from_payload(payload: &Value) -> Result<Self, ContactIdentityError> {
        let payload = as_object(payload)?;
        Ok(Self {
            identity_candidate_id: required_payload_string(payload, "identity_candidate_id")?,
            review_state: ContactIdentityReviewState::parse(required_payload_string(
                payload,
                "review_state",
            )?)?,
        })
    }
}

async fn upsert_candidate(
    pool: &PgPool,
    payload: &ContactIdentityCandidatePayload,
    identity_candidate_id: String,
    review_state: ContactIdentityReviewState,
) -> Result<(), ContactIdentityError> {
    sqlx::query(
        r#"
        INSERT INTO contact_identity_candidates (
            identity_candidate_id,
            candidate_kind,
            left_contact_id,
            right_contact_id,
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
            left_contact_id = EXCLUDED.left_contact_id,
            right_contact_id = EXCLUDED.right_contact_id,
            email_address = EXCLUDED.email_address,
            evidence_summary = EXCLUDED.evidence_summary,
            confidence = EXCLUDED.confidence,
            review_state = CASE
                WHEN contact_identity_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN contact_identity_candidates.review_state
                ELSE EXCLUDED.review_state
            END,
            event_id = CASE
                WHEN contact_identity_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN contact_identity_candidates.event_id
                ELSE NULL
            END,
            actor_id = CASE
                WHEN contact_identity_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN contact_identity_candidates.actor_id
                ELSE NULL
            END,
            reviewed_at = CASE
                WHEN contact_identity_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN contact_identity_candidates.reviewed_at
                ELSE NULL
            END,
            updated_at = now()
        "#,
    )
    .bind(identity_candidate_id)
    .bind(payload.candidate_kind.as_str())
    .bind(&payload.left_contact_id)
    .bind(&payload.right_contact_id)
    .bind(&payload.email_address)
    .bind(&payload.evidence_summary)
    .bind(payload.confidence)
    .bind(review_state.as_str())
    .execute(pool)
    .await?;

    Ok(())
}

fn row_to_contact_identity_candidate(
    row: sqlx::postgres::PgRow,
) -> Result<ContactIdentityCandidate, ContactIdentityError> {
    Ok(ContactIdentityCandidate {
        identity_candidate_id: row.try_get("identity_candidate_id")?,
        candidate_kind: row.try_get("candidate_kind")?,
        left_contact_id: row.try_get("left_contact_id")?,
        right_contact_id: row.try_get("right_contact_id")?,
        email_address: row.try_get("email_address")?,
        evidence_summary: row.try_get("evidence_summary")?,
        confidence: row.try_get("confidence")?,
        review_state: row.try_get("review_state")?,
        generated_at: row.try_get("generated_at")?,
        reviewed_at: row.try_get("reviewed_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn as_object(value: &Value) -> Result<&serde_json::Map<String, Value>, ContactIdentityError> {
    value
        .as_object()
        .ok_or_else(|| ContactIdentityError::InvalidPayload("payload".to_owned()))
}

fn required_payload_string(
    payload: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<String, ContactIdentityError> {
    let raw = payload
        .get(field)
        .ok_or_else(|| ContactIdentityError::MissingPayloadField(field.to_owned()))?;
    let value = raw
        .as_str()
        .ok_or_else(|| ContactIdentityError::InvalidPayload(field.to_owned()))?;
    validate_non_empty(field, value)
}

fn validate_non_empty(field: &str, value: &str) -> Result<String, ContactIdentityError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(ContactIdentityError::EmptyField(field.to_owned()));
    }

    Ok(normalized.to_owned())
}

fn validate_limit(limit: i64) -> Result<i64, ContactIdentityError> {
    if !(MIN_LIMIT..=MAX_LIMIT).contains(&limit) {
        return Err(ContactIdentityError::InvalidLimit);
    }

    Ok(limit)
}

fn validate_optional_limit(limit: Option<i64>) -> Result<i64, ContactIdentityError> {
    validate_limit(limit.unwrap_or(DEFAULT_LIMIT))
}

#[derive(Debug, Error)]
pub enum ContactIdentityError {
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
