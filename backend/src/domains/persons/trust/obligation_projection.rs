use serde_json::{Value, json};
use sqlx::{Postgres, Transaction};

use crate::domains::obligations::{
    NewObligation, NewObligationEvidence, ObligationEntityKind, ObligationReviewState,
    ObligationStatus, ObligationStore,
};
use crate::platform::observations::{NewObservation, ObservationOriginKind, ObservationStore};

use super::errors::PersonTrustError;
use super::models::PersonPromise;

pub(super) async fn project_promise_obligation(
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

    let evidence_observation = NewObservation::new(
        "PERSON_PROMISE",
        ObservationOriginKind::LocalRuntime,
        chrono::Utc::now(),
        json!({
            "source": "person_promise_projection",
            "person_id": promise.person_id,
            "person_promise_id": promise.id,
            "promise_status": promise.status,
            "source_message_id": promise.source_message_id,
            "due_at": promise.due_at,
        }),
        format!("person-promise://{}/{}", promise.person_id, promise.id),
    )
    .confidence(1.0)
    .provenance(json!({
        "pipeline": "person_trust",
        "person_promise_id": promise.id,
    }));

    let evidence_observation =
        ObservationStore::capture_in_transaction(transaction, &evidence_observation).await?;

    let evidence = NewObligationEvidence::observation(evidence_observation.observation_id)
        .quote(promise.description.clone())
        .confidence(1.0)
        .metadata(person_promise_metadata(promise));

    ObligationStore::upsert_with_evidence_in_transaction(transaction, &obligation, &[evidence])
        .await?;

    Ok(())
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
