use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use super::ai_state::{
    CommunicationAiStateRecord, CommunicationAiStateStore, CommunicationAiStateTransitionRequest,
};
use super::core::ProviderAccount;
use super::drafts::{
    CommunicationDraft, CommunicationDraftError, CommunicationDraftStore, DraftStatus,
    NewCommunicationDraft,
};
use super::flags::{MessageFlags, MessageFlagsError};
use super::folders::{
    CommunicationFolder, CommunicationFolderError, CommunicationFolderStore,
    FolderMessageActionResponse, NewCommunicationFolder, UpdateCommunicationFolder,
};
use super::messages::{
    MessageProjectionError, MessageProjectionStore, ProjectedMessage, WorkflowState,
};
use super::outbox::{
    CommunicationOutboxError, CommunicationOutboxItem, CommunicationOutboxStatus,
    CommunicationOutboxStore, NewCommunicationOutboxItem, ProviderSendStore,
    ProviderSendStoreError,
};
use super::saved_searches::{
    CommunicationSavedSearch, CommunicationSavedSearchError, CommunicationSavedSearchStore,
    NewCommunicationSavedSearch, UpdateCommunicationSavedSearch,
};
use super::storage::{
    AttachmentSafetyScanError, AttachmentSafetyScanRequest, AttachmentSafetyScanner,
    CommunicationStorageError, CommunicationStorageStore, HeuristicAttachmentSafetyScanner,
    ImportedCommunicationAttachment, LocalCommunicationBlobStore, NewCommunicationAttachmentImport,
    NewCommunicationBlob, new_communication_attachment_import_id,
};
use crate::domains::communications::evidence::merge_metadata;
use crate::platform::communications::{DEFAULT_MAIL_SYNC_BLOB_ROOT, OutgoingEmail};
use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, ObservationStoreError,
};

const MAX_ATTACHMENT_IMPORT_BYTES: usize = 50 * 1024 * 1024;
const LOCAL_IMPORT_ACTOR_ID: &str = "hermes-frontend";

#[derive(Clone)]
pub struct CommunicationCommandService {
    pool: PgPool,
}

