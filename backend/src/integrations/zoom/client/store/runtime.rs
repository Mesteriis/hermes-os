use super::*;

impl ZoomStore {
    pub fn new(
        pool: PgPool,
        provider_account_store: Arc<dyn ProviderAccountCommandPort>,
        provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
        imported_attachment_store: Arc<dyn ImportedAttachmentStoragePort>,
        call_store: CallIntelligenceStore,
        event_store: EventStore,
        event_bus: InMemoryEventBus,
    ) -> Self {
        Self {
            pool,
            provider_account_store,
            provider_secret_binding_store,
            imported_attachment_store,
            call_store,
            event_store,
            event_bus,
            http: zoom_http_client(),
        }
    }

    pub async fn setup_fixture_account(
        &self,
        request: &ZoomAccountSetupRequest,
    ) -> Result<ZoomAccountSetupResponse, ZoomError> {
        request.validate()?;
        let account = self
            .provider_account_store
            .upsert_runtime_account(
                request.account_id.trim().to_owned(),
                ZOOM_PROVIDER_KIND.as_str().to_owned(),
                request.display_name.trim().to_owned(),
                request.external_account_id.trim().to_owned(),
                request.account_config(),
            )
            .await?;
        Ok(ZoomAccountSetupResponse {
            account: account.into(),
        })
    }

    pub async fn setup_live_blocked_account(
        &self,
        request: &ZoomLiveAccountSetupRequest,
    ) -> Result<ZoomAccountSetupResponse, ZoomError> {
        request.validate()?;
        let provider_kind = request.provider_kind();
        let account = self
            .provider_account_store
            .upsert_runtime_account(
                request.account_id.trim().to_owned(),
                provider_kind.as_str().to_owned(),
                request.display_name.trim().to_owned(),
                request.external_account_id.trim().to_owned(),
                request.account_config(),
            )
            .await?;
        self.bind_live_secret_refs(request).await?;
        Ok(ZoomAccountSetupResponse {
            account: account.into(),
        })
    }

    pub async fn start_oauth(
        &self,
        request: &ZoomOAuthStartRequest,
    ) -> Result<ZoomOAuthPendingGrant, ZoomError> {
        request.validate()?;
        self.setup_live_blocked_account(&request.live_account_request())
            .await?;
        let setup_id = random_zoom_oauth_token()?;
        let state = random_zoom_oauth_token()?;
        let authorization_url = zoom_authorization_url(
            &request.authorization_endpoint(),
            &request.client_id,
            &request.redirect_uri,
            &request.scopes,
            &state,
        )?;
        Ok(ZoomOAuthPendingGrant {
            setup_id,
            account_id: request.account_id.trim().to_owned(),
            authorization_url,
            state,
            request: request.clone(),
        })
    }

    pub async fn list_accounts(
        &self,
        include_removed: bool,
    ) -> Result<ZoomAccountListResponse, ZoomError> {
        let mut accounts = self
            .provider_account_store
            .list()
            .await?
            .into_iter()
            .filter(|account| account.provider_kind.is_zoom())
            .map(ZoomAccount::from)
            .filter(|account| include_removed || account.lifecycle_state != "removed")
            .collect::<Vec<_>>();
        accounts.sort_by(|left, right| left.display_name.cmp(&right.display_name));
        Ok(ZoomAccountListResponse { items: accounts })
    }

    pub async fn runtime_status(&self, account_id: &str) -> Result<ZoomRuntimeStatus, ZoomError> {
        let account = self.zoom_account(account_id).await?;
        Ok(runtime_status_from_account(account))
    }

    pub async fn start_runtime(
        &self,
        request: &ZoomRuntimeStartRequest,
    ) -> Result<ZoomRuntimeStatus, ZoomError> {
        let account_id = validate_account_id(&request.account_id)?;
        let account = self.zoom_account(&account_id).await?;
        let mut config = account.config.clone();
        let live_blocked = account.auth_shape != "fixture";
        let live_authorized = live_blocked && zoom_account_is_authorized(&account);
        config["lifecycle_state"] = json!(if live_authorized {
            "running"
        } else if live_blocked {
            "blocked"
        } else {
            "running"
        });
        config["runtime_kind"] = json!(if live_authorized {
            ZOOM_LIVE_AUTHORIZED_RUNTIME_KIND
        } else if live_blocked {
            "zoom_live_blocked_runtime"
        } else {
            ZOOM_RUNTIME_KIND
        });
        config["runtime_blockers"] = if live_authorized {
            let mut blockers = account
                .config
                .get("runtime_blockers")
                .and_then(|value| value.as_array())
                .cloned()
                .unwrap_or_default();
            blockers.retain(|value| value.as_str() != Some("zoom_provider_workers_not_enabled"));
            json!(blockers)
        } else if live_blocked {
            json!(["zoom_live_authorization_required"])
        } else {
            json!([])
        };
        config["last_runtime_action"] = json!({
            "action": "start",
            "force": request.force,
            "observed_at": Utc::now(),
        });
        let updated = self.update_account_config(&account_id, config).await?;
        let status = runtime_status_from_account(updated);
        self.publish_runtime_status_event(&status, "zoom.runtime.start_requested")
            .await?;
        Ok(status)
    }

    pub async fn stop_runtime(
        &self,
        request: &ZoomRuntimeStopRequest,
    ) -> Result<ZoomRuntimeStatus, ZoomError> {
        let account_id = validate_account_id(&request.account_id)?;
        let account = self.zoom_account(&account_id).await?;
        let mut config = account.config.clone();
        if account.lifecycle_state != "removed" {
            config["lifecycle_state"] = json!("stopped");
        }
        config["last_runtime_action"] = json!({
            "action": "stop",
            "reason": &request.reason,
            "observed_at": Utc::now(),
        });
        let updated = self.update_account_config(&account_id, config).await?;
        let status = runtime_status_from_account(updated);
        self.publish_runtime_status_event(&status, "zoom.runtime.stop_requested")
            .await?;
        Ok(status)
    }

    pub async fn remove_runtime(
        &self,
        request: &ZoomRuntimeRemoveRequest,
    ) -> Result<ZoomRuntimeRemoveResponse, ZoomError> {
        let account_id = validate_account_id(&request.account_id)?;
        let account = self.zoom_account(&account_id).await?;
        let removed_at = Utc::now();
        let mut config = account.config.clone();
        config["lifecycle_state"] = json!("removed");
        config["removed_at"] = json!(removed_at);
        config["remove_reason"] = json!(&request.reason);
        let updated = self.update_account_config(&account_id, config).await?;
        let status = runtime_status_from_account(updated);
        self.publish_runtime_status_event(&status, "zoom.runtime.remove_requested")
            .await?;
        Ok(ZoomRuntimeRemoveResponse {
            account_id,
            provider_kind: status.provider_kind,
            removed: true,
            removed_at,
        })
    }

