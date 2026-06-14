use sqlx::{Postgres, Transaction};

use crate::app::ApiError;
use crate::domains::mail::messages::ProjectedMessage;
use crate::domains::persons::api::PersonProjectionStore;

use super::super::models::{
    WorkflowActionRequest, WorkflowActionResponse, WorkflowActionStatus, WorkflowActionTarget,
    WorkflowActionTargetKind,
};
use super::super::response::base_response;

pub(in crate::domains::mail::handlers::workflow_actions) async fn create_contact_response(
    transaction: &mut Transaction<'_, Postgres>,
    command_id: &str,
    event_id: &str,
    request: &WorkflowActionRequest,
    message: Option<&ProjectedMessage>,
) -> Result<WorkflowActionResponse, ApiError> {
    let email = request
        .input
        .as_ref()
        .and_then(|value| value.email.as_ref())
        .map(String::as_str)
        .or_else(|| message.map(|value| value.sender.as_str()))
        .ok_or(ApiError::InvalidCommunicationQuery(
            "create_contact requires email or source message",
        ))?;
    let display_name = request
        .input
        .as_ref()
        .and_then(|value| value.display_name.as_ref())
        .map(|value| value.trim())
        .filter(|value| !value.is_empty());
    let person = PersonProjectionStore::upsert_email_person_in_transaction(transaction, email)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "workflow contact upsert failed");
            ApiError::InvalidCommunicationQuery("workflow contact upsert failed")
        })?;
    Ok(base_response(
        command_id,
        event_id,
        request.action.clone(),
        WorkflowActionStatus::Created,
        WorkflowActionTarget {
            kind: WorkflowActionTargetKind::Person,
            id: Some(person.person_id),
        },
        message,
        vec![
            display_name
                .map(|value| format!("person upserted from communication identity for {value}"))
                .unwrap_or_else(|| "person upserted from communication identity".to_owned()),
        ],
    ))
}
