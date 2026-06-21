use sqlx::{Postgres, Transaction};

use crate::domains::communications::messages::ProjectedMessage;
use crate::domains::persons::api::{PersonProjectionError, PersonProjectionPort};
use crate::platform::observations::{NewObservation, ObservationOriginKind, ObservationPort};

pub(crate) async fn create_person_projection_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    command_id: &str,
    event_id: &str,
    email: &str,
    display_name: Option<&str>,
    message: Option<&ProjectedMessage>,
) -> Result<String, PersonProjectionError> {
    let (person, identity_id) =
        PersonProjectionPort::upsert_email_person_in_transaction(transaction, email).await?;
    let projection_observation_id = if let Some(message) = message {
        message.observation_id.clone()
    } else {
        ObservationPort::capture_in_transaction(
            transaction,
            &NewObservation::new(
                "PERSON_MUTATION",
                ObservationOriginKind::Manual,
                chrono::Utc::now(),
                serde_json::json!({
                    "command_id": command_id,
                    "event_id": event_id,
                    "email": email,
                    "display_name": display_name,
                    "operation": "workflow_action_create_person",
                }),
                format!("workflow-action://create-person/{command_id}"),
            )
            .provenance(serde_json::json!({
                "captured_by": "workflows.create_person_projection_in_transaction",
                "workflow_action": "create_person",
            })),
        )
        .await?
        .observation_id
    };
    PersonProjectionPort::link_email_person_projection_in_transaction(
        transaction,
        &projection_observation_id,
        &person,
        &identity_id,
        email,
        "workflow_action_projection",
    )
    .await?;
    Ok(person.person_id)
}
