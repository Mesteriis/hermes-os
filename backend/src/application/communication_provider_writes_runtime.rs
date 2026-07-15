use super::*;

impl TelegramMessageWriteApplicationService {
    pub(crate) fn new(
        store: TelegramStore,
        canonical_message_reads: Arc<dyn CanonicalMessageReadPort>,
        audit_log: ApiAuditLog,
        event_store: EventStore,
        event_bus: InMemoryEventBus,
    ) -> Self {
        Self {
            store,
            canonical_message_reads,
            audit_log,
            event_store,
            event_bus,
        }
    }

    pub(crate) async fn send_manual_message(
        &self,
        context: &TelegramRuntimeUseCaseContext<'_>,
        request: &TelegramManualSendRequest,
    ) -> Result<TelegramManualSendResponse, TelegramMessageWriteError> {
        let mut response = telegram_runtime::send_manual_message(context, request).await?;
        self.finish_created_message(&mut response, &request.command_id, "send_text", json!({}))
            .await?;
        Ok(response)
    }

    pub(crate) async fn send_reply_message(
        &self,
        context: &TelegramRuntimeUseCaseContext<'_>,
        request: &TelegramReplyRequest,
    ) -> Result<TelegramManualSendResponse, TelegramMessageWriteError> {
        let mut response = telegram_runtime::send_reply_message(context, request).await?;
        self.finish_created_message(
            &mut response,
            &request.command_id,
            "reply",
            json!({
                "reply_to_provider_message_id": &request.reply_to_provider_message_id,
            }),
        )
        .await?;
        Ok(response)
    }

    pub(crate) async fn send_forward_message(
        &self,
        context: &TelegramRuntimeUseCaseContext<'_>,
        request: &TelegramForwardRequest,
    ) -> Result<TelegramManualSendResponse, TelegramMessageWriteError> {
        let mut response = telegram_runtime::send_forward_message(context, request).await?;
        self.finish_created_message(
            &mut response,
            &request.command_id,
            "forward",
            json!({
                "from_provider_chat_id": &request.from_provider_chat_id,
                "from_provider_message_id": &request.from_provider_message_id,
            }),
        )
        .await?;
        Ok(response)
    }

    pub(crate) async fn send_conversation_message(
        &self,
        context: &TelegramRuntimeUseCaseContext<'_>,
        conversation_id: &str,
        request: CommunicationConversationMessageRequest,
    ) -> Result<TelegramManualSendResponse, TelegramMessageWriteError> {
        let command = TelegramManualSendRequest {
            command_id: request.command_id.unwrap_or_else(commands::new_command_id),
            account_id: request.account_id,
            provider_chat_id: conversation_id.trim().to_owned(),
            text: request.text,
        };
        command.validate()?;
        self.send_manual_message(context, &command).await
    }

    pub(crate) async fn reply_to_message(
        &self,
        context: &TelegramRuntimeUseCaseContext<'_>,
        message_id: &str,
        request: CommunicationReplyRequest,
    ) -> Result<TelegramManualSendResponse, TelegramMessageWriteError> {
        let message = self.telegram_message(message_id).await?;
        let provider_chat_id = required_provider_chat_id(&message)?;
        let command = TelegramReplyRequest {
            command_id: request.command_id.unwrap_or_else(commands::new_command_id),
            account_id: message.account_id.clone(),
            provider_chat_id,
            reply_to_provider_message_id: message.provider_message_id.clone(),
            text: request.text,
        };
        command.validate()?;
        self.send_reply_message(context, &command).await
    }

    pub(crate) async fn forward_message(
        &self,
        context: &TelegramRuntimeUseCaseContext<'_>,
        message_id: &str,
        request: CommunicationForwardRequest,
    ) -> Result<TelegramManualSendResponse, TelegramMessageWriteError> {
        let message = self.telegram_message(message_id).await?;
        let from_provider_chat_id = required_provider_chat_id(&message)?;
        let command = TelegramForwardRequest {
            command_id: request.command_id.unwrap_or_else(commands::new_command_id),
            account_id: message.account_id.clone(),
            provider_chat_id: request.conversation_id.trim().to_owned(),
            from_provider_chat_id,
            from_provider_message_id: message.provider_message_id.clone(),
        };
        command.validate()?;
        self.send_forward_message(context, &command).await
    }

