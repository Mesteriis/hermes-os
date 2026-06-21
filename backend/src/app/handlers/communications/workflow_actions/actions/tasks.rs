use sqlx::{Postgres, Transaction};

use crate::app::ApiError;
use crate::application::task_creation::{WorkflowTaskCreateInput, create_task_from_workflow_input};
use crate::domains::communications::messages::ProjectedMessage;

use super::super::models::{
    WorkflowActionRequest, WorkflowActionResponse, WorkflowActionStatus, WorkflowActionTarget,
    WorkflowActionTargetKind,
};
use super::super::response::base_response;
use super::super::validation::input_title;
use sqlx::PgPool;

pub(in crate::app::handlers::communications::workflow_actions) async fn create_task_response(
    pool: &PgPool,
    transaction: &mut Transaction<'_, Postgres>,
    command_id: &str,
    event_id: &str,
    actor_id: &str,
    request: &WorkflowActionRequest,
    message: Option<&ProjectedMessage>,
) -> Result<WorkflowActionResponse, ApiError> {
    let title = input_title(request, message, "New task")?;
    let input = request.input.as_ref();
    let task = create_task_from_workflow_input(
        pool,
        transaction,
        WorkflowTaskCreateInput {
            title,
            description: input.and_then(|value| value.body.clone()),
            provenance_kind: message.map(|_| "observation".to_owned()),
            provenance_id: message.map(|value| value.observation_id.clone()),
            source_kind: if message.is_some() {
                "observation".to_owned()
            } else {
                "manual".to_owned()
            },
            source_id: message
                .map(|value| value.observation_id.clone())
                .unwrap_or_else(|| command_id.to_owned()),
            source_type: message
                .map(|_| "observation")
                .unwrap_or("manual")
                .to_owned(),
            due_at: input.and_then(|value| value.due_at),
            created_from_event_id: event_id.to_owned(),
            created_by_actor_id: actor_id.to_owned(),
            projection_observation_id: message.map(|value| value.observation_id.clone()),
            projection_metadata: message.map(|value| {
                serde_json::json!({
                    "workflow_action": "create_task",
                    "message_id": value.message_id,
                    "created_from_event_id": event_id,
                })
            }),
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