impl CommunicationCommandService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_draft(
        &self,
        command: CommunicationDraftUpsertCommand,
    ) -> Result<CommunicationDraft, CommunicationCommandServiceError> {
        let metadata = command.metadata.clone().unwrap_or_else(|| json!({}));
        let status = command
            .status
            .as_deref()
            .and_then(DraftStatus::parse)
            .unwrap_or(DraftStatus::Draft);
        let store = CommunicationDraftStore::new(self.pool.clone());
        let existing = store.get(&command.draft_id).await?;
        let operation = if existing.is_some() {
            "draft_update"
        } else {
            "draft_create"
        };
        let observation = self
            .capture_observation(
                "draft mutation",
                "COMMUNICATION_DRAFT",
                json!({
                    "draft_id": command.draft_id.clone(),
                    "account_id": command.account_id.clone(),
                    "persona_id": command.persona_id.clone(),
                    "to_recipient_count": command.to_recipients.len(),
                    "cc_recipient_count": command.cc_recipients.as_ref().map(|items| items.len()).unwrap_or(0),
                    "bcc_recipient_count": command.bcc_recipients.as_ref().map(|items| items.len()).unwrap_or(0),
                    "subject": command.subject.clone(),
                    "has_body_text": !command.body_text.trim().is_empty(),
                    "has_body_html": command.body_html.as_deref().is_some_and(|body| !body.trim().is_empty()),
                    "in_reply_to": command.in_reply_to.clone(),
                    "reference_count": command.references.as_ref().map(|items| items.len()).unwrap_or(0),
                    "status": status.as_str(),
                    "scheduled_send_at": command.scheduled_send_at,
                    "metadata": metadata,
                    "operation": operation,
                }),
                format!("draft://{}/{}", command.draft_id, if existing.is_some() { "update" } else { "create" }),
                json!({
                    "captured_by": "mail_service.upsert_draft",
                    "operation": operation,
                }),
            )
            .await?;

        Ok(store
            .upsert_with_observation(
                &NewCommunicationDraft {
                    draft_id: command.draft_id,
                    account_id: command.account_id,
                    persona_id: command.persona_id,
                    to_recipients: command.to_recipients,
                    cc_recipients: command.cc_recipients.unwrap_or_default(),
                    bcc_recipients: command.bcc_recipients.unwrap_or_default(),
                    subject: command.subject,
                    body_text: command.body_text,
                    body_html: command.body_html,
                    in_reply_to: command.in_reply_to,
                    references: command.references.unwrap_or_default(),
                    status,
                    scheduled_send_at: command.scheduled_send_at,
                    metadata: command.metadata.unwrap_or_else(|| json!({})),
                },
                Some(&observation.observation_id),
                "draft_upsert",
                None,
            )
            .await?)
    }

    pub async fn delete_draft(
        &self,
        draft_id: &str,
    ) -> Result<bool, CommunicationCommandServiceError> {
        let store = CommunicationDraftStore::new(self.pool.clone());
        let Some(existing_draft) = store.get(draft_id).await? else {
            return Ok(false);
        };
        let observation = self
            .capture_observation(
                "draft delete",
                "COMMUNICATION_DRAFT",
                json!({
                    "draft_id": existing_draft.draft_id,
                    "account_id": existing_draft.account_id,
                    "status": existing_draft.status.as_str(),
                    "scheduled_send_at": existing_draft.scheduled_send_at,
                    "operation": "draft_delete",
                }),
                format!("draft://{draft_id}/delete"),
                json!({
                    "captured_by": "mail_service.delete_draft",
                    "operation": "draft_delete",
                }),
            )
            .await?;

        Ok(store
            .delete_with_observation(
                draft_id,
                Some(&observation.observation_id),
                "draft_delete",
                Some(json!({
                    "status": existing_draft.status.as_str(),
                })),
            )
            .await?)
    }

    pub async fn create_folder(
        &self,
        request: NewCommunicationFolder,
    ) -> Result<CommunicationFolder, CommunicationCommandServiceError> {
        let observation = self
            .capture_observation(
                "folder create",
                "COMMUNICATION_FOLDER",
                json!({
                    "account_id": request.account_id.clone(),
                    "name": request.name.clone(),
                    "description": request.description.clone(),
                    "color": request.color.clone(),
                    "sort_order": request.sort_order,
                    "operation": "folder_create",
                }),
                "folder://create".to_owned(),
                json!({
                    "captured_by": "mail_service.create_folder",
                    "operation": "folder_create",
                }),
            )
            .await?;

        Ok(CommunicationFolderStore::new(self.pool.clone())
            .create_with_observation(
                request,
                Some(&observation.observation_id),
                "folder_upsert",
                None,
            )
            .await?)
    }

    pub async fn update_folder(
        &self,
        folder_id: &str,
        request: UpdateCommunicationFolder,
    ) -> Result<Option<CommunicationFolder>, CommunicationCommandServiceError> {
        let observation = self
            .capture_observation(
                "folder update",
                "COMMUNICATION_FOLDER",
                json!({
                    "folder_id": folder_id,
                    "account_id": request.account_id.clone(),
                    "name": request.name.clone(),
                    "description": request.description.clone(),
                    "color": request.color.clone(),
                    "sort_order": request.sort_order,
                    "operation": "folder_update",
                }),
                format!("folder://{folder_id}/update"),
                json!({
                    "captured_by": "mail_service.update_folder",
                    "operation": "folder_update",
                }),
            )
            .await?;

        Ok(CommunicationFolderStore::new(self.pool.clone())
            .update_with_observation(
                folder_id,
                request,
                Some(&observation.observation_id),
                "folder_upsert",
                None,
            )
            .await?)
    }

    pub async fn delete_folder(
        &self,
        folder_id: &str,
    ) -> Result<bool, CommunicationCommandServiceError> {
        let observation = self
            .capture_observation(
                "folder delete",
                "COMMUNICATION_FOLDER",
                json!({
                    "folder_id": folder_id,
                    "operation": "folder_delete",
                }),
                format!("folder://{folder_id}/delete"),
                json!({
                    "captured_by": "mail_service.delete_folder",
                    "operation": "folder_delete",
                }),
            )
            .await?;

        Ok(CommunicationFolderStore::new(self.pool.clone())
            .delete_with_observation(
                folder_id,
                Some(&observation.observation_id),
                "folder_delete",
                None,
            )
            .await?)
    }

    pub async fn copy_message_to_folder(
        &self,
        folder_id: &str,
        message_id: &str,
    ) -> Result<Option<FolderMessageActionResponse>, CommunicationCommandServiceError> {
        let observation = self
            .capture_observation(
                "folder message copy",
                "COMMUNICATION_MESSAGE",
                json!({
                    "folder_id": folder_id,
                    "message_id": message_id,
                    "operation": "folder_message_copy",
                }),
                format!("folder://{folder_id}/messages/{message_id}/copy"),
                json!({
                    "captured_by": "mail_service.copy_message_to_folder",
                    "operation": "folder_message_copy",
                }),
            )
            .await?;

        Ok(CommunicationFolderStore::new(self.pool.clone())
            .copy_message_with_observation(
                folder_id,
                message_id,
                Some(&observation.observation_id),
                "folder_message_transition",
                None,
            )
            .await?)
    }

    pub async fn move_message_to_folder(
        &self,
        folder_id: &str,
        message_id: &str,
    ) -> Result<Option<FolderMessageActionResponse>, CommunicationCommandServiceError> {
        let observation = self
            .capture_observation(
                "folder message move",
                "COMMUNICATION_MESSAGE",
                json!({
                    "folder_id": folder_id,
                    "message_id": message_id,
                    "operation": "folder_message_move",
                }),
                format!("folder://{folder_id}/messages/{message_id}/move"),
                json!({
                    "captured_by": "mail_service.move_message_to_folder",
                    "operation": "folder_message_move",
                }),
            )
            .await?;

        Ok(CommunicationFolderStore::new(self.pool.clone())
            .move_message_with_observation(
                folder_id,
                message_id,
                Some(&observation.observation_id),
                "folder_message_transition",
                None,
            )
            .await?)
    }

    pub async fn create_saved_search(
        &self,
        request: NewCommunicationSavedSearch,
    ) -> Result<CommunicationSavedSearch, CommunicationCommandServiceError> {
        let observation = self
            .capture_observation(
                "saved search create",
                "COMMUNICATION_SAVED_SEARCH",
                json!({
                    "name": request.name.clone(),
                    "description": request.description.clone(),
                    "account_id": request.account_id.clone(),
                    "query": request.query.clone(),
                    "workflow_state": request.workflow_state.map(|state| state.as_str().to_owned()),
                    "local_state": request.local_state.map(|state| state.as_str().to_owned()),
                    "channel_kind": request.channel_kind.clone(),
                    "is_smart_folder": request.is_smart_folder,
                    "sort_order": request.sort_order,
                    "operation": "saved_search_create",
                }),
                "saved-search://create".to_owned(),
                json!({
                    "captured_by": "mail_service.create_saved_search",
                    "operation": "saved_search_create",
                }),
            )
            .await?;

        Ok(CommunicationSavedSearchStore::new(self.pool.clone())
            .create_with_observation(
                request,
                Some(&observation.observation_id),
                "saved_search_upsert",
                None,
            )
            .await?)
    }

    pub async fn update_saved_search(
        &self,
        saved_search_id: &str,
        request: UpdateCommunicationSavedSearch,
    ) -> Result<Option<CommunicationSavedSearch>, CommunicationCommandServiceError> {
        let observation = self
            .capture_observation(
                "saved search update",
                "COMMUNICATION_SAVED_SEARCH",
                json!({
                    "saved_search_id": saved_search_id,
                    "name": request.name.clone(),
                    "description": request.description.clone(),
                    "account_id": request.account_id.clone(),
                    "query": request.query.clone(),
                    "workflow_state": request.workflow_state.map(|state| state.as_str().to_owned()),
                    "local_state": request.local_state.map(|state| state.as_str().to_owned()),
                    "channel_kind": request.channel_kind.clone(),
                    "is_smart_folder": request.is_smart_folder,
                    "sort_order": request.sort_order,
                    "operation": "saved_search_update",
                }),
                format!("saved-search://{saved_search_id}/update"),
                json!({
                    "captured_by": "mail_service.update_saved_search",
                    "operation": "saved_search_update",
                }),
            )
            .await?;

        Ok(CommunicationSavedSearchStore::new(self.pool.clone())
            .update_with_observation(
                saved_search_id,
                request,
                Some(&observation.observation_id),
                "saved_search_upsert",
                None,
            )
            .await?)
    }

    pub async fn delete_saved_search(
        &self,
        saved_search_id: &str,
    ) -> Result<bool, CommunicationCommandServiceError> {
        let observation = self
            .capture_observation(
                "saved search delete",
                "COMMUNICATION_SAVED_SEARCH",
                json!({
                    "saved_search_id": saved_search_id,
                    "operation": "saved_search_delete",
                }),
                format!("saved-search://{saved_search_id}/delete"),
                json!({
                    "captured_by": "mail_service.delete_saved_search",
                    "operation": "saved_search_delete",
                }),
            )
            .await?;

        Ok(CommunicationSavedSearchStore::new(self.pool.clone())
            .delete_with_observation(
                saved_search_id,
                Some(&observation.observation_id),
                "saved_search_delete",
                None,
            )
            .await?)
    }

    pub async fn undo_outbox(
        &self,
        outbox_id: &str,
    ) -> Result<CommunicationOutboxItem, CommunicationCommandServiceError> {
        let observation = self
            .capture_observation(
                "outbox undo",
                "COMMUNICATION_OUTBOX",
                json!({
                    "outbox_id": outbox_id,
                    "operation": "outbox_undo",
                }),
                format!("outbox://{outbox_id}/undo"),
                json!({
                    "captured_by": "mail_service.undo_outbox",
                    "operation": "outbox_undo",
                }),
            )
            .await?;

        Ok(CommunicationOutboxStore::new(self.pool.clone())
            .undo_with_observation(
                outbox_id,
                Utc::now(),
                Some(&observation.observation_id),
                "outbox_status_transition",
                None,
            )
            .await?)
    }

    pub async fn enqueue_outbox_send(
        &self,
        account: &ProviderAccount,
        email: &OutgoingEmail,
        command: &CommunicationOutboxSendCommand,
    ) -> Result<CommunicationOutboxItem, CommunicationCommandServiceError> {
        if !command.metadata.is_object() {
            return Err(CommunicationCommandServiceError::InvalidRequest(
                "message metadata must be a JSON object",
            ));
        }
        let now = Utc::now();
        let undo_deadline_at = command
            .undo_send_seconds
            .filter(|seconds| *seconds > 0)
            .map(|seconds| now + chrono::Duration::seconds(seconds.clamp(1, 300)));
        let status = match command.scheduled_send_at {
            Some(scheduled_send_at) if scheduled_send_at > now => {
                CommunicationOutboxStatus::Scheduled
            }
            _ => CommunicationOutboxStatus::Queued,
        };
        let operation = if status == CommunicationOutboxStatus::Scheduled {
            "outbox_schedule"
        } else {
            "outbox_enqueue"
        };
        let observation = self
            .capture_observation(
                "outbox send enqueue",
                "COMMUNICATION_OUTBOX",
                json!({
                    "account_id": account.account_id.clone(),
                    "draft_id": command.draft_id.clone(),
                    "to_recipient_count": email.to.len(),
                    "cc_recipient_count": email.cc.len(),
                    "bcc_recipient_count": email.bcc.len(),
                    "subject": email.subject.clone(),
                    "has_body_text": !email.body_text.trim().is_empty(),
                    "has_body_html": email.body_html.as_deref().is_some_and(|body| !body.trim().is_empty()),
                    "scheduled_send_at": command.scheduled_send_at,
                    "undo_deadline_at": undo_deadline_at,
                    "status": status.as_str(),
                    "operation": operation,
                }),
                format!("outbox://{}/{}", account.account_id, operation),
                json!({
                    "captured_by": "mail_service.enqueue_outbox_send",
                    "operation": operation,
                }),
            )
            .await?;
        let outbox_id = format!(
            "outbox:{}:{}",
            account.account_id,
            now.timestamp_nanos_opt().unwrap_or_default()
        );

        Ok(CommunicationOutboxStore::new(self.pool.clone())
            .enqueue_with_observation(
                &NewCommunicationOutboxItem {
                    outbox_id,
                    account_id: account.account_id.clone(),
                    draft_id: command.draft_id.clone(),
                    to_recipients: email.to.clone(),
                    cc_recipients: email.cc.clone(),
                    bcc_recipients: email.bcc.clone(),
                    subject: email.subject.clone(),
                    body_text: email.body_text.clone(),
                    body_html: email.body_html.clone(),
                    status,
                    scheduled_send_at: command.scheduled_send_at,
                    undo_deadline_at,
                    metadata: merge_metadata(
                        json!({
                        "from": email.from,
                        "in_reply_to": email.in_reply_to,
                        "references": email.references
                        }),
                        Some(command.metadata.clone()),
                    ),
                },
                Some(&observation.observation_id),
                "outbox_status_transition",
                Some(json!({
                    "operation": operation,
                })),
            )
            .await?)
    }

    pub async fn record_provider_send_sent(
        &self,
        account: &ProviderAccount,
        email: &OutgoingEmail,
        transport: &str,
        provider_message_id: &str,
    ) -> Result<(), CommunicationCommandServiceError> {
        let operation = match transport.trim() {
            "smtp" => "provider_send_smtp".to_owned(),
            "gmail" => "provider_send_gmail".to_owned(),
            other => format!("provider_send_{other}"),
        };
        let observation = self
            .capture_observation(
                "provider send",
                "COMMUNICATION_MESSAGE",
                json!({
                    "account_id": account.account_id.clone(),
                    "transport": transport,
                    "to_recipient_count": email.to.len(),
                    "cc_recipient_count": email.cc.len(),
                    "bcc_recipient_count": email.bcc.len(),
                    "subject": email.subject.clone(),
                    "has_body_text": !email.body_text.trim().is_empty(),
                    "has_body_html": email.body_html.as_deref().is_some_and(|body| !body.trim().is_empty()),
                    "operation": operation,
                }),
                format!("provider-send://{}/{}", account.account_id, transport.trim()),
                json!({
                    "captured_by": "mail_service.record_provider_send_sent",
                    "operation": operation,
                }),
            )
            .await?;
        ProviderSendStore::new(self.pool.clone())
            .record_sent_with_observation(
                &observation.observation_id,
                provider_message_id,
                transport,
                None,
            )
            .await?;
        Ok(())
    }

    pub async fn transition_message_workflow_state(
        &self,
        message_id: &str,
        new_state: WorkflowState,
        actor_id: &str,
    ) -> Result<CommunicationWorkflowStateTransitionResult, CommunicationCommandServiceError> {
        let store = MessageProjectionStore::new(self.pool.clone());
        let current = store
            .message(message_id)
            .await?
            .ok_or(MessageProjectionError::MessageNotFound)?;

        if !WorkflowState::is_valid_transition(&current.workflow_state, &new_state) {
            return Err(CommunicationCommandServiceError::InvalidRequest(
                "invalid workflow state transition",
            ));
        }

        let previous_state = current.workflow_state.as_str().to_owned();
        let observation = self
            .capture_observation(
                "message workflow state transition",
                "COMMUNICATION_MESSAGE",
                json!({
                    "message_id": message_id,
                    "previous_state": previous_state,
                    "workflow_state": new_state.as_str(),
                    "operation": "message_workflow_state_transition",
                    "actor_id": actor_id,
                }),
                format!("message://{message_id}/workflow-state"),
                json!({
                    "captured_by": "mail_service.transition_message_workflow_state",
                    "operation": "message_workflow_state_transition",
                }),
            )
            .await?;
        let updated = store
            .transition_workflow_state_with_observation(
                message_id,
                new_state,
                Some(&observation.observation_id),
                "workflow_state_transition",
                Some(json!({
                    "previous_state": previous_state,
                })),
            )
            .await?;

        Ok(CommunicationWorkflowStateTransitionResult {
            updated,
            previous_state,
        })
    }

    pub async fn mark_message_imap_read(
        &self,
        message_id: &str,
    ) -> Result<ProjectedMessage, CommunicationCommandServiceError> {
        let store = MessageProjectionStore::new(self.pool.clone());
        let current = store
            .message(message_id)
            .await?
            .ok_or(MessageProjectionError::MessageNotFound)?;
        let observation = self
            .capture_observation(
                "imap mark read",
                "COMMUNICATION_MESSAGE",
                json!({
                    "message_id": message_id,
                    "previous_workflow_state": current.workflow_state.as_str(),
                    "workflow_state": WorkflowState::Reviewed.as_str(),
                    "operation": "imap_mark_read",
                }),
                format!("message://{message_id}/imap-mark-read"),
                json!({
                    "captured_by": "mail_service.mark_message_imap_read",
                    "operation": "imap_mark_read",
                }),
            )
            .await?;
        Ok(store
            .transition_workflow_state_with_observation(
                message_id,
                WorkflowState::Reviewed,
                Some(&observation.observation_id),
                "workflow_state_transition",
                Some(json!({
                    "previous_state": current.workflow_state.as_str(),
                })),
            )
            .await?)
    }

    pub async fn move_message_to_local_trash(
        &self,
        message_id: &str,
        operation: &'static str,
        reason: &'static str,
    ) -> Result<ProjectedMessage, CommunicationCommandServiceError> {
        let store = MessageProjectionStore::new(self.pool.clone());
        let current = store
            .message(message_id)
            .await?
            .ok_or(MessageProjectionError::MessageNotFound)?;
        let observation = self
            .capture_observation(
                operation,
                "COMMUNICATION_MESSAGE",
                json!({
                    "message_id": message_id,
                    "previous_local_state": current.local_state.as_str(),
                    "local_state": "trash",
                    "operation": operation,
                }),
                format!("message://{message_id}/{}", operation.replace('_', "-")),
                json!({
                    "captured_by": "mail_service.move_message_to_local_trash",
                    "operation": operation,
                }),
            )
            .await?;
        Ok(store
            .move_to_local_trash_with_observation(
                message_id,
                reason,
                Some(&observation.observation_id),
                "local_state_transition",
                Some(json!({
                    "previous_local_state": current.local_state.as_str(),
                })),
            )
            .await?)
    }

    pub async fn restore_message_from_local_trash(
        &self,
        message_id: &str,
    ) -> Result<ProjectedMessage, CommunicationCommandServiceError> {
        let store = MessageProjectionStore::new(self.pool.clone());
        let current = store
            .message(message_id)
            .await?
            .ok_or(MessageProjectionError::MessageNotFound)?;
        let observation = self
            .capture_observation(
                "message_restore",
                "COMMUNICATION_MESSAGE",
                json!({
                    "message_id": message_id,
                    "previous_local_state": current.local_state.as_str(),
                    "operation": "message_restore",
                }),
                format!("message://{message_id}/restore"),
                json!({
                    "captured_by": "mail_service.restore_message_from_local_trash",
                    "operation": "message_restore",
                }),
            )
            .await?;
        Ok(store
            .restore_from_local_trash_with_observation(
                message_id,
                Some(&observation.observation_id),
                "local_state_transition",
                Some(json!({
                    "previous_local_state": current.local_state.as_str(),
                })),
            )
            .await?)
    }

    pub async fn transition_message_ai_state(
        &self,
        message_id: &str,
        request: CommunicationAiStateTransitionRequest,
    ) -> Result<CommunicationAiStateRecord, CommunicationCommandServiceError> {
        let store = CommunicationAiStateStore::new(self.pool.clone());
        let current = store
            .current(message_id)
            .await?
            .ok_or(MessageProjectionError::MessageNotFound)?;
        let request_payload = json!({
            "ai_state": request.ai_state.as_str(),
            "review_reason": request.review_reason.clone(),
            "last_error": request.last_error.clone(),
        });
        let observation = self
            .capture_observation(
                "message ai state transition",
                "COMMUNICATION_MESSAGE",
                json!({
                    "message_id": message_id,
                    "previous_ai_state": current.ai_state.as_str(),
                    "request": request_payload,
                    "operation": "message_ai_state_transition",
                }),
                format!("message://{message_id}/ai-state"),
                json!({
                    "captured_by": "mail_service.transition_message_ai_state",
                    "operation": "message_ai_state_transition",
                }),
            )
            .await?;
        let record = store
            .transition_with_observation(
                message_id,
                request,
                Some(&observation.observation_id),
                "ai_state_transition",
                Some(json!({
                    "previous_ai_state": current.ai_state.as_str(),
                })),
            )
            .await?
            .ok_or(MessageProjectionError::MessageNotFound)?;
        Ok(record)
    }

    pub async fn toggle_message_pin(
        &self,
        message_id: &str,
    ) -> Result<bool, CommunicationCommandServiceError> {
        let store = MessageProjectionStore::new(self.pool.clone());
        let current = store
            .message(message_id)
            .await?
            .ok_or(MessageProjectionError::MessageNotFound)?;
        let next_pinned = !MessageFlags::is_pinned(&current);
        let observation = self
            .capture_message_flag_observation(
                message_id,
                "message_pin_toggle",
                json!({
                    "previous_pinned": MessageFlags::is_pinned(&current),
                }),
            )
            .await?;
        Ok(MessageFlags::toggle_pin_with_observation(
            &store,
            message_id,
            Some(&observation.observation_id),
            "message_flag_update",
            Some(json!({
                "pinned": next_pinned,
            })),
        )
        .await?)
    }

    pub async fn toggle_message_important(
        &self,
        message_id: &str,
    ) -> Result<bool, CommunicationCommandServiceError> {
        let store = MessageProjectionStore::new(self.pool.clone());
        let current = store
            .message(message_id)
            .await?
            .ok_or(MessageProjectionError::MessageNotFound)?;
        let next_important = !MessageFlags::is_important(&current);
        let observation = self
            .capture_message_flag_observation(
                message_id,
                "message_important_toggle",
                json!({
                    "previous_important": MessageFlags::is_important(&current),
                }),
            )
            .await?;
        Ok(MessageFlags::toggle_important_with_observation(
            &store,
            message_id,
            Some(&observation.observation_id),
            "message_flag_update",
            Some(json!({
                "important": next_important,
            })),
        )
        .await?)
    }

    pub async fn snooze_message(
        &self,
        message_id: &str,
        until: DateTime<Utc>,
    ) -> Result<(), CommunicationCommandServiceError> {
        let store = MessageProjectionStore::new(self.pool.clone());
        let current = store
            .message(message_id)
            .await?
            .ok_or(MessageProjectionError::MessageNotFound)?;
        let observation = self
            .capture_message_flag_observation(
                message_id,
                "message_snooze",
                json!({
                    "previous_snooze_until": MessageFlags::snooze_until(&current).map(|value| value.to_rfc3339()),
                    "snooze_until": until.to_rfc3339(),
                }),
            )
            .await?;
        MessageFlags::snooze_with_observation(
            &store,
            message_id,
            until,
            Some(&observation.observation_id),
            "message_flag_update",
            Some(json!({
                "snooze_until": until.to_rfc3339(),
            })),
        )
        .await?;
        Ok(())
    }

    pub async fn toggle_message_mute(
        &self,
        message_id: &str,
    ) -> Result<bool, CommunicationCommandServiceError> {
        let store = MessageProjectionStore::new(self.pool.clone());
        let current = store
            .message(message_id)
            .await?
            .ok_or(MessageProjectionError::MessageNotFound)?;
        let next_muted = !MessageFlags::is_muted(&current);
        let observation = self
            .capture_message_flag_observation(
                message_id,
                "message_mute_toggle",
                json!({
                    "previous_muted": MessageFlags::is_muted(&current),
                }),
            )
            .await?;
        Ok(MessageFlags::toggle_mute_with_observation(
            &store,
            message_id,
            Some(&observation.observation_id),
            "message_flag_update",
            Some(json!({
                "muted": next_muted,
            })),
        )
        .await?)
    }

    pub async fn add_message_label(
        &self,
        message_id: &str,
        label: &str,
    ) -> Result<(), CommunicationCommandServiceError> {
        let store = MessageProjectionStore::new(self.pool.clone());
        let current = store
            .message(message_id)
            .await?
            .ok_or(MessageProjectionError::MessageNotFound)?;
        let observation = self
            .capture_message_flag_observation(
                message_id,
                "message_add_label",
                json!({
                    "previous_labels": MessageFlags::labels(&current),
                    "label": label,
                }),
            )
            .await?;
        MessageFlags::add_label_with_observation(
            &store,
            message_id,
            label,
            Some(&observation.observation_id),
            "message_flag_update",
            Some(json!({
                "label": label,
                "action": "add",
            })),
        )
        .await?;
        Ok(())
    }

    pub async fn remove_message_label(
        &self,
        message_id: &str,
        label: &str,
    ) -> Result<(), CommunicationCommandServiceError> {
        let store = MessageProjectionStore::new(self.pool.clone());
        let current = store
            .message(message_id)
            .await?
            .ok_or(MessageProjectionError::MessageNotFound)?;
        let observation = self
            .capture_message_flag_observation(
                message_id,
                "message_remove_label",
                json!({
                    "previous_labels": MessageFlags::labels(&current),
                    "label": label,
                }),
            )
            .await?;
        MessageFlags::remove_label_with_observation(
            &store,
            message_id,
            label,
            Some(&observation.observation_id),
            "message_flag_update",
            Some(json!({
                "label": label,
                "action": "remove",
            })),
        )
        .await?;
        Ok(())
    }

    pub async fn enqueue_redirect_message(
        &self,
        message_id: &str,
        to: Vec<String>,
        cc: Vec<String>,
        bcc: Vec<String>,
    ) -> Result<CommunicationOutboxItem, CommunicationCommandServiceError> {
        let store = MessageProjectionStore::new(self.pool.clone());
        let msg = store
            .message(message_id)
            .await?
            .ok_or(MessageProjectionError::MessageNotFound)?;
        let now = Utc::now();
        let observation = self
            .capture_observation(
                "outbox redirect enqueue",
                "COMMUNICATION_OUTBOX",
                json!({
                    "message_id": msg.message_id,
                    "account_id": msg.account_id,
                    "to_recipient_count": to.len(),
                    "cc_recipient_count": cc.len(),
                    "bcc_recipient_count": bcc.len(),
                    "subject": msg.subject,
                    "operation": "outbox_redirect_enqueue",
                }),
                format!("outbox://redirect/{}/enqueue", msg.message_id),
                json!({
                    "captured_by": "mail_service.enqueue_redirect_message",
                    "operation": "outbox_redirect_enqueue",
                }),
            )
            .await?;
        let outbox_id = format!(
            "outbox:redirect:{}:{}",
            msg.account_id,
            now.timestamp_nanos_opt().unwrap_or_default()
        );
        Ok(CommunicationOutboxStore::new(self.pool.clone())
            .enqueue_with_observation(
                &NewCommunicationOutboxItem {
                    outbox_id,
                    account_id: msg.account_id.clone(),
                    draft_id: None,
                    to_recipients: to,
                    cc_recipients: cc,
                    bcc_recipients: bcc,
                    subject: msg.subject.clone(),
                    body_text: msg.body_text.clone(),
                    body_html: None,
                    status: CommunicationOutboxStatus::Queued,
                    scheduled_send_at: None,
                    undo_deadline_at: None,
                    metadata: json!({
                        "redirect_mode": "resent",
                        "redirect_of": msg.message_id,
                        "original_sender": msg.sender,
                        "original_provider_record_id": msg.provider_record_id,
                        "resent_at": now,
                    }),
                },
                Some(&observation.observation_id),
                "outbox_status_transition",
                Some(json!({
                    "operation": "outbox_redirect_enqueue",
                    "redirect_of": msg.message_id,
                })),
            )
            .await?)
    }

    pub async fn import_attachment(
        &self,
        request: CommunicationAttachmentImportCommand,
    ) -> Result<ImportedCommunicationAttachment, CommunicationCommandServiceError> {
        let bytes = decode_import_bytes(&request.content_base64)?;
        if bytes.is_empty() {
            return Err(CommunicationCommandServiceError::InvalidRequest(
                "attachment import bytes must not be empty",
            ));
        }
        if bytes.len() > MAX_ATTACHMENT_IMPORT_BYTES {
            return Err(CommunicationCommandServiceError::InvalidRequest(
                "attachment import exceeds the local size limit",
            ));
        }

        let content_type = request
            .content_type
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("application/octet-stream")
            .to_owned();
        let filename = request
            .filename
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        let metadata = request.metadata.clone().unwrap_or_else(|| json!({}));
        if !metadata.is_object() {
            return Err(CommunicationCommandServiceError::InvalidRequest(
                "attachment import metadata must be an object",
            ));
        }

        let blob_store = LocalCommunicationBlobStore::new(DEFAULT_MAIL_SYNC_BLOB_ROOT);
        let local_blob = blob_store.put_blob(&bytes).await?;
        let mail_store = CommunicationStorageStore::new(self.pool.clone());
        let stored_blob = mail_store
            .upsert_blob(
                &NewCommunicationBlob::from_local_blob(&local_blob).content_type(&content_type),
            )
            .await?;
        let scanner = HeuristicAttachmentSafetyScanner;
        let scan_report = scanner.scan(&AttachmentSafetyScanRequest {
            provider_attachment_id: "local-import",
            filename: filename.as_deref(),
            content_type: &content_type,
            size_bytes: local_blob.size_bytes,
            sha256: &local_blob.sha256,
            storage_kind: &local_blob.storage_kind,
            storage_path: &local_blob.storage_path,
            bytes: &bytes,
        })?;
        let seed = format!(
            "{}:{}:{}:{}",
            local_blob.sha256,
            filename.as_deref().unwrap_or(""),
            request.account_id.as_deref().unwrap_or(""),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );
        let attachment_id = new_communication_attachment_import_id(&seed);
        let source_kind = request
            .source_kind
            .clone()
            .unwrap_or_else(|| "local_import".to_owned());
        let observation = self
            .capture_observation(
                "attachment import",
                "COMMUNICATION_ATTACHMENT",
                json!({
                    "attachment_id": attachment_id,
                    "account_id": request.account_id.clone(),
                    "channel_kind": request.channel_kind.clone(),
                    "filename": filename.clone(),
                    "content_type": content_type.clone(),
                    "size_bytes": local_blob.size_bytes,
                    "sha256": local_blob.sha256.clone(),
                    "source_kind": source_kind.clone(),
                    "metadata": metadata.clone(),
                }),
                format!("communications://attachments/import/{attachment_id}"),
                json!({
                    "captured_by": "mail_service.import_attachment",
                    "operation": "attachment_import",
                    "storage_kind": local_blob.storage_kind,
                    "blob_id": stored_blob.blob_id,
                }),
            )
            .await?;

        let mut import = NewCommunicationAttachmentImport::new(
            attachment_id,
            stored_blob.blob_id,
            &content_type,
            local_blob.size_bytes,
            &local_blob.sha256,
            LOCAL_IMPORT_ACTOR_ID,
        )
        .source_kind(
            request
                .source_kind
                .unwrap_or_else(|| "local_import".to_owned()),
        )
        .scan_report(scan_report)
        .metadata(metadata);
        if let Some(account_id) = request.account_id {
            import = import.account_id(account_id);
        }
        if let Some(channel_kind) = request.channel_kind {
            import = import.channel_kind(channel_kind);
        }
        if let Some(filename) = filename {
            import = import.filename(filename);
        }

        Ok(mail_store
            .upsert_imported_attachment_with_observation(
                &import,
                Some(&observation.observation_id),
                "attachment_import",
                None,
            )
            .await?)
    }

    async fn capture_observation(
        &self,
        operation: &'static str,
        kind: &'static str,
        payload: Value,
        source_ref: String,
        provenance: Value,
    ) -> Result<crate::platform::observations::Observation, CommunicationCommandServiceError> {
        ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    kind,
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    payload,
                    source_ref,
                )
                .provenance(provenance),
            )
            .await
            .map_err(
                |source| CommunicationCommandServiceError::ObservationCapture { operation, source },
            )
    }

    async fn capture_message_flag_observation(
        &self,
        message_id: &str,
        operation: &'static str,
        payload: Value,
    ) -> Result<crate::platform::observations::Observation, CommunicationCommandServiceError> {
        self.capture_observation(
            "message flag action",
            "COMMUNICATION_MESSAGE",
            json!({
                "message_id": message_id,
                "operation": operation,
                "payload": payload,
            }),
            format!("message://{message_id}/{operation}"),
            json!({
                "captured_by": "mail_service.message_flags",
                "operation": operation,
            }),
        )
        .await
    }
}