    pub async fn observe_meeting(
        &self,
        request: &ZoomMeetingObservationRequest,
    ) -> Result<ZoomMeetingIngestResult, ZoomError> {
        request.validate()?;
        self.ensure_zoom_account(&request.account_id).await?;
        let observed_at = request.started_at.unwrap_or_else(Utc::now);
        let call_id = stable_zoom_call_id(&request.account_id, &request.meeting_id);
        let call: NewProviderCall = request.into_call(call_id.clone(), observed_at);
        self.call_store.upsert_call(&call).await?;

        let event_id = zoom_event_id(
            "meeting",
            &request.account_id,
            &request.event_subject_id(),
            request.observation_id.as_deref(),
        );
        let event = zoom_event(
            event_id.clone(),
            zoom_event_types::MEETING_OBSERVED,
            observed_at,
            json!({
                "provider": "zoom",
                "provider_kind": ZOOM_PROVIDER_KIND_STR,
                "account_id": &request.account_id,
            }),
            json!({
                "kind": "zoom_meeting",
                "meeting_id": &request.meeting_id,
                "meeting_uuid": &request.meeting_uuid,
                "call_id": &call_id,
            }),
            sanitize_zoom_payload(json!({
                "account_id": &request.account_id,
                "meeting_id": &request.meeting_id,
                "meeting_uuid": &request.meeting_uuid,
                "topic": &request.topic,
                "host_email": &request.host_email,
                "join_url": &request.join_url,
                "started_at": &request.started_at,
                "ended_at": &request.ended_at,
                "duration_seconds": &request.duration_seconds,
                "participants": &request.participants,
                "recording_refs": &request.recording_refs,
                "transcript_ref": &request.transcript_ref,
                "metadata": &request.metadata,
            })),
            json!({
                "source": "zoom_runtime_bridge",
                "observation_id": &request.observation_id,
                "stored_as": "provider_call_projection",
            }),
            request.causation_id.clone(),
            request
                .correlation_id
                .clone()
                .or_else(|| request.observation_id.clone())
                .or_else(|| Some(event_id.clone())),
        )?;
        self.append_and_broadcast(&event).await?;

        Ok(ZoomMeetingIngestResult {
            call_id,
            account_id: request.account_id.trim().to_owned(),
            meeting_id: request.meeting_id.trim().to_owned(),
            event_id,
            status: "recorded".to_owned(),
        })
    }

    pub async fn observe_recording(
        &self,
        request: &ZoomRecordingObservationRequest,
    ) -> Result<ZoomRecordingIngestResult, ZoomError> {
        request.validate()?;
        self.ensure_zoom_account(&request.account_id).await?;
        let observed_at = request.recording.recorded_at.unwrap_or_else(Utc::now);
        let event_id = zoom_event_id(
            "recording",
            &request.account_id,
            &format!(
                "{}:{}",
                request.meeting_id.trim(),
                request.recording.recording_id.trim()
            ),
            request.observation_id.as_deref(),
        );
        let event = zoom_event(
            event_id.clone(),
            zoom_event_types::RECORDING_OBSERVED,
            observed_at,
            json!({
                "provider": "zoom",
                "provider_kind": ZOOM_PROVIDER_KIND_STR,
                "account_id": &request.account_id,
            }),
            json!({
                "kind": "zoom_recording",
                "meeting_id": &request.meeting_id,
                "recording_id": &request.recording.recording_id,
            }),
            sanitize_zoom_payload(json!({
                "account_id": &request.account_id,
                "meeting_id": &request.meeting_id,
                "recording": &request.recording,
                "metadata": &request.metadata,
            })),
            json!({
                "source": "zoom_runtime_bridge",
                "observation_id": &request.observation_id,
            }),
            request.causation_id.clone(),
            request
                .correlation_id
                .clone()
                .or_else(|| request.observation_id.clone())
                .or_else(|| Some(event_id.clone())),
        )?;
        self.append_and_broadcast(&event).await?;

        Ok(ZoomRecordingIngestResult {
            account_id: request.account_id.trim().to_owned(),
            meeting_id: request.meeting_id.trim().to_owned(),
            recording_id: request.recording.recording_id.trim().to_owned(),
            event_id,
            status: "recorded".to_owned(),
        })
    }

    pub async fn import_recording_media_download(
        &self,
        request: &ZoomRecordingMediaDownloadRequest,
        bearer_token: Option<&str>,
    ) -> Result<ZoomRecordingMediaImportResult, ZoomError> {
        request.validate()?;
        self.ensure_zoom_account(&request.account_id).await?;
        let imported = self
            .store_recording_media_download(
                request,
                bearer_token
                    .map(str::trim)
                    .filter(|value| !value.is_empty()),
            )
            .await?;
        Ok(ZoomRecordingMediaImportResult {
            attachment_id: imported.attachment_id,
            blob_id: imported.blob_id,
            account_id: request.account_id.trim().to_owned(),
            meeting_id: request.meeting_id.trim().to_owned(),
            recording_id: request.recording.recording_id.trim().to_owned(),
            content_type: imported.content_type,
            scan_status: imported.scan_status.as_str().to_owned(),
            storage_kind: imported.storage_kind,
            storage_path: imported.storage_path,
            status: "recorded".to_owned(),
        })
    }

    pub async fn list_recording_imports(
        &self,
        account_id: &str,
        limit: i64,
    ) -> Result<ZoomRecordingImportAuditResponse, ZoomError> {
        let account_id = validate_account_id(account_id)?;
        self.ensure_zoom_account(&account_id).await?;
        let items = self
            .imported_attachment_store
            .list_imported_attachment_records(&account_id, "zoom_recording_download", limit)
            .await?;
        Ok(ZoomRecordingImportAuditResponse {
            account_id,
            items: items
                .into_iter()
                .map(zoom_recording_import_audit_item)
                .collect(),
        })
    }

    pub async fn remove_recording_import(
        &self,
        account_id: &str,
        attachment_id: &str,
        request: &ZoomRecordingImportRemoveRequest,
    ) -> Result<ZoomRecordingImportRemoveResponse, ZoomError> {
        let account_id = validate_account_id(account_id)?;
        let attachment_id = validate_required("attachment_id", attachment_id)?;
        self.ensure_zoom_account(&account_id).await?;
        let removed_at = Utc::now();
        let Some(removed) = self
            .imported_attachment_store
            .remove_imported_attachment_record(
                &attachment_id,
                &account_id,
                "zoom_recording_download",
            )
            .await?
        else {
            return Err(ZoomError::InvalidRequest(format!(
                "Zoom recording import `{attachment_id}` was not found for account `{account_id}`"
            )));
        };

        let imported = removed.imported_attachment;
        let blob_file_removed =
            if removed.blob_metadata_removed && imported.storage_kind == "local_fs" {
                delete_local_blob(DEFAULT_MAIL_SYNC_BLOB_ROOT, &imported.storage_path).await?
            } else {
                false
            };

        self.publish_recording_import_removed_event(
            &account_id,
            &imported,
            removed.blob_metadata_removed,
            blob_file_removed,
            removed_at,
            request.reason(),
        )
        .await?;

        Ok(ZoomRecordingImportRemoveResponse {
            account_id,
            attachment_id: imported.attachment_id,
            blob_id: imported.blob_id,
            recording_id: imported
                .metadata
                .get("recording_id")
                .and_then(Value::as_str)
                .map(str::to_owned),
            removed: true,
            blob_metadata_removed: removed.blob_metadata_removed,
            blob_file_removed,
            removed_at,
        })
    }

