use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonaExpertise {
    pub id: String,
    #[serde(rename = "persona_id", alias = "person_id")]
    pub person_id: String,
    pub skill: String,
    pub domain: Option<String>,
    pub evidence: Option<String>,
    pub source: String,
    pub confidence: f64,
    pub last_verified_at: Option<DateTime<Utc>>,
    #[serde(rename = "endorsed_by_persona_id", alias = "endorsed_by_person_id")]
    pub endorsed_by_person_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PersonaExpertiseStore {
    pool: PgPool,
}

impl PersonaExpertiseStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        person_id: &str,
    ) -> Result<Vec<PersonaExpertise>, PersonaExpertiseError> {
        let rows = sqlx::query(
            "SELECT id::text, person_id, skill, domain, evidence, source, confidence::float8 AS confidence,
             last_verified_at, endorsed_by_person_id, created_at, updated_at
             FROM persona_expertise WHERE person_id = $1 ORDER BY confidence DESC",
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_expertise).collect()
    }

    pub async fn search_by_skill(
        &self,
        skill: &str,
        limit: i64,
    ) -> Result<Vec<PersonaExpertise>, PersonaExpertiseError> {
        let pattern = format!("%{}%", skill.trim().to_lowercase());
        let rows = sqlx::query(
            "SELECT id::text, person_id, skill, domain, evidence, source, confidence::float8 AS confidence,
             last_verified_at, endorsed_by_person_id, created_at, updated_at
             FROM persona_expertise WHERE lower(skill) LIKE $1 ORDER BY confidence DESC LIMIT $2",
        )
        .bind(&pattern)
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_expertise).collect()
    }

    pub async fn upsert(
        &self,
        person_id: &str,
        skill: &str,
        domain: Option<&str>,
        source: &str,
        confidence: f64,
    ) -> Result<PersonaExpertise, PersonaExpertiseError> {
        let row = sqlx::query(
            "INSERT INTO persona_expertise (person_id, skill, domain, source, confidence)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT DO NOTHING
             RETURNING id::text, person_id, skill, domain, evidence, source, confidence::float8 AS confidence,
                       last_verified_at, endorsed_by_person_id, created_at, updated_at",
        )
        .bind(person_id)
        .bind(skill)
        .bind(domain)
        .bind(source)
        .bind(confidence)
        .fetch_one(&self.pool)
        .await?;
        row_to_expertise(row)
    }
}

fn row_to_expertise(row: PgRow) -> Result<PersonaExpertise, PersonaExpertiseError> {
    Ok(PersonaExpertise {
        id: row.try_get("id")?,
        person_id: row.try_get("person_id")?,
        skill: row.try_get("skill")?,
        domain: row.try_get("domain")?,
        evidence: row.try_get("evidence")?,
        source: row.try_get("source")?,
        confidence: row.try_get("confidence")?,
        last_verified_at: row.try_get("last_verified_at")?,
        endorsed_by_person_id: row.try_get("endorsed_by_person_id")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[derive(Debug, Error)]
pub enum PersonaExpertiseError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
