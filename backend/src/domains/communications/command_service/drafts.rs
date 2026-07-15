use super::*;

impl CommunicationCommandService {
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
}