    pub async fn list_audit_events(
        &self,
        account_id: &str,
        limit: i64,
    ) -> Result<ZoomAuditEventResponse, ZoomError> {
        let account_id = validate_account_id(account_id)?;
        self.ensure_zoom_account(&account_id).await?;
        let limit = limit.clamp(1, 100);
        let rows = sqlx::query(
            r#"
            SELECT
                position,
                event_id,
                event_type,
                occurred_at,
                source,
                subject,
                payload,
                provenance,
                correlation_id
            FROM event_log
            WHERE source ->> 'provider' = 'zoom'
              AND source ->> 'account_id' = $1
            ORDER BY occurred_at DESC, position DESC
            LIMIT $2
            "#,
        )
        .bind(&account_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        let items = rows
            .into_iter()
            .map(|row| -> Result<ZoomAuditEventItem, ZoomError> {
                let subject: Value = row.try_get("subject")?;
                Ok(ZoomAuditEventItem {
                    position: row.try_get("position")?,
                    event_id: row.try_get("event_id")?,
                    event_type: row.try_get("event_type")?,
                    occurred_at: row.try_get("occurred_at")?,
                    subject_kind: subject
                        .get("kind")
                        .and_then(Value::as_str)
                        .map(str::to_owned),
                    subject_entity_id: subject
                        .get("entity_id")
                        .and_then(Value::as_str)
                        .map(str::to_owned),
                    correlation_id: row.try_get("correlation_id")?,
                    source: row.try_get("source")?,
                    subject,
                    payload: row.try_get("payload")?,
                    provenance: row.try_get("provenance")?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ZoomAuditEventResponse { account_id, items })
    }

    pub async fn cleanup_retention(
        &self,
        account_id: &str,
        request: &ZoomRetentionCleanupRequest,
    ) -> Result<ZoomRetentionCleanupResponse, ZoomError> {
        let account_id = validate_account_id(account_id)?;
        request.validate()?;
        self.ensure_zoom_account(&account_id).await?;
        let checked_at = Utc::now();
        let mut items = Vec::new();
        let mut recordings_removed = 0usize;
        let mut transcripts_removed = 0usize;
        if request.remove_recordings {
            let expired = self
                .imported_attachment_store
                .list_expired_imported_attachment_records(
                    &account_id,
                    "zoom_recording_download",
                    request.limit(),
                )
                .await?;
            for imported in expired {
                let removed_at = Utc::now();
                let Some(removed) = self
                    .imported_attachment_store
                    .remove_imported_attachment_record(
                        &imported.attachment_id,
                        &account_id,
                        "zoom_recording_download",
                    )
                    .await?
                else {
                    continue;
                };

                let imported = removed.imported_attachment;
                let blob_file_removed = if removed.blob_metadata_removed
                    && imported.storage_kind == "local_fs"
                {
                    delete_local_blob(DEFAULT_MAIL_SYNC_BLOB_ROOT, &imported.storage_path).await?
                } else {
                    false
                };
                self.publish_recording_import_removed_event(
                    &account_id,
                    &imported,
                    removed.blob_metadata_removed,
                    blob_file_removed,
                    removed_at,
                    Some("retention_policy_expired".to_owned()),
                )
                .await?;
                recordings_removed += 1;
                items.push(ZoomRetentionCleanupItem {
                    evidence_kind: "recording_import".to_owned(),
                    entity_id: imported.attachment_id.clone(),
                    call_id: None,
                    meeting_id: imported
                        .metadata
                        .get("meeting_id")
                        .and_then(Value::as_str)
                        .map(str::to_owned),
                    recording_id: imported
                        .metadata
                        .get("recording_id")
                        .and_then(Value::as_str)
                        .map(str::to_owned),
                    transcript_id: None,
                    expires_at: parse_optional_datetime(
                        imported
                            .metadata
                            .get("retention_policy")
                            .and_then(|value| value.get("expires_at")),
                    ),
                    removed_at,
                });
            }
        }

        if request.remove_transcripts {
            let expired = self
                .call_store
                .list_expired_transcripts(&account_id, "zoom", request.limit())
                .await?;
            for transcript in expired {
                let removed_at = Utc::now();
                let Some(removed) = self
                    .call_store
                    .remove_transcript(&transcript.transcript_id)
                    .await?
                else {
                    continue;
                };
                self.publish_transcript_removed_event(&removed, removed_at)
                    .await?;
                transcripts_removed += 1;
                items.push(ZoomRetentionCleanupItem {
                    evidence_kind: "transcript".to_owned(),
                    entity_id: removed.transcript_id.clone(),
                    call_id: Some(removed.call_id.clone()),
                    meeting_id: removed
                        .provenance
                        .get("meeting_id")
                        .and_then(Value::as_str)
                        .map(str::to_owned),
                    recording_id: removed
                        .provenance
                        .get("source_recording_ref")
                        .and_then(Value::as_str)
                        .map(str::to_owned),
                    transcript_id: Some(removed.transcript_id.clone()),
                    expires_at: parse_optional_datetime(
                        removed
                            .provenance
                            .get("retention_policy")
                            .and_then(|value| value.get("expires_at")),
                    ),
                    removed_at,
                });
            }
        }

        let response = ZoomRetentionCleanupResponse {
            account_id: account_id.clone(),
            checked_at,
            recordings_removed,
            transcripts_removed,
            items,
        };
        self.publish_retention_cleanup_completed_event(&response)
            .await?;
        Ok(response)
    }

    pub async fn observe_transcript(
        &self,
        request: &ZoomTranscriptObservationRequest,
    ) -> Result<ZoomTranscriptIngestResult, ZoomError> {
        request.validate()?;
        self.ensure_zoom_account(&request.account_id).await?;
        let observed_at = Utc::now();
        let call_id = stable_zoom_call_id(&request.account_id, &request.meeting_id);
        self.ensure_placeholder_call(request, call_id.clone(), observed_at)
            .await?;
        let retention_policy = self
            .resolved_retention_policy(ZOOM_TRANSCRIPT_RETENTION_DAYS_SETTING_KEY, observed_at)
            .await?;
        let transcript = NewCallTranscript {
            transcript_id: request.transcript_id.trim().to_owned(),
            call_id: call_id.clone(),
            account_id: request.account_id.trim().to_owned(),
            provider_chat_id: request.provider_chat_id(),
            transcript_status: TranscriptStatus::Succeeded,
            stt_provider: "zoom-cloud-transcript".to_owned(),
            source_audio_ref: request.source_recording_ref.clone(),
            language_code: request.language_code.clone(),
            transcript_text: request.transcript_text.clone(),
            segments: request.segments.clone(),
            provenance: sanitize_zoom_payload(json!({
                "provider": "zoom",
                "provider_kind": ZOOM_PROVIDER_KIND_STR,
                "meeting_id": &request.meeting_id,
                "meeting_uuid": &request.meeting_uuid,
                "retention_policy": retention_policy,
                "metadata": &request.metadata,
            })),
        };
        self.call_store.upsert_transcript(&transcript).await?;

        let event_id = zoom_event_id(
            "transcript",
            &request.account_id,
            &request.transcript_id,
            request.observation_id.as_deref(),
        );
        let event = zoom_event(
            event_id.clone(),
            zoom_event_types::TRANSCRIPT_OBSERVED,
            observed_at,
            json!({
                "provider": "zoom",
                "provider_kind": ZOOM_PROVIDER_KIND_STR,
                "account_id": &request.account_id,
            }),
            json!({
                "kind": "zoom_transcript",
                "meeting_id": &request.meeting_id,
                "transcript_id": &request.transcript_id,
                "call_id": &call_id,
            }),
            sanitize_zoom_payload(json!({
                "account_id": &request.account_id,
                "meeting_id": &request.meeting_id,
                "meeting_uuid": &request.meeting_uuid,
                "transcript_id": &request.transcript_id,
                "source_recording_ref": &request.source_recording_ref,
                "language_code": &request.language_code,
                "segments": &request.segments,
                "retention_policy": retention_policy,
                "metadata": &request.metadata,
            })),
            json!({
                "source": "zoom_runtime_bridge",
                "observation_id": &request.observation_id,
                "stored_as": "call_transcript",
            }),
            request.causation_id.clone(),
            request
                .correlation_id
                .clone()
                .or_else(|| request.observation_id.clone())
                .or_else(|| Some(event_id.clone())),
        )?;
        self.append_and_broadcast(&event).await?;

        Ok(ZoomTranscriptIngestResult {
            transcript_id: request.transcript_id.trim().to_owned(),
            call_id,
            account_id: request.account_id.trim().to_owned(),
            meeting_id: request.meeting_id.trim().to_owned(),
            event_id,
            status: "recorded".to_owned(),
        })
    }

    pub async fn import_transcript_file(
        &self,
        request: &ZoomTranscriptFileImportRequest,
    ) -> Result<ZoomTranscriptFileImportResult, ZoomError> {
        request.validate()?;
        let parsed = request.parse_file()?;
        let transcript_request = request.to_transcript_observation(&parsed);
        let result = self.observe_transcript(&transcript_request).await?;
        Ok(ZoomTranscriptFileImportResult::from_ingest(
            result,
            parsed.format,
            parsed.parsed_segment_count,
        ))
    }

    pub async fn import_transcript_file_download(
        &self,
        request: &ZoomTranscriptFileImportRequest,
        download_url: &str,
        download_token: Option<&str>,
    ) -> Result<ZoomTranscriptFileImportResult, ZoomError> {
        let download_url = validate_required("download_url", download_url)?;
        let mut response = self
            .http
            .get(download_url)
            .header(ACCEPT, "text/vtt, application/x-subrip, text/plain, */*");
        if let Some(token) = download_token
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            response = response.header(AUTHORIZATION, format!("Bearer {token}"));
        }
        let response = response.send().await?.error_for_status()?;
        let response_content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let body = response.bytes().await?;
        if body.len() > MAX_TRANSCRIPT_FILE_TEXT_BYTES {
            return Err(ZoomError::InvalidRequest(format!(
                "downloaded transcript file must be at most {MAX_TRANSCRIPT_FILE_TEXT_BYTES} bytes"
            )));
        }
        let file_text = String::from_utf8(body.to_vec()).map_err(|_| {
            ZoomError::InvalidRequest(
                "downloaded Zoom transcript file must be valid UTF-8 text".to_owned(),
            )
        })?;

        let mut import_request = request.clone();
        import_request.file_text = file_text;
        if import_request
            .content_type
            .as_ref()
            .is_none_or(|value| value.trim().is_empty())
        {
            import_request.content_type = response_content_type;
        }
        self.import_transcript_file(&import_request).await
    }

    pub async fn complete_oauth(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        mut pending: ZoomOAuthPendingGrant,
        authorization_code: &str,
        external_account_id: Option<&str>,
    ) -> Result<ZoomAuthorizationResult, ZoomError> {
        validate_account_id(&pending.account_id)?;
        let authorization_code = validate_required("authorization_code", authorization_code)?;
        if let Some(external_account_id) = external_account_id
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            pending.request.external_account_id = external_account_id.to_owned();
        }
        let account = self.zoom_account(&pending.account_id).await?;
        if account.auth_shape != ZoomAuthShape::OAuthUser.as_str() {
            return Err(ZoomError::InvalidRequest(format!(
                "Zoom account `{}` is not an oauth_user account",
                account.account_id
            )));
        }
        let client_secret_ref = self
            .store_or_resolve_client_secret(
                secret_store,
                vault,
                &account,
                pending.request.client_secret.as_deref(),
                pending.request.client_secret_ref.as_deref(),
                "oauth_user",
            )
            .await?;
        let client_secret = resolve_client_secret(
            vault,
            pending.request.client_secret.as_deref(),
            client_secret_ref.as_deref(),
        )?;
        let token = self
            .exchange_oauth_authorization_code(
                pending.request.token_endpoint(),
                pending.request.client_id.trim(),
                &client_secret,
                &authorization_code,
                pending.request.redirect_uri.trim(),
            )
            .await?;
        let refresh_token = token.refresh_token.clone().ok_or_else(|| {
            ZoomError::InvalidRequest(
                "Zoom OAuth token response did not include refresh_token".to_owned(),
            )
        })?;
        let token_secret_ref = zoom_oauth_token_secret_ref(&account.account_id);
        let authorized_at = Utc::now();
        let expires_at = zoom_oauth_expires_at(token.expires_in);
        let token_bundle = ZoomOAuthTokenBundle {
            token_url: pending.request.token_endpoint(),
            client_id: pending.request.client_id.trim().to_owned(),
            client_secret_ref: client_secret_ref.clone(),
            auth_shape: ZoomAuthShape::OAuthUser.as_str().to_owned(),
            zoom_account_id: None,
            access_token: token.access_token,
            refresh_token: Some(refresh_token),
            expires_at,
            token_type: token.token_type,
            scope: token.scope,
        };
        let secret_metadata = zoom_oauth_secret_metadata(
            &account,
            ZoomAuthShape::OAuthUser.as_str(),
            Some(expires_at),
            &pending.request.metadata,
        );
        self.store_oauth_token_bundle(
            secret_store,
            vault,
            &account,
            &token_secret_ref,
            &token_bundle,
            &secret_metadata,
        )
        .await?;
        self.provider_secret_binding_store
            .bind(&NewProviderAccountSecretBinding::new(
                &account.account_id,
                ProviderAccountSecretPurpose::ZoomOauthToken,
                &token_secret_ref,
            ))
            .await?;
        let updated = self
            .mark_account_authorized(
                &account,
                ZoomAuthorizedAccountUpdate {
                    auth_shape: ZoomAuthShape::OAuthUser.as_str(),
                    token_secret_ref: &token_secret_ref,
                    client_secret_ref: client_secret_ref.as_deref(),
                    expires_at: Some(expires_at),
                    metadata: json!({
                    "oauth": {
                        "requested_scopes": &pending.request.scopes,
                        "token_endpoint": pending.request.token_endpoint(),
                        "redirect_uri": pending.request.redirect_uri.trim(),
                        "secret_material": "excluded",
                    }
                    }),
                    authorized_at,
                },
            )
            .await?;
        self.publish_authorization_completed_event(
            &updated,
            authorized_at,
            &pending.request.metadata,
            "oauth_complete",
        )
        .await?;

        Ok(authorization_result(
            updated,
            token_secret_ref,
            client_secret_ref,
            authorized_at,
        ))
    }

    pub async fn authorize_server_to_server(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &ZoomServerToServerAuthorizeRequest,
    ) -> Result<ZoomAuthorizationResult, ZoomError> {
        request.validate()?;
        let account = self.zoom_account(&request.account_id).await?;
        if account.auth_shape != ZoomAuthShape::ServerToServer.as_str() {
            return Err(ZoomError::InvalidRequest(format!(
                "Zoom account `{}` is not a server_to_server account",
                account.account_id
            )));
        }
        let client_secret_ref = self
            .store_or_resolve_client_secret(
                secret_store,
                vault,
                &account,
                request.client_secret.as_deref(),
                request.client_secret_ref.as_deref(),
                "server_to_server",
            )
            .await?;
        let client_secret = resolve_client_secret(
            vault,
            request.client_secret.as_deref(),
            client_secret_ref.as_deref(),
        )?;
        let zoom_account_id = request
            .zoom_account_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(account.external_account_id.trim())
            .to_owned();
        validate_required("zoom_account_id", &zoom_account_id)?;
        let token = self
            .exchange_server_to_server_token(
                request.token_endpoint(),
                request.client_id.trim(),
                &client_secret,
                &zoom_account_id,
            )
            .await?;
        let token_secret_ref = zoom_oauth_token_secret_ref(&account.account_id);
        let authorized_at = Utc::now();
        let expires_at = zoom_oauth_expires_at(token.expires_in);
        let token_bundle = ZoomOAuthTokenBundle {
            token_url: request.token_endpoint(),
            client_id: request.client_id.trim().to_owned(),
            client_secret_ref: client_secret_ref.clone(),
            auth_shape: ZoomAuthShape::ServerToServer.as_str().to_owned(),
            zoom_account_id: Some(zoom_account_id.clone()),
            access_token: token.access_token,
            refresh_token: token.refresh_token,
            expires_at,
            token_type: token.token_type,
            scope: token.scope,
        };
        let secret_metadata = zoom_oauth_secret_metadata(
            &account,
            ZoomAuthShape::ServerToServer.as_str(),
            Some(expires_at),
            &request.metadata,
        );
        self.store_oauth_token_bundle(
            secret_store,
            vault,
            &account,
            &token_secret_ref,
            &token_bundle,
            &secret_metadata,
        )
        .await?;
        self.provider_secret_binding_store
            .bind(&NewProviderAccountSecretBinding::new(
                &account.account_id,
                ProviderAccountSecretPurpose::ZoomOauthToken,
                &token_secret_ref,
            ))
            .await?;
        let updated = self
            .mark_account_authorized(
                &account,
                ZoomAuthorizedAccountUpdate {
                    auth_shape: ZoomAuthShape::ServerToServer.as_str(),
                    token_secret_ref: &token_secret_ref,
                    client_secret_ref: client_secret_ref.as_deref(),
                    expires_at: Some(expires_at),
                    metadata: json!({
                    "server_to_server": {
                        "zoom_account_id": zoom_account_id,
                        "token_endpoint": request.token_endpoint(),
                        "secret_material": "excluded",
                    }
                    }),
                    authorized_at,
                },
            )
            .await?;
        self.publish_authorization_completed_event(
            &updated,
            authorized_at,
            &request.metadata,
            "server_to_server_authorize",
        )
        .await?;

        Ok(authorization_result(
            updated,
            token_secret_ref,
            client_secret_ref,
            authorized_at,
        ))
    }

    pub async fn refresh_token(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &ZoomTokenRefreshRequest,
    ) -> Result<ZoomTokenRefreshResult, ZoomError> {
        self.refresh_token_with_provenance(secret_store, vault, request, "explicit_refresh")
            .await
    }

    pub(super) async fn refresh_token_with_provenance(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &ZoomTokenRefreshRequest,
        action: &str,
    ) -> Result<ZoomTokenRefreshResult, ZoomError> {
        request.validate()?;
        let account = self.zoom_account(&request.account_id).await?;
        if !zoom_account_is_authorized(&account) {
            return Err(ZoomError::InvalidRequest(format!(
                "Zoom account `{}` is not authorized",
                account.account_id
            )));
        }
        let token_binding = self
            .provider_secret_binding_store
            .get_for_account(
                &account.account_id,
                ProviderAccountSecretPurpose::ZoomOauthToken,
            )
            .await?
            .ok_or_else(|| {
                ZoomError::InvalidRequest(format!(
                    "Zoom account `{}` has no zoom_oauth_token secret binding",
                    account.account_id
                ))
            })?;
        let token_reference = secret_store
            .secret_reference(&token_binding.secret_ref)
            .await?
            .ok_or_else(|| {
                ZoomError::InvalidRequest(format!(
                    "Zoom token secret reference `{}` was not found",
                    token_binding.secret_ref
                ))
            })?;
        if token_reference.secret_kind != SecretKind::OauthToken
            || token_reference.store_kind != SecretStoreKind::HostVault
        {
            return Err(ZoomError::InvalidRequest(format!(
                "Zoom token secret reference `{}` must be an oauth_token in host_vault",
                token_reference.secret_ref
            )));
        }

        let mut bundle: ZoomOAuthTokenBundle =
            serde_json::from_str(&vault.read_secret(&token_reference.secret_ref)?)?;
        let checked_at = Utc::now();
        let refresh_strategy = if account.auth_shape == ZoomAuthShape::OAuthUser.as_str() {
            "oauth_refresh_token"
        } else if account.auth_shape == ZoomAuthShape::ServerToServer.as_str() {
            "server_to_server_account_credentials"
        } else {
            return Err(ZoomError::InvalidRequest(format!(
                "Zoom account `{}` is not a live account",
                account.account_id
            )));
        };

        if !request.force
            && bundle.expires_at
                > checked_at + chrono::TimeDelta::seconds(request.refresh_expiring_within_seconds())
        {
            let result = ZoomTokenRefreshResult {
                account_id: account.account_id,
                provider_kind: account.provider_kind,
                auth_shape: account.auth_shape,
                token_secret_ref: token_reference.secret_ref,
                refreshed: false,
                refresh_strategy: refresh_strategy.to_owned(),
                status: "skipped_not_expired".to_owned(),
                expires_at: bundle.expires_at,
                checked_at,
                secret_kind: SecretKind::OauthToken.as_str().to_owned(),
                store_kind: SecretStoreKind::HostVault.as_str().to_owned(),
            };
            self.publish_token_refresh_event(&result, request.force, action, None)
                .await?;
            return Ok(result);
        }

        let client_secret_ref = bundle.client_secret_ref.clone().ok_or_else(|| {
            ZoomError::InvalidRequest(format!(
                "Zoom account `{}` has no client secret reference in token bundle",
                account.account_id
            ))
        })?;
        let client_secret = vault.read_secret(&client_secret_ref)?;
        let refreshed = match if account.auth_shape == ZoomAuthShape::OAuthUser.as_str() {
            let refresh_token = bundle.refresh_token.clone().ok_or_else(|| {
                ZoomError::InvalidRequest(format!(
                    "Zoom account `{}` has no refresh token in token bundle",
                    account.account_id
                ))
            })?;
            self.exchange_oauth_refresh_token(
                bundle.token_url.clone(),
                &bundle.client_id,
                &client_secret,
                &refresh_token,
            )
            .await
        } else {
            let zoom_account_id = bundle
                .zoom_account_id
                .clone()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| account.external_account_id.trim().to_owned());
            self.exchange_server_to_server_token(
                bundle.token_url.clone(),
                &bundle.client_id,
                &client_secret,
                &zoom_account_id,
            )
            .await
        } {
            Ok(refreshed) => refreshed,
            Err(error) => {
                self.mark_account_token_refresh_failed(
                    &account,
                    refresh_strategy,
                    request.force,
                    checked_at,
                    &error.to_string(),
                )
                .await?;
                self.publish_token_refresh_failure_event(
                    &account,
                    refresh_strategy,
                    request.force,
                    checked_at,
                    &error.to_string(),
                    action,
                )
                .await?;
                return Err(error);
            }
        };

        bundle.access_token = refreshed.access_token;
        if let Some(refresh_token) = refreshed.refresh_token {
            bundle.refresh_token = Some(refresh_token);
        }
        bundle.expires_at = zoom_oauth_expires_at(refreshed.expires_in);
        bundle.token_type = refreshed.token_type;
        if refreshed.scope.is_some() {
            bundle.scope = refreshed.scope;
        }

        let refreshed_at = Utc::now();
        let secret_metadata = zoom_oauth_secret_metadata(
            &account,
            &account.auth_shape,
            Some(bundle.expires_at),
            &json!({
                "refresh": {
                    "strategy": refresh_strategy,
                    "force": request.force,
                    "refreshed_at": refreshed_at,
                    "secret_material": "excluded",
                }
            }),
        );
        self.store_oauth_token_bundle(
            secret_store,
            vault,
            &account,
            &token_reference.secret_ref,
            &bundle,
            &secret_metadata,
        )
        .await?;
        self.mark_account_token_refreshed(
            &account,
            &bundle,
            refresh_strategy,
            &token_reference.secret_ref,
            request.force,
            refreshed_at,
        )
        .await?;
        let result = ZoomTokenRefreshResult {
            account_id: account.account_id,
            provider_kind: account.provider_kind,
            auth_shape: account.auth_shape,
            token_secret_ref: token_reference.secret_ref,
            refreshed: true,
            refresh_strategy: refresh_strategy.to_owned(),
            status: "refreshed".to_owned(),
            expires_at: bundle.expires_at,
            checked_at: refreshed_at,
            secret_kind: SecretKind::OauthToken.as_str().to_owned(),
            store_kind: SecretStoreKind::HostVault.as_str().to_owned(),
        };
        self.publish_token_refresh_event(&result, request.force, action, None)
            .await?;
        Ok(result)
    }

    pub async fn maintain_tokens(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &ZoomTokenMaintenanceRequest,
    ) -> Result<ZoomTokenMaintenanceResult, ZoomError> {
        request.validate()?;
        let checked_at = Utc::now();
        let accounts = if let Some(account_id) = request
            .account_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            vec![self.zoom_account(account_id).await?]
        } else {
            self.list_accounts(false).await?.items
        };

        let mut items = Vec::new();
        for account in accounts {
            if account.auth_shape == ZoomAuthShape::Fixture.as_str()
                || account.lifecycle_state == "removed"
            {
                continue;
            }
            if !zoom_account_is_authorized(&account) {
                if request.account_id.is_some() {
                    items.push(ZoomTokenMaintenanceItem {
                        account_id: account.account_id,
                        provider_kind: account.provider_kind,
                        auth_shape: account.auth_shape,
                        status: "skipped_not_authorized".to_owned(),
                        refreshed: false,
                        expires_at: None,
                        error: None,
                    });
                }
                continue;
            }

            let refresh_request = request.refresh_request_for(&account.account_id);
            match self
                .refresh_token_with_provenance(
                    secret_store,
                    vault,
                    &refresh_request,
                    "token_maintenance",
                )
                .await
            {
                Ok(result) => items.push(ZoomTokenMaintenanceItem {
                    account_id: result.account_id,
                    provider_kind: result.provider_kind,
                    auth_shape: result.auth_shape,
                    status: result.status,
                    refreshed: result.refreshed,
                    expires_at: Some(result.expires_at),
                    error: None,
                }),
                Err(error) => items.push(ZoomTokenMaintenanceItem {
                    account_id: account.account_id,
                    provider_kind: account.provider_kind,
                    auth_shape: account.auth_shape,
                    status: "failed".to_owned(),
                    refreshed: false,
                    expires_at: None,
                    error: Some(error.to_string()),
                }),
            }
        }

        let refreshed_count = items.iter().filter(|item| item.refreshed).count();
        let failed_count = items.iter().filter(|item| item.status == "failed").count();
        let skipped_count = items
            .iter()
            .filter(|item| !item.refreshed && item.status != "failed")
            .count();
        Ok(ZoomTokenMaintenanceResult {
            checked_count: items.len(),
            refreshed_count,
            skipped_count,
            failed_count,
            refresh_expiring_within_seconds: request.refresh_expiring_within_seconds(),
            checked_at,
            items,
        })
    }

