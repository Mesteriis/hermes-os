use serde_json::json;
use sqlx::{Postgres, Row, Transaction};

use crate::domains::persons::intelligence::CommunicationFingerprint;
use crate::domains::relationships::{
    NewRelationship, NewRelationshipEvidence, RelationshipEntityKind, RelationshipReviewState,
    RelationshipStore,
};
use crate::engines::trust::TrustEngine;
use crate::platform::observations::{NewObservation, ObservationOriginKind, ObservationStore};
use crate::workflows::review_mirror::sync_relationship_review_state_in_transaction;

use super::errors::PersonEnrichmentError;
use super::models::EnrichedPerson;

pub(in crate::domains::persons::enrichment) async fn materialize_trust_relationship_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    enriched: &EnrichedPerson,
    fingerprint: &CommunicationFingerprint,
) -> Result<(), PersonEnrichmentError> {
    let Some(trust_score) = fingerprint.trust_score else {
        return Ok(());
    };
    let Some(owner_persona_id) = owner_persona_id_in_transaction(transaction).await? else {
        return Ok(());
    };
    if owner_persona_id == enriched.person_id {
        return Ok(());
    }

    let trust_signal = TrustEngine::persona_compatibility_score_signal(trust_score);
    let relationship = NewRelationship {
        source_entity_kind: RelationshipEntityKind::Persona,
        source_entity_id: owner_persona_id.clone(),
        target_entity_kind: RelationshipEntityKind::Persona,
        target_entity_id: enriched.person_id.clone(),
        relationship_type: trust_signal.relationship_type.to_owned(),
        trust_score: trust_signal.trust_score,
        strength_score: trust_signal.strength_score,
        confidence: trust_signal.confidence,
        review_state: RelationshipReviewState::Suggested,
        valid_from: None,
        valid_to: None,
        metadata: json!({
            "compatibility_source": "persons.trust_score",
            "source": "person_enrichment",
            "owner_persona_id": owner_persona_id,
            "person_id": enriched.person_id,
            "trust_score": trust_score,
        }),
    };
    let evidence_source_id = format!("person_enrichment:{}:trust_score", enriched.person_id);
    let evidence_excerpt = format!("trust_score={trust_score}");
    let source_reliability = TrustEngine::source_reliability_signal(
        &evidence_source_id,
        &evidence_excerpt,
        trust_signal.trust_score,
    )?;
    let observation = NewObservation::new(
        "PERSON_TRUST_SIGNAL",
        ObservationOriginKind::LocalRuntime,
        chrono::Utc::now(),
        json!({
            "component": "persons_enrichment_relationship",
            "evidence_source_id": evidence_source_id,
            "owner_persona_id": owner_persona_id,
            "person_id": enriched.person_id,
            "trust_score": trust_signal.trust_score,
            "strength_score": trust_signal.strength_score,
            "confidence": trust_signal.confidence,
            "evidence_excerpt": evidence_excerpt,
        }),
        format!(
            "person-enrichment://{}/relationship/trust",
            enriched.person_id
        ),
    )
    .confidence(1.0)
    .provenance(json!({
        "pipeline": "person_enrichment",
        "source_persona_id": owner_persona_id,
        "target_persona_id": enriched.person_id,
    }));
    let observation = ObservationStore::capture_in_transaction(transaction, &observation).await?;
    let observation_id = observation.observation_id.clone();
    let evidence = NewRelationshipEvidence::observation(observation_id.clone())
        .excerpt(&evidence_excerpt)
        .metadata(json!({
            "compatibility_source": "persons.trust_score",
            "source": "person_enrichment",
            "person_id": enriched.person_id,
            "trust_score": trust_score,
            "detected_language": fingerprint.detected_language,
            "typical_tone": fingerprint.typical_tone,
            "writing_style": fingerprint.writing_style,
            "trust_source_reliability": {
                "signal_type": source_reliability.kind.as_str(),
                "affected_source": source_reliability.affected_source,
                "evidence": source_reliability.evidence,
                "confidence": source_reliability.confidence,
                "direction": source_reliability.direction.as_str(),
                "explanation": source_reliability.explanation,
            },
        }));

    let stored = RelationshipStore::upsert_with_evidence_in_transaction(
        transaction,
        &relationship,
        &[evidence],
    )
    .await?;
    sync_relationship_review_state_in_transaction(transaction, &stored).await?;

    Ok(())
}

async fn owner_persona_id_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<Option<String>, PersonEnrichmentError> {
    let row = sqlx::query("SELECT person_id FROM persons WHERE is_self = true")
        .fetch_optional(&mut **transaction)
        .await?;
    row.map(|row| row.try_get("person_id"))
        .transpose()
        .map_err(Into::into)
}
