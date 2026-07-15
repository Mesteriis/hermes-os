use super::super::*;
use crate::app::handlers::communications::legal_export::{SendRequest, SendResponse};
use crate::application::communication_send::{
    CommunicationSendDependencies, CommunicationSendError, CommunicationSendRequest, send_email,
};
use serde_json::json;

pub(crate) async fn post_v1_send(
    State(state): State<AppState>,
    Json(req): Json<SendRequest>,
) -> Result<Json<SendResponse>, ApiError> {
    if req.confirmed_provider_write != Some(true) {
        return Err(ApiError::ProviderWriteConfirmationRequired);
    }

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let deps = CommunicationSendDependencies::new(pool, api_audit_log(&state)?);
    let result = send_email(
        &deps,
        CommunicationSendRequest {
            account_id: req.account_id,
            to: req.to,
            cc: req.cc.unwrap_or_default(),
            bcc: req.bcc.unwrap_or_default(),
            subject: req.subject,
            body_text: req.body_text,
            body_html: req.body_html,
            in_reply_to: req.in_reply_to,
            references: req.references.unwrap_or_default(),
            draft_id: req.draft_id,
            scheduled_send_at: req.scheduled_send_at,
            undo_send_seconds: req.undo_send_seconds,
            metadata: json!({}),
        },
    )
    .await
    .map_err(communication_send_api_error)?;

    Ok(Json(SendResponse {
        message_id: result.message_id,
        outbox_id: result.outbox_id,
        accepted: result.accepted,
        accepted_recipients: result.accepted_recipients,
        transport: result.transport,
        status: result.status,
        scheduled_send_at: result.scheduled_send_at,
        undo_deadline_at: result.undo_deadline_at,
        failure_reason: result.failure_reason,
    }))
}

fn communication_send_api_error(error: CommunicationSendError) -> ApiError {
    match error {
        CommunicationSendError::InvalidRequest(message) => {
            ApiError::InvalidCommunicationQuery(message)
        }
        CommunicationSendError::ProviderAccountNotFound => {
            ApiError::InvalidCommunicationQuery("provider account was not found")
        }
        CommunicationSendError::CommunicationIngestion(inner) => ApiError::from(inner),
        CommunicationSendError::Command(inner) => ApiError::from(inner),
        CommunicationSendError::Audit(inner) => ApiError::from(inner),
    }
}
