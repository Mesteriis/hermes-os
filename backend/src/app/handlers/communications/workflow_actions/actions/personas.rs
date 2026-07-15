use sqlx::{Postgres, Transaction};

use crate::app::error::types::ApiError;
use crate::domains::communications::messages::models::ProjectedMessage;
use crate::workflows::workflow_action_persona_projection::create_persona_projection_in_transaction;

use super::super::models::{
    WorkflowActionRequest, WorkflowActionResponse, WorkflowActionStatus, WorkflowActionTarget,
    WorkflowActionTargetKind,
};
use super::super::response::base_response;

pub(in crate::app::handlers::communications::workflow_actions) async fn create_persona_response(
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
        .or_else(|| message.map(|value| value.sender.as_str()));
    let display_name = request
        .input
        .as_ref()
        .and_then(|value| value.display_name.as_ref())
        .map(|value| value.trim())
        .filter(|value| !value.is_empty());
    if email.is_none() && display_name.is_none() {
        return Err(ApiError::InvalidCommunicationQuery(
            "create_persona requires email, source message, or display name",
        ));
    }
    let persona_id = create_persona_projection_in_transaction(
        transaction,
        command_id,
        event_id,
        email,
        display_name,
        message,
    )
    .await?;
    Ok(base_response(
        command_id,
        event_id,
        request.action.clone(),
        WorkflowActionStatus::Created,
        WorkflowActionTarget {
            kind: WorkflowActionTargetKind::Persona,
            id: Some(persona_id),
        },
        message,
        vec![
            display_name
                .map(|value| format!("persona upserted from communication identity for {value}"))
                .unwrap_or_else(|| "persona upserted from communication identity".to_owned()),
        ],
    ))
}