#[derive(Clone, Debug)]
pub struct CommunicationDraftUpsertCommand {
    pub draft_id: String,
    pub account_id: String,
    pub persona_id: Option<String>,
    pub to_recipients: Vec<String>,
    pub cc_recipients: Option<Vec<String>>,
    pub bcc_recipients: Option<Vec<String>>,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub in_reply_to: Option<String>,
    pub references: Option<Vec<String>>,
    pub status: Option<String>,
    pub scheduled_send_at: Option<DateTime<Utc>>,
    pub metadata: Option<Value>,
}

#[derive(Clone, Debug)]
pub struct CommunicationAttachmentImportCommand {
    pub account_id: Option<String>,
    pub channel_kind: Option<String>,
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub content_base64: String,
    pub source_kind: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Clone, Debug)]
pub struct CommunicationOutboxSendCommand {
    pub draft_id: Option<String>,
    pub scheduled_send_at: Option<DateTime<Utc>>,
    pub undo_send_seconds: Option<i64>,
    pub metadata: Value,
}

#[derive(Clone, Debug)]
pub struct CommunicationWorkflowStateTransitionResult {
    pub updated: ProjectedMessage,
    pub previous_state: String,
}

#[derive(Debug, Error)]
pub enum CommunicationCommandServiceError {
    #[error("{operation} observation capture failed")]
    ObservationCapture {
        operation: &'static str,
        #[source]
        source: ObservationStoreError,
    },

