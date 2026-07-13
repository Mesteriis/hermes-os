use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;
use uuid::Uuid;

use super::ai_state::{
    CommunicationAiStateRecord, CommunicationAiStateStore, CommunicationAiStateTransitionRequest,
};
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
use super::provider_commands::{
    CommunicationProviderCommandError, CommunicationProviderCommandStore,
    NewCommunicationProviderCommand,
};
use super::saved_searches::{
    CommunicationSavedSearch, CommunicationSavedSearchError, CommunicationSavedSearchStore,
    NewCommunicationSavedSearch, UpdateCommunicationSavedSearch,
};
use super::spam_reputation::{SenderReputationClassification, SenderReputationStore};
use super::storage::{
    AttachmentSafetyScanError, AttachmentSafetyScanRequest, AttachmentSafetyScanStatus,
    CommunicationStorageError, CommunicationStorageStore, ImportedCommunicationAttachment,
    LocalCommunicationBlobStore, NewCommunicationAttachmentImport, NewCommunicationBlob,
    new_communication_attachment_import_id, scan_attachment_with_configured_clamav,
};
use crate::domains::communications::evidence::{link_mail_entity_in_transaction, merge_metadata};
use crate::platform::communications::{DEFAULT_MAIL_SYNC_BLOB_ROOT, OutgoingEmail};
use hermes_communications_api::accounts::ProviderAccount;
use hermes_observations_api::models::{NewObservation, ObservationOriginKind};
use hermes_observations_postgres::errors::ObservationStoreError;
use hermes_observations_postgres::store::ObservationStore;

