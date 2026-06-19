use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::{PgPool, Postgres, Transaction};
use thiserror::Error;

use crate::platform::events::{EventStore, NewEventEnvelope};
use crate::platform::observations::{NewObservation, ObservationOriginKind, ObservationStore};

use super::evidence::link_mail_entity_in_transaction;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BulkMessageAction {
    MarkRead,
    MarkUnread,
    Archive,
    Trash,
    Restore,
    Pin,
    Unpin,
    Important,
    NotImportant,
    AddLabel(String),
    RemoveLabel(String),
    Snooze(DateTime<Utc>),
}

impl BulkMessageAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MarkRead => "mark_read",
            Self::MarkUnread => "mark_unread",
            Self::Archive => "archive",
            Self::Trash => "trash",
            Self::Restore => "restore",
            Self::Pin => "pin",
            Self::Unpin => "unpin",
            Self::Important => "important",
            Self::NotImportant => "not_important",
            Self::AddLabel(_) => "add_label",
            Self::RemoveLabel(_) => "remove_label",
            Self::Snooze(_) => "snooze",
        }
    }

    fn event_type(&self) -> &'static str {
        match self {
            Self::MarkRead => "mail.message.read",
            Self::MarkUnread => "mail.message.unread",
            Self::Archive => "mail.message.archived",
            Self::Trash => "mail.message.deleted",
            Self::Restore => "mail.message.restored",
            Self::Pin => "mail.message.pinned",
            Self::Unpin => "mail.message.unpinned",
            Self::Important => "mail.message.important",
            Self::NotImportant => "mail.message.not_important",
            Self::AddLabel(_) => "mail.message.labeled",
            Self::RemoveLabel(_) => "mail.message.unlabeled",
            Self::Snooze(_) => "mail.message.snoozed",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BulkMessageActionOutcome {
    pub action: String,
    pub requested_count: usize,
    pub matched_count: usize,
    pub updated_count: usize,
    pub not_found: Vec<String>,
}

pub struct BulkMessageActionStore {
    pool: PgPool,
}

impl BulkMessageActionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn apply(
        &self,
        message_ids: Vec<String>,
        action: BulkMessageAction,
    ) -> Result<BulkMessageActionOutcome, BulkMessageActionError> {
        let message_ids = normalize_message_ids(message_ids)?;
        let mut transaction = self.pool.begin().await?;
        let updated_ids = match &action {
            BulkMessageAction::MarkRead => {
                self.update_workflow_state(&mut transaction, &message_ids, "reviewed")
                    .await?
            }
            BulkMessageAction::MarkUnread => {
                self.update_workflow_state(&mut transaction, &message_ids, "new")
                    .await?
            }
            BulkMessageAction::Archive => {
                self.update_workflow_state(&mut transaction, &message_ids, "archived")
                    .await?
            }
            BulkMessageAction::Trash => self.move_to_trash(&mut transaction, &message_ids).await?,
            BulkMessageAction::Restore => {
                self.restore_from_trash(&mut transaction, &message_ids)
                    .await?
            }
            BulkMessageAction::Pin => {
                self.set_metadata_bool(&mut transaction, &message_ids, "pinned", true)
                    .await?
            }
            BulkMessageAction::Unpin => {
                self.set_metadata_bool(&mut transaction, &message_ids, "pinned", false)
                    .await?
            }
            BulkMessageAction::Important => {
                self.set_metadata_bool(&mut transaction, &message_ids, "important", true)
                    .await?
            }
            BulkMessageAction::NotImportant => {
                self.set_metadata_bool(&mut transaction, &message_ids, "important", false)
                    .await?
            }
            BulkMessageAction::AddLabel(label) => {
                self.add_label(&mut transaction, &message_ids, label)
                    .await?
            }
            BulkMessageAction::RemoveLabel(label) => {
                self.remove_label(&mut transaction, &message_ids, label)
                    .await?
            }
            BulkMessageAction::Snooze(until) => {
                self.snooze(&mut transaction, &message_ids, until).await?
            }
        };
        let outcome = outcome(action.as_str(), &message_ids, updated_ids.clone());

        if !updated_ids.is_empty() {
            self.capture_observation_trail(&mut transaction, &action, &outcome, &updated_ids)
                .await?;
            let event = bulk_message_action_event(&action, &outcome, &updated_ids)?;
            EventStore::append_in_transaction(&mut transaction, &event).await?;
        }
        transaction.commit().await?;

        Ok(outcome)
    }

    async fn update_workflow_state(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        message_ids: &[String],
        workflow_state: &str,
    ) -> Result<Vec<String>, BulkMessageActionError> {
        sqlx::query_scalar(
            r#"
            UPDATE communication_messages
            SET workflow_state = $2, projected_at = now()
            WHERE message_id = ANY($1)
            RETURNING message_id
            "#,
        )
        .bind(message_ids)
        .bind(workflow_state)
        .fetch_all(&mut **transaction)
        .await
        .map_err(BulkMessageActionError::Sqlx)
    }

    async fn move_to_trash(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        message_ids: &[String],
    ) -> Result<Vec<String>, BulkMessageActionError> {
        sqlx::query_scalar(
            r#"
            UPDATE communication_messages
            SET local_state = 'trash',
                local_state_changed_at = now(),
                local_state_reason = 'bulk_action',
                projected_at = now()
            WHERE message_id = ANY($1)
            RETURNING message_id
        "#,
        )
        .bind(message_ids)
        .fetch_all(&mut **transaction)
        .await
        .map_err(BulkMessageActionError::Sqlx)
    }

    async fn restore_from_trash(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        message_ids: &[String],
    ) -> Result<Vec<String>, BulkMessageActionError> {
        sqlx::query_scalar(
            r#"
            UPDATE communication_messages
            SET local_state = 'active',
                local_state_changed_at = now(),
                local_state_reason = NULL,
                projected_at = now()
            WHERE message_id = ANY($1)
            RETURNING message_id
        "#,
        )
        .bind(message_ids)
        .fetch_all(&mut **transaction)
        .await
        .map_err(BulkMessageActionError::Sqlx)
    }

    async fn set_metadata_bool(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        message_ids: &[String],
        key: &str,
        value: bool,
    ) -> Result<Vec<String>, BulkMessageActionError> {
        let path = vec![key.to_owned()];
        sqlx::query_scalar(
            r#"
            UPDATE communication_messages
            SET message_metadata = jsonb_set(
                    COALESCE(message_metadata, '{}'::jsonb),
                    $2,
                    to_jsonb($3::boolean),
                    true
                ),
                projected_at = now()
            WHERE message_id = ANY($1)
            RETURNING message_id
            "#,
        )
        .bind(message_ids)
        .bind(path)
        .bind(value)
        .fetch_all(&mut **transaction)
        .await
        .map_err(BulkMessageActionError::Sqlx)
    }

    async fn add_label(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        message_ids: &[String],
        label: &str,
    ) -> Result<Vec<String>, BulkMessageActionError> {
        validate_label(label)?;
        sqlx::query_scalar(
            r#"
            UPDATE communication_messages AS m
            SET message_metadata = jsonb_set(
                    COALESCE(m.message_metadata, '{}'::jsonb),
                    '{labels}',
                    (
                        SELECT COALESCE(jsonb_agg(label_value ORDER BY label_value), '[]'::jsonb)
                        FROM (
                            SELECT DISTINCT label_value
                            FROM (
                                SELECT jsonb_array_elements_text(
                                    CASE
                                        WHEN jsonb_typeof(COALESCE(m.message_metadata, '{}'::jsonb)->'labels') = 'array'
                                        THEN m.message_metadata->'labels'
                                        ELSE '[]'::jsonb
                                    END
                                ) AS label_value
                                UNION ALL
                                SELECT $2::text AS label_value
                            ) labels
                            WHERE trim(label_value) <> ''
                        ) distinct_labels
                    ),
                    true
                ),
                projected_at = now()
            WHERE m.message_id = ANY($1)
            RETURNING m.message_id
        "#,
        )
        .bind(message_ids)
        .bind(label.trim())
        .fetch_all(&mut **transaction)
        .await
        .map_err(BulkMessageActionError::Sqlx)
    }

    async fn remove_label(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        message_ids: &[String],
        label: &str,
    ) -> Result<Vec<String>, BulkMessageActionError> {
        validate_label(label)?;
        sqlx::query_scalar(
            r#"
            UPDATE communication_messages AS m
            SET message_metadata = jsonb_set(
                    COALESCE(m.message_metadata, '{}'::jsonb),
                    '{labels}',
                    (
                        SELECT COALESCE(jsonb_agg(label_value ORDER BY label_value), '[]'::jsonb)
                        FROM (
                            SELECT DISTINCT label_value
                            FROM jsonb_array_elements_text(
                                CASE
                                    WHEN jsonb_typeof(COALESCE(m.message_metadata, '{}'::jsonb)->'labels') = 'array'
                                    THEN m.message_metadata->'labels'
                                    ELSE '[]'::jsonb
                                END
                            ) AS label_value
                            WHERE label_value <> $2::text
                              AND trim(label_value) <> ''
                        ) remaining_labels
                    ),
                    true
                ),
                projected_at = now()
            WHERE m.message_id = ANY($1)
            RETURNING m.message_id
        "#,
        )
        .bind(message_ids)
        .bind(label.trim())
        .fetch_all(&mut **transaction)
        .await
        .map_err(BulkMessageActionError::Sqlx)
    }

    async fn snooze(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        message_ids: &[String],
        until: &DateTime<Utc>,
    ) -> Result<Vec<String>, BulkMessageActionError> {
        sqlx::query_scalar(
            r#"
            UPDATE communication_messages
            SET message_metadata = jsonb_set(
                    COALESCE(message_metadata, '{}'::jsonb),
                    '{snooze_until}',
                    to_jsonb($2::text),
                    true
                ),
                projected_at = now()
            WHERE message_id = ANY($1)
            RETURNING message_id
        "#,
        )
        .bind(message_ids)
        .bind(until.to_rfc3339())
        .fetch_all(&mut **transaction)
        .await
        .map_err(BulkMessageActionError::Sqlx)
    }

    async fn capture_observation_trail(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        action: &BulkMessageAction,
        outcome: &BulkMessageActionOutcome,
        updated_ids: &[String],
    ) -> Result<(), BulkMessageActionError> {
        let recorded_at = Utc::now();
        for message_id in updated_ids {
            let observation = ObservationStore::capture_in_transaction(
                transaction,
                &NewObservation::new(
                    "COMMUNICATION_MESSAGE",
                    ObservationOriginKind::Manual,
                    recorded_at,
                    bulk_action_observation_payload(action, outcome, message_id),
                    format!(
                        "message://{message_id}/bulk/{}/{}",
                        action.as_str(),
                        system_time_nanos()
                    ),
                )
                .provenance(json!({
                    "captured_by": "mail.bulk_actions",
                    "operation": action.as_str(),
                    "event_type": action.event_type(),
                })),
            )
            .await?;

            link_mail_entity_in_transaction(
                transaction,
                &observation.observation_id,
                "communication_message",
                message_id.to_owned(),
                bulk_action_relationship_kind(action),
                bulk_action_link_metadata(action),
                None,
            )
            .await?;
        }

        Ok(())
    }
}

