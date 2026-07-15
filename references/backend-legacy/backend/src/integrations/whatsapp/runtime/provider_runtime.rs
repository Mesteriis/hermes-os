use super::*;

impl WhatsAppProviderRuntime for WhatsappWebStore {
    fn provider_shape(&self) -> WhatsAppProviderRuntimeShape {
        WhatsAppProviderRuntimeShape::WebCompanion
    }

    fn runtime_status<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        account_id: &'a str,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus> {
        Box::pin(async move {
            let account = self.whatsapp_account(account_id).await?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            Ok(self.status_from_account(&account, "stopped", restored_session, None))
        })
    }

    fn start_runtime<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppRuntimeStartRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus> {
        Box::pin(async move {
            let account = self.whatsapp_account(&request.account_id).await?;
            let runtime_kind = account_runtime_kind(&account);
            let _provider_shape = account_provider_shape(&account, self.provider_shape());
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let status = if runtime_kind == "fixture" {
                "running"
            } else if runtime_kind == "live_blocked" {
                "blocked"
            } else {
                "stopped"
            };
            let last_error = (runtime_kind == "live_blocked").then(|| {
                "hidden WhatsApp WebView runtime requires an explicit desktop start".to_owned()
            });
            Ok(self.status_from_account(&account, status, restored_session, last_error))
        })
    }

    fn stop_runtime<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppRuntimeStopRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus> {
        Box::pin(async move {
            let account = self.whatsapp_account(&request.account_id).await?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            Ok(self.status_from_account(&account, "linked", restored_session, None))
        })
    }

    fn revoke_runtime<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppRuntimeRevokeRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus> {
        Box::pin(async move {
            self.clear_session_secret_material(secret_store, vault, &request.account_id)
                .await?;
            let updated = self
                .update_account_lifecycle_state(
                    &request.account_id,
                    "revoked",
                    "whatsapp.runtime.revoke",
                )
                .await?;
            Ok(self.status_from_account(
                &updated,
                "revoked",
                None,
                Some("WhatsApp session was revoked and must be relinked".to_owned()),
            ))
        })
    }

    fn relink_runtime<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppRuntimeRelinkRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus> {
        Box::pin(async move {
            self.clear_session_secret_material(secret_store, vault, &request.account_id)
                .await?;
            let updated = self
                .update_account_lifecycle_state(
                    &request.account_id,
                    "created",
                    "whatsapp.runtime.relink",
                )
                .await?;
            Ok(self.status_from_account(&updated, "link_required", None, None))
        })
    }

    fn remove_runtime<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppRuntimeRemoveRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeRemoveResponse> {
        Box::pin(async move {
            let account = self.whatsapp_account(&request.account_id).await?;
            let binding_refs = self
                .clear_account_secret_material(secret_store, vault, &account.account_id)
                .await?;
            let updated = self
                .update_account_lifecycle_state(
                    &account.account_id,
                    "removed",
                    "whatsapp.runtime.remove",
                )
                .await?;
            Ok(WhatsAppRuntimeRemoveResponse {
                account_id: updated.account_id,
                provider_kind: updated.provider_kind.as_str().to_owned(),
                removed: true,
                unbound_secret_refs: binding_refs,
                removed_at: Utc::now(),
            })
        })
    }

    fn runtime_health<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        account_id: &'a str,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeHealth> {
        Box::pin(async move {
            let account = self.whatsapp_account(account_id).await?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let status = self.status_from_account(&account, "stopped", restored_session, None);
            let health_status = runtime_health_status(&status);
            let healthy = health_status == "available";
            let requires_hidden_webview = status.provider_shape == "whatsapp_web_companion"
                && status.runtime_kind != "fixture";
            let provider_shape = status.provider_shape.clone();
            let runtime_kind = status.runtime_kind.clone();
            let mut checks = json!({
                "session_metadata_available": true,
                "session_restore_available": status.session_restore_available,
                "session_secret_ref": status.session_secret_ref,
                "live_runtime_available": status.live_runtime_available,
                "runtime_blockers": status.runtime_blockers,
                "session": {
                    "metadata_available": true,
                    "restore_available": status.session_restore_available,
                    "linked": matches!(status.status.as_str(), "linked" | "available" | "syncing" | "degraded"),
                    "link_required": matches!(status.status.as_str(), "created" | "link_required"),
                    "revoked": status.status == "revoked",
                    "secret_ref_bound": status.session_secret_ref.is_some(),
                },
                "storage": {
                    "binding_store": "host_vault",
                    "binding_purpose": "whatsapp_web_session_key",
                    "secret_ref_bound": status.session_secret_ref.is_some(),
                },
                "runtime": {
                    "lifecycle_state": status.status,
                    "fixture_runtime": status.fixture_runtime,
                    "kind": status.runtime_kind,
                    "provider_shape": status.provider_shape,
                    "live_runtime_available": status.live_runtime_available,
                    "live_send_available": status.live_send_available,
                    "media_download_available": status.media_download_available,
                    "media_upload_available": status.media_upload_available,
                },
                "webview": {
                    "required": requires_hidden_webview,
                    "hidden_runtime_available": status.runtime_kind == "webview_companion"
                        && status.live_runtime_available,
                    "hidden_runtime_required": status.runtime_blockers.iter().any(|blocker| blocker == "whatsapp_hidden_webview_runtime_required"),
                    "companion_runtime": status.runtime_kind == "webview_companion",
                },
                "validation": {
                    "status": health_status,
                    "healthy": healthy,
                    "blocker_count": status.runtime_blockers.len(),
                    "has_last_error": status.last_error.is_some(),
                },
            });
            if provider_shape == WhatsAppProviderRuntimeShape::WebCompanion.as_str() {
                let web_companion_bridge =
                    web_companion::web_companion_bridge_contract_health_check();
                checks["web_companion_bridge"] = web_companion_bridge.clone();
                checks["runtime"]["web_companion_bridge"] = web_companion_bridge;
            }
            Ok(WhatsAppRuntimeHealth {
                account_id: status.account_id,
                provider_shape,
                runtime_kind,
                status: health_status.to_owned(),
                healthy,
                checks,
                checked_at: Utc::now(),
            })
        })
    }

    fn request_send_text<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppTextSendRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let text = validate_non_empty("text", &request.text)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "send_text",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: None,
                    action_class: "provider_write",
                    confirmation_decision: "confirmed",
                    payload: json!({"text": text}),
                    target_ref: json!({"provider_chat_id": provider_chat_id}),
                    rendered_preview_hash: Some(whatsapp_text_preview_hash(&text)),
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(
                &account,
                &command,
                command
                    .audit_metadata
                    .get("rendered_preview_hash")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
            ))
        })
    }

    fn request_reply<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppReplyRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let reply_to_provider_message_id = validate_non_empty(
                "reply_to_provider_message_id",
                &request.reply_to_provider_message_id,
            )?;
            let text = validate_non_empty("text", &request.text)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "reply",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: Some(&reply_to_provider_message_id),
                    action_class: "provider_write",
                    confirmation_decision: "confirmed",
                    payload: json!({
                        "text": text,
                        "reply_to_provider_message_id": reply_to_provider_message_id,
                    }),
                    target_ref: json!({
                        "provider_chat_id": provider_chat_id,
                        "provider_message_id": reply_to_provider_message_id,
                    }),
                    rendered_preview_hash: Some(whatsapp_text_preview_hash(&text)),
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(
                &account,
                &command,
                command
                    .audit_metadata
                    .get("rendered_preview_hash")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
            ))
        })
    }

    fn request_forward<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppForwardRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let _from_provider_chat_id =
                validate_non_empty("from_provider_chat_id", &request.from_provider_chat_id)?;
            let from_provider_message_id = validate_non_empty(
                "from_provider_message_id",
                &request.from_provider_message_id,
            )?;
            let text = request
                .text
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty());
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "forward",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: Some(&from_provider_message_id),
                    action_class: "provider_write",
                    confirmation_decision: "confirmed",
                    payload: json!({
                        "from_provider_chat_id": _from_provider_chat_id,
                        "from_provider_message_id": from_provider_message_id,
                        "text": text,
                    }),
                    target_ref: json!({
                        "provider_chat_id": provider_chat_id,
                        "provider_message_id": from_provider_message_id,
                    }),
                    rendered_preview_hash: text.map(whatsapp_text_preview_hash),
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(
                &account,
                &command,
                command
                    .audit_metadata
                    .get("rendered_preview_hash")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
            ))
        })
    }

    fn request_edit<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppEditRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let provider_message_id =
                validate_non_empty("provider_message_id", &request.provider_message_id)?;
            let text = validate_non_empty("text", &request.text)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "edit",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: Some(&provider_message_id),
                    action_class: "provider_write",
                    confirmation_decision: "confirmed",
                    payload: json!({"text": text}),
                    target_ref: json!({
                        "provider_chat_id": provider_chat_id,
                        "provider_message_id": provider_message_id,
                    }),
                    rendered_preview_hash: Some(whatsapp_text_preview_hash(&text)),
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(
                &account,
                &command,
                command
                    .audit_metadata
                    .get("rendered_preview_hash")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
            ))
        })
    }

    fn request_delete<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppDeleteRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let confirmation_decision = request
                .confirmation_decision
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("pending")
                .to_owned();
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let provider_message_id =
                validate_non_empty("provider_message_id", &request.provider_message_id)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "delete",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: Some(&provider_message_id),
                    action_class: "destructive",
                    confirmation_decision: &confirmation_decision,
                    payload: json!({"delete_kind": "provider_delete"}),
                    target_ref: json!({
                        "provider_chat_id": provider_chat_id,
                        "provider_message_id": provider_message_id,
                    }),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn request_react<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppReactionRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let provider_message_id =
                validate_non_empty("provider_message_id", &request.provider_message_id)?;
            let reaction_emoji = validate_non_empty("reaction_emoji", &request.reaction_emoji)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "react",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: Some(&provider_message_id),
                    action_class: "provider_write",
                    confirmation_decision: "confirmed",
                    payload: json!({"reaction_emoji": reaction_emoji}),
                    target_ref: json!({
                        "provider_chat_id": provider_chat_id,
                        "provider_message_id": provider_message_id,
                    }),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn request_unreact<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppReactionRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let provider_message_id =
                validate_non_empty("provider_message_id", &request.provider_message_id)?;
            let reaction_emoji = validate_non_empty("reaction_emoji", &request.reaction_emoji)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "unreact",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: Some(&provider_message_id),
                    action_class: "provider_write",
                    confirmation_decision: "confirmed",
                    payload: json!({"reaction_emoji": reaction_emoji}),
                    target_ref: json!({
                        "provider_chat_id": provider_chat_id,
                        "provider_message_id": provider_message_id,
                    }),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn request_media_upload<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppMediaUploadRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let blob_id = validate_non_empty("blob_id", &request.blob_id)?;
            let media_type = validate_non_empty("media_type", &request.media_type)?;
            let content_type = validate_non_empty("content_type", &request.content_type)?;
            let sha256 = validate_non_empty("sha256", &request.sha256)?;
            let scan_status = validate_non_empty("scan_status", &request.scan_status)?;
            if request.size_bytes < 0 {
                return Err(WhatsappWebError::InvalidRequest(
                    "size_bytes must not be negative".to_owned(),
                ));
            }
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "send_media",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: None,
                    action_class: "provider_write",
                    confirmation_decision: "confirmed",
                    payload: json!({
                        "attachment_id": request.attachment_id,
                        "blob_id": blob_id,
                        "media_type": media_type,
                        "caption": request.caption,
                        "filename": request.filename,
                        "content_type": content_type,
                        "size_bytes": request.size_bytes,
                        "sha256": sha256,
                        "scan_status": scan_status,
                    }),
                    target_ref: json!({
                        "provider_chat_id": provider_chat_id,
                        "attachment_id": request.attachment_id,
                        "blob_id": request.blob_id,
                    }),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn request_media_download<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppMediaDownloadRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let provider_message_id =
                validate_non_empty("provider_message_id", &request.provider_message_id)?;
            if request.provider_attachment_id.is_none() && request.provider_media_id.is_none() {
                return Err(WhatsappWebError::InvalidRequest(
                    "provider_attachment_id or provider_media_id is required".to_owned(),
                ));
            }
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "download_media",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: Some(&provider_message_id),
                    action_class: "read",
                    confirmation_decision: "not_required",
                    payload: json!({
                        "provider_attachment_id": request.provider_attachment_id,
                        "provider_media_id": request.provider_media_id,
                        "filename": request.filename,
                        "content_type": request.content_type,
                    }),
                    target_ref: json!({
                        "provider_chat_id": provider_chat_id,
                        "provider_message_id": provider_message_id,
                        "provider_attachment_id": request.provider_attachment_id,
                        "provider_media_id": request.provider_media_id,
                    }),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn request_mark_read<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppConversationCommandRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "mark_read",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: None,
                    action_class: "provider_write",
                    confirmation_decision: "not_required",
                    payload: json!({"read_state": "read"}),
                    target_ref: json!({"provider_chat_id": provider_chat_id}),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn request_mark_unread<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppConversationCommandRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "mark_unread",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: None,
                    action_class: "provider_write",
                    confirmation_decision: "not_required",
                    payload: json!({"read_state": "unread"}),
                    target_ref: json!({"provider_chat_id": provider_chat_id}),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn request_archive<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppConversationCommandRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "archive",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: None,
                    action_class: "provider_write",
                    confirmation_decision: "not_required",
                    payload: json!({"archive_state": "archived"}),
                    target_ref: json!({"provider_chat_id": provider_chat_id}),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn request_unarchive<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppConversationCommandRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "unarchive",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: None,
                    action_class: "provider_write",
                    confirmation_decision: "not_required",
                    payload: json!({"archive_state": "main"}),
                    target_ref: json!({"provider_chat_id": provider_chat_id}),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn request_mute<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppConversationCommandRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "mute",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: None,
                    action_class: "provider_write",
                    confirmation_decision: "not_required",
                    payload: json!({"mute_state": "muted"}),
                    target_ref: json!({"provider_chat_id": provider_chat_id}),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn request_unmute<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppConversationCommandRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "unmute",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: None,
                    action_class: "provider_write",
                    confirmation_decision: "not_required",
                    payload: json!({"mute_state": "unmuted"}),
                    target_ref: json!({"provider_chat_id": provider_chat_id}),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn request_pin<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppConversationCommandRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "pin",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: None,
                    action_class: "provider_write",
                    confirmation_decision: "not_required",
                    payload: json!({"pin_state": "pinned"}),
                    target_ref: json!({"provider_chat_id": provider_chat_id}),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn request_unpin<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppConversationCommandRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "unpin",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: None,
                    action_class: "provider_write",
                    confirmation_decision: "not_required",
                    payload: json!({"pin_state": "unpinned"}),
                    target_ref: json!({"provider_chat_id": provider_chat_id}),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn request_join_group<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppConversationCommandRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let invite_link = request
                .invite_link
                .as_deref()
                .map(|value| validate_non_empty("invite_link", value))
                .transpose()?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "join_group",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: None,
                    action_class: "provider_write",
                    confirmation_decision: "confirmed",
                    payload: json!({"invite_link": invite_link}),
                    target_ref: json!({"provider_chat_id": provider_chat_id}),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn request_leave_group<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppConversationCommandRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let confirmation_decision = request
                .confirmation_decision
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("pending")
                .to_owned();
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "leave_group",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: None,
                    action_class: "destructive",
                    confirmation_decision: &confirmation_decision,
                    payload: json!({"membership_state": "left"}),
                    target_ref: json!({"provider_chat_id": provider_chat_id}),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn request_publish_status<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppStatusPublishRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let text = validate_non_empty("text", &request.text)?;
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "publish_status",
                    idempotency_key,
                    provider_chat_id: "status-feed",
                    provider_message_id: None,
                    action_class: "provider_write",
                    confirmation_decision: "confirmed",
                    payload: json!({"text": text}),
                    target_ref: json!({"provider_chat_id": "status-feed"}),
                    rendered_preview_hash: Some(whatsapp_text_preview_hash(&text)),
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(
                &account,
                &command,
                command
                    .audit_metadata
                    .get("rendered_preview_hash")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
            ))
        })
    }

    fn request_send_voice_note<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppVoiceNoteSendRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandResponse> {
        Box::pin(async move {
            let command_id = validated_or_generated_command_id(&request.command_id)?;
            let idempotency_key = validate_non_empty("idempotency_key", &request.idempotency_key)?;
            let account = self.whatsapp_account(&request.account_id).await?;
            let provider_chat_id =
                validate_non_empty("provider_chat_id", &request.provider_chat_id)?;
            let blob_id = validate_non_empty("blob_id", &request.blob_id)?;
            let content_type = validate_non_empty("content_type", &request.content_type)?;
            let sha256 = validate_non_empty("sha256", &request.sha256)?;
            let scan_status = validate_non_empty("scan_status", &request.scan_status)?;
            if request.size_bytes < 0 {
                return Err(WhatsappWebError::InvalidRequest(
                    "size_bytes must not be negative".to_owned(),
                ));
            }
            let restored_session = self
                .optional_restored_session_secret_ref(secret_store, vault, &account.account_id)
                .await?;
            let command = self
                .insert_blocked_provider_command(ProviderCommandInsert {
                    command_id,
                    account_id: &account.account_id,
                    command_kind: "send_voice_note",
                    idempotency_key,
                    provider_chat_id: &provider_chat_id,
                    provider_message_id: None,
                    action_class: "provider_write",
                    confirmation_decision: "confirmed",
                    payload: json!({
                        "attachment_id": request.attachment_id,
                        "blob_id": blob_id,
                        "media_type": "voice_note",
                        "filename": request.filename,
                        "content_type": content_type,
                        "size_bytes": request.size_bytes,
                        "sha256": sha256,
                        "scan_status": scan_status,
                    }),
                    target_ref: json!({
                        "provider_chat_id": provider_chat_id,
                        "attachment_id": request.attachment_id,
                        "blob_id": request.blob_id,
                    }),
                    rendered_preview_hash: None,
                    restored_session_secret_ref: restored_session,
                })
                .await?;
            Ok(self.blocked_command_response(&account, &command, None))
        })
    }

    fn list_provider_commands<'a>(
        &'a self,
        account_id: &'a str,
        provider_chat_id: Option<&'a str>,
        provider_message_id: Option<&'a str>,
        command_kinds: &'a [String],
        limit: i64,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppProviderCommandListResponse> {
        Box::pin(async move {
            let account = self.whatsapp_account(account_id).await?;
            let rows = sqlx::query(
                r#"
                SELECT *
                FROM whatsapp_provider_write_commands
                WHERE account_id = $1
                  AND ($2::text IS NULL OR provider_chat_id = $2)
                  AND ($3::text IS NULL OR provider_message_id = $3)
                  AND (cardinality($4::text[]) = 0 OR command_kind = ANY($4::text[]))
                ORDER BY created_at DESC
                LIMIT $5
                "#,
            )
            .bind(&account.account_id)
            .bind(provider_chat_id)
            .bind(provider_message_id)
            .bind(command_kinds)
            .bind(clamp_limit(limit))
            .fetch_all(self.pool())
            .await?;
            let commands = rows
                .into_iter()
                .map(row_to_whatsapp_provider_write_command)
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .map(WhatsAppProviderCommand::from)
                .collect();
            Ok(WhatsAppProviderCommandListResponse { items: commands })
        })
    }

    fn manual_retry_provider_command<'a>(
        &'a self,
        command_id: &'a str,
    ) -> WhatsAppProviderRuntimeFuture<'a, Option<WhatsAppProviderCommand>> {
        Box::pin(async move {
            let command_id = validate_non_empty("command_id", command_id)?;
            let now = Utc::now();
            let row = sqlx::query(
                r#"
                UPDATE whatsapp_provider_write_commands
                SET status = 'retrying',
                    capability_state = CASE
                        WHEN command_kind IN (
                            'send_text', 'reply', 'forward',
                            'send_media', 'download_media', 'send_voice_note',
                            'edit', 'delete', 'react', 'unreact',
                            'mark_read', 'mark_unread',
                            'archive', 'unarchive',
                            'mute', 'unmute',
                            'pin', 'unpin',
                            'join_group', 'leave_group',
                            'publish_status'
                        ) THEN 'available'
                        ELSE capability_state
                    END,
                    retry_count = 0,
                    next_attempt_at = $2,
                    last_attempt_at = NULL,
                    locked_at = NULL,
                    locked_by = NULL,
                    provider_observed_at = NULL,
                    provider_state = '{}'::jsonb,
                    reconciliation_status = 'not_observed',
                    reconciled_at = NULL,
                    dead_lettered_at = NULL,
                    completed_at = NULL,
                    last_error = NULL,
                    result_payload = result_payload || jsonb_build_object('manual_retry_at', $2),
                    updated_at = $2
                WHERE command_id = $1
                  AND status IN ('failed', 'dead_letter', 'retrying', 'cancelled')
                RETURNING *
                "#,
            )
            .bind(command_id)
            .bind(now)
            .fetch_optional(self.pool())
            .await?;
            let command = row
                .map(row_to_whatsapp_provider_write_command)
                .transpose()?;
            if let Some(command) = command {
                self.mirror_canonical_provider_command(&command).await?;
                Ok(Some(WhatsAppProviderCommand::from(command)))
            } else {
                Ok(None)
            }
        })
    }

    fn dead_letter_provider_command<'a>(
        &'a self,
        command_id: &'a str,
        reason: &'a str,
    ) -> WhatsAppProviderRuntimeFuture<'a, Option<WhatsAppProviderCommand>> {
        Box::pin(async move {
            let command_id = validate_non_empty("command_id", command_id)?;
            let reason = validate_non_empty("reason", reason)?;
            let now = Utc::now();
            let row = sqlx::query(
                r#"
                UPDATE whatsapp_provider_write_commands
                SET status = 'dead_letter',
                    locked_at = NULL,
                    locked_by = NULL,
                    last_error = $3,
                    dead_lettered_at = $2,
                    updated_at = $2
                WHERE command_id = $1
                  AND status NOT IN ('completed', 'dead_letter')
                RETURNING *
                "#,
            )
            .bind(command_id)
            .bind(now)
            .bind(reason)
            .fetch_optional(self.pool())
            .await?;
            let command = row
                .map(row_to_whatsapp_provider_write_command)
                .transpose()?;
            if let Some(command) = command {
                self.mirror_canonical_provider_command(&command).await?;
                Ok(Some(WhatsAppProviderCommand::from(command)))
            } else {
                Ok(None)
            }
        })
    }

    fn store_authorized_session_credential<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        credential: &'a WhatsAppAuthorizedSessionCredentialWrite,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppCredentialBinding> {
        Box::pin(async move {
            let account = self.whatsapp_account(&credential.account_id).await?;
            let _provider_shape = account_provider_shape(&account, self.provider_shape());
            let session_material =
                validate_non_empty("session_material", &credential.session_material)?;
            let label = validate_non_empty("label", &credential.label)?;
            let purpose = ProviderAccountSecretPurpose::WhatsappWebSessionKey;
            if !purpose.accepts_secret_kind(credential.secret_kind) {
                return Err(WhatsappWebError::InvalidRequest(format!(
                    "secret_kind `{}` is incompatible with {}",
                    credential.secret_kind.as_str(),
                    purpose.as_str()
                )));
            }

            let secret_ref = whatsapp_session_secret_ref(&account.account_id);
            let metadata = session_secret_metadata(&account, &credential.metadata);
            let runtime_kind = authorized_session_runtime_kind(&account, &credential.metadata);
            secret_store
                .upsert_secret_reference(
                    &NewSecretReference::new(
                        &secret_ref,
                        credential.secret_kind,
                        SecretStoreKind::HostVault,
                        format!("{label} for {}", account.account_id),
                    )
                    .metadata(metadata.clone()),
                )
                .await?;
            vault.store_secret(
                &secret_ref,
                &session_material,
                SecretEntryContext {
                    entry_kind: "provider_session",
                    account_id: &account.account_id,
                    purpose: purpose.as_str(),
                    secret_kind: credential.secret_kind.as_str(),
                    label: &label,
                    metadata: &metadata,
                },
            )?;
            self.provider_secret_binding_store()
                .bind(&NewProviderAccountSecretBinding::new(
                    &account.account_id,
                    purpose,
                    &secret_ref,
                ))
                .await
                .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?;
            if runtime_kind != account_runtime_kind(&account) {
                self.update_account_runtime_kind(
                    &account.account_id,
                    &runtime_kind,
                    "whatsapp.runtime.authorized_session.store",
                )
                .await?;
            }
            self.update_account_lifecycle_state(
                &account.account_id,
                "linked",
                "whatsapp.runtime.authorized_session.store",
            )
            .await?;
            self.update_session_link_state(
                &account.account_id,
                "linked",
                "whatsapp.runtime.authorized_session.store",
            )
            .await?;

            Ok(WhatsAppCredentialBinding {
                secret_purpose: purpose.as_str().to_owned(),
                secret_ref,
                secret_kind: credential.secret_kind,
                store_kind: SecretStoreKind::HostVault,
            })
        })
    }

    fn setup_fixture_account<'a>(
        &'a self,
        request: &'a WhatsappWebAccountSetupRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebAccountSetupResponse> {
        Box::pin(async move { WhatsappWebStore::setup_fixture_account(self, request).await })
    }

    fn setup_live_blocked_account<'a>(
        &'a self,
        request: &'a WhatsappLiveAccountSetupRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebAccountSetupResponse> {
        Box::pin(async move { WhatsappWebStore::setup_live_blocked_account(self, request).await })
    }

    fn list_sessions<'a>(
        &'a self,
        account_id: Option<&'a str>,
        limit: i64,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsappWebSession>> {
        Box::pin(async move { WhatsappWebStore::list_sessions(self, account_id, limit).await })
    }

    fn recent_messages<'a>(
        &'a self,
        account_id: Option<&'a str>,
        provider_chat_id: Option<&'a str>,
        limit: i64,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsappWebMessage>> {
        Box::pin(async move {
            WhatsappWebStore::recent_messages(self, account_id, provider_chat_id, limit).await
        })
    }

    fn ingest_fixture_message<'a>(
        &'a self,
        message: &'a NewWhatsappWebMessage,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedMessage> {
        Box::pin(async move { WhatsappWebStore::ingest_fixture_message(self, message).await })
    }

    fn reconcile_fixture_message_commands<'a>(
        &'a self,
        message: &'a NewWhatsappWebMessage,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>> {
        Box::pin(async move {
            WhatsappWebStore::reconcile_fixture_message_commands(self, message).await
        })
    }

    fn ingest_fixture_reaction<'a>(
        &'a self,
        reaction: &'a NewWhatsappWebReaction,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedReaction> {
        Box::pin(async move { WhatsappWebStore::ingest_fixture_reaction(self, reaction).await })
    }

    fn reconcile_fixture_reaction_commands<'a>(
        &'a self,
        reaction: &'a NewWhatsappWebReaction,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>> {
        Box::pin(async move {
            WhatsappWebStore::reconcile_fixture_reaction_commands(self, reaction).await
        })
    }

    fn ingest_fixture_media<'a>(
        &'a self,
        media: &'a NewWhatsappWebMedia,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedMedia> {
        Box::pin(async move { WhatsappWebStore::ingest_fixture_media(self, media).await })
    }

    fn reconcile_fixture_media_commands<'a>(
        &'a self,
        media: &'a NewWhatsappWebMedia,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>> {
        Box::pin(
            async move { WhatsappWebStore::reconcile_fixture_media_commands(self, media).await },
        )
    }

    fn ingest_fixture_status<'a>(
        &'a self,
        status: &'a NewWhatsappWebStatus,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedStatus> {
        Box::pin(async move { WhatsappWebStore::ingest_fixture_status(self, status).await })
    }

    fn ingest_fixture_status_view<'a>(
        &'a self,
        status_view: &'a NewWhatsappWebStatusView,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedStatusView> {
        Box::pin(
            async move { WhatsappWebStore::ingest_fixture_status_view(self, status_view).await },
        )
    }

    fn ingest_fixture_status_delete<'a>(
        &'a self,
        status_delete: &'a NewWhatsappWebStatusDelete,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedStatusDelete> {
        Box::pin(async move {
            WhatsappWebStore::ingest_fixture_status_delete(self, status_delete).await
        })
    }

    fn ingest_fixture_presence<'a>(
        &'a self,
        presence: &'a NewWhatsappWebPresence,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedPresence> {
        Box::pin(async move { WhatsappWebStore::ingest_fixture_presence(self, presence).await })
    }

    fn ingest_fixture_call<'a>(
        &'a self,
        call: &'a NewWhatsappWebCall,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedCall> {
        Box::pin(async move { WhatsappWebStore::ingest_fixture_call(self, call).await })
    }

    fn ingest_fixture_runtime_event<'a>(
        &'a self,
        runtime_event: &'a NewWhatsappWebRuntimeEvent,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedRuntimeEvent> {
        Box::pin(async move {
            WhatsappWebStore::ingest_fixture_runtime_event(self, runtime_event).await
        })
    }

    fn reconcile_fixture_status_commands<'a>(
        &'a self,
        status: &'a NewWhatsappWebStatus,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>> {
        Box::pin(
            async move { WhatsappWebStore::reconcile_fixture_status_commands(self, status).await },
        )
    }

    fn ingest_fixture_dialog<'a>(
        &'a self,
        dialog: &'a NewWhatsappWebDialog,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedDialog> {
        Box::pin(async move { WhatsappWebStore::ingest_fixture_dialog(self, dialog).await })
    }

    fn reconcile_fixture_dialog_commands<'a>(
        &'a self,
        dialog: &'a NewWhatsappWebDialog,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>> {
        Box::pin(
            async move { WhatsappWebStore::reconcile_fixture_dialog_commands(self, dialog).await },
        )
    }

    fn ingest_fixture_participant<'a>(
        &'a self,
        participant: &'a NewWhatsappWebParticipant,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedParticipant> {
        Box::pin(
            async move { WhatsappWebStore::ingest_fixture_participant(self, participant).await },
        )
    }

    fn reconcile_fixture_participant_commands<'a>(
        &'a self,
        participant: &'a NewWhatsappWebParticipant,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>> {
        Box::pin(async move {
            WhatsappWebStore::reconcile_fixture_participant_commands(self, participant).await
        })
    }

    fn ingest_fixture_message_update<'a>(
        &'a self,
        update: &'a NewWhatsappWebMessageUpdate,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedMessageUpdate> {
        Box::pin(async move { WhatsappWebStore::ingest_fixture_message_update(self, update).await })
    }

    fn reconcile_fixture_message_update_commands<'a>(
        &'a self,
        update: &'a NewWhatsappWebMessageUpdate,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>> {
        Box::pin(async move {
            WhatsappWebStore::reconcile_fixture_message_update_commands(self, update).await
        })
    }

    fn ingest_fixture_message_delete<'a>(
        &'a self,
        delete: &'a NewWhatsappWebMessageDelete,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedMessageDelete> {
        Box::pin(async move { WhatsappWebStore::ingest_fixture_message_delete(self, delete).await })
    }

    fn reconcile_fixture_message_delete_commands<'a>(
        &'a self,
        delete: &'a NewWhatsappWebMessageDelete,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>> {
        Box::pin(async move {
            WhatsappWebStore::reconcile_fixture_message_delete_commands(self, delete).await
        })
    }

    fn ingest_fixture_receipt<'a>(
        &'a self,
        receipt: &'a NewWhatsappWebReceipt,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsappWebObservedReceipt> {
        Box::pin(async move { WhatsappWebStore::ingest_fixture_receipt(self, receipt).await })
    }

    fn reconcile_fixture_receipt_commands<'a>(
        &'a self,
        receipt: &'a NewWhatsappWebReceipt,
    ) -> WhatsAppProviderRuntimeFuture<'a, Vec<WhatsAppProviderCommand>> {
        Box::pin(async move {
            WhatsappWebStore::reconcile_fixture_receipt_commands(self, receipt).await
        })
    }
}
