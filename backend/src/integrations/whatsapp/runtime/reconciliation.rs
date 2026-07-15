use super::*;

impl WhatsappWebStore {
    pub(super) async fn reconcile_fixture_message_commands(
        &self,
        message: &NewWhatsappWebMessage,
    ) -> Result<Vec<WhatsAppProviderCommand>, WhatsappWebError> {
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM whatsapp_provider_write_commands
            WHERE account_id = $1
              AND provider_chat_id = $2
              AND command_kind IN ('send_text', 'reply', 'forward')
              AND status IN ('queued', 'retrying', 'executing')
              AND confirmation_decision IN ('confirmed', 'not_required')
              AND capability_state IN ('available', 'degraded')
              AND happened_at <= $3
            ORDER BY happened_at ASC
            "#,
        )
        .bind(&message.account_id)
        .bind(&message.provider_chat_id)
        .bind(message.occurred_at + chrono::Duration::seconds(5))
        .fetch_all(self.pool())
        .await?;

        let mut reconciled = Vec::new();
        for row in rows {
            let command = row_to_whatsapp_provider_write_command(row)?;
            let should_reconcile = match command.command_kind.as_str() {
                "send_text" => {
                    command.payload.get("text").and_then(Value::as_str)
                        == Some(message.text.as_str())
                }
                "reply" => {
                    command.payload.get("text").and_then(Value::as_str)
                        == Some(message.text.as_str())
                        && command
                            .payload
                            .get("reply_to_provider_message_id")
                            .and_then(Value::as_str)
                            == message.reply_to_provider_message_id.as_deref()
                }
                "forward" => {
                    command
                        .payload
                        .get("from_provider_chat_id")
                        .and_then(Value::as_str)
                        == message.forward_origin_chat_id.as_deref()
                        && command
                            .payload
                            .get("from_provider_message_id")
                            .and_then(Value::as_str)
                            == message.forward_origin_message_id.as_deref()
                }
                _ => false,
            };
            if !should_reconcile {
                continue;
            }

            let provider_state = json!({
                "provider_chat_id": message.provider_chat_id,
                "provider_message_id": message.provider_message_id,
                "delivery_state": message.delivery_state.as_str(),
                "reply_to_provider_message_id": message.reply_to_provider_message_id,
                "forward_origin_chat_id": message.forward_origin_chat_id,
                "forward_origin_message_id": message.forward_origin_message_id,
                "observed_via": "fixture_message",
            });
            let result_payload = json!({
                "provider_chat_id": message.provider_chat_id,
                "provider_message_id": message.provider_message_id,
                "delivery_state": message.delivery_state.as_str(),
                "provider_observed_at": message.occurred_at,
                "observed_via": "fixture_message",
            });
            reconciled.push(
                self.mark_provider_command_reconciled(
                    &command.command_id,
                    message.occurred_at,
                    provider_state,
                    result_payload,
                )
                .await?,
            );
        }

