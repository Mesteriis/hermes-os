use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};

use super::errors::PersonCoreError;
use super::link_persons_entity_in_transaction;

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
        self.assign_with_observation(person_id, role, assigned_by, None)
            .await
    }

    pub async fn assign_with_observation(
        &self,
        person_id: &str,
        role: &str,
        assigned_by: Option<&str>,
        observation_id: Option<&str>,
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

        if let Some(observation_id) = observation_id {
            link_persons_entity_in_transaction(
                &mut transaction,
                observation_id,
                "role",
                role.id.clone(),
                None,
                Some(json!({
                    "person_id": person_id,
                    "role": role.role,
                    "action": "assign",
                })),
            )
            .await?;
        }
        transaction.commit().await?;

        Ok(role)
    }

    pub async fn remove(&self, person_id: &str, role: &str) -> Result<bool, PersonCoreError> {
        self.remove_with_observation(person_id, role, None).await
    }

    pub async fn remove_with_observation(
        &self,
        person_id: &str,
        role: &str,
        observation_id: Option<&str>,
    ) -> Result<bool, PersonCoreError> {
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
            && let Some(observation_id) = observation_id
        {
            link_persons_entity_in_transaction(
                &mut transaction,
                observation_id,
                "role",
                format!("{person_id}:{role}"),
                None,
                Some(json!({
                    "person_id": existing_role.person_id,
                    "role": existing_role.role,
                    "action": "delete",
                    "deleted": removed,
                })),
            )
            .await?;
        }

        transaction.commit().await?;

        Ok(removed)
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
