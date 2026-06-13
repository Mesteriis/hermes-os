use std::collections::HashSet;

use crate::domains::graph::core::{GraphNodeKind, GraphStore, NewGraphNode};
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::json;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PersonaType {
    Human,
    AiAgent,
    OrganizationProxy,
    System,
}

impl PersonaType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::AiAgent => "ai_agent",
            Self::OrganizationProxy => "organization_proxy",
            Self::System => "system",
        }
    }
}

impl TryFrom<&str> for PersonaType {
    type Error = PersonProjectionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "human" => Ok(Self::Human),
            "ai_agent" => Ok(Self::AiAgent),
            "organization_proxy" => Ok(Self::OrganizationProxy),
            "system" => Ok(Self::System),
            _ => Err(PersonProjectionError::InvalidPersonaType(value.to_owned())),
        }
    }
}

#[derive(Clone)]
pub struct PersonProjectionStore {
    pool: PgPool,
}

impl PersonProjectionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_email_person(
        &self,
        email_address: &str,
    ) -> Result<Person, PersonProjectionError> {
        let mut transaction = self.pool.begin().await?;
        let person =
            Self::upsert_email_person_in_transaction(&mut transaction, email_address).await?;
        transaction.commit().await?;
        Ok(person)
    }

    pub(crate) async fn upsert_email_person_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        email_address: &str,
    ) -> Result<Person, PersonProjectionError> {
        let normalized_email = normalize_email_address(email_address)?;
        let person_id = person_id_for_email(&normalized_email);

        let row = sqlx::query(
            r#"
            INSERT INTO persons (
                person_id,
                display_name,
                email_address
            )
            VALUES ($1, $2, $3)
            ON CONFLICT (email_address)
            DO UPDATE SET
                display_name = EXCLUDED.display_name,
                updated_at = now()
            RETURNING
                person_id,
                display_name,
                email_address,
                person_type,
                is_self,
                created_at,
                updated_at
            "#,
        )
        .bind(&person_id)
        .bind(&normalized_email)
        .bind(&normalized_email)
        .fetch_one(&mut **transaction)
        .await?;

        let person = row_to_person(row)?;
        sqlx::query(
            r#"
            INSERT INTO person_identities (person_id, identity_type, identity_value, source, confidence, status)
            VALUES ($1, 'email', $2, 'email_sync', 1.0, 'active')
            ON CONFLICT (identity_type, identity_value) WHERE status = 'active'
            DO UPDATE SET
                person_id = EXCLUDED.person_id,
                source = EXCLUDED.source,
                confidence = EXCLUDED.confidence,
                last_verified_at = now(),
                updated_at = now()
            "#,
        )
        .bind(&person.person_id)
        .bind(&normalized_email)
        .execute(&mut **transaction)
        .await?;

        Ok(person)
    }

    pub async fn owner_persona(&self) -> Result<Option<Person>, PersonProjectionError> {
        let row = sqlx::query(
            r#"
            SELECT
                person_id,
                display_name,
                email_address,
                person_type,
                is_self,
                created_at,
                updated_at
            FROM persons
            WHERE is_self = true
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_person).transpose()
    }

    pub async fn list_personas(&self, limit: i64) -> Result<Vec<Person>, PersonProjectionError> {
        let rows = sqlx::query(
            r#"
            SELECT
                person_id,
                display_name,
                email_address,
                person_type,
                is_self,
                created_at,
                updated_at
            FROM persons
            ORDER BY updated_at DESC, created_at DESC, person_id
            LIMIT $1
            "#,
        )
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_person).collect()
    }

    pub async fn get_persona(
        &self,
        persona_id: &str,
    ) -> Result<Option<Person>, PersonProjectionError> {
        let row = sqlx::query(
            r#"
            SELECT
                person_id,
                display_name,
                email_address,
                person_type,
                is_self,
                created_at,
                updated_at
            FROM persons
            WHERE person_id = $1
            "#,
        )
        .bind(persona_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_person).transpose()
    }

    pub async fn update_persona(
        &self,
        persona_id: &str,
        display_name: Option<&str>,
        set_self: bool,
    ) -> Result<Person, PersonProjectionError> {
        let mut transaction = self.pool.begin().await?;

        if let Some(display_name) = display_name {
            let display_name = validate_display_name(display_name)?;
            let result = sqlx::query(
                r#"
                UPDATE persons
                SET display_name = $2, updated_at = now()
                WHERE person_id = $1
                "#,
            )
            .bind(persona_id)
            .bind(&display_name)
            .execute(&mut *transaction)
            .await?;
            if result.rows_affected() == 0 {
                return Err(PersonProjectionError::PersonNotFound(persona_id.to_owned()));
            }

            sqlx::query(
                r#"
                UPDATE graph_nodes
                SET label = $2, updated_at = now()
                WHERE node_kind = $3 AND stable_key = $1
                "#,
            )
            .bind(persona_id)
            .bind(&display_name)
            .bind(GraphNodeKind::Person.as_str())
            .execute(&mut *transaction)
            .await?;
        }

        if set_self {
            sqlx::query(
                r#"
                UPDATE persons
                SET is_self = false, updated_at = now()
                WHERE is_self = true AND person_id <> $1
                "#,
            )
            .bind(persona_id)
            .execute(&mut *transaction)
            .await?;

            let result = sqlx::query(
                r#"
                UPDATE persons
                SET is_self = true, updated_at = now()
                WHERE person_id = $1
                "#,
            )
            .bind(persona_id)
            .execute(&mut *transaction)
            .await?;
            if result.rows_affected() == 0 {
                return Err(PersonProjectionError::PersonNotFound(persona_id.to_owned()));
            }
        }

        let row = sqlx::query(
            r#"
            SELECT
                person_id,
                display_name,
                email_address,
                person_type,
                is_self,
                created_at,
                updated_at
            FROM persons
            WHERE person_id = $1
            "#,
        )
        .bind(persona_id)
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or_else(|| PersonProjectionError::PersonNotFound(persona_id.to_owned()))?;

        let person = row_to_person(row)?;
        transaction.commit().await?;
        Ok(person)
    }

    pub async fn set_owner_persona(
        &self,
        person_id: &str,
    ) -> Result<Person, PersonProjectionError> {
        let mut transaction = self.pool.begin().await?;

        sqlx::query(
            r#"
            UPDATE persons
            SET is_self = false, updated_at = now()
            WHERE is_self = true AND person_id <> $1
            "#,
        )
        .bind(person_id)
        .execute(&mut *transaction)
        .await?;

        let row = sqlx::query(
            r#"
            UPDATE persons
            SET is_self = true, updated_at = now()
            WHERE person_id = $1
            RETURNING
                person_id,
                display_name,
                email_address,
                person_type,
                is_self,
                created_at,
                updated_at
            "#,
        )
        .bind(person_id)
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or_else(|| PersonProjectionError::PersonNotFound(person_id.to_owned()))?;

        let person = row_to_person(row)?;
        transaction.commit().await?;
        Ok(person)
    }

    pub async fn set_persona_type(
        &self,
        person_id: &str,
        persona_type: PersonaType,
    ) -> Result<Person, PersonProjectionError> {
        let row = sqlx::query(
            r#"
            UPDATE persons
            SET person_type = $2, updated_at = now()
            WHERE person_id = $1
            RETURNING
                person_id,
                display_name,
                email_address,
                person_type,
                is_self,
                created_at,
                updated_at
            "#,
        )
        .bind(person_id)
        .bind(persona_type.as_str())
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersonProjectionError::PersonNotFound(person_id.to_owned()))?;

        row_to_person(row)
    }

    pub async fn upsert_ai_agent_persona(
        &self,
        agent_id: &str,
        display_name: &str,
    ) -> Result<Person, PersonProjectionError> {
        let normalized_agent_id = normalize_ai_agent_id(agent_id)?;
        validate_display_name(display_name)?;
        let person_id = ai_agent_person_id(&normalized_agent_id);
        let email_address = ai_agent_email_address(&normalized_agent_id);
        let mut transaction = self.pool.begin().await?;

        let row = sqlx::query(
            r#"
            INSERT INTO persons (
                person_id,
                display_name,
                email_address,
                person_type,
                is_self
            )
            VALUES ($1, $2, $3, 'ai_agent', false)
            ON CONFLICT (person_id)
            DO UPDATE SET
                display_name = EXCLUDED.display_name,
                email_address = EXCLUDED.email_address,
                person_type = 'ai_agent',
                is_self = false,
                updated_at = now()
            RETURNING
                person_id,
                display_name,
                email_address,
                person_type,
                is_self,
                created_at,
                updated_at
            "#,
        )
        .bind(&person_id)
        .bind(&email_address)
        .bind(&email_address)
        .fetch_one(&mut *transaction)
        .await?;

        let person = row_to_person(row)?;
        sqlx::query(
            r#"
            INSERT INTO person_identities (
                person_id,
                identity_type,
                identity_value,
                source,
                confidence,
                status,
                metadata
            )
            VALUES (
                $1,
                'email',
                $2,
                'ai_agent_registry',
                1.0,
                'active',
                jsonb_build_object('agent_id', $3, 'persona_type', 'ai_agent')
            )
            ON CONFLICT (identity_type, identity_value) WHERE status = 'active'
            DO UPDATE SET
                person_id = EXCLUDED.person_id,
                source = EXCLUDED.source,
                confidence = EXCLUDED.confidence,
                metadata = EXCLUDED.metadata,
                last_verified_at = now(),
                updated_at = now()
            "#,
        )
        .bind(&person.person_id)
        .bind(&email_address)
        .bind(&normalized_agent_id)
        .execute(&mut *transaction)
        .await?;

        GraphStore::upsert_node_in_transaction(
            &mut transaction,
            &NewGraphNode::new(
                GraphNodeKind::Person,
                &person.person_id,
                &person.display_name,
            )
            .properties(json!({
                "persona_type": PersonaType::AiAgent.as_str(),
                "agent_id": normalized_agent_id,
                "email_address": email_address,
                "source": "ai_agent_registry"
            })),
        )
        .await?;

        transaction.commit().await?;
        Ok(person)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Person {
    pub person_id: String,
    pub display_name: String,
    pub email_address: String,
    pub persona_type: PersonaType,
    pub is_self: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn upsert_persons_from_message_participants(
    store: &PersonProjectionStore,
    email_addresses: &[String],
) -> Result<Vec<Person>, PersonProjectionError> {
    let normalized_email_addresses = normalize_email_addresses(email_addresses)?;
    let mut persons = Vec::new();

    for email_address in normalized_email_addresses {
        persons.push(store.upsert_email_person(&email_address).await?);
    }

    Ok(persons)
}

fn normalize_email_addresses(
    email_addresses: &[String],
) -> Result<Vec<String>, PersonProjectionError> {
    let mut seen = HashSet::new();
    let mut normalized_email_addresses = Vec::new();

    for email_address in email_addresses {
        let normalized_email = normalize_email_address(email_address)?;
        if seen.insert(normalized_email.clone()) {
            normalized_email_addresses.push(normalized_email);
        }
    }

    Ok(normalized_email_addresses)
}

fn row_to_person(row: PgRow) -> Result<Person, PersonProjectionError> {
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

fn normalize_email_address(email_address: &str) -> Result<String, PersonProjectionError> {
    let normalized_email = email_addr_spec(email_address).trim().to_ascii_lowercase();
    if normalized_email.is_empty() {
        return Err(PersonProjectionError::EmptyEmailAddress);
    }
    if !normalized_email.contains('@') {
        return Err(PersonProjectionError::InvalidEmailAddress(normalized_email));
    }

    Ok(normalized_email)
}

fn email_addr_spec(value: &str) -> &str {
    let value = value.trim();
    if let Some((_, tail)) = value.rsplit_once('<') {
        if let Some((addr, _)) = tail.split_once('>') {
            return addr.trim();
        }
    }
    value.trim_matches('"')
}

fn person_id_for_email(normalized_email: &str) -> String {
    let mut encoded = String::from("person:v1:email:");
    encoded.push_str(&normalized_email.len().to_string());
    encoded.push(':');
    encoded.push_str(normalized_email);
    encoded
}

fn normalize_ai_agent_id(agent_id: &str) -> Result<String, PersonProjectionError> {
    let normalized = agent_id.trim().to_ascii_uppercase();
    if normalized.is_empty() {
        return Err(PersonProjectionError::EmptyAiAgentId);
    }
    if !normalized
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        return Err(PersonProjectionError::InvalidAiAgentId(agent_id.to_owned()));
    }
    Ok(normalized)
}

fn validate_display_name(display_name: &str) -> Result<String, PersonProjectionError> {
    let display_name = display_name.trim();
    if display_name.is_empty() {
        return Err(PersonProjectionError::EmptyDisplayName);
    }
    Ok(display_name.to_owned())
}

fn ai_agent_person_id(agent_id: &str) -> String {
    format!("persona:v1:ai_agent:{agent_id}")
}

fn ai_agent_email_address(agent_id: &str) -> String {
    format!("{}@sh-inc.ru", agent_id.to_ascii_lowercase())
}

#[derive(Debug, Error)]
pub enum PersonProjectionError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("email address must not be empty")]
    EmptyEmailAddress,

    #[error("invalid email address: {0}")]
    InvalidEmailAddress(String),

    #[error("AI agent id must not be empty")]
    EmptyAiAgentId,

    #[error("invalid AI agent id: {0}")]
    InvalidAiAgentId(String),

    #[error("display name must not be empty")]
    EmptyDisplayName,

    #[error("person was not found: {0}")]
    PersonNotFound(String),

    #[error("invalid persona type: {0}")]
    InvalidPersonaType(String),

    #[error(transparent)]
    Graph(#[from] crate::domains::graph::core::GraphStoreError),
}