        Ok(reconciled)
    }

    pub(super) async fn reconcile_fixture_receipt_commands(
        &self,
        receipt: &NewWhatsappWebReceipt,
    ) -> Result<Vec<WhatsAppProviderCommand>, WhatsappWebError> {
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM whatsapp_provider_write_commands
            WHERE account_id = $1
              AND provider_chat_id = $2
              AND command_kind IN (
                  'send_text', 'send_template', 'reply', 'forward', 'send_media',
                  'send_voice_note'
              )
              AND status IN ('queued', 'retrying', 'executing', 'completed')
              AND confirmation_decision IN ('confirmed', 'not_required')
              AND capability_state IN ('available', 'degraded')
              AND created_at <= $4
              AND (
                  provider_message_id = $3
                  OR result_payload #>> '{provider_submission,provider_request_id}' = $3
                  OR result_payload #>> '{provider_submission,provider_observed_completion_target,provider_message_id}' = $3
              )
            ORDER BY created_at ASC
            "#,
        )
        .bind(&receipt.account_id)
        .bind(&receipt.provider_chat_id)
        .bind(&receipt.provider_message_id)
        .bind(receipt.observed_at + chrono::Duration::seconds(5))
        .fetch_all(self.pool())
        .await?;

        let mut reconciled = Vec::new();
        for row in rows {
            let command = row_to_whatsapp_provider_write_command(row)?;
            if !provider_request_id_matches_observed_receipt(&command, receipt) {
                continue;
            }
            let provider_state = json!({
                "provider_chat_id": receipt.provider_chat_id,
                "provider_message_id": receipt.provider_message_id,
                "delivery_state": receipt.delivery_state.as_str(),
                "provider_observed_at": receipt.observed_at,
                "observed_via": "fixture_receipt",
            });
            let result_payload = json!({
                "provider_chat_id": receipt.provider_chat_id,
                "provider_message_id": receipt.provider_message_id,
                "delivery_state": receipt.delivery_state.as_str(),
                "provider_observed_at": receipt.observed_at,
                "observed_via": "fixture_receipt",
            });
            reconciled.push(
                self.mark_provider_command_reconciled(
                    &command.command_id,
                    receipt.observed_at,
                    provider_state,
                    result_payload,
                )
                .await?,
            );
        }

        Ok(reconciled)
    }

    pub(super) async fn reconcile_fixture_reaction_commands(
        &self,
        reaction: &NewWhatsappWebReaction,
    ) -> Result<Vec<WhatsAppProviderCommand>, WhatsappWebError> {
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM whatsapp_provider_write_commands
            WHERE account_id = $1
              AND provider_chat_id = $2
              AND provider_message_id = $3
              AND command_kind IN ('react', 'unreact')
              AND status IN ('queued', 'retrying', 'executing')
              AND confirmation_decision IN ('confirmed', 'not_required')
              AND capability_state IN ('available', 'degraded')
              AND happened_at <= $4
            ORDER BY happened_at ASC
            "#,
        )
        .bind(&reaction.account_id)
        .bind(&reaction.provider_chat_id)
        .bind(&reaction.provider_message_id)
        .bind(reaction.observed_at + chrono::Duration::seconds(5))
        .fetch_all(self.pool())
        .await?;

        let mut reconciled = Vec::new();
        for row in rows {
            let command = row_to_whatsapp_provider_write_command(row)?;
            let Some(expected_reaction) = command
                .payload
                .get("reaction_emoji")
                .or_else(|| command.payload.get("reaction"))
                .and_then(Value::as_str)
            else {
                continue;
            };
            if expected_reaction != reaction.reaction {
                continue;
            }

            let expected_active = match command.command_kind.as_str() {
                "react" => true,
                "unreact" => false,
                _ => continue,
            };

            let provider_state = json!({
                "provider_chat_id": reaction.provider_chat_id,
                "provider_message_id": reaction.provider_message_id,
                "provider_actor_id": reaction.provider_actor_id,
                "reaction": reaction.reaction,
                "is_active": reaction.is_active,
                "observed_via": "fixture_reaction",
            });
            let result_payload = json!({
                "provider_chat_id": reaction.provider_chat_id,
                "provider_message_id": reaction.provider_message_id,
                "provider_actor_id": reaction.provider_actor_id,
                "reaction": reaction.reaction,
                "is_active": reaction.is_active,
                "provider_observed_at": reaction.observed_at,
                "observed_via": "fixture_reaction",
            });

            let updated = if expected_active == reaction.is_active {
                self.mark_provider_command_reconciled(
                    &command.command_id,
                    reaction.observed_at,
                    provider_state,
                    result_payload,
                )
                .await?
            } else {
                self.mark_provider_command_mismatch(
                    &command.command_id,
                    reaction.observed_at,
                    provider_state,
                    result_payload,
                    "Provider observed a different WhatsApp reaction state than requested",
                )
                .await?
            };
            reconciled.push(updated);
        }

        Ok(reconciled)
    }

    pub(super) async fn reconcile_fixture_media_commands(
        &self,
        media: &NewWhatsappWebMedia,
    ) -> Result<Vec<WhatsAppProviderCommand>, WhatsappWebError> {
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM whatsapp_provider_write_commands
            WHERE account_id = $1
              AND provider_chat_id = $2
              AND command_kind IN ('send_media', 'send_voice_note', 'download_media')
              AND status IN ('queued', 'retrying', 'executing')
              AND confirmation_decision IN ('confirmed', 'not_required')
              AND capability_state IN ('available', 'degraded')
              AND happened_at <= $3
            ORDER BY happened_at ASC
            "#,
        )
        .bind(&media.account_id)
        .bind(&media.provider_chat_id)
        .bind(media.observed_at + chrono::Duration::seconds(5))
        .fetch_all(self.pool())
        .await?;

        let mut reconciled = Vec::new();
        for row in rows {
            let command = row_to_whatsapp_provider_write_command(row)?;
            let should_reconcile = match command.command_kind.as_str() {
                "download_media" => {
                    command.provider_message_id.as_deref()
                        == Some(media.provider_message_id.as_str())
                        && command
                            .payload
                            .get("provider_attachment_id")
                            .and_then(Value::as_str)
                            .is_none_or(|value| value == media.provider_attachment_id)
                }
                "send_media" | "send_voice_note" => {
                    media.provider_message_id == format!("provider-message:{}", command.command_id)
                        || command
                            .payload
                            .get("blob_id")
                            .and_then(Value::as_str)
                            .is_some_and(|value| value == media.storage_path)
                        || provider_request_id_matches_observed_media(&command, media)
                }
                _ => false,
            };
            if !should_reconcile {
                continue;
            }

            let provider_state = json!({
                "provider_chat_id": media.provider_chat_id,
                "provider_message_id": media.provider_message_id,
                "provider_attachment_id": media.provider_attachment_id,
                "filename": media.filename,
                "content_type": media.content_type,
                "storage_path": media.storage_path,
                "sha256": media.sha256,
                "observed_via": "fixture_media",
            });
            let result_payload = json!({
                "provider_chat_id": media.provider_chat_id,
                "provider_message_id": media.provider_message_id,
                "provider_attachment_id": media.provider_attachment_id,
                "filename": media.filename,
                "content_type": media.content_type,
                "storage_path": media.storage_path,
                "sha256": media.sha256,
                "provider_observed_at": media.observed_at,
                "observed_via": "fixture_media",
            });
            reconciled.push(
                self.mark_provider_command_reconciled(
                    &command.command_id,
                    media.observed_at,
                    provider_state,
                    result_payload,
                )
                .await?,
            );
        }

        Ok(reconciled)
    }

    pub(super) async fn reconcile_fixture_dialog_commands(
        &self,
        dialog: &NewWhatsappWebDialog,
    ) -> Result<Vec<WhatsAppProviderCommand>, WhatsappWebError> {
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM whatsapp_provider_write_commands
            WHERE account_id = $1
              AND provider_chat_id = $2
              AND provider_message_id IS NULL
              AND command_kind IN (
                  'archive', 'unarchive',
                  'pin', 'unpin',
                  'mute', 'unmute',
                  'mark_read', 'mark_unread'
              )
              AND status IN ('queued', 'retrying', 'executing')
              AND confirmation_decision IN ('confirmed', 'not_required')
              AND capability_state IN ('available', 'degraded')
              AND happened_at <= $3
            ORDER BY happened_at ASC
            "#,
        )
        .bind(&dialog.account_id)
        .bind(&dialog.provider_chat_id)
        .bind(dialog.observed_at + chrono::Duration::seconds(5))
        .fetch_all(self.pool())
        .await?;

        let mut reconciled = Vec::new();
        let executor_command_id = dialog
            .import_batch_id
            .strip_prefix("whatsapp-command:")
            .and_then(|value| value.rsplit_once(':'))
            .map(|(_, command_id)| command_id);
        for row in rows {
            let command = row_to_whatsapp_provider_write_command(row)?;
            if executor_command_id.is_some_and(|command_id| command_id != command.command_id) {
                continue;
            }
            let (observed_state, state_key) = match command.command_kind.as_str() {
                "archive" | "unarchive" => match dialog.is_archived {
                    Some(state) => (state, "is_archived"),
                    None => continue,
                },
                "pin" | "unpin" => match dialog.is_pinned {
                    Some(state) => (state, "is_pinned"),
                    None => continue,
                },
                "mute" | "unmute" => match dialog.is_muted {
                    Some(state) => (state, "is_muted"),
                    None => continue,
                },
                "mark_read" | "mark_unread" => match dialog.is_unread {
                    Some(state) => (state, "is_unread"),
                    None => continue,
                },
                _ => continue,
            };
            let expected_state = match command.command_kind.as_str() {
                "archive" | "pin" | "mute" | "mark_unread" => true,
                "unarchive" | "unpin" | "unmute" | "mark_read" => false,
                _ => continue,
            };

            let provider_state = json!({
                "provider_chat_id": dialog.provider_chat_id,
                "chat_kind": dialog.chat_kind,
                "chat_title": dialog.chat_title,
                state_key: observed_state,
                "observed_via": "fixture_dialog",
            });
            let result_payload = json!({
                "provider_chat_id": dialog.provider_chat_id,
                "chat_kind": dialog.chat_kind,
                "chat_title": dialog.chat_title,
                state_key: observed_state,
                "provider_observed_at": dialog.observed_at,
                "observed_via": "fixture_dialog",
            });

            let updated = if observed_state == expected_state {
                self.mark_provider_command_reconciled(
                    &command.command_id,
                    dialog.observed_at,
                    provider_state,
                    result_payload,
                )
                .await?
            } else {
                self.mark_provider_command_mismatch(
                    &command.command_id,
                    dialog.observed_at,
                    provider_state,
                    result_payload,
                    "Provider observed a different WhatsApp dialog state than requested",
                )
                .await?
            };
            reconciled.push(updated);
        }

        Ok(reconciled)
    }

    pub(super) async fn reconcile_fixture_participant_commands(
        &self,
        participant: &NewWhatsappWebParticipant,
    ) -> Result<Vec<WhatsAppProviderCommand>, WhatsappWebError> {
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM whatsapp_provider_write_commands
            WHERE account_id = $1
              AND provider_chat_id = $2
              AND provider_message_id IS NULL
              AND command_kind IN ('join_group', 'leave_group')
              AND status IN ('queued', 'retrying', 'executing')
              AND confirmation_decision IN ('confirmed', 'not_required', 'pending')
              AND capability_state IN ('available', 'degraded')
              AND happened_at <= $3
            ORDER BY happened_at ASC
            "#,
        )
        .bind(&participant.account_id)
        .bind(&participant.provider_chat_id)
        .bind(participant.observed_at + chrono::Duration::seconds(5))
        .fetch_all(self.pool())
        .await?;

        let mut reconciled = Vec::new();
        for row in rows {
            let command = row_to_whatsapp_provider_write_command(row)?;
            let inferred_self_participant = participant.provider_member_id.trim().is_empty();
            if !participant.is_self && !inferred_self_participant {
                continue;
            }

            let observed_membership_matches = match command.command_kind.as_str() {
                "join_group" => matches!(participant.status.as_str(), "member" | "joined"),
                "leave_group" => participant.status == "left",
                _ => continue,
            };
            let provider_member_id = participant.effective_provider_member_id();
            let provider_state = json!({
                "provider_chat_id": participant.provider_chat_id,
                "provider_member_id": provider_member_id,
                "provider_identity_id": participant.provider_identity_id,
                "chat_kind": participant.effective_chat_kind(),
                "chat_title": participant.effective_chat_title(),
                "role": participant.role,
                "status": participant.status,
                "is_self": participant.is_self,
                "observed_via": "fixture_participant",
            });
            let result_payload = json!({
                "provider_chat_id": participant.provider_chat_id,
                "provider_member_id": provider_member_id,
                "provider_identity_id": participant.provider_identity_id,
                "chat_kind": participant.effective_chat_kind(),
                "chat_title": participant.effective_chat_title(),
                "role": participant.role,
                "status": participant.status,
                "is_self": participant.is_self,
                "provider_observed_at": participant.observed_at,
                "observed_via": "fixture_participant",
            });

            let updated = if observed_membership_matches {
                self.mark_provider_command_reconciled(
                    &command.command_id,
                    participant.observed_at,
                    provider_state,
                    result_payload,
                )
                .await?
            } else {
                self.mark_provider_command_mismatch(
                    &command.command_id,
                    participant.observed_at,
                    provider_state,
                    result_payload,
                    "Provider observed a different WhatsApp group membership state than requested",
                )
                .await?
            };
            reconciled.push(updated);
        }

        Ok(reconciled)
    }

    pub(super) async fn reconcile_fixture_status_commands(
        &self,
        status: &NewWhatsappWebStatus,
    ) -> Result<Vec<WhatsAppProviderCommand>, WhatsappWebError> {
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM whatsapp_provider_write_commands
            WHERE account_id = $1
              AND provider_chat_id = 'status-feed'
              AND command_kind = 'publish_status'
              AND status IN ('queued', 'retrying', 'executing')
              AND confirmation_decision IN ('confirmed', 'not_required')
              AND capability_state IN ('available', 'degraded')
              AND happened_at <= $2
            ORDER BY happened_at ASC
            "#,
        )
        .bind(&status.account_id)
        .bind(status.occurred_at + chrono::Duration::seconds(5))
        .fetch_all(self.pool())
        .await?;

        let mut reconciled = Vec::new();
        for row in rows {
            let command = row_to_whatsapp_provider_write_command(row)?;
            let Some(expected_text) = command.payload.get("text").and_then(Value::as_str) else {
                continue;
            };
            if expected_text != status.text {
                continue;
            }

            let provider_state = json!({
                "provider_status_id": status.provider_status_id,
                "sender_id": status.sender_id,
                "sender_display_name": status.sender_display_name,
                "text": status.text,
                "observed_via": "fixture_status",
            });
            let result_payload = json!({
                "provider_status_id": status.provider_status_id,
                "sender_id": status.sender_id,
                "sender_display_name": status.sender_display_name,
                "provider_observed_at": status.occurred_at,
                "observed_via": "fixture_status",
            });
            reconciled.push(
                self.mark_provider_command_reconciled(
                    &command.command_id,
                    status.occurred_at,
                    provider_state,
                    result_payload,
                )
                .await?,
            );
        }

        Ok(reconciled)
    }

    pub(super) async fn reconcile_fixture_message_update_commands(
        &self,
        update: &NewWhatsappWebMessageUpdate,
    ) -> Result<Vec<WhatsAppProviderCommand>, WhatsappWebError> {
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM whatsapp_provider_write_commands
            WHERE account_id = $1
              AND provider_chat_id = $2
              AND provider_message_id = $3
              AND command_kind = 'edit'
              AND status IN ('queued', 'retrying', 'executing')
              AND confirmation_decision IN ('confirmed', 'not_required')
              AND capability_state IN ('available', 'degraded')
              AND happened_at <= $4
            ORDER BY happened_at ASC
            "#,
        )
        .bind(&update.account_id)
        .bind(&update.provider_chat_id)
        .bind(&update.provider_message_id)
        .bind(update.observed_at + chrono::Duration::seconds(5))
        .fetch_all(self.pool())
        .await?;

        let mut reconciled = Vec::new();
        for row in rows {
            let command = row_to_whatsapp_provider_write_command(row)?;
            let Some(expected_text) = command.payload.get("text").and_then(Value::as_str) else {
                continue;
            };
            let provider_state = json!({
                "provider_chat_id": update.provider_chat_id,
                "provider_message_id": update.provider_message_id,
                "text": update.text,
                "edited": true,
                "observed_via": "fixture_message_update",
            });
            let result_payload = json!({
                "provider_chat_id": update.provider_chat_id,
                "provider_message_id": update.provider_message_id,
                "text": update.text,
                "edited": true,
                "provider_observed_at": update.observed_at,
                "observed_via": "fixture_message_update",
            });
            let updated = if expected_text == update.text {
                self.mark_provider_command_reconciled(
                    &command.command_id,
                    update.observed_at,
                    provider_state,
                    result_payload,
                )
                .await?
            } else {
                self.mark_provider_command_mismatch(
                    &command.command_id,
                    update.observed_at,
                    provider_state,
                    result_payload,
                    "Provider observed different WhatsApp edited message content than requested",
                )
                .await?
            };
            reconciled.push(updated);
        }

        Ok(reconciled)
    }

    pub(super) async fn reconcile_fixture_message_delete_commands(
        &self,
        delete: &NewWhatsappWebMessageDelete,
    ) -> Result<Vec<WhatsAppProviderCommand>, WhatsappWebError> {
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM whatsapp_provider_write_commands
            WHERE account_id = $1
              AND provider_chat_id = $2
              AND provider_message_id = $3
              AND command_kind = 'delete'
              AND status IN ('queued', 'retrying', 'executing')
              AND confirmation_decision IN ('confirmed', 'not_required')
              AND capability_state IN ('available', 'degraded')
              AND happened_at <= $4
            ORDER BY happened_at ASC
            "#,
        )
        .bind(&delete.account_id)
        .bind(&delete.provider_chat_id)
        .bind(&delete.provider_message_id)
        .bind(delete.observed_at + chrono::Duration::seconds(5))
        .fetch_all(self.pool())
        .await?;

        let mut reconciled = Vec::new();
        for row in rows {
            let command = row_to_whatsapp_provider_write_command(row)?;
            let provider_state = json!({
                "provider_chat_id": delete.provider_chat_id,
                "provider_message_id": delete.provider_message_id,
                "reason_class": delete.reason_class,
                "actor_class": delete.actor_class,
                "deleted": true,
                "observed_via": "fixture_message_delete",
            });
            let result_payload = json!({
                "provider_chat_id": delete.provider_chat_id,
                "provider_message_id": delete.provider_message_id,
                "reason_class": delete.reason_class,
                "actor_class": delete.actor_class,
                "deleted": true,
                "provider_observed_at": delete.observed_at,
                "observed_via": "fixture_message_delete",
            });
            reconciled.push(
                self.mark_provider_command_reconciled(
                    &command.command_id,
                    delete.observed_at,
                    provider_state,
                    result_payload,
                )
                .await?,
            );
        }

        Ok(reconciled)
    }

    pub(super) async fn mark_provider_command_reconciled(
        &self,
        command_id: &str,
        observed_at: DateTime<Utc>,
        provider_state: Value,
        result_payload: Value,
    ) -> Result<WhatsAppProviderCommand, WhatsappWebError> {
        let resolved_provider_message_id =
            provider_message_id_from_state(&provider_state, &result_payload);
        let row = sqlx::query(
            r#"
            UPDATE whatsapp_provider_write_commands
            SET status = 'completed',
                result_payload = $3,
                last_error = NULL,
                provider_observed_at = $2,
                provider_state = $4,
                provider_message_id = COALESCE($5, provider_message_id),
                reconciliation_status = 'observed',
                reconciled_at = $2,
                completed_at = $2,
                locked_at = NULL,
                locked_by = NULL,
                next_attempt_at = NULL,
                dead_lettered_at = NULL,
                updated_at = $2
            WHERE command_id = $1
            RETURNING *
            "#,
        )
        .bind(command_id)
        .bind(observed_at)
        .bind(&result_payload)
        .bind(&provider_state)
        .bind(resolved_provider_message_id)
        .fetch_one(self.pool())
        .await?;
        let command = row_to_whatsapp_provider_write_command(row)?;
        self.mirror_canonical_provider_command(&command).await?;
        Ok(command.into())
    }

    pub(super) async fn mark_provider_command_mismatch(
        &self,
        command_id: &str,
        observed_at: DateTime<Utc>,
        provider_state: Value,
        result_payload: Value,
        error_message: &str,
    ) -> Result<WhatsAppProviderCommand, WhatsappWebError> {
        let resolved_provider_message_id =
            provider_message_id_from_state(&provider_state, &result_payload);
        let row = sqlx::query(
            r#"
            UPDATE whatsapp_provider_write_commands
            SET status = 'failed',
                result_payload = $3,
                last_error = $4,
                provider_observed_at = $2,
                provider_state = $5,
                provider_message_id = COALESCE($6, provider_message_id),
                reconciliation_status = 'mismatch',
                reconciled_at = $2,
                completed_at = NULL,
                locked_at = NULL,
                locked_by = NULL,
                next_attempt_at = NULL,
                dead_lettered_at = NULL,
                updated_at = $2
            WHERE command_id = $1
            RETURNING *
            "#,
        )
        .bind(command_id)
        .bind(observed_at)
        .bind(&result_payload)
        .bind(error_message)
        .bind(&provider_state)
        .bind(resolved_provider_message_id)
        .fetch_one(self.pool())
        .await?;
        let command = row_to_whatsapp_provider_write_command(row)?;
        self.mirror_canonical_provider_command(&command).await?;
        Ok(command.into())
    }
}
