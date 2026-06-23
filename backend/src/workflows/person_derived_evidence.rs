use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::obligations::{
    NewObligation, NewObligationEvidence, ObligationEntityKind, ObligationReviewPort,
    ObligationReviewState, ObligationStoreError,
};
use crate::domains::persons::core::{
    PERSON_ROLE_ASSIGNED_EVENT_TYPE, PERSON_ROLE_REMOVED_EVENT_TYPE, person_role_knowledge_id,
};
use crate::domains::persons::enrichment::PERSON_TRUST_SCORE_CHANGED_EVENT_TYPE;
use crate::domains::persons::trust::PERSON_PROMISE_CREATED_EVENT_TYPE;
use crate::domains::relationships::{
    NewRelationship, NewRelationshipEvidence, RelationshipEntityKind, RelationshipReviewPort,
    RelationshipReviewState, RelationshipStoreError, relationship_id,
};
use crate::engines::trust::{TrustEngine, TrustEngineError};
use crate::platform::events::{EventStoreError, StoredEventEnvelope};
use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationPort, ObservationStoreError,
};
use crate::workflows::review_mirror::{ReviewMirrorError, ensure_relationship_review_item};

pub const PERSON_DERIVED_EVIDENCE_CONSUMER: &str = "person_derived_evidence";

