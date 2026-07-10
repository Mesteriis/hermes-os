use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::PersonaProjectionError;
use super::models::{Persona, PersonaType};

pub(super) fn row_to_persona(row: PgRow) -> Result<Persona, PersonaProjectionError> {
    Ok(Persona {
        persona_id: row.try_get("persona_id")?,
        display_name: row.try_get("display_name")?,
        email_address: row.try_get("email_address")?,
        persona_type: PersonaType::try_from(row.try_get::<String, _>("person_type")?.as_str())?,
        is_self: row.try_get("is_self")?,
        is_address_book: row.try_get("is_address_book")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
