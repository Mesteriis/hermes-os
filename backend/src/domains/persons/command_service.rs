use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, ObservationStoreError,
};

use super::api::{Person, PersonProjectionError, PersonProjectionStore};
use super::core::{
    NewPersonPersona, PersonCoreError, PersonIdentity, PersonPersona, PersonPersonaStore,
    PersonRole, PersonRoleStore, PersonsIdentityStore,
};
use super::enrichment::{PersonEnrichmentError, PersonEnrichmentStore};
use super::enrichment_engine::{EnrichmentEngineError, EnrichmentResultStore};
use super::health::{PersonHealthError, PersonHealthStore};
use super::identity::{
    PersonIdentityError, PersonIdentityReviewCommand, PersonIdentityReviewCommandResult,
    PersonIdentityStore,
};
use super::intelligence::{PersonIntelligenceService, PersonMessage};
use super::investigator::{
    DossierReviewState, DossierSnapshot, InvestigatorError, PersonInvestigator,
};
use super::memory::{
    NewRelationshipEvent, PersonFact, PersonFactStore, PersonMemoryCard, PersonMemoryCardStore,
    PersonMemoryError, PersonPreference, PersonPreferenceStore, RelationshipEvent,
    RelationshipEventStore,
};

#[derive(Clone)]
pub struct PersonCommandService {
    pool: PgPool,
}

