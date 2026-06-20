use crate::app::ApiError;
use crate::domains::communications::messages::{MessageProjectionStore, ProjectedMessage};

use super::models::WorkflowActionSource;
use super::validation::normalize_non_empty;

pub(super) async fn load_source_message(
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