const MAX_ATTACHMENT_IMPORT_BYTES: usize = 50 * 1024 * 1024;
const LOCAL_IMPORT_ACTOR_ID: &str = "hermes-frontend";
const LOCAL_USER_ACTOR_ID: &str = "hermes-local-user";

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
                    attachment_ids: command.attachment_ids,
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
        self.apply_folder_message_action_with_provider_command(
            folder_id,
            message_id,
            super::folders::FolderMessageOperation::Copy,
        )
        .await
    }

    pub async fn move_message_to_folder(
        &self,
        folder_id: &str,
        message_id: &str,
    ) -> Result<Option<FolderMessageActionResponse>, CommunicationCommandServiceError> {
        self.apply_folder_message_action_with_provider_command(
            folder_id,
            message_id,
            super::folders::FolderMessageOperation::Move,
        )
        .await
    }

    async fn apply_folder_message_action_with_provider_command(
        &self,
        folder_id: &str,
        message_id: &str,
        operation: super::folders::FolderMessageOperation,
    ) -> Result<Option<FolderMessageActionResponse>, CommunicationCommandServiceError> {
        let message_store = MessageProjectionStore::new(self.pool.clone());
        let Some(current) = message_store.message(message_id).await? else {
            return Ok(None);
        };
        let operation_name = operation.as_str();
        let mut transaction = self.pool.begin().await?;
        let observation = ObservationStore::capture_in_transaction(
            &mut transaction,
            &NewObservation::new(
                "COMMUNICATION_MESSAGE",
                ObservationOriginKind::Manual,
                Utc::now(),
                json!({
                    "folder_id": folder_id,
                    "message_id": message_id,
                    "operation": format!("folder_message_{operation_name}"),
                }),
                format!("folder://{folder_id}/messages/{message_id}/{operation_name}"),
            )
            .provenance(json!({
                "captured_by": "mail_service.folder_message_action",
                "operation": format!("folder_message_{operation_name}"),
            })),
        )
        .await
        .map_err(
            |source| CommunicationCommandServiceError::ObservationCapture {
                operation: "folder message evidence capture",
                source,
            },
        )?;
        let response = CommunicationFolderStore::apply_message_action_in_transaction(
            &mut transaction,
            folder_id,
            message_id,
            operation,
            Some(&observation.observation_id),
            "folder_message_transition",
            None,
        )
        .await?;
        let Some(response) = response else {
            transaction.rollback().await?;
            return Ok(None);
        };

        let mapped_provider_resource = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM communication_mail_provider_resources
                WHERE account_id = $1
                  AND local_folder_id = $2
                  AND writable = true
            )
            "#,
        )
        .bind(&current.account_id)
        .bind(&response.folder_id)
        .fetch_one(&mut *transaction)
        .await?;
        if mapped_provider_resource {
            let command_id = format!("mail-{operation_name}-folder:{}", Uuid::new_v4());
            let command = NewCommunicationProviderCommand::new(
                &command_id,
                &current.account_id,
                "mail",
                match operation_name {
                    "copy" => "copy_folder",
                    "move" => "move_folder",
                    _ => unreachable!("folder operation is closed"),
                },
                &command_id,
                LOCAL_USER_ACTOR_ID,
            )
            .provider_message_id(&current.provider_record_id)
            .target_ref(json!({ "message_id": current.message_id }))
            .payload(json!({
                "folder_id": response.folder_id,
                "provider_record_id": current.provider_record_id,
                "message_metadata": current.message_metadata,
            }));
            CommunicationProviderCommandStore::enqueue_in_transaction(&mut transaction, &command)
                .await?;
        }
        transaction.commit().await?;
        Ok(Some(response))
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
        let mut transaction = self.pool.begin().await?;
        let updated = MessageProjectionStore::transition_workflow_state_in_transaction(
            &mut transaction,
            message_id,
            new_state,
        )
        .await?;
        let observation = ObservationStore::capture_in_transaction(
            &mut transaction,
            &NewObservation::new(
                "COMMUNICATION_MESSAGE",
                ObservationOriginKind::Manual,
                Utc::now(),
                json!({
                    "message_id": message_id,
                    "previous_state": previous_state,
                    "workflow_state": new_state.as_str(),
                    "operation": "message_workflow_state_transition",
                    "actor_id": actor_id,
                }),
                format!("message://{message_id}/workflow-state"),
            )
            .provenance(json!({
                    "captured_by": "mail_service.transition_message_workflow_state",
                    "operation": "message_workflow_state_transition",
            })),
        )
        .await
        .map_err(
            |source| CommunicationCommandServiceError::ObservationCapture {
                operation: "workflow state evidence capture",
                source,
            },
        )?;
        link_mail_entity_in_transaction(
            &mut transaction,
            &observation.observation_id,
            "communication_message",
            updated.message_id.clone(),
            "workflow_state_transition",
            json!({ "workflow_state": updated.workflow_state.as_str() }),
            Some(json!({ "previous_state": previous_state })),
        )
        .await
        .map_err(
            |source| CommunicationCommandServiceError::ObservationCapture {
                operation: "workflow state evidence link",
                source,
            },
        )?;
        if let Some(command_kind) = match (current.workflow_state, new_state) {
            (_, WorkflowState::Spam) => Some("mark_spam"),
            (WorkflowState::Spam, WorkflowState::New) => Some("mark_not_spam"),
            (_, WorkflowState::Archived) => Some("archive"),
            _ => None,
        } {
            let command_id = format!("mail-{command_kind}:{}", Uuid::new_v4());
            Self::enqueue_mail_message_provider_command_in_transaction(
                &mut transaction,
                &command_id,
                &updated,
                command_kind,
                actor_id,
            )
            .await?;
        }
        transaction.commit().await?;

        // Reputation is derived calibration data. A failed update must never undo
        // the user's canonical local workflow decision or its provider command.
        if let Some(classification) = match (current.workflow_state, new_state) {
            (_, WorkflowState::Spam) => Some(SenderReputationClassification::Spam),
            (WorkflowState::Spam, WorkflowState::New) => {
                Some(SenderReputationClassification::NonSpam)
            }
            _ => None,
        } && let Err(error) = SenderReputationStore::new(self.pool.clone())
            .record_analysis(&updated, classification, "manual_workflow_state_transition")
            .await
        {
            tracing::warn!(
                message_id = %updated.message_id,
                error = %error,
                "mail sender reputation feedback could not be recorded"
            );
        }

        Ok(CommunicationWorkflowStateTransitionResult {
            updated,
            previous_state,
        })
    }

    pub(crate) async fn enqueue_archive_provider_command_in_transaction(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        workflow_command_id: &str,
        message: &ProjectedMessage,
        actor_id: &str,
    ) -> Result<(), CommunicationCommandServiceError> {
        if workflow_command_id.trim().is_empty() || actor_id.trim().is_empty() {
            return Err(CommunicationCommandServiceError::InvalidRequest(
                "workflow_command_id and actor_id must not be empty",
            ));
        }
        let command_id = format!("mail-archive:{}", workflow_command_id.trim());
        Self::enqueue_mail_message_provider_command_in_transaction(
            transaction,
            &command_id,
            message,
            "archive",
            actor_id,
        )
        .await
    }

    async fn enqueue_mail_message_provider_command_in_transaction(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        command_id: &str,
        message: &ProjectedMessage,
        command_kind: &str,
        actor_id: &str,
    ) -> Result<(), CommunicationCommandServiceError> {
        let command = NewCommunicationProviderCommand::new(
            command_id,
            &message.account_id,
            "mail",
            command_kind,
            command_id,
            actor_id,
        )
        .provider_message_id(&message.provider_record_id)
        .target_ref(json!({ "message_id": message.message_id }))
        .payload(json!({
            "provider_record_id": message.provider_record_id,
            "message_metadata": message.message_metadata,
        }));
        CommunicationProviderCommandStore::enqueue_in_transaction(transaction, &command).await?;
        Ok(())
    }

    pub async fn mark_message_read_local(
        &self,
        message_id: &str,
    ) -> Result<ProjectedMessage, CommunicationCommandServiceError> {
        self.set_message_read_local_with_provider_command(message_id, true, "hermes-local-user")
            .await
    }

    pub async fn set_message_read_local_with_provider_command(
        &self,
        message_id: &str,
        is_read: bool,
        actor_id: &str,
    ) -> Result<ProjectedMessage, CommunicationCommandServiceError> {
        let store = MessageProjectionStore::new(self.pool.clone());
        let current = store
            .message(message_id)
            .await?
            .ok_or(MessageProjectionError::MessageNotFound)?;
        if actor_id.trim().is_empty() {
            return Err(CommunicationCommandServiceError::InvalidRequest(
                "actor_id must not be empty",
            ));
        }

        let command_kind = if is_read { "mark_read" } else { "mark_unread" };
        let operation = if is_read {
            "local_mark_read"
        } else {
            "local_mark_unread"
        };
        let mut transaction = self.pool.begin().await?;
        let updated = MessageProjectionStore::set_read_state_in_transaction(
            &mut transaction,
            message_id,
            is_read,
            "local_user",
        )
        .await?;
        let observation = ObservationStore::capture_in_transaction(
            &mut transaction,
            &NewObservation::new(
                "COMMUNICATION_MESSAGE",
                ObservationOriginKind::Manual,
                Utc::now(),
                json!({
                    "message_id": updated.message_id,
                    "previous_is_read": current.is_read,
                    "is_read": is_read,
                    "operation": operation,
                }),
                format!("message://{message_id}/{operation}"),
            )
            .provenance(json!({
                "captured_by": "mail_service.set_message_read_local_with_provider_command",
                "operation": operation,
            })),
        )
        .await
        .map_err(
            |source| CommunicationCommandServiceError::ObservationCapture {
                operation: "read state evidence capture",
                source,
            },
        )?;
        link_mail_entity_in_transaction(
            &mut transaction,
            &observation.observation_id,
            "communication_message",
            updated.message_id.clone(),
            "read_state_transition",
            json!({ "is_read": is_read }),
            None,
        )
        .await
        .map_err(
            |source| CommunicationCommandServiceError::ObservationCapture {
                operation: "read state evidence link",
                source,
            },
        )?;

        let command_id = format!("mail-read:{}", Uuid::new_v4());
        let command = NewCommunicationProviderCommand::new(
            &command_id,
            &updated.account_id,
            "mail",
            command_kind,
            &command_id,
            actor_id,
        )
        .provider_message_id(&updated.provider_record_id)
        .target_ref(json!({ "message_id": updated.message_id }))
        .payload(json!({
            "desired_is_read": is_read,
            "read_changed_at": updated.read_changed_at.map(|value| value.to_rfc3339()),
            "provider_record_id": updated.provider_record_id,
            "message_metadata": updated.message_metadata,
        }));
        CommunicationProviderCommandStore::enqueue_in_transaction(&mut transaction, &command)
            .await?;
        transaction.commit().await?;
        Ok(updated)
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
        let mut transaction = self.pool.begin().await?;
        let updated = MessageProjectionStore::move_to_local_trash_in_transaction(
            &mut transaction,
            message_id,
            reason,
        )
        .await?;
        let observation = ObservationStore::capture_in_transaction(
            &mut transaction,
            &NewObservation::new(
                "COMMUNICATION_MESSAGE",
                ObservationOriginKind::Manual,
                Utc::now(),
                json!({
                    "message_id": message_id,
                    "previous_local_state": current.local_state.as_str(),
                    "local_state": "trash",
                    "operation": operation,
                }),
                format!("message://{message_id}/{}", operation.replace('_', "-")),
            )
            .provenance(json!({
                    "captured_by": "mail_service.move_message_to_local_trash",
                    "operation": operation,
            })),
        )
        .await
        .map_err(
            |source| CommunicationCommandServiceError::ObservationCapture {
                operation: "trash state evidence capture",
                source,
            },
        )?;
        link_mail_entity_in_transaction(
            &mut transaction,
            &observation.observation_id,
            "communication_message",
            updated.message_id.clone(),
            "local_state_transition",
            json!({
                "local_state": updated.local_state.as_str(),
                "source": reason,
            }),
            Some(json!({
                "previous_local_state": current.local_state.as_str(),
            })),
        )
        .await
        .map_err(
            |source| CommunicationCommandServiceError::ObservationCapture {
                operation: "trash state evidence link",
                source,
            },
        )?;

        let command_id = format!("mail-trash:{}", Uuid::new_v4());
        let command = NewCommunicationProviderCommand::new(
            &command_id,
            &updated.account_id,
            "mail",
            "trash",
            &command_id,
            LOCAL_USER_ACTOR_ID,
        )
        .provider_message_id(&updated.provider_record_id)
        .target_ref(json!({ "message_id": updated.message_id }))
        .payload(json!({
            "provider_record_id": updated.provider_record_id,
            "message_metadata": updated.message_metadata,
            "reason": reason,
        }));
        CommunicationProviderCommandStore::enqueue_in_transaction(&mut transaction, &command)
            .await?;
        transaction.commit().await?;
        Ok(updated)
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
        let mut metadata = current.message_metadata.clone();
        metadata["important"] = json!(next_important);
        self.persist_provider_synced_metadata(
            &current,
            &metadata,
            if next_important {
                "important"
            } else {
                "not_important"
            },
            json!({ "important": next_important }),
            &observation.observation_id,
            json!({ "important": next_important }),
        )
        .await?;
        Ok(next_important)
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
        let mut labels = MessageFlags::labels(&current);
        if !labels.iter().any(|existing| existing == label) {
            labels.push(label.to_owned());
        }
        let mut metadata = current.message_metadata.clone();
        metadata["labels"] = json!(labels);
        self.persist_provider_synced_metadata(
            &current,
            &metadata,
            "add_label",
            json!({ "label": label }),
            &observation.observation_id,
            json!({ "label": label, "action": "add" }),
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
        let mut labels = MessageFlags::labels(&current);
        labels.retain(|existing| existing != label);
        let mut metadata = current.message_metadata.clone();
        metadata["labels"] = json!(labels);
        self.persist_provider_synced_metadata(
            &current,
            &metadata,
            "remove_label",
            json!({ "label": label }),
            &observation.observation_id,
            json!({ "label": label, "action": "remove" }),
        )
        .await?;
        Ok(())
    }

    async fn persist_provider_synced_metadata(
        &self,
        current: &ProjectedMessage,
        metadata: &Value,
        command_kind: &str,
        command_payload: Value,
        observation_id: &str,
        link_metadata: Value,
    ) -> Result<ProjectedMessage, CommunicationCommandServiceError> {
        let mut transaction = self.pool.begin().await?;
        let updated = MessageProjectionStore::set_message_metadata_in_transaction(
            &mut transaction,
            &current.message_id,
            metadata,
        )
        .await?;
        link_mail_entity_in_transaction(
            &mut transaction,
            observation_id,
            "communication_message",
            updated.message_id.clone(),
            "message_flag_update",
            json!({}),
            Some(link_metadata),
        )
        .await
        .map_err(
            |source| CommunicationCommandServiceError::ObservationCapture {
                operation: "message flag evidence link",
                source,
            },
        )?;
        let command_id = format!("mail-command:{}", Uuid::new_v4());
        let mut payload = command_payload;
        payload["provider_record_id"] = json!(updated.provider_record_id);
        payload["message_metadata"] = updated.message_metadata.clone();
        let command = NewCommunicationProviderCommand::new(
            &command_id,
            &updated.account_id,
            "mail",
            command_kind,
            &command_id,
            LOCAL_USER_ACTOR_ID,
        )
        .provider_message_id(&updated.provider_record_id)
        .target_ref(json!({ "message_id": updated.message_id }))
        .payload(payload);
        CommunicationProviderCommandStore::enqueue_in_transaction(&mut transaction, &command)
            .await?;
        transaction.commit().await?;
        Ok(updated)
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
        let scan_request = AttachmentSafetyScanRequest {
            provider_attachment_id: "local-import",
            filename: filename.as_deref(),
            content_type: &content_type,
            size_bytes: local_blob.size_bytes,
            sha256: &local_blob.sha256,
            storage_kind: &local_blob.storage_kind,
            storage_path: &local_blob.storage_path,
            bytes: &bytes,
        };
        let scan_report = scan_attachment_with_configured_clamav(&scan_request).await?;
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
    ) -> Result<hermes_observations_api::models::Observation, CommunicationCommandServiceError>
    {
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
    ) -> Result<hermes_observations_api::models::Observation, CommunicationCommandServiceError>
    {
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
    pub attachment_ids: Option<Vec<String>>,
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
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

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

    #[error(transparent)]
    ProviderCommand(#[from] CommunicationProviderCommandError),
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
