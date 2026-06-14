use sqlx::postgres::PgPool;

use super::assembly;
use super::errors::InvestigatorError;
use super::meeting_prep;
use super::models::{DossierReviewState, DossierSnapshot, MeetingPrep, PersonDossier};
use super::snapshots;

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
        snapshots::review_dossier_snapshot(&self.pool, person_id, review_state).await
    }

    pub async fn meeting_prep(&self, person_id: &str) -> Result<MeetingPrep, InvestigatorError> {
        meeting_prep::meeting_prep(&self.pool, person_id).await
    }
}
