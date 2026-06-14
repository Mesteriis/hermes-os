use sqlx::postgres::PgPool;

use crate::domains::mail::messages::ProjectedMessage;

use super::participants::EmailParticipant;

pub(crate) async fn insert_relationship_event(
    pool: &PgPool,
    message: &ProjectedMessage,
    person_id: &str,
    participant: &EmailParticipant,
) -> Result<bool, sqlx::Error> {
    let event_type = if participant.role == "sender" {
        "email_sent"
    } else {
        "email_received"
    };
    let title = if participant.role == "sender" {
        "Sent email"
    } else {
        "Received email"
    };
    let occurred_at = message.occurred_at.unwrap_or(message.projected_at);
    let result = sqlx::query(
        r#"
        INSERT INTO relationship_events (
            person_id, event_type, title, description, occurred_at, source,
            related_entity_id, related_entity_kind, metadata
        )
        SELECT $1, $2, $3, $4, $5, 'email_sync', $6, 'communication_message', '{}'::jsonb
        WHERE NOT EXISTS (
            SELECT 1
            FROM relationship_events
            WHERE person_id = $1
              AND event_type = $2
              AND related_entity_id = $6
              AND related_entity_kind = 'communication_message'
        )
        "#,
    )
    .bind(person_id)
    .bind(event_type)
    .bind(title)
    .bind(Some(format!("Email subject: {}", message.subject)))
    .bind(occurred_at)
    .bind(&message.message_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}
