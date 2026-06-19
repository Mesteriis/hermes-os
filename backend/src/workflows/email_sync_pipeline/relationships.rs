use sqlx::postgres::PgPool;

use crate::domains::mail::messages::ProjectedMessage;
use crate::domains::persons::memory::RelationshipEventStore;

use super::errors::EmailSyncPipelineError;
use super::participants::EmailParticipant;

pub(crate) async fn insert_relationship_event(
    pool: &PgPool,
    message: &ProjectedMessage,
    person_id: &str,
    participant: &EmailParticipant,
) -> Result<bool, EmailSyncPipelineError> {
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
    let inserted = RelationshipEventStore::new(pool.clone())
        .upsert_email_message_event(
            &message.observation_id,
            &message.message_id,
            message.occurred_at.unwrap_or(message.projected_at),
            person_id,
            event_type,
            title,
            Some(&format!("Email subject: {}", message.subject)),
        )
        .await?;
    Ok(inserted)
}
