use super::*;

impl ZoomStore {
    pub(super) async fn mark_account_authorized(
        &self,
        account: &ZoomAccount,
        update: ZoomAuthorizedAccountUpdate<'_>,
    ) -> Result<ZoomAccount, ZoomError> {
        let mut config = account.config.clone();
        config["lifecycle_state"] = json!("authorized");
        config["runtime_kind"] = json!(ZOOM_LIVE_AUTHORIZED_RUNTIME_KIND);
        clear_zoom_provider_workers_not_enabled_blocker(&mut config);
        clear_zoom_live_authorization_required_blocker(&mut config);
        config["credential_refs_bound"] = json!({
            "zoom_oauth_token": true,
            "zoom_client_secret": update.client_secret_ref.is_some(),
            "zoom_webhook_secret": account.config
                .get("credential_refs_bound")
                .and_then(|value| value.get("zoom_webhook_secret"))
                .and_then(Value::as_bool)
                .unwrap_or(false),
        });
        config["authorization"] = json!({
            "status": "authorized",
            "auth_shape": update.auth_shape,
            "authorized_at": update.authorized_at,
            "token_secret_bound": !update.token_secret_ref.trim().is_empty(),
            "token_secret_purpose": ProviderAccountSecretPurpose::ZoomOauthToken.as_str(),
            "client_secret_bound": update.client_secret_ref.is_some(),
            "client_secret_purpose": ProviderAccountSecretPurpose::ZoomClientSecret.as_str(),
            "expires_at": update.expires_at,
            "metadata": update.metadata,
            "secret_material": "excluded",
        });
        config["last_error"] = Value::Null;
        self.update_account_config(&account.account_id, config)
            .await
    }

    pub(super) async fn mark_account_recording_sync_completed(
        &self,
        account: &ZoomAccount,
        request: &ZoomRecordingSyncRequest,
        allow_remote_recording_downloads: bool,
        result: &ZoomRecordingSyncResult,
        allow_remote_transcript_downloads: bool,
        observed_at: DateTime<Utc>,
    ) -> Result<ZoomAccount, ZoomError> {
        let mut config = account.config.clone();
        config["recording_sync"] = json!({
            "status": if result.failures.is_empty() {
                "completed"
            } else {
                "completed_with_failures"
            },
            "observed_at": observed_at,
            "from": request.from.trim(),
            "to": request.to.trim(),
            "user_id": result.user_id,
            "page_size": request.page_size(),
            "max_meetings": request.max_meetings(),
            "allow_remote_recording_downloads": allow_remote_recording_downloads,
            "allow_remote_transcript_downloads": allow_remote_transcript_downloads,
            "meetings_seen": result.meetings_seen,
            "meetings_recorded": result.meetings_recorded,
            "recordings_recorded": result.recordings_recorded,
            "media_downloads_recorded": result.media_downloads_recorded,
            "transcripts_recorded": result.transcripts_recorded,
            "failure_count": result.failures.len(),
            "failures": result.failures,
        });
        config["last_error"] = Value::Null;
        self.update_account_config(&account.account_id, config)
            .await
    }

    pub(super) async fn mark_account_recording_sync_failed(
        &self,
        account: &ZoomAccount,
        request: &ZoomRecordingSyncRequest,
        allow_remote_recording_downloads: bool,
        allow_remote_transcript_downloads: bool,
        observed_at: DateTime<Utc>,
        error: &str,
    ) -> Result<ZoomAccount, ZoomError> {
        let mut config = account.config.clone();
        config["recording_sync"] = json!({
            "status": "failed",
            "observed_at": observed_at,
            "from": request.from.trim(),
            "to": request.to.trim(),
            "user_id": request.user_id.as_deref().map(str::trim).filter(|value| !value.is_empty()),
            "page_size": request.page_size(),
            "max_meetings": request.max_meetings(),
            "allow_remote_recording_downloads": allow_remote_recording_downloads,
            "allow_remote_transcript_downloads": allow_remote_transcript_downloads,
            "error": error,
        });
        config["last_error"] = json!(error);
        self.update_account_config(&account.account_id, config)
            .await
    }

