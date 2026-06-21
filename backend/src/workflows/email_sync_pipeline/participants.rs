use sqlx::postgres::PgPool;

use crate::domains::communications::messages::{
    CommunicationMessageProjectionPort, ProjectedMessage,
};

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
) -> Result<bool, EmailSyncPipelineError> {
    let inserted = CommunicationMessageProjectionPort::new(pool.clone())
        .upsert_email_participant(
            message,
            person_id,
            &participant.email_address,
            participant.display_name.as_deref(),
            participant.role,
        )
        .await?;
    Ok(inserted)
}

fn clean_display_name(value: &str) -> Option<String> {
    let value = value.trim().trim_matches('"').trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}
