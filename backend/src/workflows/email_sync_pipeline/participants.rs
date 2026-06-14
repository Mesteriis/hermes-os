use sqlx::Row;
use sqlx::postgres::PgPool;

use crate::domains::mail::messages::ProjectedMessage;

use super::errors::EmailSyncPipelineError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EmailParticipant {
    pub(crate) email_address: String,
    pub(crate) display_name: Option<String>,
    pub(crate) role: &'static str,
}

pub(crate) fn parse_email_participant(
    raw: &str,
    role: &'static str,
) -> Result<EmailParticipant, EmailSyncPipelineError> {
    let trimmed = raw.trim();
    let (display_name, email) = if let Some((name, tail)) = trimmed.rsplit_once('<') {
        if let Some((addr, _)) = tail.split_once('>') {
            (clean_display_name(name), addr.trim())
        } else {
            (None, trimmed)
        }
    } else {
        (None, trimmed)
    };
    let email_address = email.trim_matches('"').trim().to_ascii_lowercase();
    if email_address.is_empty() || !email_address.contains('@') {
        return Err(EmailSyncPipelineError::InvalidParticipantEmail(
            raw.to_owned(),
        ));
    }
    Ok(EmailParticipant {
        email_address,
        display_name,
        role,
    })
}

pub(crate) async fn upsert_message_participant(
    pool: &PgPool,
    message: &ProjectedMessage,
    person_id: &str,
    participant: &EmailParticipant,
) -> Result<bool, sqlx::Error> {
    let row = sqlx::query(
        r#"
        INSERT INTO communication_message_participants (
            message_id, person_id, email_address, display_name, role, source, confidence
        )
        VALUES ($1, $2, $3, $4, $5, 'email_sync', 1.0)
        ON CONFLICT (message_id, person_id, role, email_address)
        DO UPDATE SET
            display_name = EXCLUDED.display_name,
            source = EXCLUDED.source,
            confidence = EXCLUDED.confidence,
            updated_at = now()
        RETURNING (xmax = 0) AS inserted
        "#,
    )
    .bind(&message.message_id)
    .bind(person_id)
    .bind(&participant.email_address)
    .bind(participant.display_name.as_deref())
    .bind(participant.role)
    .fetch_one(pool)
    .await?;
    row.try_get::<bool, _>("inserted")
}

fn clean_display_name(value: &str) -> Option<String> {
    let value = value.trim().trim_matches('"').trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}
