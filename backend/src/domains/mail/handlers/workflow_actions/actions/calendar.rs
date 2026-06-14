use sqlx::{Postgres, Transaction};

use crate::app::ApiError;
use crate::domains::calendar::events::{CalendarEventStore, NewCalendarEvent};
use crate::domains::mail::messages::ProjectedMessage;

use super::super::models::{
    WorkflowActionRequest, WorkflowActionResponse, WorkflowActionStatus, WorkflowActionTarget,
    WorkflowActionTargetKind,
};
use super::super::response::base_response;
use super::super::validation::input_title;

pub(in crate::domains::mail::handlers::workflow_actions) async fn create_event_response(
    transaction: &mut Transaction<'_, Postgres>,
    command_id: &str,
    event_id: &str,
    request: &WorkflowActionRequest,
    message: Option<&ProjectedMessage>,
) -> Result<WorkflowActionResponse, ApiError> {
    let input = request
        .input
        .as_ref()
        .ok_or(ApiError::InvalidCommunicationQuery(
            "create_event requires input",
        ))?;
    let start_at = input.starts_at.ok_or(ApiError::InvalidCommunicationQuery(
        "create_event requires starts_at",
    ))?;
    let end_at = input.ends_at.ok_or(ApiError::InvalidCommunicationQuery(
        "create_event requires ends_at",
    ))?;
    if end_at <= start_at {
        return Err(ApiError::InvalidCommunicationQuery(
            "create_event requires ends_at after starts_at",
        ));
    }
    let title = input_title(request, message, "New event")?;
    let event = CalendarEventStore::create_in_transaction(
        transaction,
        &NewCalendarEvent {
            source_event_id: Some(event_id.to_owned()),
            account_id: None,
            source_id: message.map(|value| value.message_id.clone()),
            title,
            description: input.body.clone(),
            location: None,
            start_at,
            end_at,
            timezone: None,
            all_day: Some(false),
            recurrence_rule: None,
            status: Some("scheduled".to_owned()),
            visibility: Some("private".to_owned()),
            event_type: Some("meeting".to_owned()),
            conference_url: None,
            conference_provider: None,
            preparation_reminder_minutes: None,
            travel_buffer_minutes: None,
        },
    )
    .await?;
    Ok(base_response(
        command_id,
        event_id,
        request.action.clone(),
        WorkflowActionStatus::Created,
        WorkflowActionTarget {
            kind: WorkflowActionTargetKind::CalendarEvent,
            id: Some(event.event_id),
        },
        message,
        vec!["local calendar event created through workflow action".to_owned()],
    ))
}