    pub(crate) async fn pin_message(
        &self,
        message_id: &str,
        request: &TelegramPinRequest,
    ) -> Result<TelegramLifecycleResponse, TelegramMessageWriteError> {
        request.validate()?;
        let response =
            operations::record_pin_state(&self.store, request, message_id, AUDIT_ACTOR_ID).await?;

        self.audit_log
            .record(&NewApiAuditRecord::telegram_message_pin(
                AUDIT_ACTOR_ID,
                message_id,
                &request.account_id,
                &request.provider_chat_id,
                request.is_pinned,
            ))
            .await?;

        let event = build_event(
            telegram_event_types::MESSAGE_UPDATED,
            &request.account_id,
            message_id,
            telegram_message_snapshot_payload(
                &self.store,
                message_id,
                json!({
                    "provider_chat_id": &request.provider_chat_id,
                    "provider_message_id": &request.provider_message_id,
                    "is_pinned": request.is_pinned,
                    "status": &response.status,
                }),
            )
            .await?,
        );
        self.publish_event(event).await;

        let command_event = build_command_event(CommandEventInput {
            account_id: &request.account_id,
            command_id: &request.command_id,
            command_kind: "pin",
            provider_chat_id: &request.provider_chat_id,
            message_id: Some(message_id),
            provider_message_id: Some(&request.provider_message_id),
            status: "queued",
            extra_payload: json!({
                "telegram_message_id": message_id,
                "is_pinned": request.is_pinned,
            }),
        });
        self.publish_event(command_event).await;

        Ok(response)
    }

    pub(crate) async fn edit_message(
        &self,
        message_id: &str,
        request: &TelegramEditRequest,
    ) -> Result<TelegramLifecycleResponse, TelegramMessageWriteError> {
        request.validate()?;
        let response =
            operations::record_edit(&self.store, request, message_id, AUDIT_ACTOR_ID).await?;

        self.audit_log
            .record(&NewApiAuditRecord::telegram_message_edit(
                AUDIT_ACTOR_ID,
                message_id,
                &request.account_id,
                &request.provider_chat_id,
            ))
            .await?;

        let event = build_event(
            telegram_event_types::MESSAGE_UPDATED,
            &request.account_id,
            message_id,
            telegram_message_snapshot_payload(
                &self.store,
                message_id,
                json!({
                    "provider_chat_id": &request.provider_chat_id,
                    "provider_message_id": &request.provider_message_id,
                    "version_number": response.version_number,
                }),
            )
            .await?,
        );
        self.publish_event(event).await;

        let command_event = build_command_event(CommandEventInput {
            account_id: &request.account_id,
            command_id: &request.command_id,
            command_kind: "edit",
            provider_chat_id: &request.provider_chat_id,
            message_id: Some(message_id),
            provider_message_id: Some(&request.provider_message_id),
            status: "queued",
            extra_payload: json!({
                "telegram_message_id": message_id,
                "new_text": &request.new_text,
            }),
        });
        self.publish_event(command_event).await;

        Ok(response)
    }

    pub(crate) async fn delete_message(
        &self,
        message_id: &str,
        request: &TelegramDeleteRequest,
    ) -> Result<TelegramLifecycleResponse, TelegramMessageWriteError> {
        request.validate()?;
        let response =
            operations::record_delete(self.store.pool(), request, message_id, AUDIT_ACTOR_ID)
                .await?;

        self.audit_log
            .record(&NewApiAuditRecord::telegram_message_delete(
                AUDIT_ACTOR_ID,
                message_id,
                &request.account_id,
                &request.provider_chat_id,
            ))
            .await?;

        let event = build_event(
            telegram_event_types::MESSAGE_DELETED,
            &request.account_id,
            message_id,
            telegram_message_snapshot_payload(
                &self.store,
                message_id,
                json!({
                    "provider_chat_id": &request.provider_chat_id,
                    "provider_message_id": &request.provider_message_id,
                    "reason_class": &request.reason_class,
                    "tombstone_id": &response.tombstone_id,
                }),
            )
            .await?,
        );
        self.publish_event(event).await;

        let command_event = build_command_event(CommandEventInput {
            account_id: &request.account_id,
            command_id: &request.command_id,
            command_kind: "delete",
            provider_chat_id: &request.provider_chat_id,
            message_id: Some(message_id),
            provider_message_id: Some(&request.provider_message_id),
            status: "queued",
            extra_payload: json!({
                "telegram_message_id": message_id,
                "reason_class": &request.reason_class,
                "actor_class": &request.actor_class,
                "is_provider_delete": request.is_provider_delete,
                "tombstone_id": &response.tombstone_id,
            }),
        });
        self.publish_event(command_event).await;

        Ok(response)
    }