    pub async fn sync_recordings(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &ZoomRecordingSyncRequest,
        allow_remote_recording_downloads: bool,
        allow_remote_transcript_downloads: bool,
    ) -> Result<ZoomRecordingSyncResult, ZoomError> {
        request.validate()?;
        let account = self.zoom_account(&request.account_id).await?;
        if !zoom_account_is_authorized(&account) {
            return Err(ZoomError::InvalidRequest(format!(
                "Zoom account `{}` is not authorized",
                account.account_id
            )));
        }
        let token_bundle = self
            .load_token_bundle(secret_store, vault, &account)
            .await?;
        let user_id = provider_sync_user_id(&account, request)?;
        let access_token = token_bundle.access_token.trim().to_owned();
        if access_token.is_empty() {
            return Err(ZoomError::InvalidRequest(format!(
                "Zoom account `{}` has an empty access token bundle",
                account.account_id
            )));
        }
        let sync_result: Result<ZoomRecordingSyncResult, ZoomError> = async {
            let mut next_page_token: Option<String> = None;
            let mut meetings_seen = 0usize;
            let mut meetings_recorded = 0usize;
            let mut recordings_recorded = 0usize;
            let mut media_downloads_recorded = 0usize;
            let mut transcripts_recorded = 0usize;
            let mut failures = Vec::new();
            let max_meetings = request.max_meetings();
            let api_base_url = request.api_base_url();

            while meetings_seen < max_meetings {
                let remaining = max_meetings - meetings_seen;
                let page_size = request.page_size().min(remaining);
                let response = self
                    .fetch_zoom_recordings_page(
                        &api_base_url,
                        &user_id,
                        &request.from,
                        &request.to,
                        page_size,
                        next_page_token.as_deref(),
                        &access_token,
                    )
                    .await?;
                if response.meetings.is_empty() {
                    break;
                }

                for meeting in response.meetings.into_iter().take(remaining) {
                    let meeting_id = match meeting.meeting_id() {
                        Ok(value) => value,
                        Err(error) => {
                            failures.push(ZoomRecordingSyncFailure {
                                meeting_id: "unknown".to_owned(),
                                step: "meeting_parse".to_owned(),
                                error: error.to_string(),
                            });
                            continue;
                        }
                    };
                    meetings_seen += 1;
                    let observation =
                        meeting.to_meeting_observation(&account.account_id, &meeting_id);
                    match self.observe_meeting(&observation).await {
                        Ok(_) => meetings_recorded += 1,
                        Err(error) => {
                            failures.push(ZoomRecordingSyncFailure {
                                meeting_id: meeting_id.clone(),
                                step: "meeting_observation".to_owned(),
                                error: error.to_string(),
                            });
                            continue;
                        }
                    }

                    for recording in &meeting.recording_files {
                        let mut recording_ref = match recording.to_recording_ref() {
                            Ok(value) => value,
                            Err(error) => {
                                failures.push(ZoomRecordingSyncFailure {
                                    meeting_id: meeting_id.clone(),
                                    step: "recording_parse".to_owned(),
                                    error: error.to_string(),
                                });
                                continue;
                            }
                        };
                        let recording_id = recording_ref.recording_id.clone();
                        if zoom_recording_file_is_transcript(recording) {
                            // handled below through transcript import
                        } else if let Some(download_url) = recording.download_url() {
                            if !allow_remote_recording_downloads {
                                failures.push(ZoomRecordingSyncFailure {
                                    meeting_id: meeting_id.clone(),
                                    step: "recording_download_policy".to_owned(),
                                    error: "zoom_remote_recording_download_not_enabled".to_owned(),
                                });
                            } else {
                                match self
                                    .import_recording_media_download(
                                        &ZoomRecordingMediaDownloadRequest {
                                            observation_id: Some(format!(
                                                "zoom-provider-sync-recording-download:{}:{}",
                                                meeting_id, recording_id
                                            )),
                                            account_id: account.account_id.clone(),
                                            meeting_id: meeting_id.clone(),
                                            meeting_uuid: meeting.uuid.clone(),
                                            recording: recording_ref.clone(),
                                            file_name: recording.file_name(&meeting_id),
                                            content_type: recording.content_type(),
                                            download_url,
                                            metadata: json!({
                                                "source": "zoom_provider_sync",
                                                "user_id": &user_id,
                                                "from": &request.from,
                                                "to": &request.to,
                                                "recording_type": recording.recording_type,
                                                "file_type": recording.file_type,
                                                "file_extension": recording.file_extension,
                                            }),
                                            causation_id: Some(format!(
                                                "zoom-provider-sync:{}",
                                                account.account_id
                                            )),
                                            correlation_id: Some(format!(
                                                "zoom-provider-sync-recordings:{}:{}:{}",
                                                account.account_id, request.from, request.to
                                            )),
                                        },
                                        Some(&access_token),
                                    )
                                    .await
                                {
                                    Ok(imported) => {
                                        media_downloads_recorded += 1;
                                        if let Some(metadata) =
                                            recording_ref.metadata.as_object_mut()
                                        {
                                            metadata.insert(
                                                "imported_attachment_id".to_owned(),
                                                json!(imported.attachment_id),
                                            );
                                            metadata.insert(
                                                "imported_blob_id".to_owned(),
                                                json!(imported.blob_id),
                                            );
                                            metadata.insert(
                                                "imported_scan_status".to_owned(),
                                                json!(imported.scan_status.as_str()),
                                            );
                                            metadata.insert(
                                                "imported_storage_kind".to_owned(),
                                                json!(imported.storage_kind),
                                            );
                                            metadata.insert(
                                                "imported_storage_path".to_owned(),
                                                json!(imported.storage_path),
                                            );
                                        }
                                    }
                                    Err(error) => failures.push(ZoomRecordingSyncFailure {
                                        meeting_id: meeting_id.clone(),
                                        step: "recording_download".to_owned(),
                                        error: error.to_string(),
                                    }),
                                }
                            }
                        }
                        let recording_request = ZoomRecordingObservationRequest {
                            observation_id: Some(format!(
                                "zoom-provider-sync-recording:{}:{}",
                                meeting_id, recording_id
                            )),
                            account_id: account.account_id.clone(),
                            meeting_id: meeting_id.clone(),
                            recording: recording_ref.clone(),
                            metadata: json!({
                                "source": "zoom_provider_sync",
                                "user_id": &user_id,
                                "from": &request.from,
                                "to": &request.to,
                            }),
                            causation_id: Some(format!(
                                "zoom-provider-sync:{}",
                                account.account_id
                            )),
                            correlation_id: Some(format!(
                                "zoom-provider-sync-recordings:{}:{}:{}",
                                account.account_id, request.from, request.to
                            )),
                        };
                        match self.observe_recording(&recording_request).await {
                            Ok(_) => recordings_recorded += 1,
                            Err(error) => {
                                failures.push(ZoomRecordingSyncFailure {
                                    meeting_id: meeting_id.clone(),
                                    step: "recording_observation".to_owned(),
                                    error: error.to_string(),
                                });
                                continue;
                            }
                        }

                        if !zoom_recording_file_is_transcript(recording) {
                            continue;
                        }
                        if !allow_remote_transcript_downloads {
                            failures.push(ZoomRecordingSyncFailure {
                                meeting_id: meeting_id.clone(),
                                step: "policy".to_owned(),
                                error: "zoom_remote_transcript_download_not_enabled".to_owned(),
                            });
                            continue;
                        }
                        let Some(download_url) = recording.download_url() else {
                            continue;
                        };
                        let transcript_request = ZoomTranscriptFileImportRequest {
                            observation_id: Some(format!(
                                "zoom-provider-sync-transcript:{}:{}",
                                meeting_id, recording_id
                            )),
                            transcript_id: stable_zoom_transcript_id(
                                &account.account_id,
                                &meeting_id,
                                &recording_id,
                            ),
                            account_id: account.account_id.clone(),
                            meeting_id: meeting_id.clone(),
                            meeting_uuid: meeting.uuid.clone(),
                            source_recording_ref: Some(recording_id.clone()),
                            language_code: None,
                            file_name: recording.transcript_file_name(&meeting_id),
                            content_type: recording.transcript_content_type(),
                            file_text: String::new(),
                            metadata: json!({
                                "source": "zoom_provider_sync",
                                "user_id": &user_id,
                                "from": &request.from,
                                "to": &request.to,
                                "recording_type": recording.recording_type,
                                "file_type": recording.file_type,
                                "file_extension": recording.file_extension,
                            }),
                            causation_id: Some(format!(
                                "zoom-provider-sync:{}",
                                account.account_id
                            )),
                            correlation_id: Some(format!(
                                "zoom-provider-sync-recordings:{}:{}:{}",
                                account.account_id, request.from, request.to
                            )),
                        };
                        match self
                            .import_transcript_file_download(
                                &transcript_request,
                                &download_url,
                                Some(&access_token),
                            )
                            .await
                        {
                            Ok(_) => transcripts_recorded += 1,
                            Err(error) => failures.push(ZoomRecordingSyncFailure {
                                meeting_id: meeting_id.clone(),
                                step: "transcript_download".to_owned(),
                                error: error.to_string(),
                            }),
                        }
                    }
                }

                next_page_token = response.next_page_token;
                if next_page_token
                    .as_deref()
                    .map(str::trim)
                    .is_none_or(|value| value.is_empty())
                {
                    break;
                }
            }

            Ok(ZoomRecordingSyncResult {
                account_id: account.account_id.clone(),
                user_id: user_id.clone(),
                from: request.from.trim().to_owned(),
                to: request.to.trim().to_owned(),
                meetings_seen,
                meetings_recorded,
                recordings_recorded,
                media_downloads_recorded,
                transcripts_recorded,
                failures,
            })
        }
        .await;

        match sync_result {
            Ok(result) => {
                self.mark_account_recording_sync_completed(
                    &account,
                    request,
                    allow_remote_recording_downloads,
                    &result,
                    allow_remote_transcript_downloads,
                    Utc::now(),
                )
                .await?;
                Ok(result)
            }
            Err(error) => {
                self.mark_account_recording_sync_failed(
                    &account,
                    request,
                    allow_remote_recording_downloads,
                    allow_remote_transcript_downloads,
                    Utc::now(),
                    &error.to_string(),
                )
                .await?;
                Err(error)
            }
        }
    }

