use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::PersonProjectionError;
use super::models::{Person, PersonaType};

pub(super) fn row_to_person(row: PgRow) -> Result<Person, PersonProjectionError> {
    Ok(Person {
        person_id: row.try_get("person_id")?,
        display_name: row.try_get("display_name")?,
        email_address: row.try_get("email_address")?,
        persona_type: PersonaType::try_from(row.try_get::<String, _>("person_type")?.as_str())?,
        is_self: row.try_get("is_self")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
