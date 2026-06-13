use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};
use thiserror::Error;

use crate::domains::obligations::{
    NewObligation, NewObligationEvidence, ObligationEntityKind, ObligationEvidenceSourceKind,
    ObligationReviewState, ObligationStatus, ObligationStore, ObligationStoreError,
};
use crate::engines::risk::{RiskEngine, RiskEngineError, RiskSeverity, RiskSignal};

// ── PersonPromises ─────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonPromise {
    pub id: String,
    pub person_id: String,
    pub description: String,
    pub source_message_id: Option<String>,
    pub promised_at: DateTime<Utc>,
    pub due_at: Option<DateTime<Utc>>,
    pub fulfilled_at: Option<DateTime<Utc>>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PersonPromiseStore {
    pool: PgPool,
}

impl PersonPromiseStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, person_id: &str) -> Result<Vec<PersonPromise>, PersonTrustError> {
        let rows = sqlx::query(
            "SELECT id::text, person_id, description, source_message_id, promised_at,
             due_at, fulfilled_at, status, created_at, updated_at
             FROM person_promises WHERE person_id = $1 ORDER BY promised_at DESC",
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_promise).collect()
    }

    pub async fn create(
        &self,
        person_id: &str,
        description: &str,
        due_at: Option<DateTime<Utc>>,
    ) -> Result<PersonPromise, PersonTrustError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            "INSERT INTO person_promises (person_id, description, due_at)
             VALUES ($1, $2, $3)
             RETURNING id::text, person_id, description, source_message_id, promised_at,
                       due_at, fulfilled_at, status, created_at, updated_at",
        )
        .bind(person_id)
        .bind(description)
        .bind(due_at)
        .fetch_one(&mut *transaction)
        .await?;
        let promise = row_to_promise(row)?;
        Self::project_promise_obligation(&mut transaction, &promise).await?;
        transaction.commit().await?;

        Ok(promise)
    }

    pub async fn fulfill(&self, id: &str) -> Result<(), PersonTrustError> {
        sqlx::query("UPDATE person_promises SET status = 'fulfilled', fulfilled_at = now(), updated_at = now() WHERE id::text = $1")
            .bind(id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn mark_broken(&self, id: &str) -> Result<(), PersonTrustError> {
        sqlx::query(
            "UPDATE person_promises SET status = 'broken', updated_at = now() WHERE id::text = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn project_promise_obligation(
        transaction: &mut Transaction<'_, Postgres>,
        promise: &PersonPromise,
    ) -> Result<(), PersonTrustError> {
        let mut obligation = NewObligation::new(
            ObligationEntityKind::Persona,
            promise.person_id.clone(),
            promise.description.clone(),
            1.0,
            ObligationReviewState::UserConfirmed,
        )
        .status(person_promise_status_to_obligation_status(&promise.status))
        .metadata(person_promise_metadata(promise));
        if let Some(due_at) = promise.due_at {
            obligation = obligation.due_at(due_at);
        }

        let evidence =
            NewObligationEvidence::new(ObligationEvidenceSourceKind::RawRecord, promise.id.clone())
                .quote(promise.description.clone())
                .confidence(1.0)
                .metadata(person_promise_metadata(promise));

        ObligationStore::upsert_with_evidence_in_transaction(transaction, &obligation, &[evidence])
            .await?;

        Ok(())
    }
}

fn person_promise_metadata(promise: &PersonPromise) -> Value {
    json!({
        "source": "person_promise_adapter",
        "person_promise_id": promise.id,
        "person_id": promise.person_id,
        "promise_status": promise.status,
        "source_message_id": promise.source_message_id,
    })
}

fn person_promise_status_to_obligation_status(status: &str) -> ObligationStatus {
    match status {
        "fulfilled" => ObligationStatus::Fulfilled,
        "broken" => ObligationStatus::Disputed,
        _ => ObligationStatus::Open,
    }
}

fn row_to_promise(row: PgRow) -> Result<PersonPromise, PersonTrustError> {
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

// ── PersonRisks ────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonRisk {
    pub id: String,
    pub person_id: String,
    pub risk_type: String,
    pub description: String,
    pub severity: String,
    pub source: String,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution: Option<String>,
}

#[derive(Clone)]
pub struct PersonRiskStore {
    pool: PgPool,
}

impl PersonRiskStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, person_id: &str) -> Result<Vec<PersonRisk>, PersonTrustError> {
        let rows = sqlx::query(
            "SELECT id::text, person_id, risk_type, description, severity, source, confidence::float8 AS confidence,
             created_at, resolved_at, resolution
             FROM person_risks WHERE person_id = $1 ORDER BY created_at DESC",
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_risk).collect()
    }

    pub async fn report(
        &self,
        person_id: &str,
        risk_type: &str,
        description: &str,
        severity: &str,
        source: &str,
    ) -> Result<PersonRisk, PersonTrustError> {
        let observation =
            RiskEngine::persona_observation(person_id, risk_type, description, severity, source)?;
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            "INSERT INTO person_risks (person_id, risk_type, description, severity, source, confidence)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id::text, person_id, risk_type, description, severity, source, confidence::float8 AS confidence,
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
        sync_person_health_status_in_transaction(&mut transaction, person_id).await?;
        transaction.commit().await?;
        Ok(risk)
    }

    pub async fn resolve(&self, id: &str, resolution: &str) -> Result<(), PersonTrustError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            "UPDATE person_risks SET resolved_at = now(), resolution = $2 WHERE id::text = $1 RETURNING person_id",
        )
        .bind(id)
        .bind(resolution)
        .fetch_optional(&mut *transaction)
        .await?;
        if let Some(row) = row {
            let person_id: String = row.try_get("person_id")?;
            sync_person_health_status_in_transaction(&mut transaction, &person_id).await?;
        }
        transaction.commit().await?;
        Ok(())
    }
}

async fn sync_person_health_status_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    person_id: &str,
) -> Result<(), PersonTrustError> {
    let rows = sqlx::query(
        r#"
        SELECT severity
        FROM person_risks
        WHERE person_id = $1
          AND resolved_at IS NULL
        "#,
    )
    .bind(person_id)
    .fetch_all(&mut **transaction)
    .await?;
    let risks = rows
        .into_iter()
        .map(|row| {
            let severity: String = row.try_get("severity")?;
            Ok(RiskSignal::unresolved(RiskSeverity::parse(&severity)?))
        })
        .collect::<Result<Vec<_>, PersonTrustError>>()?;
    let health_status = RiskEngine::derive_attention_status(&risks).as_persona_health_status();

    sqlx::query(
        "UPDATE persons
         SET health_status = $2, last_health_check = now(), updated_at = now()
         WHERE person_id = $1",
    )
    .bind(person_id)
    .bind(health_status)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

fn row_to_risk(row: PgRow) -> Result<PersonRisk, PersonTrustError> {
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

#[derive(Debug, Error)]
pub enum PersonTrustError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Obligation(#[from] ObligationStoreError),
    #[error(transparent)]
    RiskEngine(#[from] RiskEngineError),
}
