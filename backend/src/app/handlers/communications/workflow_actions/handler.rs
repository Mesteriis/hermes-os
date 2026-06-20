use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use chrono::Utc;
use serde_json::json;

use crate::app::{ApiError, AppState};
use crate::domains::communications::messages::MessageProjectionStore;
use crate::platform::events::{EventStore, NewEventEnvelope};

use super::actions::{
    archive_response, create_contact_response, create_document_response, create_event_response,
    create_task_response, link_document_response, reply_response,
};
use super::constants::WORKFLOW_EVENT_TYPE;
use super::models::{WorkflowActionKind, WorkflowActionRequest, WorkflowActionResponse};
use super::response::response_from_event;
use super::source::load_source_message;
use super::validation::{actor_id_from_headers, normalize_non_empty};

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
    let response = match request.action.clone() {
        WorkflowActionKind::Reply => {
            reply_response(&command_id, &event_id, &request, source_message.as_ref())?
        }
        WorkflowActionKind::CreateTask => {
            create_task_response(
                &pool,
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
