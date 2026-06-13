use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Transaction};
use thiserror::Error;

use crate::domains::relationships::{
    NewRelationship, NewRelationshipEvidence, RelationshipEntityKind,
    RelationshipEvidenceSourceKind, RelationshipReviewState, RelationshipStore,
    RelationshipStoreError,
};

// ── PersonIdentity ──────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonIdentity {
    pub id: String,
    pub person_id: Option<String>,
    pub identity_type: String,
    pub identity_value: String,
    pub source: String,
    pub confidence: f64,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub status: String,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PersonsIdentityStore {
    pool: PgPool,
}

impl PersonsIdentityStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_by_person(
        &self,
        person_id: &str,
    ) -> Result<Vec<PersonIdentity>, PersonCoreError> {
        let rows = sqlx::query(
            r#"SELECT id::text, person_id, identity_type, identity_value, source,
               confidence::float8 AS confidence,
               last_verified_at, status, metadata, created_at, updated_at
               FROM person_identities WHERE person_id = $1 ORDER BY identity_type"#,
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_identity).collect()
    }

    pub async fn list_unattached(
        &self,
        limit: i64,
    ) -> Result<Vec<PersonIdentity>, PersonCoreError> {
        let limit = limit.clamp(1, 200);
        let rows = sqlx::query(
            r#"SELECT id::text, person_id, identity_type, identity_value, source,
               confidence::float8 AS confidence,
               last_verified_at, status, metadata, created_at, updated_at
               FROM person_identities
               WHERE person_id IS NULL
               ORDER BY updated_at DESC, id
               LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_identity).collect()
    }

    pub async fn upsert(
        &self,
        person_id: &str,
        identity_type: &str,
        identity_value: &str,
        source: &str,
    ) -> Result<PersonIdentity, PersonCoreError> {
        let row = sqlx::query(
            r#"INSERT INTO person_identities (person_id, identity_type, identity_value, source)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (identity_type, identity_value) WHERE status = 'active'
               DO UPDATE SET updated_at = now()
               RETURNING id::text, person_id, identity_type, identity_value, source,
                         confidence::float8 AS confidence,
                         last_verified_at, status, metadata, created_at, updated_at"#,
        )
        .bind(person_id)
        .bind(identity_type)
        .bind(identity_value)
        .bind(source)
        .fetch_one(&self.pool)
        .await?;
        row_to_identity(row)
    }

    pub async fn create_unattached(
        &self,
        identity_type: &str,
        identity_value: &str,
        source: &str,
    ) -> Result<PersonIdentity, PersonCoreError> {
        let row = sqlx::query(
            r#"INSERT INTO person_identities (person_id, identity_type, identity_value, source)
               VALUES (NULL, $1, $2, $3)
               ON CONFLICT (identity_type, identity_value) WHERE status = 'active'
               DO UPDATE SET updated_at = now()
               RETURNING id::text, person_id, identity_type, identity_value, source,
                         confidence::float8 AS confidence,
                         last_verified_at, status, metadata, created_at, updated_at"#,
        )
        .bind(identity_type)
        .bind(identity_value)
        .bind(source)
        .fetch_one(&self.pool)
        .await?;
        row_to_identity(row)
    }

    pub async fn attach_to_persona(
        &self,
        identity_id: &str,
        person_id: &str,
    ) -> Result<PersonIdentity, PersonCoreError> {
        let row = sqlx::query(
            r#"UPDATE person_identities
               SET person_id = $2, status = 'active', updated_at = now()
               WHERE id::text = $1
               RETURNING id::text, person_id, identity_type, identity_value, source,
                         confidence::float8 AS confidence,
                         last_verified_at, status, metadata, created_at, updated_at"#,
        )
        .bind(identity_id)
        .bind(person_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(PersonCoreError::IdentityNotFound)?;
        row_to_identity(row)
    }

    pub async fn update_status(
        &self,
        identity_id: &str,
        status: &str,
    ) -> Result<(), PersonCoreError> {
        sqlx::query(
            "UPDATE person_identities SET status = $2, updated_at = now() WHERE id::text = $1",
        )
        .bind(identity_id)
        .bind(status)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, identity_id: &str) -> Result<bool, PersonCoreError> {
        let result = sqlx::query("DELETE FROM person_identities WHERE id::text = $1")
            .bind(identity_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}

fn row_to_identity(row: PgRow) -> Result<PersonIdentity, PersonCoreError> {
    Ok(PersonIdentity {
        id: row.try_get("id")?,
        person_id: row.try_get("person_id")?,
        identity_type: row.try_get("identity_type")?,
        identity_value: row.try_get("identity_value")?,
        source: row.try_get("source")?,
        confidence: row.try_get("confidence")?,
        last_verified_at: row.try_get("last_verified_at")?,
        status: row.try_get("status")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

// ── PersonRole ──────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonRole {
    pub id: String,
    pub person_id: String,
    pub role: String,
    pub assigned_by: Option<String>,
    pub assigned_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PersonRoleStore {
    pool: PgPool,
}

impl PersonRoleStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_by_person(
        &self,
        person_id: &str,
    ) -> Result<Vec<PersonRole>, PersonCoreError> {
        let rows = sqlx::query(
            r#"SELECT id::text, person_id, role, assigned_by, assigned_at
               FROM person_roles WHERE person_id = $1 ORDER BY assigned_at"#,
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_role).collect()
    }

    pub async fn assign(
        &self,
        person_id: &str,
        role: &str,
        assigned_by: Option<&str>,
    ) -> Result<PersonRole, PersonCoreError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"INSERT INTO person_roles (person_id, role, assigned_by)
               VALUES ($1, $2, $3)
               ON CONFLICT (person_id, role) DO UPDATE SET assigned_by = EXCLUDED.assigned_by
               RETURNING id::text, person_id, role, assigned_by, assigned_at"#,
        )
        .bind(person_id)
        .bind(role)
        .bind(assigned_by)
        .fetch_one(&mut *transaction)
        .await?;
        let role = row_to_role(row)?;

        Self::materialize_role_relationship_in_transaction(
            &mut transaction,
            &role,
            RelationshipReviewState::UserConfirmed,
        )
        .await?;
        transaction.commit().await?;

        Ok(role)
    }

    pub async fn remove(&self, person_id: &str, role: &str) -> Result<bool, PersonCoreError> {
        let mut transaction = self.pool.begin().await?;
        let existing_role = sqlx::query(
            r#"SELECT id::text, person_id, role, assigned_by, assigned_at
               FROM person_roles
               WHERE person_id = $1 AND role = $2
               FOR UPDATE"#,
        )
        .bind(person_id)
        .bind(role)
        .fetch_optional(&mut *transaction)
        .await?
        .map(row_to_role)
        .transpose()?;

        let result = sqlx::query("DELETE FROM person_roles WHERE person_id = $1 AND role = $2")
            .bind(person_id)
            .bind(role)
            .execute(&mut *transaction)
            .await?;
        let removed = result.rows_affected() > 0;

        if let Some(existing_role) = existing_role
            && removed
        {
            Self::materialize_role_relationship_in_transaction(
                &mut transaction,
                &existing_role,
                RelationshipReviewState::UserRejected,
            )
            .await?;
        }

        transaction.commit().await?;

        Ok(removed)
    }

    async fn materialize_role_relationship_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        role: &PersonRole,
        review_state: RelationshipReviewState,
    ) -> Result<(), PersonCoreError> {
        let relationship = NewRelationship {
            source_entity_kind: RelationshipEntityKind::Persona,
            source_entity_id: role.person_id.clone(),
            target_entity_kind: RelationshipEntityKind::Knowledge,
            target_entity_id: person_role_knowledge_id(&role.role),
            relationship_type: "has_role".to_owned(),
            trust_score: 1.0,
            strength_score: 0.7,
            confidence: 1.0,
            review_state,
            valid_from: Some(role.assigned_at),
            valid_to: None,
            metadata: json!({
                "compatibility_source": "person_roles",
                "compatibility_record_id": role.id,
                "role": role.role,
                "assigned_by": role.assigned_by,
                "assigned_at": role.assigned_at,
            }),
        };
        let evidence = NewRelationshipEvidence::new(
            RelationshipEvidenceSourceKind::RawRecord,
            role.id.clone(),
        )
        .excerpt(role.role.clone())
        .metadata(json!({
            "compatibility_source": "person_roles",
            "person_id": role.person_id,
            "role": role.role,
            "assigned_by": role.assigned_by,
            "review_state": review_state.as_str(),
        }));

        RelationshipStore::upsert_with_evidence_in_transaction(
            transaction,
            &relationship,
            &[evidence],
        )
        .await?;

        Ok(())
    }
}

fn row_to_role(row: PgRow) -> Result<PersonRole, PersonCoreError> {
    Ok(PersonRole {
        id: row.try_get("id")?,
        person_id: row.try_get("person_id")?,
        role: row.try_get("role")?,
        assigned_by: row.try_get("assigned_by")?,
        assigned_at: row.try_get("assigned_at")?,
    })
}

fn person_role_knowledge_id(role: &str) -> String {
    let mut slug = String::new();
    let mut previous_was_separator = false;

    for character in role.trim().to_ascii_lowercase().chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            previous_was_separator = false;
        } else if !slug.is_empty() && !previous_was_separator {
            slug.push('_');
            previous_was_separator = true;
        }
    }

    while slug.ends_with('_') {
        slug.pop();
    }

    if slug.is_empty() {
        "person_role:unspecified".to_owned()
    } else {
        format!("person_role:{slug}")
    }
}