    pub async fn webhook_subscription_status(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &ZoomWebhookSubscriptionStatusRequest,
    ) -> Result<ZoomWebhookSubscriptionStatusResult, ZoomError> {
        request.validate()?;
        let checked_at = Utc::now();
        let account = self.zoom_account(&request.account_id).await?;
        ensure_zoom_account_is_authorized(&account)?;
        self.ensure_webhook_secret_available(secret_store, vault, &account)
            .await?;
        let access_token = self
            .load_webhook_management_access_token(secret_store, vault, &account)
            .await?;
        let subscriptions = self
            .fetch_zoom_webhook_subscriptions(&request.api_base_url(), &access_token)
            .await?;
        let managed_subscription_id = account
            .config
            .get("webhook_subscription")
            .and_then(|value| value.get("managed_subscription_id"))
            .and_then(Value::as_str)
            .map(str::to_owned);

        self.mark_account_webhook_subscription_checked(
            &account,
            managed_subscription_id.as_deref(),
            None,
            checked_at,
            None,
        )
        .await?;

        Ok(ZoomWebhookSubscriptionStatusResult {
            account_id: account.account_id,
            provider_kind: account.provider_kind,
            auth_shape: account.auth_shape,
            checked_at,
            managed_subscription_id,
            subscriptions,
        })
    }