fn bulk_action_relationship_kind(action: &BulkMessageAction) -> &'static str {
    match action {
        BulkMessageAction::MarkRead
        | BulkMessageAction::MarkUnread
        | BulkMessageAction::Archive => "workflow_state_transition",
        BulkMessageAction::Trash | BulkMessageAction::Restore => "local_state_transition",
        BulkMessageAction::Pin
        | BulkMessageAction::Unpin
        | BulkMessageAction::Important
        | BulkMessageAction::NotImportant
        | BulkMessageAction::AddLabel(_)
        | BulkMessageAction::RemoveLabel(_)
        | BulkMessageAction::Snooze(_) => "message_flag_update",
    }
}

fn bulk_action_link_metadata(action: &BulkMessageAction) -> Value {
    match action {
        BulkMessageAction::MarkRead => json!({ "workflow_state": "reviewed" }),
        BulkMessageAction::MarkUnread => json!({ "workflow_state": "new" }),
        BulkMessageAction::Archive => json!({ "workflow_state": "archived" }),
        BulkMessageAction::Trash => json!({ "local_state": "trash" }),
        BulkMessageAction::Restore => json!({ "local_state": "active" }),
        BulkMessageAction::Pin => json!({ "pinned": true }),
        BulkMessageAction::Unpin => json!({ "pinned": false }),
        BulkMessageAction::Important => json!({ "important": true }),
        BulkMessageAction::NotImportant => json!({ "important": false }),
        BulkMessageAction::AddLabel(label) => json!({ "label": label.trim(), "action": "add" }),
        BulkMessageAction::RemoveLabel(label) => {
            json!({ "label": label.trim(), "action": "remove" })
        }
        BulkMessageAction::Snooze(until) => json!({ "snooze_until": until.to_rfc3339() }),
    }
}

