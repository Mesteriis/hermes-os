use super::*;

impl CommunicationCommandService {
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
}