    pub async fn reconcile_webhook_subscription(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &ZoomWebhookSubscriptionReconcileRequest,
    ) -> Result<ZoomWebhookSubscriptionReconcileResult, ZoomError> {
        request.validate()?;
        let checked_at = Utc::now();
        let account = self.zoom_account(&request.account_id).await?;
        ensure_zoom_account_is_authorized(&account)?;
        self.ensure_webhook_secret_available(secret_store, vault, &account)
            .await?;
        let access_token = self
            .load_webhook_management_access_token(secret_store, vault, &account)
            .await?;
        let desired_subscription_name = request.resolved_subscription_name();
        let desired_endpoint_url = request.endpoint_url.trim().to_owned();
        let desired_event_types = request.resolved_event_types();
        let subscriptions = self
            .fetch_zoom_webhook_subscriptions(&request.api_base_url(), &access_token)
            .await?;
        let existing = find_managed_zoom_webhook_subscription(
            &subscriptions,
            account
                .config
                .get("webhook_subscription")
                .and_then(|value| value.get("managed_subscription_id"))
                .and_then(Value::as_str),
            &desired_subscription_name,
        );
        let replaced_existing = existing.is_some();

        if let Some(existing) = existing
            && existing.endpoint_url == desired_endpoint_url
            && canonical_zoom_webhook_event_types(&existing.event_types) == desired_event_types
        {
            self.mark_account_webhook_subscription_checked(
                &account,
                Some(&existing.subscription_id),
                Some(existing),
                checked_at,
                None,
            )
            .await?;
            return Ok(ZoomWebhookSubscriptionReconcileResult {
                account_id: account.account_id,
                provider_kind: account.provider_kind,
                auth_shape: account.auth_shape,
                status: "unchanged".to_owned(),
                checked_at,
                subscription: existing.clone(),
            });
        }

        if let Some(existing) = existing {
            self.delete_zoom_webhook_subscription(
                &request.api_base_url(),
                &existing.subscription_id,
                &access_token,
            )
            .await?;
        }

        let created = self
            .create_zoom_webhook_subscription(
                &request.api_base_url(),
                &desired_subscription_name,
                &desired_endpoint_url,
                &desired_event_types,
                &access_token,
            )
            .await?;
        self.mark_account_webhook_subscription_checked(
            &account,
            Some(&created.subscription_id),
            Some(&created),
            checked_at,
            None,
        )
        .await?;

        Ok(ZoomWebhookSubscriptionReconcileResult {
            account_id: account.account_id,
            provider_kind: account.provider_kind,
            auth_shape: account.auth_shape,
            status: if replaced_existing {
                "recreated".to_owned()
            } else {
                "created".to_owned()
            },
            checked_at,
            subscription: created,
        })
    }

