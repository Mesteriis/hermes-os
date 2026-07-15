use super::*;

impl WhatsappFixtureIngestApplicationService {
    pub(crate) async fn ingest_status(
        &self,
        request: &NewWhatsappWebStatus,
    ) -> Result<WhatsappWebStatusIngestResult, CommunicationFixtureIngestError> {
        self.ingest_status_with_reconciliation_source(request, "provider_observed.fixture_status")
            .await
    }

    pub(crate) async fn ingest_runtime_bridge_status(
        &self,
        request: &NewWhatsappWebStatus,
    ) -> Result<WhatsappWebStatusIngestResult, CommunicationFixtureIngestError> {
        self.ingest_status_with_reconciliation_source(
            request,
            "provider_observed.runtime_bridge_status",
        )
        .await
    }

    async fn ingest_status_with_reconciliation_source(
        &self,
        request: &NewWhatsappWebStatus,
        reconciliation_source: &str,
    ) -> Result<WhatsappWebStatusIngestResult, CommunicationFixtureIngestError> {
        let account_context = self
            .whatsapp_account_projection_context(&request.account_id)
            .await?;
        let observed = self.runtime.ingest_fixture_status(request).await?;
        let observed_raw =
            annotate_whatsapp_raw_observed_source(&observed.raw, reconciliation_source)?;
        let stored_raw = self.record_and_accept_whatsapp_raw(&observed_raw).await?;
        let channel_id = self.ensure_whatsapp_channel(&request.account_id).await?;
        let status_author_identity_id = self
            .upsert_whatsapp_status_identity(&request.account_id, request, &stored_raw)
            .await?;
        self.upsert_whatsapp_persona_identity_traces_for_status(request, &stored_raw)
            .await?;
        let status_feed_conversation_id = self
            .upsert_whatsapp_status_feed_conversation(
                &request.account_id,
                &channel_id,
                request,
                status_author_identity_id.as_deref(),
                &stored_raw,
            )
            .await?;
        let message = MessageProjectionPort::new(self.pool.clone())
            .upsert_channel_message(&NewProjectedMessage {
                message_id: whatsapp_status_message_id(
                    &request.account_id,
                    &request.provider_status_id,
                ),
                raw_record_id: stored_raw.raw_record_id.clone(),
                account_id: request.account_id.clone(),
                provider_record_id: request.provider_status_id.clone(),
                subject: "WhatsApp Status".to_owned(),
                sender: request.sender_id.clone(),
                recipients: vec![status_feed_conversation_id.clone()],
                body_text: request.text.clone(),
                occurred_at: Some(request.occurred_at),
                channel_kind: account_context.channel_kind.clone(),
                conversation_id: Some(status_feed_conversation_id),
                sender_display_name: Some(request.sender_display_name.clone()),
                delivery_state: "received".to_owned(),
                message_metadata: json!({
                    "communication_object_type": "status",
                    "provider_status_id": request.provider_status_id,
                    "accepted_signal_event_id": stored_raw.accepted_event_id,
                    "status_author_identity_id": status_author_identity_id,
                    "status_author_identity_kind": request.sender_identity_kind,
                    "status_author_address": request.sender_address,
                    "status_author_push_name": request.sender_push_name,
                    "status_author_business_profile": request.sender_business_profile,
                    "status_author_profile_photo_ref": request.sender_profile_photo_ref,
                }),
            })
            .await?;
        let message_ids = vec![message.message_id.clone()];
        refresh_message_decisions_into_review(&self.pool, &message_ids).await?;
        refresh_message_task_candidates_into_review(&self.pool, &message_ids).await?;
        if let Some(projected_message) = MessageProjectionPort::new(self.pool.clone())
            .message(&message.message_id)
            .await?
        {
            let _ = refresh_message_people_candidates_into_review(
                &self.pool,
                std::slice::from_ref(&projected_message),
            )
            .await?;
            let _ = refresh_message_knowledge_candidates_into_review(
                &self.pool,
                std::slice::from_ref(&projected_message),
            )
            .await?;
        }
        let reconciled_commands = self
            .runtime
            .reconcile_fixture_status_commands(request)
            .await?;
        self.publish_whatsapp_command_reconciled_events(
            reconciled_commands.clone(),
            reconciliation_source,
        )
        .await?;
        self.publish_whatsapp_status_runtime_events(&reconciled_commands, reconciliation_source)
            .await?;

        Ok(WhatsappWebStatusIngestResult {
            raw_record_id: stored_raw.raw_record_id,
            message_id: message.message_id,
        })
    }

