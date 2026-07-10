use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::GraphProjectionError;
use super::models::{DocumentRow, MessageRow, PersonaRow};

pub(super) fn row_to_persona(row: PgRow) -> Result<PersonaRow, GraphProjectionError> {
    Ok(PersonaRow {
        person_id: row.try_get("person_id")?,
        display_name: row.try_get("display_name")?,
        email_address: row.try_get("email_address")?,
    })
}

pub(super) fn row_to_message(row: PgRow) -> Result<MessageRow, GraphProjectionError> {
    Ok(MessageRow {
        message_id: row.try_get("message_id")?,
        raw_record_id: row.try_get("raw_record_id")?,
        observation_id: row.try_get("observation_id")?,
        account_id: row.try_get("account_id")?,
        provider_record_id: row.try_get("provider_record_id")?,
        subject: row.try_get("subject")?,
        sender: row.try_get("sender")?,
        recipients: recipients_from_value(row.try_get("recipients")?)?,
        body_text: row.try_get("body_text")?,
        occurred_at: row.try_get("occurred_at")?,
    })
}

pub(super) fn row_to_document(row: PgRow) -> Result<DocumentRow, GraphProjectionError> {
    Ok(DocumentRow {
        document_id: row.try_get("document_id")?,
        document_kind: row.try_get("document_kind")?,
        title: row.try_get("title")?,
        source_fingerprint: row.try_get("source_fingerprint")?,
        observation_id: row.try_get("observation_id")?,
        imported_at: row.try_get("imported_at")?,
    })
}

fn recipients_from_value(value: Value) -> Result<Vec<String>, GraphProjectionError> {
    let Some(values) = value.as_array() else {
        return Err(GraphProjectionError::InvalidRecipients);
    };

    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or(GraphProjectionError::InvalidRecipients)
        })
        .collect()
}