fn bulk_action_observation_payload(
    action: &BulkMessageAction,
    outcome: &BulkMessageActionOutcome,
    message_id: &str,
) -> Value {
    json!({
        "message_id": message_id,
        "operation": format!("bulk_{}", action.as_str()),
        "action": action.as_str(),
        "action_parameters": action_parameters(action),
        "requested_count": outcome.requested_count,
        "matched_count": outcome.matched_count,
        "updated_count": outcome.updated_count,
        "not_found": outcome.not_found,
    })
}

fn bulk_message_action_event(
    action: &BulkMessageAction,
    outcome: &BulkMessageActionOutcome,
    updated_ids: &[String],
) -> Result<NewEventEnvelope, BulkMessageActionError> {
    let event_id = format!(
        "mail_message_action_event:{}:{:x}",
        outcome.action,
        system_time_nanos()
    );

    Ok(NewEventEnvelope::builder(
        event_id.clone(),
        action.event_type(),
        Utc::now(),
        json!({ "kind": "mail_bulk_action_api" }),
        json!({
            "kind": "mail_message_bulk_action",
            "id": event_id,
            "message_ids": updated_ids,
        }),
    )
    .actor(json!({ "actor_id": "hermes-frontend" }))
    .payload(json!({
        "action": outcome.action,
        "action_parameters": action_parameters(action),
        "requested_count": outcome.requested_count,
        "matched_count": outcome.matched_count,
        "updated_count": outcome.updated_count,
        "message_ids": updated_ids,
        "not_found": outcome.not_found,
    }))
    .provenance(json!({
        "source_kind": "local_api",
        "source_id": outcome.action,
    }))
    .correlation_id(outcome.action.clone())
    .build()?)
}

