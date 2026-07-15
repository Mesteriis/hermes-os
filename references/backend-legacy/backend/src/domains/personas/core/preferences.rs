use sqlx::{Postgres, Transaction};

use super::errors::PersonaCoreError;
use super::interaction_contexts::PersonaInteractionContext;

pub(super) async fn materialize_interaction_preferences_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    persona: &PersonaInteractionContext,
    source: &str,
) -> Result<(), PersonaCoreError> {
    upsert_interaction_preference_in_transaction(
        transaction,
        persona,
        "name",
        Some(persona.name.as_str()),
        source,
    )
    .await?;
    upsert_interaction_preference_in_transaction(
        transaction,
        persona,
        "context",
        persona.context.as_deref(),
        source,
    )
    .await?;
    upsert_interaction_preference_in_transaction(
        transaction,
        persona,
        "default_tone",
        persona.default_tone.as_deref(),
        source,
    )
    .await?;
    upsert_interaction_preference_in_transaction(
        transaction,
        persona,
        "default_language",
        persona.default_language.as_deref(),
        source,
    )
    .await?;
    upsert_interaction_preference_in_transaction(
        transaction,
        persona,
        "preferred_channel",
        persona.preferred_channel.as_deref(),
        source,
    )
    .await?;

    Ok(())
}

pub(super) async fn delete_interaction_preferences_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    persona: &PersonaInteractionContext,
    source: &str,
) -> Result<(), PersonaCoreError> {
    for field in [
        "name",
        "context",
        "default_tone",
        "default_language",
        "preferred_channel",
    ] {
        sqlx::query(
            "DELETE FROM persona_preferences
             WHERE persona_id = $1 AND preference_type = $2 AND source = $3",
        )
        .bind(&persona.source_persona_id)
        .bind(interaction_context_preference_type(
            &persona.interaction_context_id,
            field,
        ))
        .bind(source)
        .execute(&mut **transaction)
        .await?;
    }

    Ok(())
}

async fn upsert_interaction_preference_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    persona: &PersonaInteractionContext,
    field: &str,
    value: Option<&str>,
    source: &str,
) -> Result<(), PersonaCoreError> {
    let preference_type =
        interaction_context_preference_type(&persona.interaction_context_id, field);
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        sqlx::query(
            "DELETE FROM persona_preferences
             WHERE persona_id = $1 AND preference_type = $2 AND source = $3",
        )
        .bind(&persona.source_persona_id)
        .bind(preference_type)
        .bind(source)
        .execute(&mut **transaction)
        .await?;
        return Ok(());
    };

    sqlx::query(
        "INSERT INTO persona_preferences (persona_id, preference_type, value, source)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (persona_id, preference_type)
         DO UPDATE SET value = EXCLUDED.value, source = EXCLUDED.source, updated_at = now()",
    )
    .bind(&persona.source_persona_id)
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
    format!("persona_interaction_contexts:{persona_id}")
}