    pub(super) async fn mark_account_token_refreshed(
        &self,
        account: &ZoomAccount,
        bundle: &ZoomOAuthTokenBundle,
        refresh_strategy: &str,
        token_secret_ref: &str,
        force: bool,
        refreshed_at: DateTime<Utc>,
    ) -> Result<ZoomAccount, ZoomError> {
        let mut config = account.config.clone();
        if !config
            .get("authorization")
            .is_some_and(|authorization| authorization.is_object())
        {
            config["authorization"] = json!({});
        }
        config["authorization"]["status"] = json!("authorized");
        config["authorization"]["auth_shape"] = json!(&account.auth_shape);
        config["authorization"]["expires_at"] = json!(bundle.expires_at);
        config["authorization"]["token_secret_bound"] = json!(!token_secret_ref.trim().is_empty());
        config["authorization"]["token_secret_purpose"] =
            json!(ProviderAccountSecretPurpose::ZoomOauthToken.as_str());
        config["authorization"]["last_token_refresh"] = json!({
            "status": "refreshed",
            "strategy": refresh_strategy,
            "force": force,
            "refreshed_at": refreshed_at,
            "expires_at": bundle.expires_at,
            "secret_material": "excluded",
        });
        if !config
            .get("credential_refs_bound")
            .is_some_and(|bindings| bindings.is_object())
        {
            config["credential_refs_bound"] = json!({});
        }
        config["credential_refs_bound"]["zoom_oauth_token"] = json!(true);
        clear_zoom_token_refresh_blocker(&mut config);
        config["last_error"] = Value::Null;
        self.update_account_config(&account.account_id, config)
            .await
    }

    pub(super) async fn mark_account_token_refresh_failed(
        &self,
        account: &ZoomAccount,
        refresh_strategy: &str,
        force: bool,
        refreshed_at: DateTime<Utc>,
        error: &str,
    ) -> Result<(), ZoomError> {
        let mut config = account.config.clone();
        if !config
            .get("authorization")
            .is_some_and(|authorization| authorization.is_object())
        {
            config["authorization"] = json!({});
        }
        config["authorization"]["status"] = json!("authorized");
        config["authorization"]["auth_shape"] = json!(&account.auth_shape);
        config["authorization"]["last_token_refresh"] = json!({
            "status": "failed",
            "strategy": refresh_strategy,
            "force": force,
            "refreshed_at": refreshed_at,
            "secret_material": "excluded",
            "error": error,
        });
        add_zoom_token_refresh_blocker(&mut config);
        config["last_error"] = json!(error);
        self.update_account_config(&account.account_id, config)
            .await?;
        Ok(())
    }

    pub(super) async fn mark_account_webhook_subscription_checked(
        &self,
        account: &ZoomAccount,
        managed_subscription_id: Option<&str>,
        subscription: Option<&ZoomWebhookSubscription>,
        checked_at: DateTime<Utc>,
        error: Option<&str>,
    ) -> Result<ZoomAccount, ZoomError> {
        let mut config = account.config.clone();
        config["webhook_subscription"] = json!({
            "managed_subscription_id": managed_subscription_id,
            "subscription_name": subscription.map(|value| value.subscription_name.as_str()),
            "endpoint_url": subscription.map(|value| value.endpoint_url.as_str()),
            "event_types": subscription.map(|value| value.event_types.clone()).unwrap_or_default(),
            "checked_at": checked_at,
            "status": if error.is_some() {
                "failed"
            } else if subscription.is_some() {
                "managed"
            } else {
                "cleared"
            },
            "error": error,
        });
        if let Some(error) = error {
            config["last_error"] = json!(error);
        } else {
            config["last_error"] = Value::Null;
        }
        self.update_account_config(&account.account_id, config)
            .await
    }