// ── PersonPersona ───────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonPersona {
    pub persona_id: String,
    pub person_id: String,
    pub name: String,
    pub context: Option<String>,
    pub default_tone: Option<String>,
    pub default_language: Option<String>,
    pub preferred_channel: Option<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PersonPersonaStore {
    pool: PgPool,
}

impl PersonPersonaStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_by_person(
        &self,
        person_id: &str,
    ) -> Result<Vec<PersonPersona>, PersonCoreError> {
        let rows = sqlx::query(
            r#"SELECT persona_id, person_id, name, context, default_tone, default_language,
               preferred_channel, metadata, created_at, updated_at
               FROM person_personas WHERE person_id = $1 ORDER BY name"#,
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_persona).collect()
    }

    pub async fn upsert(
        &self,
        persona: &NewPersonPersona,
    ) -> Result<PersonPersona, PersonCoreError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"INSERT INTO person_personas (persona_id, person_id, name, context, default_tone,
               default_language, preferred_channel)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               ON CONFLICT (persona_id)
               DO UPDATE SET name = EXCLUDED.name, context = EXCLUDED.context,
                             default_tone = EXCLUDED.default_tone,
                             default_language = EXCLUDED.default_language,
                             preferred_channel = EXCLUDED.preferred_channel,
                             updated_at = now()
               RETURNING persona_id, person_id, name, context, default_tone, default_language,
                         preferred_channel, metadata, created_at, updated_at"#,
        )
        .bind(&persona.persona_id)
        .bind(&persona.person_id)
        .bind(&persona.name)
        .bind(&persona.context)
        .bind(&persona.default_tone)
        .bind(&persona.default_language)
        .bind(&persona.preferred_channel)
        .fetch_one(&mut *transaction)
        .await?;
        let persona = row_to_persona(row)?;

        Self::materialize_interaction_preferences_in_transaction(&mut transaction, &persona)
            .await?;
        transaction.commit().await?;

        Ok(persona)
    }

    pub async fn delete(&self, persona_id: &str) -> Result<bool, PersonCoreError> {
        let mut transaction = self.pool.begin().await?;
        let existing_persona = sqlx::query(
            r#"SELECT persona_id, person_id, name, context, default_tone, default_language,
               preferred_channel, metadata, created_at, updated_at
               FROM person_personas
               WHERE persona_id = $1
               FOR UPDATE"#,
        )
        .bind(persona_id)
        .fetch_optional(&mut *transaction)
        .await?
        .map(row_to_persona)
        .transpose()?;

        let result = sqlx::query("DELETE FROM person_personas WHERE persona_id = $1")
            .bind(persona_id)
            .execute(&mut *transaction)
            .await?;
        let deleted = result.rows_affected() > 0;

        if let Some(existing_persona) = existing_persona
            && deleted
        {
            Self::delete_interaction_preferences_in_transaction(
                &mut transaction,
                &existing_persona,
            )
            .await?;
        }

        transaction.commit().await?;

        Ok(deleted)
    }

    async fn materialize_interaction_preferences_in_transaction(
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

    async fn delete_interaction_preferences_in_transaction(
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
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewPersonPersona {
    pub persona_id: String,
    pub person_id: String,
    pub name: String,
    pub context: Option<String>,
    pub default_tone: Option<String>,
    pub default_language: Option<String>,
    pub preferred_channel: Option<String>,
}

fn row_to_persona(row: PgRow) -> Result<PersonPersona, PersonCoreError> {
    Ok(PersonPersona {
        persona_id: row.try_get("persona_id")?,
        person_id: row.try_get("person_id")?,
        name: row.try_get("name")?,
        context: row.try_get("context")?,
        default_tone: row.try_get("default_tone")?,
        default_language: row.try_get("default_language")?,
        preferred_channel: row.try_get("preferred_channel")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
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

// ── Error type ──────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum PersonCoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Relationship(#[from] RelationshipStoreError),
    #[error("person identity not found")]
    IdentityNotFound,
    #[error("person persona not found")]
    PersonaNotFound,
}