    pub async fn remove_webhook_subscription(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &ZoomWebhookSubscriptionRemoveRequest,
    ) -> Result<ZoomWebhookSubscriptionRemoveResult, ZoomError> {
        request.validate()?;
        let checked_at = Utc::now();
        let account = self.zoom_account(&request.account_id).await?;
        ensure_zoom_account_is_authorized(&account)?;
        self.ensure_webhook_secret_available(secret_store, vault, &account)
            .await?;
        let access_token = self
            .load_webhook_management_access_token(secret_store, vault, &account)
            .await?;
        let stored_managed_subscription_id = account
            .config
            .get("webhook_subscription")
            .and_then(|value| value.get("managed_subscription_id"))
            .and_then(Value::as_str)
            .map(str::to_owned);
        let subscription_id = request
            .subscription_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .or(stored_managed_subscription_id.clone());

        let removed = if let Some(subscription_id) = subscription_id.as_deref() {
            self.delete_zoom_webhook_subscription(
                &request.api_base_url(),
                subscription_id,
                &access_token,
            )
            .await?;
            true
        } else {
            false
        };

        self.mark_account_webhook_subscription_checked(&account, None, None, checked_at, None)
            .await?;

        Ok(ZoomWebhookSubscriptionRemoveResult {
            account_id: account.account_id,
            provider_kind: account.provider_kind,
            auth_shape: account.auth_shape,
            removed,
            checked_at,
            subscription_id,
        })
    }

