use sqlx::postgres::PgPool;

use super::errors::InvestigatorError;
use super::models::MeetingPrep;
use crate::domains::persons::enrichment::PersonEnrichmentStore;
use crate::domains::persons::trust::{PersonPromiseStore, PersonRiskStore};

pub(super) async fn meeting_prep(
    pool: &PgPool,
    person_id: &str,
) -> Result<MeetingPrep, InvestigatorError> {
    let enrichment = PersonEnrichmentStore::new(pool.clone());
    let person = enrichment
        .get_enriched(person_id)
        .await?
        .ok_or(InvestigatorError::PersonNotFound)?;

    let last_interaction_days = person
        .last_interaction_at
        .map(|dt| (chrono::Utc::now() - dt).num_days());

    let promises = PersonPromiseStore::new(pool.clone());
    let risks = PersonRiskStore::new(pool.clone());
    let open_promises = promises
        .list(person_id)
        .await
        .unwrap_or_default()
        .iter()
        .filter(|promise| promise.status == "pending")
        .count() as i64;
    let open_risks = risks
        .list(person_id)
        .await
        .unwrap_or_default()
        .iter()
        .filter(|risk| risk.resolved_at.is_none())
        .count() as i64;

    let mut tips = person
        .frequent_topics
        .iter()
        .map(|topic| format!("Discuss topic: {topic}"))
        .collect::<Vec<_>>();
    if let Some(tone) = &person.tone {
        tips.push(format!("Match tone: {tone}"));
    }
    if let Some(style) = &person.writing_style {
        tips.push(format!("Style: {style}"));
    }

    Ok(MeetingPrep {
        person_id: person.person_id,
        display_name: person.display_name,
        last_interaction_days,
        open_promises,
        open_risks,
        recent_topics: person.frequent_topics,
        communication_tips: tips,
        shared_projects: person.linked_projects,
    })
}