#[derive(Debug, Error)]
pub enum PersonDerivedEvidenceWorkflowError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    Relationship(#[from] RelationshipStoreError),

    #[error(transparent)]
    Obligation(#[from] ObligationStoreError),

    #[error(transparent)]
    ReviewMirror(#[from] ReviewMirrorError),

    #[error(transparent)]
    Trust(#[from] TrustEngineError),

    #[error("event payload is missing required field {0}")]
    MissingPayloadField(&'static str),

    #[error("event payload field {field} is invalid: {value}")]
    InvalidPayloadField { field: &'static str, value: String },
}

pub async fn project_person_derived_evidence_event(
    pool: PgPool,
    event: StoredEventEnvelope,
) -> Result<(), EventStoreError> {
    project_person_derived_evidence_event_inner(&pool, &event)
        .await
        .map_err(|error| EventStoreError::ConsumerHandlerFailed(error.to_string()))
}

async fn project_person_derived_evidence_event_inner(
    pool: &PgPool,
    event: &StoredEventEnvelope,
) -> Result<(), PersonDerivedEvidenceWorkflowError> {
    match event.event.event_type.as_str() {
        PERSON_ROLE_ASSIGNED_EVENT_TYPE => materialize_role_assigned(pool, event).await,
        PERSON_ROLE_REMOVED_EVENT_TYPE => materialize_role_removed(pool, event).await,
        PERSON_TRUST_SCORE_CHANGED_EVENT_TYPE => materialize_trust_score(pool, event).await,
        PERSON_PROMISE_CREATED_EVENT_TYPE => materialize_promise(pool, event).await,
        _ => Ok(()),
    }
}

async fn materialize_role_assigned(
    pool: &PgPool,
    event: &StoredEventEnvelope,
) -> Result<(), PersonDerivedEvidenceWorkflowError> {
    let person_id = required_string(&event.event.payload, "person_id")?;
    let role = required_string(&event.event.payload, "role")?;
    let assigned_by = optional_string(&event.event.payload, "assigned_by");
    let role_knowledge_id = optional_string(&event.event.payload, "role_knowledge_id")
        .map(str::to_owned)
        .unwrap_or_else(|| person_role_knowledge_id(role));

    let observation = ObservationPort::new(pool.clone())
        .capture(
            &NewObservation::new(
                "PERSON_ROLE",
                ObservationOriginKind::LocalRuntime,
                event.event.occurred_at,
                json!({
                    "person_id": person_id,
                    "role": role,
                    "assigned_by": assigned_by,
                    "action": "assign",
                }),
                format!("person://{person_id}/roles/{role_knowledge_id}"),
            )
            .provenance(json!({
                "captured_by": "person_derived_evidence.role_assigned",
                "event_id": event.event.event_id,
            })),
        )
        .await?;

    let relationship = NewRelationship {
        source_entity_kind: RelationshipEntityKind::Persona,
        source_entity_id: person_id.to_owned(),
        target_entity_kind: RelationshipEntityKind::Knowledge,
        target_entity_id: role_knowledge_id,
        relationship_type: "has_role".to_owned(),
        trust_score: 1.0,
        strength_score: 0.7,
        confidence: 1.0,
        review_state: RelationshipReviewState::UserConfirmed,
        valid_from: None,
        valid_to: None,
        metadata: json!({
            "compatibility_source": "person_roles",
            "role": role,
            "assigned_by": assigned_by,
        }),
    };
    let evidence = NewRelationshipEvidence::observation(observation.observation_id)
        .excerpt(role)
        .metadata(json!({
            "compatibility_source": "person_roles",
        }));

    let _ = RelationshipReviewPort::new(pool.clone())
        .upsert_with_evidence(&relationship, &[evidence])
        .await?;
    Ok(())
}

async fn materialize_role_removed(
    pool: &PgPool,
    event: &StoredEventEnvelope,
) -> Result<(), PersonDerivedEvidenceWorkflowError> {
    let person_id = required_string(&event.event.payload, "person_id")?;
    let role = required_string(&event.event.payload, "role")?;
    let role_knowledge_id = optional_string(&event.event.payload, "role_knowledge_id")
        .map(str::to_owned)
        .unwrap_or_else(|| person_role_knowledge_id(role));
    let relationship_id = relationship_id(
        RelationshipEntityKind::Persona,
        person_id,
        "has_role",
        RelationshipEntityKind::Knowledge,
        &role_knowledge_id,
    );

    let _ = RelationshipReviewPort::new(pool.clone())
        .set_review_state(&relationship_id, RelationshipReviewState::UserRejected)
        .await?;
    Ok(())
}

async fn materialize_trust_score(
    pool: &PgPool,
    event: &StoredEventEnvelope,
) -> Result<(), PersonDerivedEvidenceWorkflowError> {
    let person_id = required_string(&event.event.payload, "person_id")?;
    let trust_score = required_i16(&event.event.payload, "trust_score")?;
    let normalized_confidence = f64::from(trust_score.clamp(0, 100)) / 100.0;
    let source_observation_id = optional_string(&event.event.payload, "source_observation_id");
    let evidence_text = format!("trust_score={trust_score}");
    let source_reliability = TrustEngine::source_reliability_signal(
        &format!("person_enrichment:{person_id}:trust_score"),
        &evidence_text,
        normalized_confidence,
    )?;

    let observation = ObservationPort::new(pool.clone())
        .capture(
            &NewObservation::new(
                "PERSON_TRUST_SIGNAL",
                ObservationOriginKind::LocalRuntime,
                event.event.occurred_at,
                json!({
                    "person_id": person_id,
                    "trust_score": trust_score,
                    "source_observation_id": source_observation_id,
                    "action": "trust_score_enrichment",
                }),
                format!("person://{person_id}/trust-score"),
            )
            .confidence(normalized_confidence)
            .provenance(json!({
                "captured_by": "person_derived_evidence.trust_score",
                "event_id": event.event.event_id,
            })),
        )
        .await?;

    let Some(owner_person_id) = owner_persona_id(pool, person_id).await? else {
        return Ok(());
    };

    let relationship_signal = TrustEngine::persona_compatibility_score_signal(trust_score);
    let relationship = NewRelationship::between_personas(
        owner_person_id.clone(),
        person_id.to_owned(),
        relationship_signal.relationship_type,
        relationship_signal.trust_score,
        relationship_signal.strength_score,
        relationship_signal.confidence,
        RelationshipReviewState::Suggested,
    )
    .metadata(json!({
        "compatibility_source": "persons.trust_score",
        "trust_score": trust_score,
    }));
    let evidence = NewRelationshipEvidence::observation(observation.observation_id.clone())
        .excerpt(evidence_text)
        .metadata(json!({
            "compatibility_source": "persons.trust_score",
            "source_observation_id": source_observation_id,
            "trust_source_reliability": {
                "signal_type": source_reliability.kind.as_str(),
                "affected_source": source_reliability.affected_source,
                "direction": source_reliability.direction.as_str(),
                "confidence": source_reliability.confidence,
            }
        }));
    let relationship = RelationshipReviewPort::new(pool.clone())
        .upsert_with_evidence(&relationship, &[evidence])
        .await?;
    let _ = ensure_relationship_review_item(
        pool,
        &relationship.relationship_id,
        &relationship.relationship_type,
        relationship.source_entity_kind.as_str(),
        &relationship.source_entity_id,
        relationship.target_entity_kind.as_str(),
        &relationship.target_entity_id,
        relationship.confidence,
        Some("trust_score enrichment suggests a persona trust relationship"),
        &observation.observation_id,
    )
    .await?;

    Ok(())
}

async fn materialize_promise(
    pool: &PgPool,
    event: &StoredEventEnvelope,
) -> Result<(), PersonDerivedEvidenceWorkflowError> {
    let promise_id = required_string(&event.event.payload, "promise_id")?;
    let person_id = required_string(&event.event.payload, "person_id")?;
    let description = required_string(&event.event.payload, "description")?;
    let due_at: Option<DateTime<Utc>> = serde_json::from_value(
        event
            .event
            .payload
            .get("due_at")
            .cloned()
            .unwrap_or(Value::Null),
    )?;

    let observation = ObservationPort::new(pool.clone())
        .capture(
            &NewObservation::new(
                "PERSON_PROMISE",
                ObservationOriginKind::LocalRuntime,
                event.event.occurred_at,
                json!({
                    "promise_id": promise_id,
                    "person_id": person_id,
                    "description": description,
                    "due_at": &due_at,
                    "action": "create",
                }),
                format!("person://{person_id}/promises/{promise_id}"),
            )
            .provenance(json!({
                "captured_by": "person_derived_evidence.promise_created",
                "event_id": event.event.event_id,
            })),
        )
        .await?;

    let mut obligation = NewObligation::new(
        ObligationEntityKind::Persona,
        person_id.to_owned(),
        description.to_owned(),
        1.0,
        ObligationReviewState::UserConfirmed,
    )
    .metadata(json!({
        "compatibility_source": "person_promises",
        "person_promise_id": promise_id,
    }));
    if let Some(due_at) = due_at {
        obligation = obligation.due_at(due_at);
    }
    let evidence = NewObligationEvidence::observation(observation.observation_id)
        .quote(description)
        .metadata(json!({
            "compatibility_source": "person_promises",
            "person_promise_id": promise_id,
        }));
    let _ = ObligationReviewPort::new(pool.clone())
        .upsert_with_evidence(&obligation, &[evidence])
        .await?;

    Ok(())
}

async fn owner_persona_id(
    pool: &PgPool,
    target_person_id: &str,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT person_id
        FROM persons
        WHERE is_self = true
          AND person_id <> $1
        LIMIT 1
        "#,
    )
    .bind(target_person_id)
    .fetch_optional(pool)
    .await
}

fn required_string<'a>(
    payload: &'a Value,
    field: &'static str,
) -> Result<&'a str, PersonDerivedEvidenceWorkflowError> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or(PersonDerivedEvidenceWorkflowError::MissingPayloadField(
            field,
        ))
}

fn optional_string<'a>(payload: &'a Value, field: &'static str) -> Option<&'a str> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
}

fn required_i16(
    payload: &Value,
    field: &'static str,
) -> Result<i16, PersonDerivedEvidenceWorkflowError> {
    let value = payload.get(field).and_then(Value::as_i64).ok_or(
        PersonDerivedEvidenceWorkflowError::MissingPayloadField(field),
    )?;
    i16::try_from(value).map_err(
        |_| PersonDerivedEvidenceWorkflowError::InvalidPayloadField {
            field,
            value: value.to_string(),
        },
    )
}