    pub(crate) async fn restore_message_visibility(
        &self,
        message_id: &str,
        request: &TelegramRestoreVisibilityRequest,
    ) -> Result<TelegramLifecycleResponse, TelegramMessageWriteError> {
        request.validate()?;
        let response = operations::record_restore_visibility(
            self.store.pool(),
            request,
            message_id,
            AUDIT_ACTOR_ID,
        )
        .await?;

        self.audit_log
            .record(&NewApiAuditRecord::telegram_message_restore_visibility(
                AUDIT_ACTOR_ID,
                message_id,
                &request.account_id,
                &request.provider_chat_id,
            ))
            .await?;

        let event = build_event(
            telegram_event_types::MESSAGE_VISIBILITY_RESTORED,
            &request.account_id,
            message_id,
            telegram_message_snapshot_payload(
                &self.store,
                message_id,
                json!({
                    "provider_chat_id": &request.provider_chat_id,
                    "provider_message_id": &request.provider_message_id,
                    "tombstone_id": &response.tombstone_id,
                }),
            )
            .await?,
        );
        self.publish_event(event).await;

        let command_event = build_command_event(CommandEventInput {
            account_id: &request.account_id,
            command_id: &request.command_id,
            command_kind: "restore_visibility",
            provider_chat_id: &request.provider_chat_id,
            message_id: Some(message_id),
            provider_message_id: Some(&request.provider_message_id),
            status: "queued",
            extra_payload: json!({
                "telegram_message_id": message_id,
                "reason": &request.reason,
                "tombstone_id": &response.tombstone_id,
            }),
        });
        self.publish_event(command_event).await;

        Ok(response)
    }

    pub(crate) async fn add_reaction(
        &self,
        message_id: &str,
        mut request: TelegramReactionRequest,
    ) -> Result<TelegramReactionResponse, TelegramMessageWriteError> {
        request.validate()?;
        let command_id = request
            .command_id
            .clone()
            .unwrap_or_else(commands::new_command_id);
        request.command_id = Some(command_id.clone());
        let response = reactions::add_reaction(self.store.pool(), &request, message_id).await?;
        self.finish_reaction(message_id, &request, &command_id, true, "react")
            .await?;
        Ok(response)
    }

    pub(crate) async fn remove_reaction(
        &self,
        message_id: &str,
        mut request: TelegramReactionRequest,
    ) -> Result<TelegramReactionResponse, TelegramMessageWriteError> {
        request.validate()?;
        let command_id = request
            .command_id
            .clone()
            .unwrap_or_else(commands::new_command_id);
        request.command_id = Some(command_id.clone());
        let response = reactions::remove_reaction(self.store.pool(), &request, message_id).await?;
        self.finish_reaction(message_id, &request, &command_id, false, "unreact")
            .await?;
        Ok(response)
    }