    pub(super) async fn store_or_resolve_client_secret(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        account: &ZoomAccount,
        client_secret: Option<&str>,
        client_secret_ref: Option<&str>,
        auth_shape: &str,
    ) -> Result<Option<String>, ZoomError> {
        if let Some(secret_ref) = client_secret_ref
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            vault.read_secret(secret_ref)?;
            self.provider_secret_binding_store
                .bind(&NewProviderAccountSecretBinding::new(
                    &account.account_id,
                    ProviderAccountSecretPurpose::ZoomClientSecret,
                    secret_ref,
                ))
                .await?;
            return Ok(Some(secret_ref.to_owned()));
        }

        let Some(client_secret) = client_secret
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            return Ok(None);
        };
        let secret_ref = zoom_client_secret_ref(&account.account_id);
        let metadata = zoom_client_secret_metadata(account, auth_shape);
        secret_store
            .upsert_secret_reference(
                &NewSecretReference::new(
                    &secret_ref,
                    SecretKind::ApiToken,
                    SecretStoreKind::HostVault,
                    format!("Zoom client secret for {}", account.account_id),
                )
                .metadata(metadata.clone()),
            )
            .await?;
        vault.store_secret(
            &secret_ref,
            client_secret,
            SecretEntryContext {
                entry_kind: "provider_client_secret",
                account_id: &account.account_id,
                purpose: ProviderAccountSecretPurpose::ZoomClientSecret.as_str(),
                secret_kind: SecretKind::ApiToken.as_str(),
                label: "Zoom client secret",
                metadata: &metadata,
            },
        )?;
        self.provider_secret_binding_store
            .bind(&NewProviderAccountSecretBinding::new(
                &account.account_id,
                ProviderAccountSecretPurpose::ZoomClientSecret,
                &secret_ref,
            ))
            .await?;
        Ok(Some(secret_ref))
    }

    pub(super) async fn store_recording_media_download(
        &self,
        request: &ZoomRecordingMediaDownloadRequest,
        bearer_token: Option<&str>,
    ) -> Result<ImportedAttachmentRecord, ZoomError> {
        let observed_at = Utc::now();
        let download_url = validate_required("download_url", &request.download_url)?;
        let mut http_request = self.http.get(download_url).header(ACCEPT, "*/*");
        if let Some(token) = bearer_token {
            http_request = http_request.bearer_auth(token);
        }
        let response = http_request.send().await?.error_for_status()?;
        let response_content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let body = response.bytes().await?;
        if body.len() > ZOOM_MAX_RECORDING_MEDIA_DOWNLOAD_BYTES {
            return Err(ZoomError::InvalidRequest(format!(
                "downloaded Zoom recording file must be at most {ZOOM_MAX_RECORDING_MEDIA_DOWNLOAD_BYTES} bytes"
            )));
        }

        let bytes = body.to_vec();
        let local_blob = put_local_blob(DEFAULT_MAIL_SYNC_BLOB_ROOT, &bytes).await?;
        let content_type = response_content_type
            .or_else(|| request.content_type.clone())
            .or_else(|| zoom_recording_content_type(request.recording.file_extension.as_deref()))
            .unwrap_or_else(|| "application/octet-stream".to_owned());
        let stored_blob = self
            .imported_attachment_store
            .upsert_blob_record(&local_blob, &content_type)
            .await?;
        let filename = request.file_name.clone();
        let scan_report = scan_attachment(&SafetyScanRequest {
            filename: filename.as_deref(),
            content_type: &content_type,
            size_bytes: local_blob.size_bytes,
            bytes: &bytes,
        })?;
        let attachment_id = new_attachment_import_id(&format!(
            "zoom-recording-download:{}:{}:{}",
            request.account_id.trim(),
            request.meeting_id.trim(),
            request.recording.recording_id.trim()
        ));
        let retention_policy = self
            .resolved_retention_policy(
                ZOOM_RECORDING_IMPORT_RETENTION_DAYS_SETTING_KEY,
                observed_at,
            )
            .await?;
        let mut import = ImportedAttachmentUpsert {
            attachment_id,
            account_id: request.account_id.trim().to_owned(),
            channel_kind: "zoom".to_owned(),
            blob_id: stored_blob.blob_id,
            filename: None,
            content_type: content_type.clone(),
            size_bytes: local_blob.size_bytes,
            sha256: local_blob.sha256.clone(),
            source_kind: "zoom_recording_download".to_owned(),
            imported_by: "zoom-recording-download".to_owned(),
            scan_report,
            metadata: json!({
                "provider": "zoom",
                "meeting_id": &request.meeting_id,
                "meeting_uuid": &request.meeting_uuid,
                "recording_id": &request.recording.recording_id,
                "recording_type": &request.recording.recording_type,
                "file_type": request.recording.metadata.get("file_type"),
                "file_extension": &request.recording.file_extension,
                "source": request.metadata.get("source").cloned().unwrap_or_else(|| json!("zoom_recording_download")),
                "retention_policy": retention_policy,
                "metadata": sanitize_zoom_payload(request.metadata.clone()),
                "secret_material": "excluded",
            }),
        };
        if let Some(filename) = filename {
            import.filename = Some(filename);
        }
        Ok(self
            .imported_attachment_store
            .upsert_imported_attachment_record(&import)
            .await?)
    }
}
