use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Postgres, Transaction};

use crate::app::{ApiError, AppState};
use crate::domains::calendar::events::{CalendarEventStore, NewCalendarEvent};
use crate::domains::documents::core::{DocumentImportStore, NewDocumentImport};
use crate::domains::mail::messages::{MessageProjectionStore, ProjectedMessage, WorkflowState};
use crate::domains::persons::api::PersonProjectionStore;
use crate::domains::tasks::api::{NewTask, TaskStore};
use crate::platform::events::{EventEnvelope, EventStore, NewEventEnvelope};

const WORKFLOW_EVENT_TYPE: &str = "workflow.action_executed";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorkflowActionKind {
    Reply,
    CreateTask,
    CreateNote,
    CreateDocument,
    CreateEvent,
    LinkDocument,
    CreateContact,
    Archive,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WorkflowActionSource {
    kind: String,
    id: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct WorkflowActionInput {
    title: Option<String>,
    body: Option<String>,
    email: Option<String>,
    display_name: Option<String>,
    starts_at: Option<DateTime<Utc>>,
    ends_at: Option<DateTime<Utc>>,
    due_at: Option<DateTime<Utc>>,
    document_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WorkflowActionRequest {
    command_id: String,
    action: WorkflowActionKind,
    source: Option<WorkflowActionSource>,
    input: Option<WorkflowActionInput>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorkflowActionStatus {
    Created,
    Updated,
    Linked,
    Opened,
    Archived,
    Noop,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorkflowActionTargetKind {
    Compose,
    Message,
    Task,
    Document,
    CalendarEvent,
    Person,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct WorkflowActionTarget {
    kind: WorkflowActionTargetKind,
    id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct WorkflowActionProvenance {
    #[serde(skip_serializing_if = "Option::is_none")]
    source_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_id: Option<String>,
    confidence: Option<f64>,
    evidence: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct WorkflowActionResponse {
    command_id: String,
    event_id: String,
    action: WorkflowActionKind,
    status: WorkflowActionStatus,
    target: WorkflowActionTarget,
    provenance: WorkflowActionProvenance,
}

pub(crate) async fn post_v1_workflow_action(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<WorkflowActionRequest>,
) -> Result<Json<WorkflowActionResponse>, ApiError> {
    let Some(pool) = state.database.pool().cloned() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    let command_id = normalize_non_empty("command_id", &request.command_id)?;
    let event_id = format!("workflow_action:{command_id}");
    let event_store = EventStore::new(pool.clone());
    if let Some(existing) = event_store.get_by_id(&event_id).await? {
        return Ok(Json(response_from_event(existing)?));
    }

    let actor_id = actor_id_from_headers(&headers);
    let message_store = MessageProjectionStore::new(pool.clone());
    let source_message = load_source_message(&message_store, request.source.as_ref()).await?;
    let mut transaction = pool
        .begin()
        .await
        .map_err(|error| ApiError::Store(error.into()))?;
    let response = match request.action {
        WorkflowActionKind::Reply => {
            reply_response(&command_id, &event_id, &request, source_message.as_ref())?
        }
        WorkflowActionKind::CreateTask => {
            create_task_response(
                &mut transaction,
                &command_id,
                &event_id,
                &actor_id,
                &request,
                source_message.as_ref(),
            )
            .await?
        }
        WorkflowActionKind::CreateNote => {
            create_document_response(
                &mut transaction,
                &command_id,
                &event_id,
                &request,
                source_message.as_ref(),
                true,
            )
            .await?
        }
        WorkflowActionKind::CreateDocument => {
            create_document_response(
                &mut transaction,
                &command_id,
                &event_id,
                &request,
                source_message.as_ref(),
                false,
            )
            .await?
        }
        WorkflowActionKind::CreateEvent => {
            create_event_response(
                &mut transaction,
                &command_id,
                &event_id,
                &request,
                source_message.as_ref(),
            )
            .await?
        }
        WorkflowActionKind::LinkDocument => {
            link_document_response(
                &mut transaction,
                &command_id,
                &event_id,
                &request,
                source_message.as_ref(),
            )
            .await?
        }
        WorkflowActionKind::CreateContact => {
            create_contact_response(
                &mut transaction,
                &command_id,
                &event_id,
                &request,
                source_message.as_ref(),
            )
            .await?
        }
        WorkflowActionKind::Archive => {
            archive_response(
                &mut transaction,
                &command_id,
                &event_id,
                &request,
                source_message.as_ref(),
            )
            .await?
        }
    };

    let event = NewEventEnvelope::builder(
        event_id.clone(),
        WORKFLOW_EVENT_TYPE,
        Utc::now(),
        json!({
            "kind": "workflow_action",
            "source_id": command_id,
        }),
        json!({
            "kind": "workflow_action",
            "id": command_id,
        }),
    )
    .actor(json!({ "actor_id": actor_id }))
    .payload(serde_json::to_value(&response).map_err(|_| {
        ApiError::InvalidCommunicationQuery("invalid workflow action response payload")
    })?)
    .provenance(json!({
        "source_kind": response.provenance.source_kind.as_deref(),
        "source_id": response.provenance.source_id.as_deref(),
        "confidence": response.provenance.confidence,
        "evidence": response.provenance.evidence.clone(),
    }))
    .correlation_id(command_id.clone())
    .build()
    .map_err(ApiError::InvalidEnvelope)?;

    match EventStore::append_in_transaction(&mut transaction, &event).await {
        Ok(_) => {
            transaction
                .commit()
                .await
                .map_err(|error| ApiError::Store(error.into()))?;
            Ok(Json(response))
        }
        Err(error) if error.is_unique_violation() => {
            let _ = transaction.rollback().await;
            let Some(existing) = event_store.get_by_id(&event_id).await? else {
                return Err(ApiError::Store(error));
            };
            Ok(Json(response_from_event(existing)?))
        }
        Err(error) => Err(ApiError::Store(error)),
    }
}

async fn load_source_message(
    store: &MessageProjectionStore,
    source: Option<&WorkflowActionSource>,
) -> Result<Option<ProjectedMessage>, ApiError> {
    let Some(source) = source else {
        return Ok(None);
    };
    if source.kind != "communication_message" {
        return Err(ApiError::InvalidCommunicationQuery(
            "workflow action source kind must be communication_message",
        ));
    }
    let source_id = normalize_non_empty("source.id", &source.id)?;
    Ok(Some(
        store
            .message(&source_id)
            .await?
            .ok_or(ApiError::CommunicationMessageNotFound)?,
    ))
}

fn reply_response(
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

async fn create_task_response(
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

async fn create_document_response(
    transaction: &mut Transaction<'_, Postgres>,
    command_id: &str,
    event_id: &str,
    request: &WorkflowActionRequest,
    message: Option<&ProjectedMessage>,
    note_mode: bool,
) -> Result<WorkflowActionResponse, ApiError> {
    let title = input_title(
        request,
        message,
        if note_mode {
            "New note"
        } else {
            "New document"
        },
    )?;
    let input = request.input.as_ref();
    let document_id = input
        .and_then(|value| value.document_id.as_ref())
        .map(|value| normalize_non_empty("document_id", value))
        .transpose()?
        .unwrap_or_else(|| {
            let prefix = if note_mode {
                "document:workflow-note"
            } else {
                "document:workflow"
            };
            format!("{prefix}:{command_id}")
        });
    let body = input
        .and_then(|value| value.body.as_ref())
        .map(String::as_str)
        .or_else(|| message.map(|value| value.body_text.as_str()))
        .unwrap_or(&title);
    let markdown = format!("# {title}\n\n{body}");
    let document = DocumentImportStore::import_document_in_transaction(
        transaction,
        &NewDocumentImport::markdown(document_id, title, markdown),
    )
    .await
    .map_err(|error| {
        tracing::error!(error = %error, "workflow document import failed");
        ApiError::InvalidCommunicationQuery("workflow document import failed")
    })?;
    Ok(base_response(
        command_id,
        event_id,
        request.action.clone(),
        WorkflowActionStatus::Created,
        WorkflowActionTarget {
            kind: WorkflowActionTargetKind::Document,
            id: Some(document.document_id),
        },
        message,
        vec!["markdown document imported through documents boundary".to_owned()],
    ))
}

async fn create_event_response(
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

async fn link_document_response(
    transaction: &mut Transaction<'_, Postgres>,
    command_id: &str,
    event_id: &str,
    request: &WorkflowActionRequest,
    message: Option<&ProjectedMessage>,
) -> Result<WorkflowActionResponse, ApiError> {
    let message = require_source_message(request, message)?;
    let title = input_title(request, Some(message), "Linked communication document")?;
    let document_id = request
        .input
        .as_ref()
        .and_then(|value| value.document_id.as_ref())
        .map(|value| normalize_non_empty("document_id", value))
        .transpose()?
        .unwrap_or_else(|| format!("document:mail-message:{}", message.message_id));
    let markdown = format!("# {title}\n\n{}", message.body_text);
    let document = DocumentImportStore::import_document_in_transaction(
        transaction,
        &NewDocumentImport::markdown(document_id, title, markdown),
    )
    .await
    .map_err(|error| {
        tracing::error!(error = %error, "workflow link document import failed");
        ApiError::InvalidCommunicationQuery("workflow document import failed")
    })?;
    Ok(base_response(
        command_id,
        event_id,
        request.action.clone(),
        WorkflowActionStatus::Linked,
        WorkflowActionTarget {
            kind: WorkflowActionTargetKind::Document,
            id: Some(document.document_id),
        },
        Some(message),
        vec!["message-to-document relation recorded in workflow event payload".to_owned()],
    ))
}

async fn create_contact_response(
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

async fn archive_response(
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

fn require_source_message<'a>(
    request: &WorkflowActionRequest,
    message: Option<&'a ProjectedMessage>,
) -> Result<&'a ProjectedMessage, ApiError> {
    if request.source.is_none() {
        return Err(ApiError::InvalidCommunicationQuery(
            "workflow action requires source message",
        ));
    }
    message.ok_or(ApiError::CommunicationMessageNotFound)
}

fn input_title(
    request: &WorkflowActionRequest,
    message: Option<&ProjectedMessage>,
    fallback: &str,
) -> Result<String, ApiError> {
    if let Some(title) = request
        .input
        .as_ref()
        .and_then(|value| value.title.as_ref())
    {
        return normalize_non_empty("title", title);
    }
    if let Some(message) = message {
        return normalize_non_empty("title", &message.subject);
    }
    Ok(fallback.to_owned())
}

fn base_response(
    command_id: &str,
    event_id: &str,
    action: WorkflowActionKind,
    status: WorkflowActionStatus,
    target: WorkflowActionTarget,
    message: Option<&ProjectedMessage>,
    evidence: Vec<String>,
) -> WorkflowActionResponse {
    WorkflowActionResponse {
        command_id: command_id.to_owned(),
        event_id: event_id.to_owned(),
        action,
        status,
        target,
        provenance: WorkflowActionProvenance {
            source_kind: message.map(|_| "communication_message".to_owned()),
            source_id: message.map(|value| value.message_id.clone()),
            confidence: None,
            evidence,
        },
    }
}

fn response_from_event(event: EventEnvelope) -> Result<WorkflowActionResponse, ApiError> {
    let event_id = event.event_id.clone();
    serde_json::from_value::<WorkflowActionResponse>(event.payload).map_err(|error| {
        tracing::error!(error = %error, event_id = %event_id, "stored workflow action payload is invalid");
        ApiError::InvalidCommunicationQuery("stored workflow action payload is invalid")
    })
}

fn normalize_non_empty(field: &'static str, value: &str) -> Result<String, ApiError> {
    let normalized = value.trim().to_owned();
    if normalized.is_empty() {
        return Err(ApiError::InvalidCommunicationQuery(match field {
            "command_id" => "command_id must not be empty",
            "source.id" => "source id must not be empty",
            "document_id" => "document_id must not be empty",
            "title" => "title must not be empty",
            _ => "workflow action field must not be empty",
        }));
    }
    Ok(normalized)
}

fn actor_id_from_headers(headers: &HeaderMap) -> String {
    headers
        .get("x-hermes-actor-id")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("hermes-frontend")
        .to_owned()
}
