use hermes_communications_api::accounts::ProviderSecretBindingCommandPort;
use hermes_communications_api::accounts::{
    NewProviderAccountSecretBinding, ProviderAccountCommandPort, ProviderAccountSecretPurpose,
};
use hermes_events_api::NewEventEnvelope;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::Row;
use sqlx::postgres::PgPool;
use url::form_urlencoded::byte_serialize;

use crate::platform::calls::{
    CallIntelligenceStore, NewCallTranscript, NewProviderCall, TranscriptStatus,
};
use crate::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use crate::platform::events::bus::InMemoryEventBus;
use crate::platform::events::bus::zoom_event_types;
use crate::platform::secrets::{
    NewSecretReference, SecretKind, SecretReferenceStore, SecretStoreKind,
};
use crate::platform::settings::ApplicationSettingsStore;
use crate::platform::storage::{
    ImportedAttachmentRecord, ImportedAttachmentStoragePort, ImportedAttachmentUpsert,
    SafetyScanRequest, delete_local_blob, new_attachment_import_id, put_local_blob,
    scan_attachment,
};
use crate::vault::{HostVault, SecretEntryContext};
use hermes_events_postgres::store::EventStore;

use super::errors::ZoomError;
use super::models::{
    MAX_TRANSCRIPT_FILE_TEXT_BYTES, ZOOM_EXPLICIT_TOKEN_REFRESH_THRESHOLD_SECONDS,
    ZOOM_LIVE_AUTHORIZED_RUNTIME_KIND, ZOOM_MAX_RECORDING_MEDIA_DOWNLOAD_BYTES,
    ZOOM_MAX_TOKEN_REFRESH_THRESHOLD_SECONDS, ZOOM_PROVIDER_KIND, ZOOM_PROVIDER_KIND_STR,
    ZOOM_RUNTIME_KIND, ZOOM_TOKEN_EXPIRY_SAFETY_MARGIN_SECONDS,
    ZOOM_TOKEN_MAINTENANCE_REFRESH_THRESHOLD_SECONDS, ZOOM_TOKEN_ROTATION_REQUIRED_BLOCKER,
    ZoomAccount, ZoomAccountListResponse, ZoomAccountSetupRequest, ZoomAccountSetupResponse,
    ZoomAuditEventItem, ZoomAuditEventResponse, ZoomAuthShape, ZoomAuthorizationResult,
    ZoomLiveAccountSetupRequest, ZoomMeetingIngestResult, ZoomMeetingObservationRequest,
    ZoomOAuthPendingGrant, ZoomOAuthStartRequest, ZoomOAuthTokenBundle, ZoomOAuthTokenResponse,
    ZoomRecordingImportAuditItem, ZoomRecordingImportAuditResponse,
    ZoomRecordingImportRemoveRequest, ZoomRecordingImportRemoveResponse, ZoomRecordingIngestResult,
    ZoomRecordingMediaDownloadRequest, ZoomRecordingMediaImportResult,
    ZoomRecordingObservationRequest, ZoomRecordingRef, ZoomRecordingSyncFailure,
    ZoomRecordingSyncRequest, ZoomRecordingSyncResult, ZoomRetentionCleanupItem,
    ZoomRetentionCleanupRequest, ZoomRetentionCleanupResponse, ZoomRuntimeRemoveRequest,
    ZoomRuntimeRemoveResponse, ZoomRuntimeStartRequest, ZoomRuntimeStatus, ZoomRuntimeStopRequest,
    ZoomServerToServerAuthorizeRequest, ZoomTokenMaintenanceItem, ZoomTokenMaintenanceRequest,
    ZoomTokenMaintenanceResult, ZoomTokenRefreshRequest, ZoomTokenRefreshResult,
    ZoomTranscriptFileImportRequest, ZoomTranscriptFileImportResult, ZoomTranscriptIngestResult,
    ZoomTranscriptObservationRequest, ZoomWebhookSubscription,
    ZoomWebhookSubscriptionReconcileRequest, ZoomWebhookSubscriptionReconcileResult,
    ZoomWebhookSubscriptionRemoveRequest, ZoomWebhookSubscriptionRemoveResult,
    ZoomWebhookSubscriptionStatusRequest, ZoomWebhookSubscriptionStatusResult,
    random_zoom_oauth_token, sanitize_zoom_payload, zoom_authorization_url, zoom_client_secret_ref,
    zoom_oauth_expires_at, zoom_oauth_token_secret_ref,
};

#[derive(Clone)]
pub struct ZoomStore {
    pool: PgPool,
    provider_account_store: Arc<dyn ProviderAccountCommandPort>,
    provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
    imported_attachment_store: Arc<dyn ImportedAttachmentStoragePort>,
    call_store: CallIntelligenceStore,
    event_store: EventStore,
    event_bus: InMemoryEventBus,
    http: reqwest::Client,
}

struct ZoomAuthorizedAccountUpdate<'a> {
    auth_shape: &'a str,
    token_secret_ref: &'a str,
    client_secret_ref: Option<&'a str>,
    expires_at: Option<DateTime<Utc>>,
    metadata: Value,
    authorized_at: DateTime<Utc>,
}

const ZOOM_RECORDING_IMPORT_RETENTION_DAYS_SETTING_KEY: &str =
    "privacy.zoom_recording_import_retention_days";
