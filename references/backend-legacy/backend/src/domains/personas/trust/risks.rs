use sqlx::Row;
use sqlx::postgres::PgPool;

use crate::engines::risk::engine::RiskEngine;

use super::errors::PersonaTrustError;
use super::health_projection::sync_persona_health_status_in_transaction;
use super::models::PersonaRisk;
use super::rows::row_to_risk;

#[derive(Clone)]
pub struct PersonaRiskStore {
    pool: PgPool,
}

impl PersonaRiskStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, persona_id: &str) -> Result<Vec<PersonaRisk>, PersonaTrustError> {
        let rows = sqlx::query(
            "SELECT id::text, persona_id, risk_type, description, severity, source, confidence::float8 AS confidence,
             created_at, resolved_at, resolution
             FROM persona_risks WHERE persona_id = $1 ORDER BY created_at DESC",
        )
        .bind(persona_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_risk).collect()
    }

    pub async fn report(
        &self,
        persona_id: &str,
        risk_type: &str,
        description: &str,
        severity: &str,
        source: &str,
    ) -> Result<PersonaRisk, PersonaTrustError> {
        let observation =
            RiskEngine::persona_observation(persona_id, risk_type, description, severity, source)?;
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            "INSERT INTO persona_risks (persona_id, risk_type, description, severity, source, confidence)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id::text, persona_id, risk_type, description, severity, source, confidence::float8 AS confidence,
                       created_at, resolved_at, resolution",
        )
        .bind(&observation.affected_entity_id)
        .bind(&observation.risk_type)
        .bind(&observation.evidence)
        .bind(observation.severity.as_str())
        .bind(&observation.source)
        .bind(observation.confidence)
        .fetch_one(&mut *transaction)
        .await?;
        let risk = row_to_risk(row)?;
        sync_persona_health_status_in_transaction(&mut transaction, persona_id).await?;
        transaction.commit().await?;
        Ok(risk)
    }

    pub async fn resolve(&self, id: &str, resolution: &str) -> Result<(), PersonaTrustError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            "UPDATE persona_risks
             SET resolved_at = now(), resolution = $2
             WHERE id::text = $1
             RETURNING persona_id",
        )
        .bind(id)
        .bind(resolution)
        .fetch_optional(&mut *transaction)
        .await?;
        if let Some(row) = row {
            let persona_id: String = row.try_get("persona_id")?;
            sync_persona_health_status_in_transaction(&mut transaction, &persona_id).await?;
        }
        transaction.commit().await?;
        Ok(())
    }
}
