use super::*;

impl CommunicationCommandService {
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

    pub(crate) async fn enqueue_mail_message_provider_command_in_transaction(
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
}