impl PersonCommandService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_identity_trace_manual(
        &self,
        identity_type: &str,
        identity_value: &str,
        requested_source: &str,
    ) -> Result<PersonIdentity, PersonCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSON_RECORD_MUTATION",
                Utc::now(),
                json!({
                    "identity_type": identity_type,
                    "identity_value": identity_value,
                    "source": requested_source,
                }),
                format!("person-identity://trace/{identity_type}/{identity_value}"),
                json!({
                    "captured_by": "persons_service.create_identity_trace_manual",
                    "operation": "create_identity_trace_manual",
                    "requested_source": requested_source,
                }),
            )
            .await?;

        Ok(PersonsIdentityStore::new(self.pool.clone())
            .create_unattached_with_observation(
                identity_type,
                identity_value,
                &format!("observation:{}", observation.observation_id),
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn assign_identity_trace_manual(
        &self,
        identity_id: &str,
        person_id: &str,
    ) -> Result<PersonIdentity, PersonCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSON_RECORD_MUTATION",
                Utc::now(),
                json!({
                    "identity_id": identity_id,
                    "person_id": person_id,
                    "action": "attach_identity_trace",
                }),
                format!("person-identity://trace/{identity_id}/assignment"),
                json!({
                    "captured_by": "persons_service.assign_identity_trace_manual",
                    "operation": "assign_identity_trace_manual",
                }),
            )
            .await?;

        Ok(PersonsIdentityStore::new(self.pool.clone())
            .attach_to_persona_with_observation(identity_id, person_id, &observation.observation_id)
            .await?)
    }

    pub async fn upsert_person_identity_manual(
        &self,
        person_id: &str,
        identity_type: &str,
        identity_value: &str,
        requested_source: &str,
    ) -> Result<PersonIdentity, PersonCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSON_RECORD_MUTATION",
                Utc::now(),
                json!({
                    "person_id": person_id,
                    "identity_type": identity_type,
                    "identity_value": identity_value,
                    "source": requested_source,
                }),
                format!("person://{person_id}/identities/{identity_type}"),
                json!({
                    "captured_by": "persons_service.upsert_person_identity_manual",
                    "operation": "upsert_person_identity_manual",
                    "requested_source": requested_source,
                }),
            )
            .await?;

        Ok(PersonsIdentityStore::new(self.pool.clone())
            .upsert_with_observation(
                person_id,
                identity_type,
                identity_value,
                &format!("observation:{}", observation.observation_id),
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn delete_person_identity_manual(
        &self,
        person_id: &str,
        identity_id: &str,
    ) -> Result<bool, PersonCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSON_RECORD_MUTATION",
                Utc::now(),
                json!({
                    "person_id": person_id,
                    "identity_id": identity_id,
                    "action": "delete_identity",
                }),
                format!("person://{person_id}/identities/{identity_id}/delete"),
                json!({
                    "captured_by": "persons_service.delete_person_identity_manual",
                    "operation": "delete_person_identity_manual",
                }),
            )
            .await?;

        Ok(PersonsIdentityStore::new(self.pool.clone())
            .delete_with_observation(person_id, identity_id, &observation.observation_id)
            .await?)
    }

    pub async fn assign_role_manual(
        &self,
        person_id: &str,
        role: &str,
    ) -> Result<PersonRole, PersonCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSON_RECORD_MUTATION",
                Utc::now(),
                json!({
                    "person_id": person_id,
                    "role": role,
                    "action": "assign_role",
                }),
                format!("person://{person_id}/roles/{role}"),
                json!({
                    "captured_by": "persons_service.assign_role_manual",
                    "operation": "assign_role_manual",
                }),
            )
            .await?;

        Ok(PersonRoleStore::new(self.pool.clone())
            .assign_with_observation(person_id, role, None, Some(&observation.observation_id))
            .await?)
    }

    pub async fn remove_role_manual(
        &self,
        person_id: &str,
        role: &str,
    ) -> Result<bool, PersonCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSON_RECORD_MUTATION",
                Utc::now(),
                json!({
                    "person_id": person_id,
                    "role": role,
                    "action": "remove_role",
                }),
                format!("person://{person_id}/roles/{role}/delete"),
                json!({
                    "captured_by": "persons_service.remove_role_manual",
                    "operation": "remove_role_manual",
                }),
            )
            .await?;

        Ok(PersonRoleStore::new(self.pool.clone())
            .remove_with_observation(person_id, role, Some(&observation.observation_id))
            .await?)
    }

    pub async fn upsert_person_persona_manual(
        &self,
        persona: &NewPersonPersona,
    ) -> Result<PersonPersona, PersonCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSON_RECORD_MUTATION",
                Utc::now(),
                json!({
                    "person_id": persona.person_id,
                    "persona_id": persona.persona_id,
                    "name": persona.name,
                    "context": persona.context,
                    "default_tone": persona.default_tone,
                    "default_language": persona.default_language,
                    "preferred_channel": persona.preferred_channel,
                    "action": "upsert_persona",
                }),
                format!(
                    "person://{}/personas/{}",
                    persona.person_id, persona.persona_id
                ),
                json!({
                    "captured_by": "persons_service.upsert_person_persona_manual",
                    "operation": "upsert_person_persona_manual",
                }),
            )
            .await?;

        Ok(PersonPersonaStore::new(self.pool.clone())
            .upsert_with_observation(
                persona,
                Some(&format!("observation:{}", observation.observation_id)),
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn delete_person_persona_manual(
        &self,
        person_id: &str,
        persona_id: &str,
    ) -> Result<bool, PersonCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSON_RECORD_MUTATION",
                Utc::now(),
                json!({
                    "person_id": person_id,
                    "persona_id": persona_id,
                    "action": "delete_persona",
                }),
                format!("person://{person_id}/personas/{persona_id}/delete"),
                json!({
                    "captured_by": "persons_service.delete_person_persona_manual",
                    "operation": "delete_person_persona_manual",
                }),
            )
            .await?;

        Ok(PersonPersonaStore::new(self.pool.clone())
            .delete_with_observation(
                person_id,
                persona_id,
                Some(&format!("observation:{}", observation.observation_id)),
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn upsert_person_fact_manual(
        &self,
        person_id: &str,
        fact_type: &str,
        value: &str,
        requested_source: &str,
        confidence: f64,
    ) -> Result<PersonFact, PersonCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSON_RECORD_MUTATION",
                Utc::now(),
                json!({
                    "person_id": person_id,
                    "fact_type": fact_type,
                    "value": value,
                    "source": requested_source,
                    "confidence": confidence,
                }),
                format!("person://{person_id}/facts/{fact_type}"),
                json!({
                    "captured_by": "persons_service.upsert_person_fact_manual",
                    "operation": "upsert_person_fact_manual",
                    "requested_source": requested_source,
                }),
            )
            .await?;

        Ok(PersonFactStore::new(self.pool.clone())
            .upsert_with_observation(
                person_id,
                fact_type,
                value,
                &format!("observation:{}", observation.observation_id),
                confidence,
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn upsert_person_memory_card_manual(
        &self,
        person_id: &str,
        title: &str,
        description: &str,
        requested_source: &str,
        importance: i16,
    ) -> Result<PersonMemoryCard, PersonCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSON_MEMORY_CARD",
                Utc::now(),
                json!({
                    "person_id": person_id,
                    "title": title,
                    "description": description,
                    "source": requested_source,
                    "importance": importance,
                }),
                format!("person://{person_id}/memory-cards/{title}"),
                json!({
                    "captured_by": "persons_service.upsert_person_memory_card_manual",
                    "operation": "upsert_person_memory_card_manual",
                    "requested_source": requested_source,
                }),
            )
            .await?;

        Ok(PersonMemoryCardStore::new(self.pool.clone())
            .upsert_with_observation(
                person_id,
                title,
                description,
                &format!("observation:{}", observation.observation_id),
                importance,
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn upsert_person_preference_manual(
        &self,
        person_id: &str,
        preference_type: &str,
        value: &str,
        requested_source: &str,
    ) -> Result<PersonPreference, PersonCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSON_RECORD_MUTATION",
                Utc::now(),
                json!({
                    "person_id": person_id,
                    "preference_type": preference_type,
                    "value": value,
                    "source": requested_source,
                }),
                format!("person://{person_id}/preferences/{preference_type}"),
                json!({
                    "captured_by": "persons_service.upsert_person_preference_manual",
                    "operation": "upsert_person_preference_manual",
                    "requested_source": requested_source,
                }),
            )
            .await?;

        Ok(PersonPreferenceStore::new(self.pool.clone())
            .upsert_with_observation(
                person_id,
                preference_type,
                value,
                &format!("observation:{}", observation.observation_id),
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn add_relationship_event_manual(
        &self,
        event: &NewRelationshipEvent,
    ) -> Result<RelationshipEvent, PersonCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSON_RECORD_MUTATION",
                event.occurred_at,
                json!({
                    "person_id": event.person_id,
                    "event_type": event.event_type,
                    "title": event.title,
                    "description": event.description,
                    "occurred_at": event.occurred_at,
                    "requested_source": event.source,
                    "related_entity_id": event.related_entity_id,
                    "related_entity_kind": event.related_entity_kind,
                }),
                format!("person://{}/timeline", event.person_id),
                json!({
                    "captured_by": "persons_service.add_relationship_event_manual",
                    "operation": "add_relationship_event_manual",
                    "requested_source": event.source,
                }),
            )
            .await?;

        Ok(RelationshipEventStore::new(self.pool.clone())
            .add_with_observation(
                &NewRelationshipEvent {
                    person_id: event.person_id.clone(),
                    event_type: event.event_type.clone(),
                    title: event.title.clone(),
                    description: event.description.clone(),
                    occurred_at: event.occurred_at,
                    source: format!("observation:{}", observation.observation_id),
                    related_entity_id: event.related_entity_id.clone(),
                    related_entity_kind: event.related_entity_kind.clone(),
                },
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn apply_enrichment_manual(
        &self,
        person_id: &str,
        result_id: &str,
    ) -> Result<(), PersonCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "REVIEW_TRANSITION",
                Utc::now(),
                json!({
                    "person_id": person_id,
                    "result_id": result_id,
                    "operation": "enrichment_apply",
                }),
                format!("persona://{person_id}/enrichment/{result_id}/apply"),
                json!({
                    "captured_by": "persons_service.apply_enrichment_manual",
                    "operation": "apply_enrichment_manual",
                }),
            )
            .await?;

        EnrichmentResultStore::new(self.pool.clone())
            .apply_with_observation(
                result_id,
                Some(&observation.observation_id),
                Some(json!({
                    "captured_by": "persons_service.apply_enrichment_manual",
                    "operation": "apply_enrichment_manual",
                })),
            )
            .await?;
        Ok(())
    }

    pub async fn reject_enrichment_manual(
        &self,
        person_id: &str,
        result_id: &str,
    ) -> Result<(), PersonCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "REVIEW_TRANSITION",
                Utc::now(),
                json!({
                    "person_id": person_id,
                    "result_id": result_id,
                    "operation": "enrichment_reject",
                }),
                format!("persona://{person_id}/enrichment/{result_id}/reject"),
                json!({
                    "captured_by": "persons_service.reject_enrichment_manual",
                    "operation": "reject_enrichment_manual",
                }),
            )
            .await?;

        EnrichmentResultStore::new(self.pool.clone())
            .reject_with_observation(
                result_id,
                Some(&observation.observation_id),
                Some(json!({
                    "captured_by": "persons_service.reject_enrichment_manual",
                    "operation": "reject_enrichment_manual",
                })),
            )
            .await?;
        Ok(())
    }

    pub async fn toggle_watchlist_manual(
        &self,
        person_id: &str,
    ) -> Result<bool, PersonCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSON_MUTATION",
                Utc::now(),
                json!({
                    "person_id": person_id,
                    "action": "toggle_watchlist",
                }),
                format!("person://{person_id}/watchlist"),
                json!({
                    "captured_by": "persons_service.toggle_watchlist_manual",
                    "operation": "toggle_watchlist_manual",
                }),
            )
            .await?;

        Ok(PersonHealthStore::new(self.pool.clone())
            .toggle_watchlist_with_observation(
                person_id,
                &format!("observation:{}", observation.observation_id),
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn fingerprint_person_manual(
        &self,
        person_id: &str,
        person_messages: &[PersonMessage],
    ) -> Result<Value, PersonCommandServiceError> {
        let fingerprint = PersonIntelligenceService::heuristic_fingerprint(person_messages);

        let observation = self
            .capture_manual_at(
                "PERSON_MUTATION",
                Utc::now(),
                json!({
                    "person_id": person_id,
                    "action": "fingerprint_enrichment",
                    "detected_language": fingerprint.detected_language,
                    "typical_tone": fingerprint.typical_tone,
                    "trust_score": fingerprint.trust_score,
                    "avg_response_hours": fingerprint.avg_response_hours,
                    "writing_style": fingerprint.writing_style,
                }),
                format!("person://{person_id}/fingerprint"),
                json!({
                    "captured_by": "persons_service.fingerprint_person_manual",
                    "operation": "fingerprint_person_manual",
                }),
            )
            .await?;

        PersonEnrichmentStore::new(self.pool.clone())
            .enrich_person_with_observation(person_id, &fingerprint, &observation.observation_id)
            .await?;

        Ok(json!({
            "enriched": true,
            "fingerprint": fingerprint,
        }))
    }

    pub async fn toggle_favorite_manual(
        &self,
        person_id: &str,
    ) -> Result<bool, PersonCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSON_MUTATION",
                Utc::now(),
                json!({
                    "person_id": person_id,
                    "action": "toggle_favorite",
                }),
                format!("person://{person_id}/favorite"),
                json!({
                    "captured_by": "persons_service.toggle_favorite_manual",
                    "operation": "toggle_favorite_manual",
                }),
            )
            .await?;

        Ok(PersonEnrichmentStore::new(self.pool.clone())
            .toggle_favorite_with_observation(
                person_id,
                &format!("observation:{}", observation.observation_id),
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn set_notes_manual(
        &self,
        person_id: &str,
        notes: &str,
    ) -> Result<(), PersonCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSON_MEMORY_CARD",
                Utc::now(),
                json!({
                    "person_id": person_id,
                    "title": "Persona notes",
                    "body": notes,
                }),
                format!("person://{person_id}/notes"),
                json!({
                    "captured_by": "persons_service.set_notes_manual",
                    "operation": "set_notes_manual",
                }),
            )
            .await?;

        PersonEnrichmentStore::new(self.pool.clone())
            .set_notes_with_observation(
                person_id,
                notes,
                &format!("observation:{}", observation.observation_id),
                &observation.observation_id,
            )
            .await?;
        Ok(())
    }

    pub async fn set_owner_persona_manual(
        &self,
        person_id: &str,
    ) -> Result<Person, PersonCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSON_MUTATION",
                Utc::now(),
                json!({
                    "person_id": person_id,
                    "operation": "set_owner_persona",
                }),
                format!("persona://{person_id}/owner"),
                json!({
                    "captured_by": "persons_service.set_owner_persona_manual",
                    "operation": "set_owner_persona_manual",
                }),
            )
            .await?;

        Ok(PersonProjectionStore::new(self.pool.clone())
            .set_owner_persona_with_observation(person_id, &observation.observation_id)
            .await?)
    }

    pub async fn update_persona_manual(
        &self,
        persona_id: &str,
        display_name: Option<&str>,
        set_self: bool,
    ) -> Result<Person, PersonCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSON_MUTATION",
                Utc::now(),
                json!({
                    "persona_id": persona_id,
                    "display_name": display_name,
                    "is_self": set_self,
                }),
                format!("persona://{persona_id}/update"),
                json!({
                    "captured_by": "persons_service.update_persona_manual",
                    "operation": "update_persona_manual",
                }),
            )
            .await?;

        Ok(PersonProjectionStore::new(self.pool.clone())
            .update_persona_with_observation(
                persona_id,
                display_name,
                set_self,
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn review_identity_candidate_manual(
        &self,
        command: &PersonIdentityReviewCommand,
    ) -> Result<PersonIdentityReviewCommandResult, PersonCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "REVIEW_TRANSITION",
                Utc::now(),
                json!({
                    "identity_candidate_id": command.identity_candidate_id,
                    "command_id": command.command_id,
                    "review_state": command.review_state.as_str(),
                    "actor_id": command.actor_id,
                    "operation": "identity_candidate_review",
                }),
                format!(
                    "identity-candidate://{}/review/{}",
                    command.identity_candidate_id, command.command_id
                ),
                json!({
                    "captured_by": "persons_service.review_identity_candidate_manual",
                    "operation": "review_identity_candidate_manual",
                }),
            )
            .await?;

        Ok(PersonIdentityStore::new(self.pool.clone())
            .set_review_state_with_observation(
                command,
                Some(&observation.observation_id),
                Some(json!({
                    "captured_by": "persons_service.review_identity_candidate_manual",
                    "operation": "review_identity_candidate_manual",
                })),
            )
            .await?)
    }

    pub async fn review_dossier_manual(
        &self,
        person_id: &str,
        review_state: DossierReviewState,
    ) -> Result<DossierSnapshot, PersonCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "REVIEW_TRANSITION",
                Utc::now(),
                json!({
                    "person_id": person_id,
                    "review_state": review_state.as_str(),
                    "operation": "dossier_review",
                }),
                format!("persona://{person_id}/dossier/review"),
                json!({
                    "captured_by": "persons_service.review_dossier_manual",
                    "operation": "review_dossier_manual",
                }),
            )
            .await?;

        Ok(PersonInvestigator::new(self.pool.clone())
            .review_dossier_snapshot_with_observation(
                person_id,
                review_state,
                Some(&observation.observation_id),
                Some(json!({
                    "captured_by": "persons_service.review_dossier_manual",
                    "operation": "review_dossier_manual",
                })),
            )
            .await?)
    }

    async fn capture_manual_at(
        &self,
        kind: &str,
        observed_at: DateTime<Utc>,
        payload: Value,
        source_ref: String,
        provenance: Value,
    ) -> Result<crate::platform::observations::Observation, PersonCommandServiceError> {
        Ok(ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    kind,
                    ObservationOriginKind::Manual,
                    observed_at,
                    payload,
                    source_ref,
                )
                .provenance(provenance),
            )
            .await?)
    }
}

#[derive(Debug, Error)]
pub enum PersonCommandServiceError {
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error(transparent)]
    Projection(#[from] PersonProjectionError),
    #[error(transparent)]
    Core(#[from] PersonCoreError),
    #[error(transparent)]
    Enrichment(#[from] PersonEnrichmentError),
    #[error(transparent)]
    EnrichmentEngine(#[from] EnrichmentEngineError),
    #[error(transparent)]
    Memory(#[from] PersonMemoryError),
    #[error(transparent)]
    Health(#[from] PersonHealthError),
    #[error(transparent)]
    Identity(#[from] PersonIdentityError),
    #[error(transparent)]
    Investigator(#[from] InvestigatorError),
}