const ZOOM_TRANSCRIPT_RETENTION_DAYS_SETTING_KEY: &str = "privacy.zoom_transcript_retention_days";

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
        let authorization_url = zoom_authorization_url(request, &state)?;
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

    async fn refresh_token_with_provenance(
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

    async fn exchange_oauth_authorization_code(
        &self,
        token_endpoint: String,
        client_id: &str,
        client_secret: &str,
        authorization_code: &str,
        redirect_uri: &str,
    ) -> Result<ZoomOAuthTokenResponse, ZoomError> {
        Ok(self
            .http
            .post(token_endpoint)
            .basic_auth(client_id.trim(), Some(client_secret.trim()))
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", authorization_code.trim()),
                ("redirect_uri", redirect_uri.trim()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json::<ZoomOAuthTokenResponse>()
            .await?)
    }

    async fn exchange_oauth_refresh_token(
        &self,
        token_endpoint: String,
        client_id: &str,
        client_secret: &str,
        refresh_token: &str,
    ) -> Result<ZoomOAuthTokenResponse, ZoomError> {
        Ok(self
            .http
            .post(token_endpoint)
            .basic_auth(client_id.trim(), Some(client_secret.trim()))
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token.trim()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json::<ZoomOAuthTokenResponse>()
            .await?)
    }

    async fn exchange_server_to_server_token(
        &self,
        token_endpoint: String,
        client_id: &str,
        client_secret: &str,
        zoom_account_id: &str,
    ) -> Result<ZoomOAuthTokenResponse, ZoomError> {
        Ok(self
            .http
            .post(token_endpoint)
            .basic_auth(client_id.trim(), Some(client_secret.trim()))
            .query(&[
                ("grant_type", "account_credentials"),
                ("account_id", zoom_account_id.trim()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json::<ZoomOAuthTokenResponse>()
            .await?)
    }

    async fn exchange_client_credentials_token(
        &self,
        token_endpoint: String,
        client_id: &str,
        client_secret: &str,
    ) -> Result<ZoomOAuthTokenResponse, ZoomError> {
        Ok(self
            .http
            .post(token_endpoint)
            .basic_auth(client_id.trim(), Some(client_secret.trim()))
            .form(&[("grant_type", "client_credentials")])
            .send()
            .await?
            .error_for_status()?
            .json::<ZoomOAuthTokenResponse>()
            .await?)
    }

    async fn store_or_resolve_client_secret(
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

    async fn load_token_bundle(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        account: &ZoomAccount,
    ) -> Result<ZoomOAuthTokenBundle, ZoomError> {
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
        Ok(serde_json::from_str(
            &vault.read_secret(&token_reference.secret_ref)?,
        )?)
    }

    async fn ensure_webhook_secret_available(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        account: &ZoomAccount,
    ) -> Result<(), ZoomError> {
        let webhook_binding = self
            .provider_secret_binding_store
            .get_for_account(
                &account.account_id,
                ProviderAccountSecretPurpose::ZoomWebhookSecret,
            )
            .await?
            .ok_or_else(|| {
                ZoomError::InvalidRequest(format!(
                    "Zoom account `{}` has no zoom_webhook_secret binding",
                    account.account_id
                ))
            })?;
        let webhook_reference = secret_store
            .secret_reference(&webhook_binding.secret_ref)
            .await?
            .ok_or_else(|| {
                ZoomError::InvalidRequest(format!(
                    "Zoom webhook secret reference `{}` was not found",
                    webhook_binding.secret_ref
                ))
            })?;
        if webhook_reference.secret_kind != SecretKind::ApiToken
            || webhook_reference.store_kind != SecretStoreKind::HostVault
        {
            return Err(ZoomError::InvalidRequest(format!(
                "Zoom webhook secret reference `{}` must be an api_token in host_vault",
                webhook_reference.secret_ref
            )));
        }
        let _ = vault.read_secret(&webhook_reference.secret_ref)?;
        Ok(())
    }

    async fn load_webhook_management_access_token(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        account: &ZoomAccount,
    ) -> Result<String, ZoomError> {
        let bundle = self.load_token_bundle(secret_store, vault, account).await?;
        let client_secret_ref = bundle.client_secret_ref.clone().ok_or_else(|| {
            ZoomError::InvalidRequest(format!(
                "Zoom account `{}` has no client secret reference in token bundle",
                account.account_id
            ))
        })?;
        let client_secret = vault.read_secret(&client_secret_ref)?;
        let token = if account.auth_shape == ZoomAuthShape::ServerToServer.as_str() {
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
            .await?
        } else if account.auth_shape == ZoomAuthShape::OAuthUser.as_str() {
            self.exchange_client_credentials_token(
                bundle.token_url.clone(),
                &bundle.client_id,
                &client_secret,
            )
            .await?
        } else {
            return Err(ZoomError::InvalidRequest(format!(
                "Zoom account `{}` is not a live account",
                account.account_id
            )));
        };
        let access_token = token.access_token.trim().to_owned();
        if access_token.is_empty() {
            return Err(ZoomError::InvalidRequest(format!(
                "Zoom account `{}` returned an empty webhook-management access token",
                account.account_id
            )));
        }
        Ok(access_token)
    }

    #[allow(clippy::too_many_arguments)]
    async fn fetch_zoom_recordings_page(
        &self,
        api_base_url: &str,
        user_id: &str,
        from: &str,
        to: &str,
        page_size: usize,
        next_page_token: Option<&str>,
        access_token: &str,
    ) -> Result<ZoomApiRecordingListResponse, ZoomError> {
        let user_id = byte_serialize(user_id.trim().as_bytes()).collect::<String>();
        let endpoint = format!("{api_base_url}/users/{user_id}/recordings");
        let mut query = vec![
            ("from", from.trim().to_owned()),
            ("to", to.trim().to_owned()),
            ("page_size", page_size.to_string()),
        ];
        if let Some(token) = next_page_token
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            query.push(("next_page_token", token.to_owned()));
        }
        Ok(self
            .http
            .get(endpoint)
            .bearer_auth(access_token.trim())
            .query(&query)
            .send()
            .await?
            .error_for_status()?
            .json::<ZoomApiRecordingListResponse>()
            .await?)
    }

    async fn fetch_zoom_webhook_subscriptions(
        &self,
        api_base_url: &str,
        access_token: &str,
    ) -> Result<Vec<ZoomWebhookSubscription>, ZoomError> {
        let endpoint = format!("{api_base_url}/marketplace/app/event_subscription");
        let response = self
            .http
            .get(endpoint)
            .bearer_auth(access_token.trim())
            .header(ACCEPT, "application/json")
            .send()
            .await?
            .error_for_status()?
            .json::<ZoomApiEventSubscriptionListResponse>()
            .await?;
        Ok(response
            .event_subscriptions
            .into_iter()
            .filter_map(|subscription| subscription.into_public())
            .collect())
    }

    async fn create_zoom_webhook_subscription(
        &self,
        api_base_url: &str,
        subscription_name: &str,
        endpoint_url: &str,
        event_types: &[String],
        access_token: &str,
    ) -> Result<ZoomWebhookSubscription, ZoomError> {
        let endpoint = format!("{api_base_url}/marketplace/app/event_subscription");
        let response = self
            .http
            .post(endpoint)
            .bearer_auth(access_token.trim())
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .json(&json!({
                "subscription_name": subscription_name.trim(),
                "event_webhook_url": endpoint_url.trim(),
                "event_types": event_types,
            }))
            .send()
            .await?
            .error_for_status()?
            .json::<ZoomApiEventSubscription>()
            .await?;
        response.into_public().ok_or_else(|| {
            ZoomError::InvalidRequest(
                "Zoom webhook subscription create response was missing required fields".to_owned(),
            )
        })
    }

    async fn delete_zoom_webhook_subscription(
        &self,
        api_base_url: &str,
        subscription_id: &str,
        access_token: &str,
    ) -> Result<(), ZoomError> {
        let subscription_id = byte_serialize(subscription_id.trim().as_bytes()).collect::<String>();
        let endpoint =
            format!("{api_base_url}/marketplace/app/event_subscription/{subscription_id}");
        self.http
            .delete(endpoint)
            .bearer_auth(access_token.trim())
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    async fn store_recording_media_download(
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

    async fn store_oauth_token_bundle(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        account: &ZoomAccount,
        secret_ref: &str,
        token_bundle: &ZoomOAuthTokenBundle,
        metadata: &Value,
    ) -> Result<(), ZoomError> {
        secret_store
            .upsert_secret_reference(
                &NewSecretReference::new(
                    secret_ref,
                    SecretKind::OauthToken,
                    SecretStoreKind::HostVault,
                    format!("Zoom OAuth credential for {}", account.account_id),
                )
                .metadata(metadata.clone()),
            )
            .await?;
        vault.store_secret(
            secret_ref,
            &serde_json::to_string(token_bundle)?,
            SecretEntryContext {
                entry_kind: "provider_credential",
                account_id: &account.account_id,
                purpose: ProviderAccountSecretPurpose::ZoomOauthToken.as_str(),
                secret_kind: SecretKind::OauthToken.as_str(),
                label: "Zoom OAuth credential",
                metadata,
            },
        )?;
        Ok(())
    }

    async fn mark_account_authorized(
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

    async fn mark_account_recording_sync_completed(
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

    async fn mark_account_recording_sync_failed(
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

    async fn mark_account_token_refreshed(
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

    async fn mark_account_token_refresh_failed(
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

    async fn mark_account_webhook_subscription_checked(
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

    async fn bind_live_secret_refs(
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

    async fn bind_optional_secret_ref(
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

    async fn zoom_account(&self, account_id: &str) -> Result<ZoomAccount, ZoomError> {
        let account_id = validate_account_id(account_id)?;
        let account = self
            .provider_account_store
            .get(&account_id)
            .await?
            .ok_or_else(|| {
                ZoomError::InvalidRequest(format!("Zoom account `{account_id}` was not found"))
            })?;
        if !account.provider_kind.is_zoom() {
            return Err(ZoomError::InvalidRequest(format!(
                "account `{account_id}` is not a Zoom provider account"
            )));
        }
        Ok(account.into())
    }

    async fn ensure_zoom_account(&self, account_id: &str) -> Result<(), ZoomError> {
        self.zoom_account(account_id).await.map(|_| ())
    }

    async fn update_account_config(
        &self,
        account_id: &str,
        config: Value,
    ) -> Result<ZoomAccount, ZoomError> {
        let account = self
            .provider_account_store
            .update_config(account_id, &config)
            .await?
            .ok_or_else(|| {
                ZoomError::InvalidRequest(format!("Zoom account `{account_id}` was not found"))
            })?;
        Ok(account.into())
    }

    async fn ensure_placeholder_call(
        &self,
        request: &ZoomTranscriptObservationRequest,
        call_id: String,
        observed_at: DateTime<Utc>,
    ) -> Result<(), ZoomError> {
        let call_exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM telegram_calls
                WHERE account_id = $1
                  AND provider_call_id = $2
            )
            "#,
        )
        .bind(request.account_id.trim())
        .bind(request.meeting_id.trim())
        .fetch_one(&self.pool)
        .await?;
        if call_exists {
            return Ok(());
        }
        let call = NewProviderCall {
            call_id,
            account_id: request.account_id.trim().to_owned(),
            provider_call_id: request.meeting_id.trim().to_owned(),
            provider_chat_id: request.provider_chat_id(),
            direction: crate::platform::calls::CallDirection::Outgoing,
            call_state: crate::platform::calls::CallState::Ended,
            started_at: None,
            ended_at: None,
            transcription_policy_id: None,
            metadata: json!({
                "provider": "zoom",
                "provider_kind": ZOOM_PROVIDER_KIND_STR,
                "meeting_id": &request.meeting_id,
                "meeting_uuid": &request.meeting_uuid,
                "observed_at": observed_at,
                "placeholder": true,
            }),
        };
        self.call_store
            .upsert_call(&call)
            .await
            .map(|_| ())
            .map_err(Into::into)
    }

    async fn publish_runtime_status_event(
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

    async fn publish_authorization_completed_event(
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

    async fn publish_token_refresh_event(
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

    async fn publish_token_refresh_failure_event(
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

    async fn publish_recording_import_removed_event(
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

    async fn publish_transcript_removed_event(
        &self,
        transcript: &crate::platform::calls::CallTranscript,
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

    async fn publish_retention_cleanup_completed_event(
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

    async fn append_and_broadcast(&self, event: &NewEventEnvelope) -> Result<(), ZoomError> {
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

fn runtime_status_from_account(account: ZoomAccount) -> ZoomRuntimeStatus {
    let checked_at = Utc::now();
    let mut blockers = account
        .config
        .get("runtime_blockers")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let last_error = account
        .config
        .get("last_error")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let fixture = account.auth_shape == "fixture";
    let live_authorized = !fixture && zoom_account_is_authorized(&account);
    let status = if account.lifecycle_state == "fixture_ready" {
        "stopped".to_owned()
    } else {
        account.lifecycle_state.clone()
    };
    let token_rotation = zoom_token_rotation_status(&account, checked_at);
    blockers.retain(|value| value != "zoom_provider_workers_not_enabled");
    if live_authorized {
        blockers.retain(|value| value != "zoom_live_authorization_required");
    }
    if live_authorized
        && token_rotation.rotation_required
        && !blockers
            .iter()
            .any(|value| value == ZOOM_TOKEN_ROTATION_REQUIRED_BLOCKER)
    {
        blockers.push(ZOOM_TOKEN_ROTATION_REQUIRED_BLOCKER.to_owned());
    }
    let mut metadata = account.config.clone();
    metadata["token_rotation_policy"] = token_rotation.metadata;
    let healthy = account.lifecycle_state != "removed"
        && ((fixture && account.lifecycle_state != "removed")
            || (live_authorized && status == "running" && blockers.is_empty()));

    ZoomRuntimeStatus {
        account_id: account.account_id,
        provider_kind: account.provider_kind,
        runtime_kind: account.runtime_kind,
        status,
        healthy,
        auth_shape: account.auth_shape,
        live_runtime_available: live_authorized,
        recording_ingest_available: account.lifecycle_state != "removed",
        transcript_ingest_available: account.lifecycle_state != "removed",
        runtime_blockers: blockers,
        last_error,
        checked_at,
        metadata,
    }
}

fn parse_optional_datetime(value: Option<&Value>) -> Option<DateTime<Utc>> {
    value
        .and_then(Value::as_str)
        .and_then(|raw| DateTime::parse_from_rfc3339(raw).ok())
        .map(|value| value.with_timezone(&Utc))
}

struct ZoomTokenRotationStatus {
    rotation_required: bool,
    metadata: Value,
}

fn zoom_token_rotation_status(
    account: &ZoomAccount,
    checked_at: DateTime<Utc>,
) -> ZoomTokenRotationStatus {
    let authorization = account.config.get("authorization");
    let expires_at = authorization
        .and_then(|value| value.get("expires_at"))
        .and_then(zoom_datetime_from_json);
    let token_secret_bound = authorization
        .and_then(|value| value.get("token_secret_bound"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let last_refresh_status = authorization
        .and_then(|value| value.get("last_token_refresh"))
        .and_then(|value| value.get("status"))
        .and_then(Value::as_str);
    let last_refresh_failed = last_refresh_status == Some("failed");
    let expired = expires_at.is_some_and(|value| value <= checked_at);
    let refresh_due = expires_at.is_some_and(|value| {
        value
            <= checked_at
                + chrono::TimeDelta::seconds(ZOOM_TOKEN_MAINTENANCE_REFRESH_THRESHOLD_SECONDS)
    });
    let live_authorized = account.auth_shape != ZoomAuthShape::Fixture.as_str()
        && zoom_account_is_authorized(account);
    let missing_token_secret = live_authorized && !token_secret_bound;
    let rotation_required = last_refresh_failed || expired || missing_token_secret;
    let status = if rotation_required {
        "required"
    } else if refresh_due {
        "due"
    } else if live_authorized {
        "current"
    } else {
        "not_applicable"
    };

    ZoomTokenRotationStatus {
        rotation_required,
        metadata: json!({
            "status": status,
            "rotation_required": rotation_required,
            "refresh_due": refresh_due,
            "expired": expired,
            "missing_token_secret": missing_token_secret,
            "last_refresh_status": last_refresh_status,
            "expires_at": expires_at,
            "checked_at": checked_at,
            "policy": {
                "explicit_refresh_threshold_seconds": ZOOM_EXPLICIT_TOKEN_REFRESH_THRESHOLD_SECONDS,
                "maintenance_refresh_threshold_seconds": ZOOM_TOKEN_MAINTENANCE_REFRESH_THRESHOLD_SECONDS,
                "max_refresh_threshold_seconds": ZOOM_MAX_TOKEN_REFRESH_THRESHOLD_SECONDS,
                "provider_expiry_safety_margin_seconds": ZOOM_TOKEN_EXPIRY_SAFETY_MARGIN_SECONDS,
                "failure_blocker": ZOOM_TOKEN_ROTATION_REQUIRED_BLOCKER,
            },
        }),
    }
}

fn zoom_datetime_from_json(value: &Value) -> Option<DateTime<Utc>> {
    value
        .as_str()
        .and_then(|candidate| DateTime::parse_from_rfc3339(candidate).ok())
        .map(|value| value.with_timezone(&Utc))
}

fn add_zoom_token_refresh_blocker(config: &mut Value) {
    let mut blockers = config
        .get("runtime_blockers")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    let blocker = Value::String(ZOOM_TOKEN_ROTATION_REQUIRED_BLOCKER.to_owned());
    if !blockers.iter().any(|value| value == &blocker) {
        blockers.push(blocker);
    }
    config["runtime_blockers"] = json!(blockers);
}

fn clear_zoom_token_refresh_blocker(config: &mut Value) {
    let mut blockers = config
        .get("runtime_blockers")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    blockers.retain(|value| value.as_str() != Some(ZOOM_TOKEN_ROTATION_REQUIRED_BLOCKER));
    config["runtime_blockers"] = json!(blockers);
}

fn clear_zoom_provider_workers_not_enabled_blocker(config: &mut Value) {
    let mut blockers = config
        .get("runtime_blockers")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    blockers.retain(|value| value.as_str() != Some("zoom_provider_workers_not_enabled"));
    config["runtime_blockers"] = json!(blockers);
}

fn clear_zoom_live_authorization_required_blocker(config: &mut Value) {
    let mut blockers = config
        .get("runtime_blockers")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    blockers.retain(|value| value.as_str() != Some("zoom_live_authorization_required"));
    config["runtime_blockers"] = json!(blockers);
}

fn zoom_account_is_authorized(account: &ZoomAccount) -> bool {
    account
        .config
        .get("authorization")
        .and_then(|authorization| authorization.get("status"))
        .and_then(Value::as_str)
        == Some("authorized")
}

fn ensure_zoom_account_is_authorized(account: &ZoomAccount) -> Result<(), ZoomError> {
    if zoom_account_is_authorized(account) {
        return Ok(());
    }
    Err(ZoomError::InvalidRequest(format!(
        "Zoom account `{}` is not authorized",
        account.account_id
    )))
}

fn canonical_zoom_webhook_event_types(event_types: &[String]) -> Vec<String> {
    let mut normalized = event_types
        .iter()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized
}

fn find_managed_zoom_webhook_subscription<'a>(
    subscriptions: &'a [ZoomWebhookSubscription],
    managed_subscription_id: Option<&str>,
    subscription_name: &str,
) -> Option<&'a ZoomWebhookSubscription> {
    managed_subscription_id
        .and_then(|subscription_id| {
            subscriptions
                .iter()
                .find(|subscription| subscription.subscription_id == subscription_id.trim())
        })
        .or_else(|| {
            subscriptions
                .iter()
                .find(|subscription| subscription.subscription_name == subscription_name.trim())
        })
}

fn authorization_result(
    account: ZoomAccount,
    token_secret_ref: String,
    client_secret_ref: Option<String>,
    authorized_at: DateTime<Utc>,
) -> ZoomAuthorizationResult {
    ZoomAuthorizationResult {
        account_id: account.account_id,
        provider_kind: account.provider_kind,
        auth_shape: account.auth_shape,
        lifecycle_state: account.lifecycle_state,
        runtime_kind: account.runtime_kind,
        token_secret_ref,
        client_secret_ref,
        secret_kind: SecretKind::OauthToken.as_str().to_owned(),
        store_kind: SecretStoreKind::HostVault.as_str().to_owned(),
        authorized_at,
    }
}

fn refresh_flow_label(auth_shape: &str) -> &'static str {
    if auth_shape == ZoomAuthShape::OAuthUser.as_str() {
        "oauth_user"
    } else if auth_shape == ZoomAuthShape::ServerToServer.as_str() {
        "server_to_server"
    } else {
        "unknown"
    }
}

impl ZoomStore {
    async fn resolved_retention_policy(
        &self,
        setting_key: &str,
        observed_at: DateTime<Utc>,
    ) -> Result<Value, ZoomError> {
        let settings = ApplicationSettingsStore::new(self.pool.clone());
        settings.repair_declared_settings().await?;
        let retention_days = settings
            .setting(setting_key)
            .await?
            .and_then(|setting| setting.value.as_i64())
            .unwrap_or(0)
            .max(0);
        let expires_at = if retention_days > 0 {
            Some(observed_at + chrono::TimeDelta::days(retention_days))
        } else {
            None
        };
        Ok(json!({
            "setting_key": setting_key,
            "retention_days": retention_days,
            "mode": if retention_days > 0 {
                "delete_after_n_days"
            } else {
                "explicit_remove_only"
            },
            "observed_at": observed_at,
            "expires_at": expires_at,
        }))
    }
}

fn zoom_oauth_secret_metadata(
    account: &ZoomAccount,
    auth_shape: &str,
    expires_at: Option<DateTime<Utc>>,
    metadata: &Value,
) -> Value {
    json!({
        "provider": "zoom",
        "provider_kind": account.provider_kind,
        "account_id": account.account_id,
        "external_account_id": account.external_account_id,
        "auth_shape": auth_shape,
        "secret_purpose": ProviderAccountSecretPurpose::ZoomOauthToken.as_str(),
        "secret_material": "excluded",
        "expires_at": expires_at,
        "metadata": metadata,
    })
}

fn zoom_client_secret_metadata(account: &ZoomAccount, auth_shape: &str) -> Value {
    json!({
        "provider": "zoom",
        "provider_kind": account.provider_kind,
        "account_id": account.account_id,
        "external_account_id": account.external_account_id,
        "auth_shape": auth_shape,
        "secret_purpose": ProviderAccountSecretPurpose::ZoomClientSecret.as_str(),
        "secret_material": "excluded",
    })
}

fn resolve_client_secret(
    vault: &HostVault,
    client_secret: Option<&str>,
    client_secret_ref: Option<&str>,
) -> Result<String, ZoomError> {
    if let Some(client_secret) = client_secret
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Ok(client_secret.to_owned());
    }
    let secret_ref = client_secret_ref
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            ZoomError::InvalidRequest(
                "Zoom client secret is required for token exchange".to_owned(),
            )
        })?;
    vault.read_secret(secret_ref).map_err(Into::into)
}

fn validate_required(field: &'static str, value: &str) -> Result<String, ZoomError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ZoomError::InvalidRequest(format!(
            "{field} must not be empty"
        )));
    }
    Ok(trimmed.to_owned())
}

fn zoom_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest client configuration must be valid")
}

fn validate_account_id(account_id: &str) -> Result<String, ZoomError> {
    let trimmed = account_id.trim();
    if trimmed.is_empty() {
        return Err(ZoomError::InvalidRequest(
            "account_id must not be empty".to_owned(),
        ));
    }
    Ok(trimmed.to_owned())
}

#[derive(Clone, Debug, serde::Deserialize)]
struct ZoomApiEventSubscriptionListResponse {
    #[serde(default, alias = "subscriptions")]
    event_subscriptions: Vec<ZoomApiEventSubscription>,
}

#[derive(Clone, Debug, serde::Deserialize)]
struct ZoomApiEventSubscription {
    #[serde(alias = "id", alias = "event_subscription_id")]
    subscription_id: Option<String>,
    #[serde(alias = "subscription_name", alias = "event_subscription_name")]
    subscription_name: Option<String>,
    #[serde(alias = "event_webhook_url", alias = "webhook_url")]
    endpoint_url: Option<String>,
    #[serde(default, alias = "events")]
    event_types: Vec<String>,
}

impl ZoomApiEventSubscription {
    fn into_public(self) -> Option<ZoomWebhookSubscription> {
        let subscription_id = self
            .subscription_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())?
            .to_owned();
        let subscription_name = self
            .subscription_name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())?
            .to_owned();
        let endpoint_url = self
            .endpoint_url
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())?
            .to_owned();
        Some(ZoomWebhookSubscription {
            subscription_id,
            subscription_name,
            endpoint_url,
            event_types: canonical_zoom_webhook_event_types(&self.event_types),
        })
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
struct ZoomApiRecordingListResponse {
    #[serde(default)]
    meetings: Vec<ZoomApiRecordingMeeting>,
    #[serde(default)]
    next_page_token: Option<String>,
}

#[derive(Clone, Debug, serde::Deserialize)]
struct ZoomApiRecordingMeeting {
    id: Value,
    uuid: Option<String>,
    topic: Option<String>,
    host_email: Option<String>,
    join_url: Option<String>,
    start_time: Option<String>,
    end_time: Option<String>,
    duration: Option<i64>,
    #[serde(default)]
    recording_files: Vec<ZoomApiRecordingFile>,
}

impl ZoomApiRecordingMeeting {
    fn meeting_id(&self) -> Result<String, ZoomError> {
        json_string_or_number("meeting.id", &self.id)
    }

    fn to_meeting_observation(
        &self,
        account_id: &str,
        meeting_id: &str,
    ) -> ZoomMeetingObservationRequest {
        let recording_refs = self
            .recording_files
            .iter()
            .filter_map(|item| item.to_recording_ref().ok())
            .collect::<Vec<ZoomRecordingRef>>();
        ZoomMeetingObservationRequest {
            observation_id: Some(format!("zoom-provider-sync-meeting:{meeting_id}")),
            account_id: account_id.to_owned(),
            meeting_id: meeting_id.to_owned(),
            meeting_uuid: self.uuid.clone(),
            topic: self.topic.clone(),
            host_email: self.host_email.clone(),
            join_url: self.join_url.clone(),
            started_at: zoom_api_datetime(self.start_time.as_deref()),
            ended_at: zoom_api_datetime(self.end_time.as_deref()),
            duration_seconds: self.duration.map(|minutes| minutes.saturating_mul(60)),
            participants: Vec::new(),
            recording_refs,
            transcript_ref: None,
            metadata: json!({
                "source": "zoom_provider_sync",
            }),
            causation_id: Some(format!("zoom-provider-sync:{account_id}")),
            correlation_id: Some(format!(
                "zoom-provider-sync-meeting:{account_id}:{meeting_id}"
            )),
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
struct ZoomApiRecordingFile {
    id: Value,
    file_type: Option<String>,
    recording_type: Option<String>,
    download_url: Option<String>,
    file_extension: Option<String>,
    file_size: Option<i64>,
    recording_start: Option<String>,
}

impl ZoomApiRecordingFile {
    fn recording_id(&self) -> Result<String, ZoomError> {
        json_string_or_number("recording_file.id", &self.id)
    }

    fn to_recording_ref(&self) -> Result<ZoomRecordingRef, ZoomError> {
        Ok(ZoomRecordingRef {
            recording_id: self.recording_id()?,
            recording_type: self.recording_type.clone(),
            download_ref: self.download_url(),
            file_extension: self
                .file_extension
                .clone()
                .or_else(|| self.file_type.clone()),
            file_size_bytes: self.file_size,
            recorded_at: zoom_api_datetime(self.recording_start.as_deref()),
            metadata: json!({
                "file_type": self.file_type,
                "source": "zoom_provider_sync",
            }),
        })
    }

    fn download_url(&self) -> Option<String> {
        self.download_url
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    }

    fn transcript_file_name(&self, meeting_id: &str) -> Option<String> {
        let extension = self
            .file_extension
            .as_deref()
            .or(self.file_type.as_deref())
            .map(str::trim)
            .filter(|value| !value.is_empty())?
            .to_ascii_lowercase();
        Some(format!(
            "{}-{}.{}",
            meeting_id.trim(),
            self.recording_id().ok()?,
            extension
        ))
    }

    fn transcript_content_type(&self) -> Option<String> {
        zoom_transcript_content_type(self.file_extension.as_deref().or(self.file_type.as_deref()))
    }

    fn file_name(&self, meeting_id: &str) -> Option<String> {
        let extension = self
            .file_extension
            .as_deref()
            .or(self.file_type.as_deref())
            .map(str::trim)
            .filter(|value| !value.is_empty())?
            .to_ascii_lowercase();
        Some(format!(
            "{}-{}.{}",
            meeting_id.trim(),
            self.recording_id().ok()?,
            extension
        ))
    }

    fn content_type(&self) -> Option<String> {
        zoom_recording_content_type(self.file_extension.as_deref().or(self.file_type.as_deref()))
    }
}

fn provider_sync_user_id(
    account: &ZoomAccount,
    request: &ZoomRecordingSyncRequest,
) -> Result<String, ZoomError> {
    if let Some(user_id) = request
        .user_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Ok(user_id.to_owned());
    }
    if account.auth_shape == ZoomAuthShape::OAuthUser.as_str() {
        return Ok("me".to_owned());
    }
    let external_account_id = account.external_account_id.trim();
    if !external_account_id.is_empty() {
        return Ok(external_account_id.to_owned());
    }
    Err(ZoomError::InvalidRequest(
        "user_id is required for server_to_server recording sync when external_account_id is empty"
            .to_owned(),
    ))
}

fn zoom_recording_content_type(extension: Option<&str>) -> Option<String> {
    let normalized = extension
        .map(str::trim)
        .filter(|value| !value.is_empty())?
        .to_ascii_lowercase();
    let content_type = match normalized.as_str() {
        "mp4" => "video/mp4",
        "m4a" => "audio/mp4",
        "m4v" => "video/x-m4v",
        "txt" => "text/plain",
        "csv" => "text/csv",
        "json" => "application/json",
        "chat" => "text/plain",
        _ => "application/octet-stream",
    };
    Some(content_type.to_owned())
}

fn zoom_recording_import_audit_item(
    item: ImportedAttachmentRecord,
) -> ZoomRecordingImportAuditItem {
    let retention_policy = item.metadata.get("retention_policy");
    ZoomRecordingImportAuditItem {
        attachment_id: item.attachment_id,
        account_id: item.account_id.unwrap_or_default(),
        meeting_id: item
            .metadata
            .get("meeting_id")
            .and_then(Value::as_str)
            .map(str::to_owned),
        meeting_uuid: item
            .metadata
            .get("meeting_uuid")
            .and_then(Value::as_str)
            .map(str::to_owned),
        recording_id: item
            .metadata
            .get("recording_id")
            .and_then(Value::as_str)
            .map(str::to_owned),
        filename: item.filename,
        content_type: item.content_type,
        size_bytes: item.size_bytes,
        sha256: item.sha256,
        source: item
            .metadata
            .get("source")
            .and_then(Value::as_str)
            .map(str::to_owned),
        scan_status: item.scan_status.as_str().to_owned(),
        scan_summary: item.scan_summary,
        storage_kind: item.storage_kind,
        storage_path: item.storage_path,
        retention_mode: retention_policy
            .and_then(|value| value.get("mode"))
            .and_then(Value::as_str)
            .unwrap_or("explicit_remove_only")
            .to_owned(),
        retention_days: retention_policy
            .and_then(|value| value.get("retention_days"))
            .and_then(Value::as_i64)
            .unwrap_or(0),
        expires_at: retention_policy
            .and_then(|value| value.get("expires_at"))
            .and_then(Value::as_str)
            .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
            .map(|value| value.with_timezone(&Utc)),
        created_at: item.created_at,
    }
}

fn json_string_or_number(field: &str, value: &Value) -> Result<String, ZoomError> {
    match value {
        Value::String(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                Err(ZoomError::InvalidRequest(format!(
                    "{field} must not be empty"
                )))
            } else {
                Ok(trimmed.to_owned())
            }
        }
        Value::Number(value) => Ok(value.to_string()),
        _ => Err(ZoomError::InvalidRequest(format!(
            "{field} must be a string or number"
        ))),
    }
}

fn zoom_api_datetime(value: Option<&str>) -> Option<DateTime<Utc>> {
    value
        .map(str::trim)
        .filter(|candidate| !candidate.is_empty())
        .and_then(|candidate| {
            DateTime::parse_from_rfc3339(candidate)
                .ok()
                .map(|value| value.with_timezone(&Utc))
        })
}

fn zoom_recording_file_is_transcript(recording: &ZoomApiRecordingFile) -> bool {
    [
        recording.file_extension.as_deref(),
        recording.file_type.as_deref(),
    ]
    .into_iter()
    .flatten()
    .map(|value| value.trim().to_ascii_lowercase())
    .any(|value| matches!(value.as_str(), "vtt" | "srt" | "txt" | "transcript" | "cc"))
        || recording
            .recording_type
            .as_deref()
            .map(str::trim)
            .map(str::to_ascii_lowercase)
            .is_some_and(|value| value.contains("transcript") || value == "audio_transcript")
}

fn zoom_transcript_content_type(extension_or_type: Option<&str>) -> Option<String> {
    match extension_or_type
        .map(str::trim)
        .filter(|value| !value.is_empty())?
        .to_ascii_lowercase()
        .as_str()
    {
        "vtt" => Some("text/vtt".to_owned()),
        "srt" => Some("application/x-subrip".to_owned()),
        "txt" | "transcript" | "cc" => Some("text/plain".to_owned()),
        _ => None,
    }
}

fn stable_zoom_transcript_id(account_id: &str, meeting_id: &str, recording_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(account_id.trim().as_bytes());
    hasher.update(b":");
    hasher.update(meeting_id.trim().as_bytes());
    hasher.update(b":");
    hasher.update(recording_id.trim().as_bytes());
    format!("zoom_transcript_{:x}", hasher.finalize())
}

pub fn stable_zoom_call_id(account_id: &str, meeting_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(account_id.trim().as_bytes());
    hasher.update(b":");
    hasher.update(meeting_id.trim().as_bytes());
    format!("zoom_call_{:x}", hasher.finalize())
}

fn zoom_event_id(
    kind: &str,
    account_id: &str,
    subject_id: &str,
    observation_id: Option<&str>,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(kind.as_bytes());
    hasher.update(b":");
    hasher.update(account_id.trim().as_bytes());
    hasher.update(b":");
    hasher.update(subject_id.trim().as_bytes());
    if let Some(observation_id) = observation_id {
        hasher.update(b":");
        hasher.update(observation_id.trim().as_bytes());
    }
    format!("evt_zoom_{}_{:x}", kind, hasher.finalize())
}

#[allow(clippy::too_many_arguments)]
fn zoom_event(
    event_id: String,
    event_type: &str,
    occurred_at: DateTime<Utc>,
    source: Value,
    subject: Value,
    payload: Value,
    provenance: Value,
    causation_id: Option<String>,
    correlation_id: Option<String>,
) -> Result<NewEventEnvelope, ZoomError> {
    let mut builder = NewEventEnvelope::builder(event_id, event_type, occurred_at, source, subject)
        .payload(payload)
        .provenance(provenance);
    if let Some(causation_id) = causation_id {
        builder = builder.causation_id(causation_id);
    }
    if let Some(correlation_id) = correlation_id {
        builder = builder.correlation_id(correlation_id);
    }
    Ok(builder.build()?)
}
