use crate::app::error::types::ApiError;
use crate::domains::communications::messages::models::ProjectedMessage;

use super::super::models::{
    WorkflowActionRequest, WorkflowActionResponse, WorkflowActionStatus, WorkflowActionTarget,
    WorkflowActionTargetKind,
};
use super::super::response::base_response;
use super::super::validation::require_source_message;

pub(in crate::app::handlers::communications::workflow_actions) fn reply_response(
    command_id: &str,
    event_id: &str,
    request: &WorkflowActionRequest,
    message: Option<&ProjectedMessage>,
) -> Result<WorkflowActionResponse, ApiError> {
    let message = require_source_message(request, message)?;
    Ok(base_response(
        command_id,
        event_id,
        request.action.clone(),
        WorkflowActionStatus::Opened,
        WorkflowActionTarget {
            kind: WorkflowActionTargetKind::Compose,
            id: Some(message.message_id.clone()),
        },
        Some(message),
        vec!["reply compose opened from selected communication message".to_owned()],
    ))
}