    pub(crate) async fn mark_message_read(
        &self,
        message_id: &str,
        request: &TelegramMessageMarkReadRequest,
    ) -> Result<TelegramMessageMarkReadResponse, TelegramMessageWriteError> {
        let message = self.telegram_message(message_id).await?;
        let provider_chat_id = required_provider_chat_id(&message)?;
        if message.account_id != request.account_id {
            return Err(TelegramError::InvalidRequest(
                "message account_id does not match mark-read request".to_owned(),
            )
            .into());
        }
        if provider_chat_id != request.provider_chat_id {
            return Err(TelegramError::InvalidRequest(
                "message provider_chat_id does not match mark-read request".to_owned(),
            )
            .into());
        }

        let chat = self
            .store
            .telegram_chat(&message.account_id, &provider_chat_id)
            .await?
            .ok_or_else(|| {
                TelegramError::InvalidRequest(format!(
                    "Telegram chat projection for message `{message_id}` was not found"
                ))
            })?;

        self.store
            .set_chat_last_read_at(&chat.telegram_chat_id, Some(Utc::now()))
            .await?;
        self.store
            .apply_provider_unread_counts(
                &chat.telegram_chat_id,
                None,
                None,
                Some(&message.provider_message_id),
                "api.telegram.message.mark_read",
            )
            .await?;
        let metadata = self
            .store
            .recompute_chat_unread_count(&chat.telegram_chat_id)
            .await?;

        let command_id = commands::new_command_id();
        commands::insert_command(
            self.store.pool(),
            &command_id,
            &request.account_id,
            "mark_read",
            &format!(
                "mark_read:{}:{}",
                message.message_id,
                Utc::now().timestamp_millis()
            ),
            &provider_chat_id,
            Some(&message.provider_message_id),
            "available",
            "provider_write",
            "confirmed",
            AUDIT_ACTOR_ID,
            json!({
                "source": "telegram_message_mark_read",
                "message_id": &message.message_id,
                "last_read_inbox_provider_message_id": &message.provider_message_id,
            }),
            json!({
                "message_id": &message.message_id,
                "telegram_chat_id": &chat.telegram_chat_id,
                "provider_chat_id": &provider_chat_id,
                "provider_message_id": &message.provider_message_id,
            }),
            json!({
                "source": "telegram_message_mark_read",
                "message_id": &message.message_id,
                "last_read_inbox_provider_message_id": &message.provider_message_id,
            }),
        )
        .await?;

        self.audit_log
            .record(&NewApiAuditRecord::telegram_message_mark_read(
                AUDIT_ACTOR_ID,
                &message.message_id,
                &request.account_id,
                &provider_chat_id,
                &message.provider_message_id,
            ))
            .await?;

        let command_event = build_event(
            telegram_event_types::COMMAND_STATUS_CHANGED,
            &request.account_id,
            &command_id,
            json!({
                "command_id": &command_id,
                "command_kind": "mark_read",
                "action": "mark_read",
                "provider_chat_id": &provider_chat_id,
                "telegram_chat_id": &chat.telegram_chat_id,
                "message_id": &message.provider_message_id,
                "status": "queued",
                "chat": &chat,
            }),
        );
        self.publish_event(command_event).await;

        let refreshed_chat = self
            .store
            .telegram_chat_by_id(&chat.telegram_chat_id)
            .await?;
        let chat_updated_event = build_event(
            telegram_event_types::CHAT_UPDATED,
            &request.account_id,
            &chat.telegram_chat_id,
            json!({
                "provider_chat_id": &provider_chat_id,
                "telegram_chat_id": &chat.telegram_chat_id,
                "action": "mark_read",
                "chat": refreshed_chat,
            }),
        );
        self.publish_event(chat_updated_event).await;

        Ok(TelegramMessageMarkReadResponse {
            telegram_chat_id: chat.telegram_chat_id,
            action: "mark_read".to_owned(),
            status: "read".to_owned(),
            metadata,
        })
    }

    pub(crate) async fn message_versions(
        &self,
        message_id: &str,
    ) -> Result<TelegramMessageVersionListResponse, TelegramMessageWriteError> {
        let mut versions =
            message_versions::list_message_versions(self.store.pool(), message_id).await?;
        if versions.is_empty() {
            versions = self
                .canonical_message_reads
                .list_message_versions(message_id)
                .await
                .map_err(|error| TelegramError::InvalidRequest(error.to_string()))?
                .into_iter()
                .map(|record| TelegramMessageVersion {
                    version_id: record.version_id,
                    message_id: record.message_id,
                    account_id: record.account_id,
                    provider_message_id: record.provider_message_id,
                    provider_chat_id: record.provider_chat_id,
                    version_number: record.version_number,
                    body_text: record.body_text,
                    edit_timestamp: record.edit_timestamp,
                    source_event: record.source_event,
                    raw_diff_payload: record.raw_diff_payload,
                    provenance: record.provenance,
                    created_at: record.created_at,
                })
                .collect();
        }
        Ok(TelegramMessageVersionListResponse {
            message_id: message_id.to_owned(),
            versions,
        })
    }