    pub(super) async fn bind_live_secret_refs(
        &self,
        request: &ZoomLiveAccountSetupRequest,
    ) -> Result<(), ZoomError> {
        self.bind_optional_secret_ref(
            &request.account_id,
            ProviderAccountSecretPurpose::ZoomOauthToken,
            request.token_secret_ref.as_deref(),
        )
        .await?;
        self.bind_optional_secret_ref(
            &request.account_id,
            ProviderAccountSecretPurpose::ZoomClientSecret,
            request.client_secret_ref.as_deref(),
        )
        .await?;
        self.bind_optional_secret_ref(
            &request.account_id,
            ProviderAccountSecretPurpose::ZoomWebhookSecret,
            request.webhook_secret_ref.as_deref(),
        )
        .await?;
        Ok(())
    }

    pub(super) async fn bind_optional_secret_ref(
        &self,
        account_id: &str,
        secret_purpose: ProviderAccountSecretPurpose,
        secret_ref: Option<&str>,
    ) -> Result<(), ZoomError> {
        let Some(secret_ref) = secret_ref.map(str::trim).filter(|value| !value.is_empty()) else {
            return Ok(());
        };
        self.provider_secret_binding_store
            .bind(&NewProviderAccountSecretBinding::new(
                account_id.trim(),
                secret_purpose,
                secret_ref,
            ))
            .await?;
        Ok(())
    }

    pub(super) async fn publish_runtime_status_event(
        &self,
        status: &ZoomRuntimeStatus,
        action: &str,
    ) -> Result<(), ZoomError> {
        let observed_at = Utc::now();
        let event_id = zoom_event_id("runtime", &status.account_id, &status.status, None);
        let event = zoom_event(
            event_id,
            zoom_event_types::RUNTIME_STATUS_CHANGED,
            observed_at,
            json!({
                "provider": "zoom",
                "provider_kind": &status.provider_kind,
                "account_id": &status.account_id,
            }),
            json!({
                "kind": "zoom_runtime",
                "account_id": &status.account_id,
            }),
            sanitize_zoom_payload(json!({
                "status": status,
            })),
            json!({
                "source": "zoom_runtime_control",
                "action": action,
            }),
            None,
            Some(format!("zoom-runtime:{}", status.account_id)),
        )?;
        self.append_and_broadcast(&event).await
    }

    pub(super) async fn publish_authorization_completed_event(
        &self,
        account: &ZoomAccount,
        authorized_at: DateTime<Utc>,
        metadata: &Value,
        action: &str,
    ) -> Result<(), ZoomError> {
        let event_id = zoom_event_id(
            "authorization_completed",
            &account.account_id,
            &account.auth_shape,
            Some(&authorized_at.timestamp_millis().to_string()),
        );
        let event = zoom_event(
            event_id,
            zoom_event_types::AUTHORIZATION_COMPLETED,
            authorized_at,
            json!({
                "provider": "zoom",
                "provider_kind": &account.provider_kind,
                "account_id": &account.account_id,
            }),
            json!({
                "kind": "zoom_account",
                "entity_id": &account.account_id,
                "account_id": &account.account_id,
            }),
            sanitize_zoom_payload(json!({
                "account_id": &account.account_id,
                "auth_shape": &account.auth_shape,
                "lifecycle_state": &account.lifecycle_state,
                "runtime_kind": &account.runtime_kind,
                "authorized_at": authorized_at,
                "expires_at": account.config
                    .get("authorization")
                    .and_then(|authorization| authorization.get("expires_at"))
                    .cloned(),
                "client_credential_bound": account
                    .config
                    .get("credential_refs_bound")
                    .and_then(|bindings| bindings.get("zoom_client_secret"))
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                "credential_bundle_bound": account
                    .config
                    .get("credential_refs_bound")
                    .and_then(|bindings| bindings.get("zoom_oauth_token"))
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                "metadata": metadata,
            })),
            json!({
                "source": "zoom_authorization",
                "action": action,
            }),
            None,
            Some(format!("zoom-account:{}", account.account_id)),
        )?;
        self.append_and_broadcast(&event).await
    }

