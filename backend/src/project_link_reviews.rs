use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, Postgres};
use sqlx::{Row, Transaction};
use thiserror::Error;

use crate::event_log::{
    EventEnvelope, EventEnvelopeError, EventStore, EventStoreError, NewEventEnvelope,
};

const PROJECT_LINK_REVIEW_EVENT_TYPE: &str = "project.link_review_state_changed";
const PROJECT_LINK_REVIEW_SOURCE_KIND: &str = "project_link_review";
const PROJECT_LINK_REVIEW_SOURCE_PROVIDER: &str = "local_api";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectLinkTargetKind {
    Message,
    Document,
}

impl ProjectLinkTargetKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Message => "message",
            Self::Document => "document",
        }
    }

    fn parse(value: impl AsRef<str>) -> Result<Self, ProjectLinkReviewError> {
        match value.as_ref() {
            "message" => Ok(Self::Message),
            "document" => Ok(Self::Document),
            _ => Err(ProjectLinkReviewError::InvalidTargetKind(
                value.as_ref().to_owned(),
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectLinkReviewState {
    Suggested,
    UserConfirmed,
    UserRejected,
}

impl ProjectLinkReviewState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Suggested => "suggested",
            Self::UserConfirmed => "user_confirmed",
            Self::UserRejected => "user_rejected",
        }
    }

    fn parse(value: impl AsRef<str>) -> Result<Self, ProjectLinkReviewError> {
        match value.as_ref() {
            "suggested" => Ok(Self::Suggested),
            "user_confirmed" => Ok(Self::UserConfirmed),
            "user_rejected" => Ok(Self::UserRejected),
            _ => Err(ProjectLinkReviewError::InvalidReviewState(
                value.as_ref().to_owned(),
            )),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectLinkReviewCommand {
    pub command_id: String,
    pub project_id: String,
    pub target_kind: ProjectLinkTargetKind,
    pub target_id: String,
    pub review_state: ProjectLinkReviewState,
    pub actor_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectLinkReviewCommandResult {
    pub project_id: String,
    pub target_kind: ProjectLinkTargetKind,
    pub target_id: String,
    pub review_state: ProjectLinkReviewState,
    pub event_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectLinkReview {
    pub project_id: String,
    pub target_kind: ProjectLinkTargetKind,
    pub target_id: String,
    pub review_state: ProjectLinkReviewState,
    pub event_id: String,
    pub actor_id: String,
    pub reviewed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectReviewedTarget {
    pub target_id: String,
    pub review_state: ProjectLinkReviewState,
}

struct ReviewEventApplication<'a> {
    target_kind: ProjectLinkTargetKind,
    project_id: &'a str,
    target_id: &'a str,
    review_state: ProjectLinkReviewState,
    event_id: &'a str,
    actor_id: &'a str,
    reviewed_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct ProjectLinkReviewStore {
    pool: PgPool,
}

impl ProjectLinkReviewStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn set_review_state(
        &self,
        command: &ProjectLinkReviewCommand,
    ) -> Result<ProjectLinkReviewCommandResult, ProjectLinkReviewError> {
        let command_id = validate_non_empty("command_id", &command.command_id)?;
        let project_id = validate_non_empty("project_id", &command.project_id)?;
        let target_id = validate_non_empty("target_id", &command.target_id)?;
        let actor_id = validate_non_empty("actor_id", &command.actor_id)?;

        let mut transaction = self.pool.begin().await?;

        self.ensure_project_exists(&mut transaction, &project_id)
            .await?;
        self.ensure_target_exists(&mut transaction, command.target_kind, &target_id)
            .await?;

        let event_id = format!("project_link_review:{command_id}");
        let event = ProjectLinkReviewCommand {
            command_id,
            project_id: project_id.clone(),
            target_kind: command.target_kind,
            target_id: target_id.clone(),
            review_state: command.review_state,
            actor_id: actor_id.clone(),
        }
        .to_review_event(&event_id)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        self.apply_review_event_in_transaction(
            &mut transaction,
            ReviewEventApplication {
                target_kind: command.target_kind,
                project_id: &project_id,
                target_id: &target_id,
                review_state: command.review_state,
                event_id: &event.event_id,
                actor_id: &actor_id,
                reviewed_at: event.occurred_at,
            },
        )
        .await?;

        transaction.commit().await?;

        Ok(ProjectLinkReviewCommandResult {
            project_id,
            target_kind: command.target_kind,
            target_id,
            review_state: command.review_state,
            event_id,
        })
    }

    pub async fn apply_review_event(
        &self,
        event: &EventEnvelope,
    ) -> Result<(), ProjectLinkReviewError> {
        let parsed = ReviewEvent::from_payload(&event.payload)?;
        if event.event_type != PROJECT_LINK_REVIEW_EVENT_TYPE {
            return Err(ProjectLinkReviewError::InvalidEventType);
        }

        let actor_id = event
            .actor
            .as_ref()
            .and_then(|value| value.get("actor_id"))
            .and_then(Value::as_str)
            .ok_or(ProjectLinkReviewError::MissingActorId)?;
        let actor_id = validate_non_empty("actor_id", actor_id)?;
        let mut transaction = self.pool.begin().await?;

        self.ensure_project_exists(&mut transaction, &parsed.project_id)
            .await?;
        self.ensure_target_exists(&mut transaction, parsed.target_kind, &parsed.target_id)
            .await?;
        self.apply_review_event_in_transaction(
            &mut transaction,
            ReviewEventApplication {
                target_kind: parsed.target_kind,
                project_id: &parsed.project_id,
                target_id: &parsed.target_id,
                review_state: parsed.review_state,
                event_id: &event.event_id,
                actor_id: &actor_id,
                reviewed_at: event.occurred_at,
            },
        )
        .await?;

        transaction.commit().await?;
        Ok(())
    }

    pub async fn explicit_review(
        &self,
        project_id: &str,
        target_kind: ProjectLinkTargetKind,
        target_id: &str,
    ) -> Result<Option<ProjectLinkReview>, ProjectLinkReviewError> {
        let project_id = validate_non_empty("project_id", project_id)?;
        let target_id = validate_non_empty("target_id", target_id)?;

        let row = sqlx::query(
            r#"
            SELECT
                project_id,
                target_kind,
                target_id,
                review_state,
                event_id,
                actor_id,
                reviewed_at,
                created_at,
                updated_at
            FROM project_link_reviews
            WHERE project_id = $1 AND target_kind = $2 AND target_id = $3
            "#,
        )
        .bind(&project_id)
        .bind(target_kind.as_str())
        .bind(&target_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_project_link_review).transpose()
    }

    pub async fn active_message_ids_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<ProjectReviewedTarget>, ProjectLinkReviewError> {
        let project_id = validate_non_empty("project_id", project_id)?;

        let rows = sqlx::query(
            r#"
            WITH keyword_matches AS (
                SELECT message_id AS target_id
                FROM communication_messages message
                WHERE EXISTS (
                    SELECT 1
                    FROM project_keywords keyword
                    WHERE keyword.project_id = $1
                      AND (
                          position(lower(keyword.keyword) in lower(message.subject)) > 0
                          OR position(lower(keyword.keyword) in lower(message.body_text)) > 0
                      )
                )
            ),
            confirmed AS (
                SELECT target_id
                FROM project_link_reviews
                WHERE project_id = $1
                  AND target_kind = 'message'
                  AND review_state = 'user_confirmed'
            ),
            rejected AS (
                SELECT target_id
                FROM project_link_reviews
                WHERE project_id = $1
                  AND target_kind = 'message'
                  AND review_state = 'user_rejected'
            ),
            active AS (
                SELECT target_id, 'suggested' AS review_state FROM keyword_matches
                UNION ALL
                SELECT target_id, 'user_confirmed' AS review_state FROM confirmed
            )
            SELECT active.target_id, max(active.review_state) AS review_state
            FROM active
            WHERE NOT EXISTS (
                SELECT 1
                FROM rejected
                WHERE rejected.target_id = active.target_id
            )
            GROUP BY active.target_id
            ORDER BY active.target_id
            "#,
        )
        .bind(&project_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_project_reviewed_target)
            .collect()
    }

    pub async fn active_document_ids_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<ProjectReviewedTarget>, ProjectLinkReviewError> {
        let project_id = validate_non_empty("project_id", project_id)?;

        let rows = sqlx::query(
            r#"
            WITH keyword_matches AS (
                SELECT document_id AS target_id
                FROM documents document
                WHERE EXISTS (
                    SELECT 1
                    FROM project_keywords keyword
                    WHERE keyword.project_id = $1
                      AND (
                          position(lower(keyword.keyword) in lower(document.title)) > 0
                          OR position(lower(keyword.keyword) in lower(document.extracted_text)) > 0
                      )
                )
            ),
            confirmed AS (
                SELECT target_id
                FROM project_link_reviews
                WHERE project_id = $1
                  AND target_kind = 'document'
                  AND review_state = 'user_confirmed'
            ),
            rejected AS (
                SELECT target_id
                FROM project_link_reviews
                WHERE project_id = $1
                  AND target_kind = 'document'
                  AND review_state = 'user_rejected'
            ),
            active AS (
                SELECT target_id, 'suggested' AS review_state FROM keyword_matches
                UNION ALL
                SELECT target_id, 'user_confirmed' AS review_state FROM confirmed
            )
            SELECT active.target_id, max(active.review_state) AS review_state
            FROM active
            WHERE NOT EXISTS (
                SELECT 1
                FROM rejected
                WHERE rejected.target_id = active.target_id
            )
            GROUP BY active.target_id
            ORDER BY active.target_id
            "#,
        )
        .bind(&project_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_project_reviewed_target)
            .collect()
    }

    async fn apply_review_event_in_transaction(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        application: ReviewEventApplication<'_>,
    ) -> Result<(), ProjectLinkReviewError> {
        match application.review_state {
            ProjectLinkReviewState::Suggested => {
                sqlx::query(
                    r#"
                    DELETE FROM project_link_reviews
                    WHERE project_id = $1
                      AND target_kind = $2
                      AND target_id = $3
                    "#,
                )
                .bind(application.project_id)
                .bind(application.target_kind.as_str())
                .bind(application.target_id)
                .execute(&mut **transaction)
                .await?;
            }
            ProjectLinkReviewState::UserConfirmed | ProjectLinkReviewState::UserRejected => {
                sqlx::query(
                    r#"
                    INSERT INTO project_link_reviews (
                        project_id,
                        target_kind,
                        target_id,
                        review_state,
                        event_id,
                        actor_id,
                        reviewed_at
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                    ON CONFLICT (project_id, target_kind, target_id)
                    DO UPDATE SET
                        review_state = EXCLUDED.review_state,
                        event_id = EXCLUDED.event_id,
                        actor_id = EXCLUDED.actor_id,
                        reviewed_at = EXCLUDED.reviewed_at,
                        updated_at = now()
                    "#,
                )
                .bind(application.project_id)
                .bind(application.target_kind.as_str())
                .bind(application.target_id)
                .bind(application.review_state.as_str())
                .bind(application.event_id)
                .bind(application.actor_id)
                .bind(application.reviewed_at)
                .execute(&mut **transaction)
                .await?;
            }
        }

        Ok(())
    }

    async fn ensure_project_exists(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        project_id: &str,
    ) -> Result<(), ProjectLinkReviewError> {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (SELECT 1 FROM projects WHERE project_id = $1)",
        )
        .bind(project_id)
        .fetch_one(&mut **transaction)
        .await?;

        if !exists {
            return Err(ProjectLinkReviewError::ProjectNotFound);
        }

        Ok(())
    }

    async fn ensure_target_exists(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        target_kind: ProjectLinkTargetKind,
        target_id: &str,
    ) -> Result<(), ProjectLinkReviewError> {
        let exists =
            match target_kind {
                ProjectLinkTargetKind::Message => sqlx::query_scalar::<_, bool>(
                    "SELECT EXISTS (SELECT 1 FROM communication_messages WHERE message_id = $1)",
                )
                .bind(target_id)
                .fetch_one(&mut **transaction)
                .await?,
                ProjectLinkTargetKind::Document => {
                    sqlx::query_scalar::<_, bool>(
                        "SELECT EXISTS (SELECT 1 FROM documents WHERE document_id = $1)",
                    )
                    .bind(target_id)
                    .fetch_one(&mut **transaction)
                    .await?
                }
            };

        if !exists {
            return Err(ProjectLinkReviewError::TargetNotFound);
        }

        Ok(())
    }
}

impl ProjectLinkReviewCommand {
    fn to_review_event(&self, event_id: &str) -> Result<NewEventEnvelope, ProjectLinkReviewError> {
        Ok(NewEventEnvelope::builder(
            event_id,
            PROJECT_LINK_REVIEW_EVENT_TYPE,
            Utc::now(),
            json!({
                "kind": PROJECT_LINK_REVIEW_SOURCE_KIND,
                "provider": PROJECT_LINK_REVIEW_SOURCE_PROVIDER,
                "source_id": self.command_id,
            }),
            json!({
                "kind": "project_link_review",
                "project_id": self.project_id,
            }),
        )
        .actor(json!({ "actor_id": self.actor_id }))
        .payload(self.review_payload())
        .build()?)
    }

    fn review_payload(&self) -> Value {
        json!({
            "project_id": self.project_id,
            "target_kind": self.target_kind.as_str(),
            "target_id": self.target_id,
            "review_state": self.review_state.as_str(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReviewEvent {
    project_id: String,
    target_kind: ProjectLinkTargetKind,
    target_id: String,
    review_state: ProjectLinkReviewState,
}

impl ReviewEvent {
    fn from_payload(payload: &Value) -> Result<Self, ProjectLinkReviewError> {
        let payload = as_object(payload)?;
        Ok(Self {
            project_id: required_payload_string(payload, "project_id")?,
            target_kind: ProjectLinkTargetKind::parse(required_payload_string(
                payload,
                "target_kind",
            )?)?,
            target_id: required_payload_string(payload, "target_id")?,
            review_state: ProjectLinkReviewState::parse(required_payload_string(
                payload,
                "review_state",
            )?)?,
        })
    }
}

fn row_to_project_link_review(
    row: sqlx::postgres::PgRow,
) -> Result<ProjectLinkReview, ProjectLinkReviewError> {
    let target_kind = ProjectLinkTargetKind::parse(row.try_get::<String, _>("target_kind")?)?;
    let review_state = ProjectLinkReviewState::parse(row.try_get::<String, _>("review_state")?)?;
    Ok(ProjectLinkReview {
        project_id: row.try_get("project_id")?,
        target_kind,
        target_id: row.try_get("target_id")?,
        review_state,
        event_id: row.try_get("event_id")?,
        actor_id: row.try_get("actor_id")?,
        reviewed_at: row.try_get("reviewed_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_project_reviewed_target(
    row: sqlx::postgres::PgRow,
) -> Result<ProjectReviewedTarget, ProjectLinkReviewError> {
    let review_state = ProjectLinkReviewState::parse(row.try_get::<String, _>("review_state")?)?;

    Ok(ProjectReviewedTarget {
        target_id: row.try_get("target_id")?,
        review_state,
    })
}

fn as_object(value: &Value) -> Result<&serde_json::Map<String, Value>, ProjectLinkReviewError> {
    value
        .as_object()
        .ok_or_else(|| ProjectLinkReviewError::InvalidPayload("payload".to_owned()))
}

fn required_payload_string(
    payload: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<String, ProjectLinkReviewError> {
    let raw = payload
        .get(field)
        .ok_or_else(|| ProjectLinkReviewError::MissingPayloadField(field.to_owned()))?;
    let value = raw
        .as_str()
        .ok_or_else(|| ProjectLinkReviewError::InvalidPayload(field.to_owned()))?;
    validate_non_empty(field, value)
}

fn validate_non_empty(field: &str, value: &str) -> Result<String, ProjectLinkReviewError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(ProjectLinkReviewError::EmptyField(field.to_owned()));
    }

    Ok(normalized.to_owned())
}

#[derive(Debug, Error)]
pub enum ProjectLinkReviewError {
    #[error("project_id does not exist")]
    ProjectNotFound,

    #[error("project link target does not exist")]
    TargetNotFound,

    #[error("target_kind must be one of message or document")]
    InvalidTargetKind(String),

    #[error("review_state must be suggested, user_confirmed, or user_rejected")]
    InvalidReviewState(String),

    #[error("field must not be empty: {0}")]
    EmptyField(String),

    #[error("field missing from payload: {0}")]
    MissingPayloadField(String),

    #[error("field must be a string: {0}")]
    InvalidPayload(String),

    #[error("actor_id is missing from event")]
    MissingActorId,

    #[error("invalid review event type")]
    InvalidEventType,

    #[error(transparent)]
    EventEnvelope(#[from] EventEnvelopeError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),
}