    pub(crate) async fn message_tombstones(
        &self,
        message_id: &str,
    ) -> Result<TelegramMessageTombstoneListResponse, TelegramMessageWriteError> {
        let mut tombstones = tombstones::list_tombstones(self.store.pool(), message_id).await?;
        if tombstones.is_empty() {
            tombstones = self
                .canonical_message_reads
                .list_message_tombstones(message_id)
                .await
                .map_err(|error| TelegramError::InvalidRequest(error.to_string()))?
                .into_iter()
                .map(|record| TelegramMessageTombstone {
                    tombstone_id: record.tombstone_id,
                    message_id: record.message_id,
                    account_id: record.account_id,
                    provider_message_id: record.provider_message_id,
                    provider_chat_id: record.provider_chat_id,
                    reason_class: record.reason_class,
                    actor_class: record.actor_class,
                    observed_at: record.observed_at,
                    source_event: record.source_event,
                    is_provider_delete: record.is_provider_delete,
                    is_local_visible: record.is_local_visible,
                    metadata: record.metadata,
                    provenance: record.provenance,
                    created_at: record.created_at,
                })
                .collect();
        }
        Ok(TelegramMessageTombstoneListResponse {
            message_id: message_id.to_owned(),
            tombstones,
        })
    }

    pub(crate) async fn reply_chain(
        &self,
        message_id: &str,
    ) -> Result<TelegramReplyChainResponse, TelegramMessageWriteError> {
        match references::reply_chain(&self.store, message_id).await {
            Ok(response) if !response.replies.is_empty() || !response.reply_to.is_empty() => {
                Ok(response)
            }
            Ok(_) => Ok(reply_chain(self.canonical_message_reads.as_ref(), message_id).await?),
            Err(_) => Ok(reply_chain(self.canonical_message_reads.as_ref(), message_id).await?),
        }
    }

    pub(crate) async fn forward_chain(
        &self,
        message_id: &str,
    ) -> Result<TelegramForwardChainResponse, TelegramMessageWriteError> {
        match references::forward_chain(&self.store, message_id).await {
            Ok(response) if !response.forwards.is_empty() => Ok(response),
            Ok(_) => Ok(forward_chain(self.canonical_message_reads.as_ref(), message_id).await?),
            Err(_) => Ok(forward_chain(self.canonical_message_reads.as_ref(), message_id).await?),
        }
    }

    pub(crate) async fn reactions(
        &self,
        message_id: &str,
    ) -> Result<TelegramReactionListResponse, TelegramMessageWriteError> {
        let mut reactions = reactions::list_reactions(self.store.pool(), message_id).await?;
        if reactions.is_empty() {
            reactions = self
                .canonical_message_reads
                .list_message_reactions(message_id)
                .await
                .map_err(|error| TelegramError::InvalidRequest(error.to_string()))?
                .into_iter()
                .map(|record| TelegramReaction {
                    reaction_id: record.reaction_id,
                    message_id: record.message_id,
                    account_id: record.account_id,
                    provider_message_id: record.provider_message_id,
                    provider_chat_id: record.provider_chat_id,
                    sender_id: record.sender_id,
                    sender_display_name: record.sender_display_name,
                    reaction_emoji: record.reaction_emoji,
                    is_active: record.is_active,
                    observed_at: record.observed_at,
                    source_event: record.source_event,
                    provider_actor_id: record.provider_actor_id,
                    metadata: record.metadata,
                    provenance: record.provenance,
                    created_at: record.created_at,
                    updated_at: record.updated_at,
                })
                .collect();
        }
        let summary = if reactions.is_empty() {
            reactions::reaction_summary(self.store.pool(), message_id).await?
        } else {
            canonical_reaction_summary(message_id, &reactions)
        };
        Ok(TelegramReactionListResponse {
            message_id: message_id.to_owned(),
            reactions,
            summary,
        })
    }