    pub(super) async fn publish_token_refresh_event(
        &self,
        result: &ZoomTokenRefreshResult,
        force: bool,
        action: &str,
        error: Option<&str>,
    ) -> Result<(), ZoomError> {
        let event_type = match result.status.as_str() {
            "refreshed" => zoom_event_types::TOKEN_REFRESHED,
            "skipped_not_expired" => zoom_event_types::TOKEN_REFRESH_SKIPPED,
            _ => zoom_event_types::TOKEN_REFRESHED,
        };
        let event_id = zoom_event_id(
            "token_refresh",
            &result.account_id,
            &result.status,
            Some(&result.checked_at.timestamp_millis().to_string()),
        );
        let event = zoom_event(
            event_id,
            event_type,
            result.checked_at,
            json!({
                "provider": "zoom",
                "provider_kind": &result.provider_kind,
                "account_id": &result.account_id,
            }),
            json!({
                "kind": "zoom_account",
                "entity_id": &result.account_id,
                "account_id": &result.account_id,
            }),
            sanitize_zoom_payload(json!({
                "account_id": &result.account_id,
                "auth_shape": &result.auth_shape,
                "status": &result.status,
                "refreshed": result.refreshed,
                "force": force,
                "checked_at": result.checked_at,
                "expires_at": result.expires_at,
                "refresh_flow": refresh_flow_label(&result.auth_shape),
                "store_kind": &result.store_kind,
                "credential_format": "oauth_bundle",
                "error": error,
            })),
            json!({
                "source": "zoom_token_refresh",
                "action": action,
            }),
            None,
            Some(format!("zoom-account:{}", result.account_id)),
        )?;
        self.append_and_broadcast(&event).await
    }

    pub(super) async fn publish_token_refresh_failure_event(
        &self,
        account: &ZoomAccount,
        refresh_strategy: &str,
        force: bool,
        checked_at: DateTime<Utc>,
        error: &str,
        action: &str,
    ) -> Result<(), ZoomError> {
        let event_id = zoom_event_id(
            "token_refresh_failed",
            &account.account_id,
            refresh_strategy,
            Some(&checked_at.timestamp_millis().to_string()),
        );
        let event = zoom_event(
            event_id,
            zoom_event_types::TOKEN_REFRESH_FAILED,
            checked_at,
            json!({
                "provider": "zoom",
                "provider_kind": &account.provider_kind,
                "account_id": &account.account_id,
            }),
            json!({
                "kind": "zoom_account",
                "entity_id": &account.account_id,
                "account_id": &account.account_id,
            }),
            sanitize_zoom_payload(json!({
                "account_id": &account.account_id,
                "auth_shape": &account.auth_shape,
                "status": "failed",
                "refreshed": false,
                "force": force,
                "checked_at": checked_at,
                "refresh_flow": refresh_flow_label(&account.auth_shape),
                "error": error,
            })),
            json!({
                "source": "zoom_token_refresh",
                "action": action,
            }),
            None,
            Some(format!("zoom-account:{}", account.account_id)),
        )?;
        self.append_and_broadcast(&event).await
    }

    pub(super) async fn publish_recording_import_removed_event(
        &self,
        account_id: &str,
        imported: &ImportedAttachmentRecord,
        blob_metadata_removed: bool,
        blob_file_removed: bool,
        removed_at: DateTime<Utc>,
        reason: Option<String>,
    ) -> Result<(), ZoomError> {
        let recording_id = imported
            .metadata
            .get("recording_id")
            .and_then(Value::as_str)
            .unwrap_or(imported.attachment_id.as_str());
        let event_id = zoom_event_id(
            "recording_import_removed",
            account_id,
            recording_id,
            Some(imported.attachment_id.as_str()),
        );
        let event = zoom_event(
            event_id,
            zoom_event_types::RECORDING_IMPORT_REMOVED,
            removed_at,
            json!({
                "provider": "zoom",
                "account_id": account_id,
                "source_kind": imported.source_kind,
            }),
            json!({
                "kind": "zoom_recording_import",
                "entity_id": imported.attachment_id,
                "account_id": account_id,
            }),
            sanitize_zoom_payload(json!({
                "attachment_id": imported.attachment_id,
                "blob_id": imported.blob_id,
                "recording_id": imported.metadata.get("recording_id").cloned(),
                "meeting_id": imported.metadata.get("meeting_id").cloned(),
                "meeting_uuid": imported.metadata.get("meeting_uuid").cloned(),
                "filename": imported.filename,
                "content_type": imported.content_type,
                "size_bytes": imported.size_bytes,
                "storage_kind": imported.storage_kind,
                "storage_path": imported.storage_path,
                "blob_metadata_removed": blob_metadata_removed,
                "blob_file_removed": blob_file_removed,
                "removed_at": removed_at,
                "reason": reason,
            })),
            json!({
                "source": "zoom_recording_import_retention_control",
            }),
            Some(format!("zoom-recording-import:{}", imported.attachment_id)),
            Some(format!("zoom-recording-import:{}", account_id)),
        )?;
        self.append_and_broadcast(&event).await
    }