    async fn upsert_whatsapp_status_feed_conversation(
        &self,
        account_id: &str,
        channel_id: &str,
        request: &NewWhatsappWebStatus,
        status_author_identity_id: Option<&str>,
        stored_raw: &AcceptedWhatsappRawRecord,
    ) -> Result<String, CommunicationFixtureIngestError> {
        let account_context = self.whatsapp_account_projection_context(account_id).await?;
        let conversation_id = whatsapp_status_feed_conversation_id(account_id);
        let metadata = json!({
            "provider": account_context.provider_kind,
            "chat_kind": "status_feed",
            "is_status_feed": true,
            "raw_record_id": stored_raw.raw_record_id,
            "accepted_signal_event_id": stored_raw.accepted_event_id,
            "provider_status_id": request.provider_status_id,
            "status_author_identity_id": status_author_identity_id,
            "status_author_identity_kind": request.sender_identity_kind,
            "status_author_address": request.sender_address,
            "status_author_push_name": request.sender_push_name,
            "status_author_business_profile": request.sender_business_profile,
            "status_author_profile_photo_ref": request.sender_profile_photo_ref,
        });
        crate::domains::communications::fixtures::whatsapp_projection::upsert_whatsapp_status_feed_conversation(
            &self.pool,
            &conversation_id,
            account_id,
            channel_id,
            &account_context.channel_kind,
            request.occurred_at,
            metadata,
        )
        .await?;
        Ok(conversation_id)
    }

    pub(crate) async fn ingest_status_view(
        &self,
        request: &NewWhatsappWebStatusView,
    ) -> Result<WhatsappWebStatusViewIngestResult, CommunicationFixtureIngestError> {
        self.ingest_status_view_with_observed_source(
            request,
            "provider_observed.fixture_status_view",
        )
        .await
    }

    pub(crate) async fn ingest_runtime_bridge_status_view(
        &self,
        request: &NewWhatsappWebStatusView,
    ) -> Result<WhatsappWebStatusViewIngestResult, CommunicationFixtureIngestError> {
        self.ingest_status_view_with_observed_source(
            request,
            "provider_observed.runtime_bridge_status_view",
        )
        .await
    }

    async fn ingest_status_view_with_observed_source(
        &self,
        request: &NewWhatsappWebStatusView,
        observed_source: &str,
    ) -> Result<WhatsappWebStatusViewIngestResult, CommunicationFixtureIngestError> {
        let observed = self.runtime.ingest_fixture_status_view(request).await?;
        let observed_raw = annotate_whatsapp_raw_observed_source(&observed.raw, observed_source)?;
        let stored_raw = self.record_and_accept_whatsapp_raw(&observed_raw).await?;
        let store = ProviderChannelMessagePort::new(self.pool.clone());
        let message = store
            .message_by_provider_record_id(
                &request.account_id,
                &request.provider_status_id,
                WHATSAPP_CHANNEL_KINDS,
            )
            .await?
            .ok_or_else(|| {
                CommunicationFixtureIngestError::SignalControlBlocked(format!(
                    "whatsapp status view target `{}` is not projected",
                    request.provider_status_id
                ))
            })?;
        let mut viewer_ids = message
            .message_metadata
            .get("status_viewer_ids")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if !viewer_ids
            .iter()
            .any(|value| value.as_str() == Some(request.viewer_id.as_str()))
        {
            viewer_ids.push(Value::String(request.viewer_id.clone()));
        }
        let updated_metadata = merged_object_metadata(
            &message.message_metadata,
            json!({
                "status_viewed": true,
                "status_last_viewed_at": request.observed_at,
                "status_view_count": viewer_ids.len(),
                "status_viewer_ids": viewer_ids,
                "status_last_viewer_id": request.viewer_id,
                "status_last_viewer_display_name": request.viewer_display_name,
                "status_view_observation_event_id": stored_raw.accepted_event_id,
            }),
        )?;
        let updated_message = MessageProjectionPort::new(self.pool.clone())
            .set_message_metadata_with_observation(
                &message.message_id,
                &updated_metadata,
                None,
                "status_view_observed",
                None,
            )
            .await?;

        Ok(WhatsappWebStatusViewIngestResult {
            raw_record_id: stored_raw.raw_record_id,
            message_id: updated_message.message_id,
        })
    }