fn action_parameters(action: &BulkMessageAction) -> Value {
    match action {
        BulkMessageAction::AddLabel(label) => json!({ "label": label.trim() }),
        BulkMessageAction::RemoveLabel(label) => json!({ "label": label.trim() }),
        BulkMessageAction::Snooze(until) => json!({ "snooze_until": until.to_rfc3339() }),
        _ => json!({}),
    }
}

fn normalize_message_ids(message_ids: Vec<String>) -> Result<Vec<String>, BulkMessageActionError> {
    if message_ids.is_empty() {
        return Err(BulkMessageActionError::Invalid(
            "message_ids must not be empty",
        ));
    }
    if message_ids.len() > 500 {
        return Err(BulkMessageActionError::Invalid(
            "message_ids must contain at most 500 items",
        ));
    }

    let mut seen = HashSet::new();
    let mut normalized = Vec::new();
    for message_id in message_ids {
        let message_id = message_id.trim();
        if message_id.is_empty() {
            return Err(BulkMessageActionError::Invalid(
                "message_ids must not contain empty values",
            ));
        }
        if seen.insert(message_id.to_owned()) {
            normalized.push(message_id.to_owned());
        }
    }

    Ok(normalized)
}

fn validate_label(label: &str) -> Result<(), BulkMessageActionError> {
    if label.trim().is_empty() {
        return Err(BulkMessageActionError::Invalid("label must not be empty"));
    }
    Ok(())
}

fn outcome(
    action: &str,
    requested_ids: &[String],
    updated_ids: Vec<String>,
) -> BulkMessageActionOutcome {
    let updated = updated_ids.into_iter().collect::<HashSet<_>>();
    let not_found = requested_ids
        .iter()
        .filter(|message_id| !updated.contains(*message_id))
        .cloned()
        .collect::<Vec<_>>();
    let updated_count = updated.len();

    BulkMessageActionOutcome {
        action: action.to_owned(),
        requested_count: requested_ids.len(),
        matched_count: updated_count,
        updated_count,
        not_found,
    }
}

#[derive(Debug, Error)]
pub enum BulkMessageActionError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    ObservationStore(#[from] crate::platform::observations::ObservationStoreError),
    #[error(transparent)]
    EventStore(#[from] crate::platform::events::EventStoreError),
    #[error(transparent)]
    EventEnvelope(#[from] crate::platform::events::EventEnvelopeError),
    #[error("invalid bulk message action request: {0}")]
    Invalid(&'static str),
}

fn system_time_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}