    async fn finish_created_message(
        &self,
        response: &mut TelegramManualSendResponse,
        command_id: &str,
        command_kind: &str,
        extra_payload: serde_json::Value,
    ) -> Result<(), TelegramMessageWriteError> {
        self.project_manual_send_response(response).await?;
        self.audit_log
            .record(&NewApiAuditRecord::telegram_message_send(
                AUDIT_ACTOR_ID,
                &response.message_id,
                &response.account_id,
                &response.provider_chat_id,
                &response.rendered_preview_hash,
            ))
            .await?;

        let event = build_event(
            telegram_event_types::MESSAGE_CREATED,
            &response.account_id,
            &response.message_id,
            telegram_message_snapshot_payload(
                &self.store,
                &response.message_id,
                merge_json_objects(
                    json!({
                        "provider_chat_id": &response.provider_chat_id,
                        "delivery_state": &response.delivery_state,
                        "runtime_kind": &response.runtime_kind,
                        "status": &response.status,
                    }),
                    extra_payload.clone(),
                ),
            )
            .await?,
        );
        self.publish_event(event).await;

        let command_event = build_command_event(CommandEventInput {
            account_id: &response.account_id,
            command_id,
            command_kind,
            provider_chat_id: &response.provider_chat_id,
            message_id: Some(&response.message_id),
            provider_message_id: None,
            status: &response.status,
            extra_payload: merge_json_objects(
                json!({
                    "telegram_message_id": &response.message_id,
                    "runtime_kind": &response.runtime_kind,
                }),
                extra_payload,
            ),
        });
        self.publish_event(command_event).await;
        Ok(())
    }

    async fn finish_reaction(
        &self,
        message_id: &str,
        request: &TelegramReactionRequest,
        command_id: &str,
        is_active: bool,
        command_kind: &str,
    ) -> Result<(), TelegramMessageWriteError> {
        self.audit_log
            .record(&NewApiAuditRecord::telegram_message_reaction(
                AUDIT_ACTOR_ID,
                message_id,
                &request.account_id,
                &request.provider_chat_id,
                &request.reaction_emoji,
                is_active,
            ))
            .await?;

        let event = build_event(
            telegram_event_types::REACTION_CHANGED,
            &request.account_id,
            message_id,
            json!({
                "provider_chat_id": &request.provider_chat_id,
                "reaction_emoji": &request.reaction_emoji,
                "is_active": is_active,
            }),
        );
        self.publish_event(event).await;

        let command_event = build_command_event(CommandEventInput {
            account_id: &request.account_id,
            command_id,
            command_kind,
            provider_chat_id: &request.provider_chat_id,
            message_id: Some(message_id),
            provider_message_id: Some(&request.provider_message_id),
            status: "queued",
            extra_payload: json!({
                "telegram_message_id": message_id,
                "reaction_emoji": &request.reaction_emoji,
                "sender_id": &request.sender_id,
                "sender_display_name": &request.sender_display_name,
                "is_active": is_active,
            }),
        });
        self.publish_event(command_event).await;
        Ok(())
    }

    async fn project_manual_send_response(
        &self,
        response: &mut TelegramManualSendResponse,
    ) -> Result<(), TelegramMessageWriteError> {
        let Some(raw) = response.raw.take() else {
            return Err(TelegramError::InvalidRequest(
                "Telegram send response is missing raw provider observation".to_owned(),
            )
            .into());
        };
        let stored_raw = CommunicationIngestionStore::new(self.store.pool().clone())
            .record_raw_source(&raw)
            .await?;
        let Some(accepted_event) =
            dispatch_telegram_raw_signal(self.store.pool().clone(), &stored_raw).await?
        else {
            return Err(TelegramMessageWriteError::SignalControlBlocked(
                "telegram manual send raw signal was not accepted by Signal Hub".to_owned(),
            ));
        };
        let Some(projected) =
            project_accepted_signal_if_runtime_allows(self.store.pool().clone(), &accepted_event)
                .await?
        else {
            return Err(TelegramMessageWriteError::SignalControlBlocked(
                "telegram manual send accepted signal did not produce a message projection"
                    .to_owned(),
            ));
        };
        response.raw_record_id = projected.raw_record_id;
        response.message_id = projected.message_id;
        Ok(())
    }

    async fn telegram_message(
        &self,
        message_id: &str,
    ) -> Result<
        crate::integrations::telegram::client::models::messages::TelegramMessage,
        TelegramMessageWriteError,
    > {
        self.store.message_by_id(message_id).await?.ok_or_else(|| {
            TelegramMessageWriteError::Telegram(TelegramError::InvalidRequest(format!(
                "Telegram message `{message_id}` was not found"
            )))
        })
    }

    async fn publish_event(&self, event: NewEventEnvelope) {
        if let Err(error) = self.event_store.append(&event).await {
            tracing::warn!(error = %error, "failed to append Telegram message write event");
        }
        let _ = self.event_bus.broadcast(event);
    }
}
