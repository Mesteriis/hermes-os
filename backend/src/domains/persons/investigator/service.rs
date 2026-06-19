use serde_json::{Value, json};
use sqlx::postgres::PgPool;

use super::assembly;
use super::errors::InvestigatorError;
use super::meeting_prep;
use super::models::{DossierReviewState, DossierSnapshot, MeetingPrep, PersonDossier};
use super::snapshots;
use crate::domains::persons::core::link_persons_entity;
use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, materialize_review_transition_link,
};

#[derive(Clone)]
pub struct PersonInvestigator {
    pool: PgPool,
}

impl PersonInvestigator {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn assemble_dossier(
        &self,
        person_id: &str,
    ) -> Result<PersonDossier, InvestigatorError> {
        assembly::assemble_dossier(&self.pool, person_id).await
    }

    pub async fn assemble_and_cache_dossier(
        &self,
        person_id: &str,
    ) -> Result<(PersonDossier, DossierSnapshot), InvestigatorError> {
        let dossier = self.assemble_dossier(person_id).await?;
        let snapshot = self.cache_dossier_snapshot(&dossier).await?;
        Ok((dossier, snapshot))
    }

    pub async fn assemble_cache_and_record_refresh(
        &self,
        person_id: &str,
        operation: &str,
        captured_by: &str,
        endpoint: &str,
        source_ref: String,
    ) -> Result<(PersonDossier, DossierSnapshot), InvestigatorError> {
        let observation = ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "PERSON_MUTATION",
                    ObservationOriginKind::Manual,
                    chrono::Utc::now(),
                    json!({
                        "person_id": person_id,
                        "operation": operation,
                    }),
                    source_ref,
                )
                .provenance(json!({
                    "captured_by": captured_by,
                    "endpoint": endpoint,
                })),
            )
            .await?;
        let (dossier, snapshot) = self.assemble_and_cache_dossier(person_id).await?;
        link_persons_entity(
            &self.pool,
            &observation.observation_id,
            "dossier_snapshot",
            snapshot.dossier_snapshot_id.clone(),
            Some("dossier_refresh"),
            Some(json!({
                "person_id": person_id,
                "trigger": endpoint,
            })),
        )
        .await?;
        Ok((dossier, snapshot))
    }

    pub async fn cache_dossier_snapshot(
        &self,
        dossier: &PersonDossier,
    ) -> Result<DossierSnapshot, InvestigatorError> {
        snapshots::cache_dossier_snapshot(&self.pool, dossier).await
    }

    pub async fn review_dossier_snapshot(
        &self,
        person_id: &str,
        review_state: DossierReviewState,
    ) -> Result<DossierSnapshot, InvestigatorError> {
        self.review_dossier_snapshot_with_observation(person_id, review_state, None, None)
            .await
    }

    pub async fn review_dossier_snapshot_with_observation(
        &self,
        person_id: &str,
        review_state: DossierReviewState,
        observation_id: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<DossierSnapshot, InvestigatorError> {
        let snapshot =
            snapshots::review_dossier_snapshot(&self.pool, person_id, review_state).await?;
        materialize_review_transition_link(
            &self.pool,
            observation_id,
            "persons",
            "dossier_snapshot",
            &snapshot.dossier_snapshot_id,
            "review_state",
            snapshot.review_state.as_str(),
            metadata
                .map(|extra| {
                    json!({
                        "person_id": person_id,
                        "context": extra,
                    })
                })
                .or_else(|| {
                    Some(json!({
                        "person_id": person_id,
                    }))
                }),
        )
        .await?;
        Ok(snapshot)
    }

    pub async fn meeting_prep(&self, person_id: &str) -> Result<MeetingPrep, InvestigatorError> {
        meeting_prep::meeting_prep(&self.pool, person_id).await
    }
}