    pub(super) async fn publish_transcript_removed_event(
        &self,
        transcript: &crate::platform::calls::models::CallTranscript,
        removed_at: DateTime<Utc>,
    ) -> Result<(), ZoomError> {
        let account_id = transcript.account_id.as_str();
        let event_id = zoom_event_id(
            "transcript_removed",
            account_id,
            &transcript.transcript_id,
            Some(transcript.call_id.as_str()),
        );
        let event = zoom_event(
            event_id,
            zoom_event_types::TRANSCRIPT_REMOVED,
            removed_at,
            json!({
                "provider": "zoom",
                "provider_kind": ZOOM_PROVIDER_KIND_STR,
                "account_id": account_id,
            }),
            json!({
                "kind": "zoom_transcript",
                "entity_id": transcript.transcript_id,
                "transcript_id": transcript.transcript_id,
                "call_id": transcript.call_id,
            }),
            sanitize_zoom_payload(json!({
                "call_id": transcript.call_id,
                "transcript_id": transcript.transcript_id,
                "meeting_id": transcript.provenance.get("meeting_id").cloned(),
                "source_recording_ref": transcript.source_audio_ref,
                "language_code": transcript.language_code,
                "retention_policy": transcript.provenance.get("retention_policy").cloned(),
                "removed_at": removed_at,
                "reason": "retention_policy_expired",
            })),
            json!({
                "source": "zoom_retention_cleanup",
            }),
            Some(format!("zoom-transcript:{}", transcript.transcript_id)),
            Some(format!("zoom-retention:{}", account_id)),
        )?;
        self.append_and_broadcast(&event).await
    }

    pub(super) async fn publish_retention_cleanup_completed_event(
        &self,
        result: &ZoomRetentionCleanupResponse,
    ) -> Result<(), ZoomError> {
        let event_id = zoom_event_id(
            "retention_cleanup_completed",
            &result.account_id,
            &result.account_id,
            Some(&format!(
                "{}:{}:{}",
                result.checked_at.timestamp_micros(),
                result.recordings_removed,
                result.transcripts_removed
            )),
        );
        let event = zoom_event(
            event_id,
            zoom_event_types::RETENTION_CLEANUP_COMPLETED,
            result.checked_at,
            json!({
                "provider": "zoom",
                "provider_kind": ZOOM_PROVIDER_KIND_STR,
                "account_id": result.account_id,
            }),
            json!({
                "kind": "zoom_retention_cleanup",
                "entity_id": result.account_id,
                "account_id": result.account_id,
            }),
            json!({
                "recordings_removed": result.recordings_removed,
                "transcripts_removed": result.transcripts_removed,
                "item_count": result.items.len(),
            }),
            json!({
                "source": "zoom_retention_cleanup",
            }),
            Some(format!("zoom-retention:{}", result.account_id)),
            Some(format!("zoom-retention:{}", result.account_id)),
        )?;
        self.append_and_broadcast(&event).await
    }

    pub(super) async fn append_and_broadcast(
        &self,
        event: &NewEventEnvelope,
    ) -> Result<(), ZoomError> {
        if self
            .event_store
            .append_for_dispatch_idempotent(event)
            .await?
            .is_some()
        {
            self.event_bus.broadcast(event.clone());
        }
        Ok(())
    }
}