    #[error("{0}")]
    InvalidRequest(&'static str),

    #[error(transparent)]
    Draft(#[from] CommunicationDraftError),

    #[error(transparent)]
    Folder(#[from] CommunicationFolderError),

    #[error(transparent)]
    SavedSearch(#[from] CommunicationSavedSearchError),

    #[error(transparent)]
    Outbox(#[from] CommunicationOutboxError),

    #[error(transparent)]
    CommunicationStorage(#[from] CommunicationStorageError),

    #[error(transparent)]
    AttachmentScan(#[from] AttachmentSafetyScanError),

    #[error(transparent)]
    ProviderSendStore(#[from] ProviderSendStoreError),

    #[error(transparent)]
    MessageProjection(#[from] MessageProjectionError),

    #[error(transparent)]
    CommunicationAiState(#[from] super::ai_state::CommunicationAiStateError),

    #[error(transparent)]
    MessageFlags(#[from] MessageFlagsError),
}

fn decode_import_bytes(content_base64: &str) -> Result<Vec<u8>, CommunicationCommandServiceError> {
    let encoded = content_base64
        .split_once(',')
        .map(|(_, value)| value)
        .unwrap_or(content_base64)
        .trim();
    BASE64_STANDARD.decode(encoded).map_err(|_| {
        CommunicationCommandServiceError::InvalidRequest("invalid attachment import base64")
    })
}
