use sqlx::{Postgres, Transaction};

use crate::app::ApiError;
use crate::domains::documents::core::{DocumentImportStore, NewDocumentImport};
use crate::domains::mail::messages::ProjectedMessage;

use super::super::models::{
    WorkflowActionRequest, WorkflowActionResponse, WorkflowActionStatus, WorkflowActionTarget,
    WorkflowActionTargetKind,
};
use super::super::response::base_response;
use super::super::validation::{input_title, normalize_non_empty, require_source_message};

pub(in crate::domains::mail::handlers::workflow_actions) async fn create_document_response(
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

pub(in crate::domains::mail::handlers::workflow_actions) async fn link_document_response(
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
