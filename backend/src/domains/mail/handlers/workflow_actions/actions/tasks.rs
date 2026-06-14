use sqlx::{Postgres, Transaction};

use crate::app::ApiError;
use crate::domains::mail::messages::ProjectedMessage;
use crate::domains::tasks::api::{NewTask, TaskStore};

use super::super::models::{
    WorkflowActionRequest, WorkflowActionResponse, WorkflowActionStatus, WorkflowActionTarget,
    WorkflowActionTargetKind,
};
use super::super::response::base_response;
use super::super::validation::input_title;

pub(in crate::domains::mail::handlers::workflow_actions) async fn create_task_response(
    transaction: &mut Transaction<'_, Postgres>,
    command_id: &str,
    event_id: &str,
    actor_id: &str,
    request: &WorkflowActionRequest,
    message: Option<&ProjectedMessage>,
) -> Result<WorkflowActionResponse, ApiError> {
    let title = input_title(request, message, "New task")?;
    let input = request.input.as_ref();
    let task = TaskStore::create_in_transaction(
        transaction,
        &NewTask {
            title,
            description: input.and_then(|value| value.body.clone()),
            source_kind: Some(
                if message.is_some() {
                    "communication"
                } else {
                    "manual"
                }
                .to_owned(),
            ),
            source_id: Some(
                message
                    .map(|value| value.message_id.clone())
                    .unwrap_or_else(|| command_id.to_owned()),
            ),
            source_type: Some(message.map(|_| "message").unwrap_or("manual").to_owned()),
            project_id: None,
            hermes_status: Some("new".to_owned()),
            priority_score: None,
            area: None,
            why: None,
            due_at: input.and_then(|value| value.due_at),
            energy_type: None,
            confidentiality: Some("private_local".to_owned()),
            tags: None,
            linked_person_id: None,
            linked_organization_id: None,
            created_from_event_id: Some(event_id.to_owned()),
            created_by_actor_id: Some(actor_id.to_owned()),
        },
    )
    .await?;
    Ok(base_response(
        command_id,
        event_id,
        request.action.clone(),
        WorkflowActionStatus::Created,
        WorkflowActionTarget {
            kind: WorkflowActionTargetKind::Task,
            id: Some(task.task_id),
        },
        message,
        vec!["task created through local workflow action".to_owned()],
    ))
}
