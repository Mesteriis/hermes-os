use sqlx::{Postgres, Transaction};

use crate::app::ApiError;
use crate::domains::communications::messages::{
    MessageProjectionStore, ProjectedMessage, WorkflowState,
};

use super::super::models::{
    WorkflowActionRequest, WorkflowActionResponse, WorkflowActionStatus, WorkflowActionTarget,
    WorkflowActionTargetKind,
};
use super::super::response::base_response;
use super::super::validation::require_source_message;

pub(in crate::app::handlers::communications::workflow_actions) async fn archive_response(
    transaction: &mut Transaction<'_, Postgres>,
    command_id: &str,
    event_id: &str,
    request: &WorkflowActionRequest,
    message: Option<&ProjectedMessage>,
) -> Result<WorkflowActionResponse, ApiError> {
    let message = require_source_message(request, message)?;
    let updated = if message.workflow_state == WorkflowState::Archived {
        message.clone()
    } else {
        if !WorkflowState::is_valid_transition(&message.workflow_state, &WorkflowState::Archived) {
            return Err(ApiError::InvalidCommunicationQuery(
                "invalid workflow state transition",
            ));
        }
        MessageProjectionStore::transition_workflow_state_in_transaction(
            transaction,
            &message.message_id,
            WorkflowState::Archived,
        )
        .await?
    };
    Ok(base_response(
        command_id,
        event_id,
        request.action.clone(),
        if updated.workflow_state == WorkflowState::Archived {
            WorkflowActionStatus::Archived
        } else {
            WorkflowActionStatus::Noop
        },
        WorkflowActionTarget {
            kind: WorkflowActionTargetKind::Message,
            id: Some(updated.message_id),
        },
        Some(message),
        vec!["message workflow state transitioned locally".to_owned()],
    ))
}