    pub(crate) async fn ingest_status_delete(
        &self,
        request: &NewWhatsappWebStatusDelete,
    ) -> Result<WhatsappWebStatusDeleteIngestResult, CommunicationFixtureIngestError> {
        self.ingest_status_delete_with_observed_source(
            request,
            "provider_observed.fixture_status_delete",
        )
        .await
    }

    pub(crate) async fn ingest_runtime_bridge_status_delete(
        &self,
        request: &NewWhatsappWebStatusDelete,
    ) -> Result<WhatsappWebStatusDeleteIngestResult, CommunicationFixtureIngestError> {
        self.ingest_status_delete_with_observed_source(
            request,
            "provider_observed.runtime_bridge_status_delete",
        )
        .await
    }

    async fn ingest_status_delete_with_observed_source(
        &self,
        request: &NewWhatsappWebStatusDelete,
        observed_source: &str,
    ) -> Result<WhatsappWebStatusDeleteIngestResult, CommunicationFixtureIngestError> {
        let account_context = self
            .whatsapp_account_projection_context(&request.account_id)
            .await?;
        let observed = self.runtime.ingest_fixture_status_delete(request).await?;
        let observed_raw = annotate_whatsapp_raw_observed_source(&observed.raw, observed_source)?;
        let stored_raw = self.record_and_accept_whatsapp_raw(&observed_raw).await?;
        let store = ProviderChannelMessagePort::new(self.pool.clone());
        let message = store
            .message_by_provider_record_id(
                &request.account_id,
                &request.provider_status_id,
                WHATSAPP_CHANNEL_KINDS,
            )
            .await?
            .ok_or_else(|| {
                CommunicationFixtureIngestError::SignalControlBlocked(format!(
                    "whatsapp status delete target `{}` is not projected",
                    request.provider_status_id
                ))
            })?;
        let updated_metadata = merged_object_metadata(
            &message.message_metadata,
            json!({
                "status_deleted": true,
                "status_deleted_at": request.observed_at,
                "status_delete_actor_class": request.actor_class,
                "status_delete_reason_class": request.reason_class,
                "status_delete_observation_event_id": stored_raw.accepted_event_id,
            }),
        )?;
        MessageProjectionPort::new(self.pool.clone())
            .set_message_metadata_with_observation(
                &message.message_id,
                &updated_metadata,
                None,
                "status_delete_observed",
                None,
            )
            .await?;
        let tombstone_id = whatsapp_message_tombstone_id(&stored_raw.accepted_event_id);
        sqlx::query(
            r#"
            INSERT INTO communication_message_tombstones (
                tombstone_id, message_id, account_id, provider_message_id,
                provider_conversation_id, reason_class, actor_class, observed_at,
                source_event, is_provider_delete, is_local_visible, metadata, provenance
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, TRUE, FALSE, $10, $11)
            ON CONFLICT (tombstone_id) DO NOTHING
            "#,
        )
        .bind(&tombstone_id)
        .bind(&message.message_id)
        .bind(&message.account_id)
        .bind(&message.provider_record_id)
        .bind(&message.conversation_id)
        .bind(&request.reason_class)
        .bind(&request.actor_class)
        .bind(request.observed_at)
        .bind(&stored_raw.accepted_event_id)
        .bind(json!({
            "provider": account_context.provider_kind,
            "communication_object_type": "status",
            "provider_status_id": request.provider_status_id,
            "raw_record_id": stored_raw.raw_record_id,
            "accepted_signal_event_id": stored_raw.accepted_event_id,
        }))
        .bind(json!({
            "provider": account_context.provider_kind,
            "raw_record_id": stored_raw.raw_record_id,
            "accepted_signal_event_id": stored_raw.accepted_event_id,
        }))
        .execute(&self.pool)
        .await?;

        Ok(WhatsappWebStatusDeleteIngestResult {
            raw_record_id: stored_raw.raw_record_id,
            message_id: message.message_id,
            tombstone_id,
        })
    }

