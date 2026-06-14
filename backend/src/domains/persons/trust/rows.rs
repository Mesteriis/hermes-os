use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::PersonTrustError;
use super::models::{PersonPromise, PersonRisk};

pub(super) fn row_to_promise(row: PgRow) -> Result<PersonPromise, PersonTrustError> {
    Ok(PersonPromise {
        id: row.try_get("id")?,
        person_id: row.try_get("person_id")?,
        description: row.try_get("description")?,
        source_message_id: row.try_get("source_message_id")?,
        promised_at: row.try_get("promised_at")?,
        due_at: row.try_get("due_at")?,
        fulfilled_at: row.try_get("fulfilled_at")?,
        status: row.try_get("status")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_to_risk(row: PgRow) -> Result<PersonRisk, PersonTrustError> {
    Ok(PersonRisk {
        id: row.try_get("id")?,
        person_id: row.try_get("person_id")?,
        risk_type: row.try_get("risk_type")?,
        description: row.try_get("description")?,
        severity: row.try_get("severity")?,
        source: row.try_get("source")?,
        confidence: row.try_get("confidence")?,
        created_at: row.try_get("created_at")?,
        resolved_at: row.try_get("resolved_at")?,
        resolution: row.try_get("resolution")?,
    })
}
