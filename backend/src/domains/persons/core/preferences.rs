use sqlx::{Postgres, Transaction};

use super::errors::PersonCoreError;
use super::interaction_contexts::PersonPersona;

pub(super) async fn materialize_interaction_preferences_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    persona: &PersonPersona,
) -> Result<(), PersonCoreError> {
    let source = interaction_context_source(&persona.persona_id);
    upsert_interaction_preference_in_transaction(
        transaction,
        persona,
        "name",
        Some(persona.name.as_str()),
        &source,
    )
    .await?;
    upsert_interaction_preference_in_transaction(
        transaction,
        persona,
        "context",
        persona.context.as_deref(),
        &source,
    )
    .await?;
    upsert_interaction_preference_in_transaction(
        transaction,
        persona,
        "default_tone",
        persona.default_tone.as_deref(),
        &source,
    )
    .await?;
    upsert_interaction_preference_in_transaction(
        transaction,
        persona,
        "default_language",
        persona.default_language.as_deref(),
        &source,
    )
    .await?;
    upsert_interaction_preference_in_transaction(
        transaction,
        persona,
        "preferred_channel",
        persona.preferred_channel.as_deref(),
        &source,
    )
    .await?;

    Ok(())
}

pub(super) async fn delete_interaction_preferences_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    persona: &PersonPersona,
) -> Result<(), PersonCoreError> {
    let source = interaction_context_source(&persona.persona_id);
    for field in [
        "name",
        "context",
        "default_tone",
        "default_language",
        "preferred_channel",
    ] {
        sqlx::query(
            "DELETE FROM person_preferences
             WHERE person_id = $1 AND preference_type = $2 AND source = $3",
        )
        .bind(&persona.person_id)
        .bind(interaction_context_preference_type(
            &persona.persona_id,
            field,
        ))
        .bind(&source)
        .execute(&mut **transaction)
        .await?;
    }

    Ok(())
}

async fn upsert_interaction_preference_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    persona: &PersonPersona,
    field: &str,
    value: Option<&str>,
    source: &str,
) -> Result<(), PersonCoreError> {
    let preference_type = interaction_context_preference_type(&persona.persona_id, field);
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        sqlx::query(
            "DELETE FROM person_preferences
             WHERE person_id = $1 AND preference_type = $2 AND source = $3",
        )
        .bind(&persona.person_id)
        .bind(preference_type)
        .bind(source)
        .execute(&mut **transaction)
        .await?;
        return Ok(());
    };

    sqlx::query(
        "INSERT INTO person_preferences (person_id, preference_type, value, source)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (person_id, preference_type)
         DO UPDATE SET value = EXCLUDED.value, source = EXCLUDED.source, updated_at = now()",
    )
    .bind(&persona.person_id)
    .bind(preference_type)
    .bind(value)
    .bind(source)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

fn interaction_context_preference_type(persona_id: &str, field: &str) -> String {
    format!("interaction_context:{persona_id}:{field}")
}

fn interaction_context_source(persona_id: &str) -> String {
    format!("person_personas:{persona_id}")
}