    pub(crate) async fn ingest_presence(
        &self,
        request: &NewWhatsappWebPresence,
    ) -> Result<WhatsappWebPresenceIngestResult, CommunicationFixtureIngestError> {
        self.ingest_presence_with_observed_source(request, "provider_observed.fixture_presence")
            .await
    }

    pub(crate) async fn ingest_runtime_bridge_presence(
        &self,
        request: &NewWhatsappWebPresence,
    ) -> Result<WhatsappWebPresenceIngestResult, CommunicationFixtureIngestError> {
        self.ingest_presence_with_observed_source(
            request,
            "provider_observed.runtime_bridge_presence",
        )
        .await
    }

    async fn ingest_presence_with_observed_source(
        &self,
        request: &NewWhatsappWebPresence,
        observed_source: &str,
    ) -> Result<WhatsappWebPresenceIngestResult, CommunicationFixtureIngestError> {
        let observed = self.runtime.ingest_fixture_presence(request).await?;
        let observed_raw = annotate_whatsapp_raw_observed_source(&observed.raw, observed_source)?;
        let stored_raw = self.record_and_accept_whatsapp_raw(&observed_raw).await?;
        let identity_row: Option<(String, Option<String>, Value)> = sqlx::query_as(
            r#"
            SELECT identity_id, display_name, metadata
            FROM communication_identities
            WHERE account_id = $1
              AND identity_kind = $2
              AND provider_identity_id = $3
            LIMIT 1
            "#,
        )
        .bind(&request.account_id)
        .bind(&request.identity_kind)
        .bind(&request.provider_identity_id)
        .fetch_optional(&self.pool)
        .await?;

        let identity_id =
            if let Some((identity_id, current_display_name, current_metadata)) = identity_row {
                let updated_metadata = merged_identity_display_name_metadata(
                    current_display_name.as_deref(),
                    &current_metadata,
                    Some(request.display_name.as_str()),
                    json!({
                        "presence_state": request.presence_state,
                        "presence_provider_chat_id": request.provider_chat_id,
                        "presence_observed_at": request.observed_at,
                        "last_seen_at": request.last_seen_at,
                        "presence_observation_event_id": stored_raw.accepted_event_id,
                    }),
                    request.observed_at,
                )?;
                sqlx::query(
                    r#"
                UPDATE communication_identities
                SET display_name = $2,
                    metadata = $3,
                    updated_at = now()
                WHERE identity_id = $1
                "#,
                )
                .bind(&identity_id)
                .bind(&request.display_name)
                .bind(updated_metadata)
                .execute(&self.pool)
                .await?;
                Some(identity_id)
            } else {
                None
            };

        Ok(WhatsappWebPresenceIngestResult {
            raw_record_id: stored_raw.raw_record_id,
            identity_id,
        })
    }

    pub(crate) async fn ingest_call(
        &self,
        request: &NewWhatsappWebCall,
    ) -> Result<WhatsappWebCallIngestResult, CommunicationFixtureIngestError> {
        self.ingest_call_with_observed_source(request, "provider_observed.fixture_call")
            .await
    }

    pub(crate) async fn ingest_runtime_bridge_call(
        &self,
        request: &NewWhatsappWebCall,
    ) -> Result<WhatsappWebCallIngestResult, CommunicationFixtureIngestError> {
        self.ingest_call_with_observed_source(request, "provider_observed.runtime_bridge_call")
            .await
    }

    async fn ingest_call_with_observed_source(
        &self,
        request: &NewWhatsappWebCall,
        observed_source: &str,
    ) -> Result<WhatsappWebCallIngestResult, CommunicationFixtureIngestError> {
        let account_context = self
            .whatsapp_account_projection_context(&request.account_id)
            .await?;
        let observed = self.runtime.ingest_fixture_call(request).await?;
        let observed_raw = annotate_whatsapp_raw_observed_source(&observed.raw, observed_source)?;
        let stored_raw = self.record_and_accept_whatsapp_raw(&observed_raw).await?;
        let call_id = whatsapp_call_id(&request.account_id, &request.provider_call_id);
        let direction =
            crate::application::whatsapp_fixture_policy::call_direction(&request.direction)?;
        let call_state =
            crate::application::whatsapp_fixture_policy::call_state(&request.call_state)?;
        let metadata = merged_object_metadata(
            &request.metadata,
            json!({
                "provider": account_context.provider_kind,
                "raw_record_id": stored_raw.raw_record_id,
                "accepted_signal_event_id": stored_raw.accepted_event_id,
                "observed_at": request.observed_at,
            }),
        )?;
        CallIntelligencePort::new(self.pool.clone())
            .upsert_call(&NewTelegramCall {
                call_id: call_id.clone(),
                account_id: request.account_id.clone(),
                provider_call_id: request.provider_call_id.clone(),
                provider_chat_id: request.provider_chat_id.clone(),
                direction,
                call_state,
                started_at: request.started_at,
                ended_at: request.ended_at,
                transcription_policy_id: None,
                metadata,
            })
            .await?;

        Ok(WhatsappWebCallIngestResult {
            raw_record_id: stored_raw.raw_record_id,
            call_id,
        })
    }

    pub(crate) async fn ingest_runtime_event(
        &self,
        request: &NewWhatsappWebRuntimeEvent,
    ) -> Result<WhatsappWebRuntimeEventIngestResult, CommunicationFixtureIngestError> {
        self.ingest_runtime_event_with_observed_source(
            request,
            "provider_observed.fixture_runtime_event",
        )
        .await
    }

    pub(crate) async fn ingest_runtime_bridge_runtime_event(
        &self,
        request: &NewWhatsappWebRuntimeEvent,
    ) -> Result<WhatsappWebRuntimeEventIngestResult, CommunicationFixtureIngestError> {
        self.ingest_runtime_event_with_observed_source(
            request,
            "provider_observed.runtime_bridge_runtime_event",
        )
        .await
    }

    async fn ingest_runtime_event_with_observed_source(
        &self,
        request: &NewWhatsappWebRuntimeEvent,
        observed_source: &str,
    ) -> Result<WhatsappWebRuntimeEventIngestResult, CommunicationFixtureIngestError> {
        let observed = self.runtime.ingest_fixture_runtime_event(request).await?;
        let observed_raw = annotate_whatsapp_raw_observed_source(&observed.raw, observed_source)?;
        let stored_raw = self.record_and_accept_whatsapp_raw(&observed_raw).await?;
        Ok(WhatsappWebRuntimeEventIngestResult {
            raw_record_id: stored_raw.raw_record_id,
            accepted_event_id: stored_raw.accepted_event_id,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn capture_runtime_lifecycle_event(
        &self,
        account_id: &str,
        provider_event_id: &str,
        runtime_event_kind: &str,
        runtime_status: Option<&str>,
        lifecycle_state: Option<&str>,
        severity: Option<&str>,
        metadata: Value,
        import_batch_id: &str,
        observed_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<WhatsappWebRuntimeEventIngestResult, CommunicationFixtureIngestError> {
        let account_context = self.whatsapp_account_projection_context(account_id).await?;
        let raw_record_id = whatsapp_runtime_event_raw_record_id(account_id, provider_event_id);
        let source_fingerprint = stable_whatsapp_id(
            "source_fingerprint:v5:whatsapp_web_runtime_event",
            &[
                account_id,
                provider_event_id,
                runtime_event_kind,
                runtime_status.unwrap_or(""),
                lifecycle_state.unwrap_or(""),
                severity.unwrap_or(""),
            ],
        );
        let observed_source = metadata
            .get("source")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .unwrap_or(import_batch_id)
            .to_owned();
        let raw = NewRawCommunicationRecord::new(
            &raw_record_id,
            account_id,
            "whatsapp_web_runtime_event",
            provider_event_id,
            source_fingerprint,
            import_batch_id,
            json!({
                "provider_event_id": provider_event_id,
                "runtime_event_kind": runtime_event_kind,
                "runtime_status": runtime_status,
                "lifecycle_state": lifecycle_state,
                "severity": severity,
                "metadata": crate::application::communication_fixture_metadata::redact_secret_material(metadata),
            }),
        )
        .occurred_at(observed_at)
        .provenance(json!({
            "provider": account_context.provider_kind,
            "provider_kind": account_context.provider_kind,
            "account_id": account_id,
            "observed_source": observed_source,
            "captured_by": "application.communication_fixture_ingest.whatsapp_runtime_lifecycle",
        }));
        let stored_raw = self.record_and_accept_whatsapp_raw(&raw).await?;
        Ok(WhatsappWebRuntimeEventIngestResult {
            raw_record_id: stored_raw.raw_record_id,
            accepted_event_id: stored_raw.accepted_event_id,
        })
    }

    pub(crate) async fn capture_media_lifecycle_event(
        &self,
        account_id: &str,
        command_id: &str,
        event_type: &str,
        metadata: Value,
        source: &str,
        observed_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<WhatsappWebRuntimeEventIngestResult, CommunicationFixtureIngestError> {
        let Some(runtime_event_kind) =
            crate::application::whatsapp_fixture_runtime_policy::media_runtime_event_kind(
                event_type,
            )
        else {
            return Err(CommunicationFixtureIngestError::SignalControlBlocked(
                format!("unsupported whatsapp media lifecycle event type `{event_type}`"),
            ));
        };
        let lifecycle_state =
            crate::application::whatsapp_fixture_runtime_policy::media_lifecycle_state(event_type);
        let has_runtime_blockers = metadata
            .get("runtime_blockers")
            .and_then(Value::as_array)
            .is_some_and(|blockers| !blockers.is_empty());
        let runtime_status = match lifecycle_state {
            "failed" if has_runtime_blockers => Some("blocked"),
            "failed" => Some("degraded"),
            _ => None,
        };
        let severity = match lifecycle_state {
            "failed" if has_runtime_blockers => Some("blocked"),
            "failed" => Some("warning"),
            _ => Some("info"),
        };
        let provider_event_id = format!(
            "{command_id}:{runtime_event_kind}:{}",
            observed_at.timestamp_micros()
        );
        self.capture_runtime_lifecycle_event(
            account_id,
            &provider_event_id,
            runtime_event_kind,
            runtime_status,
            Some(lifecycle_state),
            severity,
            merged_object_metadata(
                &metadata,
                json!({
                    "command_id": command_id,
                    "event_type": event_type,
                }),
            )?,
            source,
            observed_at,
        )
        .await
    }

    pub(crate) async fn ingest_dialog(
        &self,
        request: &NewWhatsappWebDialog,
    ) -> Result<WhatsappWebDialogIngestResult, CommunicationFixtureIngestError> {
        self.ingest_dialog_with_reconciliation_source(request, "provider_observed.fixture_dialog")
            .await
    }

    pub(crate) async fn ingest_runtime_bridge_dialog(
        &self,
        request: &NewWhatsappWebDialog,
    ) -> Result<WhatsappWebDialogIngestResult, CommunicationFixtureIngestError> {
        self.ingest_dialog_with_reconciliation_source(
            request,
            "provider_observed.runtime_bridge_dialog",
        )
        .await
    }

    async fn ingest_dialog_with_reconciliation_source(
        &self,
        request: &NewWhatsappWebDialog,
        reconciliation_source: &str,
    ) -> Result<WhatsappWebDialogIngestResult, CommunicationFixtureIngestError> {
        let observed = self.runtime.ingest_fixture_dialog(request).await?;
        let observed_raw =
            annotate_whatsapp_raw_observed_source(&observed.raw, reconciliation_source)?;
        let stored_raw = self.record_and_accept_whatsapp_raw(&observed_raw).await?;
        let channel_id = self.ensure_whatsapp_channel(&request.account_id).await?;
        let conversation_id = self
            .upsert_whatsapp_conversation(
                &request.account_id,
                &channel_id,
                &request.provider_chat_id,
                &request.chat_title,
                &request.chat_kind,
                request.is_archived,
                request.is_pinned,
                request.is_muted,
                request.is_unread,
                request.unread_count,
                request.participant_count,
                request.community_parent_chat_id.as_deref(),
                request.community_parent_title.as_deref(),
                request.invite_link.as_deref(),
                request.is_community_root,
                request.is_broadcast,
                request.is_newsletter,
                &request.avatar_metadata,
                &request.provider_labels,
                request.observed_at,
                &stored_raw,
            )
            .await?;
        let reconciled_commands = self
            .runtime
            .reconcile_fixture_dialog_commands(request)
            .await?;
        self.publish_whatsapp_command_reconciled_events(
            reconciled_commands.clone(),
            reconciliation_source,
        )
        .await?;
        self.publish_whatsapp_conversation_runtime_events(
            &reconciled_commands,
            reconciliation_source,
        )
        .await?;

        Ok(WhatsappWebDialogIngestResult {
            raw_record_id: stored_raw.raw_record_id,
            channel_id,
            conversation_id,
        })
    }

    pub(crate) async fn ingest_participant(
        &self,
        request: &NewWhatsappWebParticipant,
    ) -> Result<WhatsappWebParticipantIngestResult, CommunicationFixtureIngestError> {
        self.ingest_participant_with_reconciliation_source(
            request,
            "provider_observed.fixture_participant",
        )
        .await
    }

    pub(crate) async fn ingest_runtime_bridge_participant(
        &self,
        request: &NewWhatsappWebParticipant,
    ) -> Result<WhatsappWebParticipantIngestResult, CommunicationFixtureIngestError> {
        self.ingest_participant_with_reconciliation_source(
            request,
            "provider_observed.runtime_bridge_participant",
        )
        .await
    }

    async fn ingest_participant_with_reconciliation_source(
        &self,
        request: &NewWhatsappWebParticipant,
        reconciliation_source: &str,
    ) -> Result<WhatsappWebParticipantIngestResult, CommunicationFixtureIngestError> {
        let observed = self.runtime.ingest_fixture_participant(request).await?;
        let observed_raw =
            annotate_whatsapp_raw_observed_source(&observed.raw, reconciliation_source)?;
        let stored_raw = self.record_and_accept_whatsapp_raw(&observed_raw).await?;
        let channel_id = self.ensure_whatsapp_channel(&request.account_id).await?;
        let conversation_id =
            whatsapp_conversation_id(&request.account_id, &request.provider_chat_id);
        let previous_last_message_at = self
            .whatsapp_conversation_last_message_at(&conversation_id)
            .await?;
        let conversation_id = self
            .upsert_whatsapp_conversation(
                &request.account_id,
                &channel_id,
                &request.provider_chat_id,
                request.effective_chat_title(),
                request.effective_chat_kind(),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                &json!({}),
                &[],
                request.observed_at,
                &stored_raw,
            )
            .await?;
        self.restore_whatsapp_conversation_last_message_at(
            &request.account_id,
            &conversation_id,
            previous_last_message_at,
        )
        .await?;
        let identity_id = self
            .upsert_whatsapp_identity(&request.account_id, &channel_id, request, &stored_raw)
            .await?;
        self.upsert_whatsapp_persona_identity_traces_for_participant(request, &stored_raw)
            .await?;
        let participant_upsert = self
            .upsert_whatsapp_conversation_participant(
                &conversation_id,
                &identity_id,
                request,
                &stored_raw,
            )
            .await?;
        let reconciled_commands = self
            .runtime
            .reconcile_fixture_participant_commands(request)
            .await?;
        self.publish_whatsapp_command_reconciled_events(
            reconciled_commands.clone(),
            reconciliation_source,
        )
        .await?;
        self.publish_whatsapp_group_runtime_events(&reconciled_commands, reconciliation_source)
            .await?;

        Ok(WhatsappWebParticipantIngestResult {
            raw_record_id: stored_raw.raw_record_id,
            conversation_id,
            participant_id: participant_upsert.participant_id,
            identity_id,
            previous_role: participant_upsert.previous_role,
            current_role: request.role.clone(),
            previous_status: participant_upsert.previous_status,
            current_status: request.status.clone(),
            role_changed: participant_upsert.role_changed,
            membership_changed: participant_upsert.membership_changed,
        })
    }
}
