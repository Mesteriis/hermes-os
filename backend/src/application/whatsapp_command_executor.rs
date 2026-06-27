use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chrono::Utc;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::domains::communications::storage::{
    CommunicationStorageStore, LocalCommunicationBlobStore,
};
use crate::integrations::whatsapp::client::{
    NewWhatsappWebDialog, NewWhatsappWebMedia, NewWhatsappWebMessage, NewWhatsappWebMessageDelete,
    NewWhatsappWebMessageUpdate, NewWhatsappWebReaction, NewWhatsappWebStatus,
};
use crate::integrations::whatsapp::runtime::{
    WhatsAppProviderApiAccessToken, WhatsAppProviderCommandExecutionError,
    WhatsAppProviderExecutableCommand, WhatsAppProviderInMemoryMediaBytes,
    WhatsAppProviderMediaDownloadRef, WhatsAppProviderWriteCommand,
    claim_due_business_cloud_commands_for_execution, claim_due_commands_for_execution,
    claim_due_native_md_commands_for_execution, dead_letter_failed_command,
    import_canonical_provider_commands, record_live_provider_command_submitted,
    recover_stale_fixture_executing_commands, recover_stale_live_executing_commands,
    reschedule_failed_command, whatsapp_business_cloud_access_token_secret_ref,
    whatsapp_native_md_media_download_secret_ref,
};
use crate::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use crate::platform::events::bus::whatsapp_event_types;
use crate::platform::events::{EventBus, EventStore, NewEventEnvelope};
use crate::vault::{HostVault, HostVaultError};

use super::communication_fixture_ingest::CommunicationFixtureIngestError;
use super::communication_fixture_ingest::WhatsappFixtureIngestApplicationService;
use super::provider_runtime_contracts::WhatsAppProviderRuntimeRef;

const WHATSAPP_COMMAND_EXECUTOR_RUNTIME: &str = "whatsapp_command_executor";
static WHATSAPP_COMMAND_EXECUTOR_EVENT_SEQUENCE: AtomicU64 = AtomicU64::new(1);

pub(crate) async fn execute_due_fixture_commands(
    pool: PgPool,
    runtime: WhatsAppProviderRuntimeRef,
    event_store: EventStore,
    event_bus: EventBus,
    limit: i64,
) {
    let now = Utc::now();
    match import_canonical_provider_commands(&pool, now, limit).await {
        Ok(commands) => {
            for command in commands {
                let _ = publish_command_event(
                    &event_store,
                    &event_bus,
                    whatsapp_event_types::COMMAND_STATUS_CHANGED,
                    &command,
                    json!({"source": "canonical_provider_command_import"}),
                )
                .await;
            }
        }
        Err(error) => {
            tracing::warn!(error = %error, "whatsapp command executor: canonical import failed");
        }
    }

    match recover_stale_fixture_executing_commands(&pool, now).await {
        Ok(commands) => {
            for command in commands {
                let _ = publish_command_event(
                    &event_store,
                    &event_bus,
                    whatsapp_event_types::COMMAND_STATUS_CHANGED,
                    &command,
                    json!({"source": "stale_recovery"}),
                )
                .await;
            }
        }
        Err(error) => {
            tracing::warn!(error = %error, "whatsapp command executor: stale recovery failed");
        }
    }

    let commands = match claim_due_commands_for_execution(&pool, now, limit).await {
        Ok(commands) => commands,
        Err(error) => {
            tracing::warn!(error = %error, "whatsapp command executor: failed to claim commands");
            return;
        }
    };
    if commands.is_empty() {
        return;
    }

    let fixture_ingest = WhatsappFixtureIngestApplicationService::new(
        pool.clone(),
        runtime,
        event_store.clone(),
        event_bus.clone(),
    );

    for command in commands {
        let _ = publish_command_event(
            &event_store,
            &event_bus,
            whatsapp_event_types::COMMAND_STATUS_CHANGED,
            &command,
            json!({"source": "command_executor", "phase": "claimed"}),
        )
        .await;
        let _ = publish_media_execution_started_event(&fixture_ingest, &command).await;
        if let Err(error) = execute_claimed_command(&fixture_ingest, &command).await {
            tracing::warn!(
                error = %error,
                command_id = %command.command_id,
                command_kind = %command.command_kind,
                "whatsapp command executor: command execution failed"
            );
            let _ = publish_media_execution_failed_event(&fixture_ingest, &command, &error).await;
            match reschedule_failed_command(
                &pool,
                &command.command_id,
                Utc::now(),
                &error.to_string(),
                None,
                None,
            )
            .await
            {
                Ok(Some(updated)) => {
                    let _ = publish_command_event(
                        &event_store,
                        &event_bus,
                        whatsapp_event_types::COMMAND_STATUS_CHANGED,
                        &updated,
                        json!({"source": "command_executor", "error": error.to_string()}),
                    )
                    .await;
                }
                Ok(None) => {}
                Err(update_error) => {
                    tracing::warn!(
                        error = %update_error,
                        command_id = %command.command_id,
                        "whatsapp command executor: failed to reschedule failed command"
                    );
                }
            }
        }
    }
}

pub(crate) async fn execute_due_live_native_md_commands(
    pool: PgPool,
    runtime: WhatsAppProviderRuntimeRef,
    vault: HostVault,
    event_store: EventStore,
    event_bus: EventBus,
    limit: i64,
) {
    let now = Utc::now();
    match recover_stale_live_executing_commands(&pool, now, None).await {
        Ok(commands) => {
            for command in commands {
                let _ = publish_command_event(
                    &event_store,
                    &event_bus,
                    whatsapp_event_types::COMMAND_STATUS_CHANGED,
                    &command,
                    json!({"source": "stale_live_recovery"}),
                )
                .await;
            }
        }
        Err(error) => {
            tracing::warn!(error = %error, "whatsapp native command executor: stale recovery failed");
        }
    }

    let commands = match claim_due_native_md_commands_for_execution(&pool, now, limit).await {
        Ok(commands) => commands,
        Err(error) => {
            tracing::warn!(
                error = %error,
                "whatsapp native command executor: failed to claim commands"
            );
            return;
        }
    };
    if commands.is_empty() {
        return;
    }

    let media_event_ingest = WhatsappFixtureIngestApplicationService::new(
        pool.clone(),
        runtime.clone(),
        event_store.clone(),
        event_bus.clone(),
    );

    for command in commands {
        let _ = publish_command_event(
            &event_store,
            &event_bus,
            whatsapp_event_types::COMMAND_STATUS_CHANGED,
            &command,
            json!({"source": "native_md_command_executor", "phase": "claimed"}),
        )
        .await;
        let _ = publish_media_execution_started_event(&media_event_ingest, &command).await;
        let mut executable = WhatsAppProviderExecutableCommand::from(&command);
        if let Err(error) =
            prepare_live_native_md_media_upload(&pool, &command, &mut executable).await
        {
            tracing::warn!(
                error_code = error.error_code.as_deref().unwrap_or("unknown"),
                command_id = %command.command_id,
                command_kind = %command.command_kind,
                "whatsapp native command executor: media upload preparation failed"
            );
            record_live_native_md_command_failure(
                &pool,
                &event_store,
                &event_bus,
                &media_event_ingest,
                &command,
                &error,
            )
            .await;
            continue;
        }
        if let Err(error) =
            prepare_live_native_md_media_download(&vault, &command, &mut executable).await
        {
            tracing::warn!(
                error_code = error.error_code.as_deref().unwrap_or("unknown"),
                command_id = %command.command_id,
                command_kind = %command.command_kind,
                "whatsapp native command executor: media download preparation failed"
            );
            record_live_native_md_command_failure(
                &pool,
                &event_store,
                &event_bus,
                &media_event_ingest,
                &command,
                &error,
            )
            .await;
            continue;
        }
        match runtime.execute_live_provider_command(&executable).await {
            Ok(outcome) => {
                if command.command_kind == "download_media" {
                    if let Err(error) = persist_live_native_md_media_download(
                        &media_event_ingest,
                        &command,
                        &outcome,
                    )
                    .await
                    {
                        tracing::warn!(
                            error_code = error.error_code.as_deref().unwrap_or("unknown"),
                            command_id = %command.command_id,
                            command_kind = %command.command_kind,
                            "whatsapp native command executor: media download persistence failed"
                        );
                        record_live_native_md_command_failure(
                            &pool,
                            &event_store,
                            &event_bus,
                            &media_event_ingest,
                            &command,
                            &error,
                        )
                        .await;
                    }
                    continue;
                }
                match record_live_provider_command_submitted(&pool, Utc::now(), &outcome).await {
                    Ok(Some(updated)) => {
                        let _ = publish_media_execution_progress(
                            &media_event_ingest,
                            &command,
                            "submitted_to_provider_awaiting_observed_evidence",
                            95,
                            None,
                            None,
                            None,
                        )
                        .await;
                        let _ = publish_command_event(
                            &event_store,
                            &event_bus,
                            whatsapp_event_types::COMMAND_STATUS_CHANGED,
                            &updated,
                            json!({
                                "source": "native_md_command_executor",
                                "phase": "submitted_to_provider",
                                "provider_request_id": outcome.provider_request_id,
                                "completion_rule": "provider_observed_event_reconciliation_required",
                                "payload_policy": "sanitized_metadata_only",
                            }),
                        )
                        .await;
                    }
                    Ok(None) => {}
                    Err(error) => {
                        tracing::warn!(
                            error = %error,
                            command_id = %command.command_id,
                            "whatsapp native command executor: failed to record provider submission"
                        );
                    }
                }
            }
            Err(error) => {
                tracing::warn!(
                    error_code = error.error_code.as_deref().unwrap_or("unknown"),
                    command_id = %command.command_id,
                    command_kind = %command.command_kind,
                    "whatsapp native command executor: command execution failed"
                );
                record_live_native_md_command_failure(
                    &pool,
                    &event_store,
                    &event_bus,
                    &media_event_ingest,
                    &command,
                    &error,
                )
                .await;
            }
        }
    }
}

pub(crate) async fn execute_due_live_business_cloud_commands(
    pool: PgPool,
    runtime: WhatsAppProviderRuntimeRef,
    vault: HostVault,
    event_store: EventStore,
    event_bus: EventBus,
    limit: i64,
) {
    let now = Utc::now();
    let commands = match claim_due_business_cloud_commands_for_execution(&pool, now, limit).await {
        Ok(commands) => commands,
        Err(error) => {
            tracing::warn!(
                error = %error,
                "whatsapp business cloud command executor: failed to claim commands"
            );
            return;
        }
    };
    if commands.is_empty() {
        return;
    }

    let media_event_ingest = WhatsappFixtureIngestApplicationService::new(
        pool.clone(),
        runtime.clone(),
        event_store.clone(),
        event_bus.clone(),
    );

    for command in commands {
        let _ = publish_command_event(
            &event_store,
            &event_bus,
            whatsapp_event_types::COMMAND_STATUS_CHANGED,
            &command,
            json!({"source": "business_cloud_command_executor", "phase": "claimed"}),
        )
        .await;
        let mut executable = WhatsAppProviderExecutableCommand::from(&command);
        if let Err(error) =
            prepare_live_business_cloud_access_token(&vault, &command, &mut executable).await
        {
            tracing::warn!(
                error_code = error.error_code.as_deref().unwrap_or("unknown"),
                command_id = %command.command_id,
                command_kind = %command.command_kind,
                "whatsapp business cloud command executor: access token preparation failed"
            );
            record_live_business_cloud_command_failure(
                &pool,
                &event_store,
                &event_bus,
                &media_event_ingest,
                &command,
                &error,
            )
            .await;
            continue;
        }
        if let Err(error) =
            prepare_live_business_cloud_media_upload(&pool, &command, &mut executable).await
        {
            tracing::warn!(
                error_code = error.error_code.as_deref().unwrap_or("unknown"),
                command_id = %command.command_id,
                command_kind = %command.command_kind,
                "whatsapp business cloud command executor: media upload preparation failed"
            );
            record_live_business_cloud_command_failure(
                &pool,
                &event_store,
                &event_bus,
                &media_event_ingest,
                &command,
                &error,
            )
            .await;
            continue;
        }
        match runtime.execute_live_provider_command(&executable).await {
            Ok(outcome) => {
                match record_live_provider_command_submitted(&pool, Utc::now(), &outcome).await {
                    Ok(Some(updated)) => {
                        let _ = publish_command_event(
                            &event_store,
                            &event_bus,
                            whatsapp_event_types::COMMAND_STATUS_CHANGED,
                            &updated,
                            json!({
                                "source": "business_cloud_command_executor",
                                "phase": "submitted_to_provider",
                                "provider_request_id": outcome.provider_request_id,
                                "completion_rule": "provider_observed_event_reconciliation_required",
                                "payload_policy": "sanitized_metadata_only",
                            }),
                        )
                        .await;
                    }
                    Ok(None) => {}
                    Err(error) => {
                        tracing::warn!(
                            error = %error,
                            command_id = %command.command_id,
                            "whatsapp business cloud command executor: failed to record provider submission"
                        );
                    }
                }
            }
            Err(error) => {
                tracing::warn!(
                    error_code = error.error_code.as_deref().unwrap_or("unknown"),
                    command_id = %command.command_id,
                    command_kind = %command.command_kind,
                    "whatsapp business cloud command executor: command execution failed"
                );
                record_live_business_cloud_command_failure(
                    &pool,
                    &event_store,
                    &event_bus,
                    &media_event_ingest,
                    &command,
                    &error,
                )
                .await;
            }
        }
    }
}

async fn prepare_live_business_cloud_access_token(
    vault: &HostVault,
    command: &WhatsAppProviderWriteCommand,
    executable: &mut WhatsAppProviderExecutableCommand,
) -> Result<(), WhatsAppProviderCommandExecutionError> {
    if !matches!(
        command.command_kind.as_str(),
        "send_text" | "send_template" | "send_media" | "send_voice_note"
    ) {
        return Ok(());
    }

    let secret_ref = whatsapp_business_cloud_access_token_secret_ref(&command.account_id);
    let token = vault.read_secret(&secret_ref).map_err(|error| {
        let retry_after_seconds = match &error {
            HostVaultError::Locked | HostVaultError::Uninitialized => Some(30),
            HostVaultError::MissingSecret { .. } => None,
            _ => Some(30),
        };
        WhatsAppProviderCommandExecutionError::new(
            "business_cloud_access_token_unavailable",
            format!("failed to read WhatsApp Business Cloud access token from host vault: {error}"),
            retry_after_seconds,
        )
    })?;
    if token.trim().is_empty() {
        return Err(WhatsAppProviderCommandExecutionError::new(
            "business_cloud_access_token_empty",
            "WhatsApp Business Cloud access token host-vault payload is empty",
            None,
        ));
    }
    executable.api_access_token = Some(WhatsAppProviderApiAccessToken::new(secret_ref, token));
    Ok(())
}

async fn prepare_live_business_cloud_media_upload(
    pool: &PgPool,
    command: &WhatsAppProviderWriteCommand,
    executable: &mut WhatsAppProviderExecutableCommand,
) -> Result<(), WhatsAppProviderCommandExecutionError> {
    if !matches!(
        command.command_kind.as_str(),
        "send_media" | "send_voice_note"
    ) {
        return Ok(());
    }

    let media_blob = resolve_upload_media_blob_descriptor(pool, command)
        .await
        .map_err(|error| {
            WhatsAppProviderCommandExecutionError::new(
                "business_cloud_media_blob_unavailable",
                error,
                Some(30),
            )
        })?;
    if media_blob.storage_kind != "local_fs" {
        return Err(WhatsAppProviderCommandExecutionError::new(
            "business_cloud_media_storage_kind_unsupported",
            format!(
                "Business Cloud media upload supports local_fs blobs only, got `{}`",
                media_blob.storage_kind
            ),
            None,
        ));
    }

    let blob_store = LocalCommunicationBlobStore::new(DEFAULT_MAIL_SYNC_BLOB_ROOT);
    let bytes = blob_store
        .read_blob(&media_blob.storage_path)
        .await
        .map_err(|error| {
            WhatsAppProviderCommandExecutionError::new(
                "business_cloud_media_blob_read_failed",
                format!("failed to read local media blob for Business Cloud upload: {error}"),
                Some(30),
            )
        })?;
    let actual_size = i64::try_from(bytes.len()).map_err(|_| {
        WhatsAppProviderCommandExecutionError::new(
            "business_cloud_media_blob_too_large",
            "local media blob size exceeds supported command metadata range",
            None,
        )
    })?;
    if actual_size != media_blob.size_bytes {
        return Err(WhatsAppProviderCommandExecutionError::new(
            "business_cloud_media_blob_size_mismatch",
            format!(
                "local media blob size mismatch: expected {}, got {}",
                media_blob.size_bytes, actual_size
            ),
            None,
        ));
    }

    let actual_sha256 = upload_media_sha256(&bytes);
    if actual_sha256 != media_blob.sha256 {
        return Err(WhatsAppProviderCommandExecutionError::new(
            "business_cloud_media_blob_sha256_mismatch",
            "local media blob digest does not match command metadata",
            None,
        ));
    }

    executable.media_bytes = Some(WhatsAppProviderInMemoryMediaBytes::new(bytes));
    Ok(())
}

async fn prepare_live_native_md_media_upload(
    pool: &PgPool,
    command: &WhatsAppProviderWriteCommand,
    executable: &mut WhatsAppProviderExecutableCommand,
) -> Result<(), WhatsAppProviderCommandExecutionError> {
    if !matches!(
        command.command_kind.as_str(),
        "send_media" | "send_voice_note"
    ) {
        return Ok(());
    }

    let media_blob = resolve_upload_media_blob_descriptor(pool, command)
        .await
        .map_err(|error| {
            WhatsAppProviderCommandExecutionError::new(
                "native_md_media_blob_unavailable",
                error,
                Some(30),
            )
        })?;
    if media_blob.storage_kind != "local_fs" {
        return Err(WhatsAppProviderCommandExecutionError::new(
            "native_md_media_storage_kind_unsupported",
            format!(
                "native_md media upload supports local_fs blobs only, got `{}`",
                media_blob.storage_kind
            ),
            None,
        ));
    }

    let blob_store = LocalCommunicationBlobStore::new(DEFAULT_MAIL_SYNC_BLOB_ROOT);
    let bytes = blob_store
        .read_blob(&media_blob.storage_path)
        .await
        .map_err(|error| {
            WhatsAppProviderCommandExecutionError::new(
                "native_md_media_blob_read_failed",
                format!("failed to read local media blob for WhatsApp upload: {error}"),
                Some(30),
            )
        })?;
    let actual_size = i64::try_from(bytes.len()).map_err(|_| {
        WhatsAppProviderCommandExecutionError::new(
            "native_md_media_blob_too_large",
            "local media blob size exceeds supported command metadata range",
            None,
        )
    })?;
    if actual_size != media_blob.size_bytes {
        return Err(WhatsAppProviderCommandExecutionError::new(
            "native_md_media_blob_size_mismatch",
            format!(
                "local media blob size mismatch: expected {}, got {}",
                media_blob.size_bytes, actual_size
            ),
            None,
        ));
    }

    let actual_sha256 = upload_media_sha256(&bytes);
    if actual_sha256 != media_blob.sha256 {
        return Err(WhatsAppProviderCommandExecutionError::new(
            "native_md_media_blob_sha256_mismatch",
            "local media blob digest does not match command metadata",
            None,
        ));
    }

    executable.media_bytes = Some(WhatsAppProviderInMemoryMediaBytes::new(bytes));
    Ok(())
}

async fn prepare_live_native_md_media_download(
    vault: &HostVault,
    command: &WhatsAppProviderWriteCommand,
    executable: &mut WhatsAppProviderExecutableCommand,
) -> Result<(), WhatsAppProviderCommandExecutionError> {
    if command.command_kind != "download_media" {
        return Ok(());
    }

    let secret_ref = media_download_secret_ref_for_command(command)?;
    let secret_payload = vault.read_secret(&secret_ref).map_err(|error| {
        let retry_after_seconds = match &error {
            HostVaultError::Locked | HostVaultError::Uninitialized => Some(30),
            HostVaultError::MissingSecret { .. } => None,
            _ => Some(30),
        };
        WhatsAppProviderCommandExecutionError::new(
            "native_md_media_download_ref_unavailable",
            format!("failed to read WhatsApp media download ref from host vault: {error}"),
            retry_after_seconds,
        )
    })?;
    let secret_payload: Value = serde_json::from_str(&secret_payload).map_err(|error| {
        WhatsAppProviderCommandExecutionError::new(
            "native_md_media_download_ref_invalid_json",
            format!("WhatsApp media download ref host-vault payload is invalid JSON: {error}"),
            None,
        )
    })?;
    let download_ref =
        media_download_ref_from_secret_payload(command, &secret_ref, &secret_payload)?;
    executable.media_download_ref = Some(download_ref);
    Ok(())
}

async fn persist_live_native_md_media_download(
    fixture_ingest: &WhatsappFixtureIngestApplicationService,
    command: &WhatsAppProviderWriteCommand,
    outcome: &crate::integrations::whatsapp::runtime::WhatsAppProviderCommandExecutionOutcome,
) -> Result<(), WhatsAppProviderCommandExecutionError> {
    let Some(media_bytes) = outcome.downloaded_media_bytes.as_ref() else {
        return Err(WhatsAppProviderCommandExecutionError::new(
            "native_md_downloaded_media_bytes_missing",
            "native_md media download returned no in-memory bytes for local blob persistence",
            None,
        ));
    };
    if media_bytes.is_empty() {
        return Err(WhatsAppProviderCommandExecutionError::new(
            "native_md_downloaded_media_bytes_empty",
            "native_md media download returned an empty byte payload",
            None,
        ));
    }

    let provider_message_id = command
        .provider_message_id
        .as_deref()
        .or_else(|| {
            command
                .target_ref
                .get("provider_message_id")
                .and_then(Value::as_str)
        })
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            WhatsAppProviderCommandExecutionError::new(
                "native_md_download_requires_provider_message_id",
                "native_md media download requires provider_message_id before projection",
                None,
            )
        })?;
    let provider_attachment_id = media_download_provider_attachment_id(command);
    let content_type = media_download_content_type(command, outcome);
    let blob_store = LocalCommunicationBlobStore::new(DEFAULT_MAIL_SYNC_BLOB_ROOT);
    let local_blob = blob_store
        .put_blob(&media_bytes.clone_bytes())
        .await
        .map_err(|error| {
            WhatsAppProviderCommandExecutionError::new(
                "native_md_download_blob_write_failed",
                format!(
                    "failed to persist WhatsApp downloaded media to local blob storage: {error}"
                ),
                Some(30),
            )
        })?;

    publish_media_execution_progress(
        fixture_ingest,
        command,
        "provider_downloaded_local_blob_persisted",
        90,
        Some(provider_message_id),
        Some(&provider_attachment_id),
        None,
    )
    .await
    .map_err(|error| {
        WhatsAppProviderCommandExecutionError::new(
            "native_md_download_progress_event_failed",
            error,
            Some(30),
        )
    })?;

    let import_batch_id = format!(
        "whatsapp-native-md-download:{}:{}",
        command.account_id, command.command_id
    );
    let observed = fixture_ingest
        .ingest_runtime_bridge_media(&NewWhatsappWebMedia {
            account_id: command.account_id.clone(),
            provider_chat_id: command.provider_chat_id.clone(),
            provider_message_id: provider_message_id.to_owned(),
            provider_attachment_id: provider_attachment_id.clone(),
            filename: command
                .payload
                .get("filename")
                .and_then(Value::as_str)
                .map(str::to_owned),
            content_type,
            size_bytes: local_blob.size_bytes,
            sha256: local_blob.sha256.clone(),
            storage_kind: local_blob.storage_kind,
            storage_path: local_blob.storage_path,
            import_batch_id,
            observed_at: Utc::now(),
        })
        .await
        .map_err(|error| {
            WhatsAppProviderCommandExecutionError::new(
                "native_md_download_media_observation_failed",
                format!("failed to ingest WhatsApp downloaded media observation: {error}"),
                Some(30),
            )
        })?;

    publish_media_execution_progress(
        fixture_ingest,
        command,
        "provider_observed",
        100,
        Some(provider_message_id),
        Some(&provider_attachment_id),
        Some(&observed.message_id),
    )
    .await
    .map_err(|error| {
        WhatsAppProviderCommandExecutionError::new(
            "native_md_download_progress_event_failed",
            error,
            Some(30),
        )
    })?;
    publish_media_execution_completed(
        fixture_ingest,
        command,
        Some(provider_message_id),
        Some(&provider_attachment_id),
        Some(&observed.message_id),
    )
    .await
    .map_err(|error| {
        WhatsAppProviderCommandExecutionError::new(
            "native_md_download_completed_event_failed",
            error,
            Some(30),
        )
    })?;
    Ok(())
}

fn media_download_secret_ref_for_command(
    command: &WhatsAppProviderWriteCommand,
) -> Result<String, WhatsAppProviderCommandExecutionError> {
    if let Some(secret_ref) = command_value_string(command, "media_download_secret_ref") {
        return Ok(secret_ref);
    }
    let fingerprint = command_value_string(command, "provider_media_ref_fingerprint")
        .or_else(|| command_value_string(command, "provider_media_id"))
        .ok_or_else(|| {
            WhatsAppProviderCommandExecutionError::new(
                "native_md_media_download_ref_missing",
                "native_md media download requires provider_media_id or media_download_secret_ref",
                None,
            )
        })?;
    Ok(whatsapp_native_md_media_download_secret_ref(
        &command.account_id,
        &fingerprint,
    ))
}

fn media_download_ref_from_secret_payload(
    command: &WhatsAppProviderWriteCommand,
    secret_ref: &str,
    payload: &Value,
) -> Result<WhatsAppProviderMediaDownloadRef, WhatsAppProviderCommandExecutionError> {
    let version = payload
        .get("version")
        .and_then(Value::as_i64)
        .ok_or_else(|| media_download_ref_error("version", "missing_or_invalid"))?;
    if version != 1 {
        return Err(media_download_ref_error("version", "unsupported"));
    }
    let account_id = required_secret_string(payload, "account_id")?;
    if account_id != command.account_id {
        return Err(media_download_ref_error("account_id", "mismatch"));
    }
    let secret_purpose = required_secret_string(payload, "secret_purpose")?;
    if secret_purpose != "whatsapp_media_download_ref" {
        return Err(media_download_ref_error("secret_purpose", "mismatch"));
    }
    let provider_shape = required_secret_string(payload, "provider_shape")?;
    if provider_shape != "whatsapp_native_md" {
        return Err(media_download_ref_error("provider_shape", "unsupported"));
    }

    let direct_path = required_secret_string(payload, "direct_path")?;
    let media_key = required_secret_base64(payload, "media_key_base64")?;
    let file_sha256 = required_secret_base64(payload, "file_sha256_base64")?;
    let file_enc_sha256 = required_secret_base64(payload, "file_enc_sha256_base64")?;
    let file_length = payload
        .get("file_length")
        .and_then(Value::as_u64)
        .ok_or_else(|| media_download_ref_error("file_length", "missing_or_invalid"))?;
    let media_type = required_secret_string(payload, "media_type")?;
    let content_type = payload
        .get("content_type")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .or_else(|| command_value_string(command, "content_type"))
        .unwrap_or_else(|| "application/octet-stream".to_owned());
    let provider_media_ref_fingerprint = payload
        .get("provider_media_ref_fingerprint")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .or_else(|| command_value_string(command, "provider_media_ref_fingerprint"))
        .or_else(|| command_value_string(command, "provider_media_id"))
        .ok_or_else(|| media_download_ref_error("provider_media_ref_fingerprint", "missing"))?;
    let static_url = payload
        .get("static_url")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    let media_key_timestamp = payload.get("media_key_timestamp").and_then(Value::as_i64);

    Ok(WhatsAppProviderMediaDownloadRef {
        secret_ref: secret_ref.to_owned(),
        provider_media_ref_fingerprint,
        media_type,
        content_type,
        file_length,
        file_sha256,
        file_enc_sha256,
        direct_path,
        static_url,
        media_key,
        media_key_timestamp,
    })
}

fn required_secret_string(
    payload: &Value,
    field: &'static str,
) -> Result<String, WhatsAppProviderCommandExecutionError> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| media_download_ref_error(field, "missing"))
}

fn required_secret_base64(
    payload: &Value,
    field: &'static str,
) -> Result<Vec<u8>, WhatsAppProviderCommandExecutionError> {
    let encoded = required_secret_string(payload, field)?;
    BASE64_STANDARD.decode(encoded.as_bytes()).map_err(|_| {
        WhatsAppProviderCommandExecutionError::new(
            "native_md_media_download_ref_invalid_base64",
            format!("WhatsApp media download ref host-vault field `{field}` is invalid base64"),
            None,
        )
    })
}

fn media_download_ref_error(
    field: &'static str,
    reason: &'static str,
) -> WhatsAppProviderCommandExecutionError {
    WhatsAppProviderCommandExecutionError::new(
        "native_md_media_download_ref_invalid",
        format!("WhatsApp media download ref host-vault field `{field}` is {reason}"),
        None,
    )
}

fn command_value_string(command: &WhatsAppProviderWriteCommand, field: &str) -> Option<String> {
    command
        .payload
        .get(field)
        .and_then(Value::as_str)
        .or_else(|| command.target_ref.get(field).and_then(Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn media_download_provider_attachment_id(command: &WhatsAppProviderWriteCommand) -> String {
    command_value_string(command, "provider_attachment_id")
        .or_else(|| command_value_string(command, "provider_media_id"))
        .unwrap_or_else(|| command.command_id.clone())
}

fn media_download_content_type(
    command: &WhatsAppProviderWriteCommand,
    outcome: &crate::integrations::whatsapp::runtime::WhatsAppProviderCommandExecutionOutcome,
) -> String {
    command_value_string(command, "content_type")
        .or_else(|| {
            outcome
                .result_payload
                .pointer("/provider_submission/operation/content_type")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "application/octet-stream".to_owned())
}

async fn record_live_native_md_command_failure(
    pool: &PgPool,
    event_store: &EventStore,
    event_bus: &EventBus,
    media_event_ingest: &WhatsappFixtureIngestApplicationService,
    command: &WhatsAppProviderWriteCommand,
    error: &WhatsAppProviderCommandExecutionError,
) {
    let _ = publish_media_execution_failed_event(media_event_ingest, command, &error.error_message)
        .await;
    let is_terminal_unsupported =
        error.error_code.as_deref() == Some("native_md_command_kind_unsupported");
    let update_result = if is_terminal_unsupported {
        dead_letter_failed_command(
            pool,
            &command.command_id,
            Utc::now(),
            &error.error_message,
            error.error_code.as_deref(),
        )
        .await
    } else {
        reschedule_failed_command(
            pool,
            &command.command_id,
            Utc::now(),
            &error.error_message,
            error.error_code.as_deref(),
            error.retry_after_seconds,
        )
        .await
    };

    match update_result {
        Ok(Some(updated)) => {
            let phase = if is_terminal_unsupported {
                "terminal_unsupported_before_provider_observation"
            } else {
                "failed_before_provider_observation"
            };
            let retry_policy = if is_terminal_unsupported {
                "terminal"
            } else {
                "retry_or_dead_letter"
            };
            let _ = publish_command_event(
                event_store,
                event_bus,
                whatsapp_event_types::COMMAND_STATUS_CHANGED,
                &updated,
                json!({
                    "source": "native_md_command_executor",
                    "phase": phase,
                    "error_code": error.error_code,
                    "retry_after_seconds": error.retry_after_seconds,
                    "retry_policy": retry_policy,
                    "payload_policy": "sanitized_metadata_only",
                }),
            )
            .await;
        }
        Ok(None) => {}
        Err(update_error) => {
            tracing::warn!(
                error = %update_error,
                command_id = %command.command_id,
                "whatsapp native command executor: failed to update failed command"
            );
        }
    }
}

async fn record_live_business_cloud_command_failure(
    pool: &PgPool,
    event_store: &EventStore,
    event_bus: &EventBus,
    media_event_ingest: &WhatsappFixtureIngestApplicationService,
    command: &WhatsAppProviderWriteCommand,
    error: &WhatsAppProviderCommandExecutionError,
) {
    let _ = publish_media_execution_failed_event(media_event_ingest, command, &error.error_message)
        .await;
    match reschedule_failed_command(
        pool,
        &command.command_id,
        Utc::now(),
        &error.error_message,
        error.error_code.as_deref(),
        error.retry_after_seconds,
    )
    .await
    {
        Ok(Some(updated)) => {
            let _ = publish_command_event(
                event_store,
                event_bus,
                whatsapp_event_types::COMMAND_STATUS_CHANGED,
                &updated,
                json!({
                    "source": "business_cloud_command_executor",
                    "phase": "failed_before_provider_observation",
                    "error_code": error.error_code,
                    "retry_after_seconds": error.retry_after_seconds,
                    "payload_policy": "sanitized_metadata_only",
                }),
            )
            .await;
        }
        Ok(None) => {}
        Err(update_error) => {
            tracing::warn!(
                error = %update_error,
                command_id = %command.command_id,
                "whatsapp business cloud command executor: failed to reschedule failed command"
            );
        }
    }
}

pub(crate) fn command_executor_runtime_name() -> &'static str {
    WHATSAPP_COMMAND_EXECUTOR_RUNTIME
}

async fn execute_claimed_command(
    fixture_ingest: &WhatsappFixtureIngestApplicationService,
    command: &WhatsAppProviderWriteCommand,
) -> Result<(), String> {
    let import_batch_id = format!(
        "whatsapp-command:{}:{}",
        command.account_id, command.command_id
    );
    match command.command_kind.as_str() {
        "edit" => {
            fixture_ingest
                .ingest_message_update(&NewWhatsappWebMessageUpdate {
                    account_id: command.account_id.clone(),
                    provider_chat_id: command.provider_chat_id.clone(),
                    provider_message_id: command
                        .provider_message_id
                        .clone()
                        .ok_or_else(|| "provider_message_id is required".to_owned())?,
                    text: command
                        .payload
                        .get("text")
                        .and_then(Value::as_str)
                        .ok_or_else(|| "payload.text is required".to_owned())?
                        .to_owned(),
                    import_batch_id,
                    observed_at: Utc::now(),
                })
                .await
                .map_err(fixture_error)?;
        }
        "send_text" => {
            fixture_ingest
                .ingest_message(&NewWhatsappWebMessage {
                    account_id: command.account_id.clone(),
                    provider_chat_id: command.provider_chat_id.clone(),
                    provider_message_id: format!("provider-message:{}", command.command_id),
                    chat_title: command.provider_chat_id.clone(),
                    sender_id: command.account_id.clone(),
                    sender_display_name: "Hermes Owner".to_owned(),
                    text: command
                        .payload
                        .get("text")
                        .and_then(Value::as_str)
                        .ok_or_else(|| "payload.text is required".to_owned())?
                        .to_owned(),
                    reply_to_provider_message_id: None,
                    forward_origin_chat_id: None,
                    forward_origin_message_id: None,
                    forward_origin_sender_id: None,
                    forward_origin_sender_name: None,
                    forwarded_at: None,
                    message_metadata: json!({}),
                    import_batch_id,
                    occurred_at: Utc::now(),
                    delivery_state:
                        crate::integrations::whatsapp::client::WhatsappWebDeliveryState::Sent,
                })
                .await
                .map_err(fixture_error)?;
        }
        "reply" => {
            fixture_ingest
                .ingest_message(&NewWhatsappWebMessage {
                    account_id: command.account_id.clone(),
                    provider_chat_id: command.provider_chat_id.clone(),
                    provider_message_id: format!("provider-message:{}", command.command_id),
                    chat_title: command.provider_chat_id.clone(),
                    sender_id: command.account_id.clone(),
                    sender_display_name: "Hermes Owner".to_owned(),
                    text: command
                        .payload
                        .get("text")
                        .and_then(Value::as_str)
                        .ok_or_else(|| "payload.text is required".to_owned())?
                        .to_owned(),
                    reply_to_provider_message_id: command
                        .payload
                        .get("reply_to_provider_message_id")
                        .and_then(Value::as_str)
                        .map(str::to_owned),
                    forward_origin_chat_id: None,
                    forward_origin_message_id: None,
                    forward_origin_sender_id: None,
                    forward_origin_sender_name: None,
                    forwarded_at: None,
                    message_metadata: json!({}),
                    import_batch_id,
                    occurred_at: Utc::now(),
                    delivery_state:
                        crate::integrations::whatsapp::client::WhatsappWebDeliveryState::Sent,
                })
                .await
                .map_err(fixture_error)?;
        }
        "forward" => {
            fixture_ingest
                .ingest_message(&NewWhatsappWebMessage {
                    account_id: command.account_id.clone(),
                    provider_chat_id: command.provider_chat_id.clone(),
                    provider_message_id: format!("provider-message:{}", command.command_id),
                    chat_title: command.provider_chat_id.clone(),
                    sender_id: command.account_id.clone(),
                    sender_display_name: "Hermes Owner".to_owned(),
                    text: "Forwarded message".to_owned(),
                    reply_to_provider_message_id: None,
                    forward_origin_chat_id: command
                        .payload
                        .get("from_provider_chat_id")
                        .and_then(Value::as_str)
                        .map(str::to_owned),
                    forward_origin_message_id: command
                        .payload
                        .get("from_provider_message_id")
                        .and_then(Value::as_str)
                        .map(str::to_owned),
                    forward_origin_sender_id: None,
                    forward_origin_sender_name: None,
                    forwarded_at: Some(Utc::now()),
                    message_metadata: json!({}),
                    import_batch_id,
                    occurred_at: Utc::now(),
                    delivery_state:
                        crate::integrations::whatsapp::client::WhatsappWebDeliveryState::Sent,
                })
                .await
                .map_err(fixture_error)?;
        }
        "send_media" | "send_voice_note" => {
            let provider_message_id = format!("provider-message:{}", command.command_id);
            let provider_attachment_id = format!("provider-attachment:{}", command.command_id);
            let payload_filename = command
                .payload
                .get("filename")
                .and_then(Value::as_str)
                .unwrap_or("attachment.bin")
                .to_owned();
            let media_blob = resolve_fixture_upload_media_blob(fixture_ingest, command).await?;
            fixture_ingest
                .ingest_message(&NewWhatsappWebMessage {
                    account_id: command.account_id.clone(),
                    provider_chat_id: command.provider_chat_id.clone(),
                    provider_message_id: provider_message_id.clone(),
                    chat_title: command.provider_chat_id.clone(),
                    sender_id: command.account_id.clone(),
                    sender_display_name: "Hermes Owner".to_owned(),
                    text: command
                        .payload
                        .get("caption")
                        .and_then(Value::as_str)
                        .filter(|value| !value.trim().is_empty())
                        .map(str::to_owned)
                        .unwrap_or_else(|| {
                            if command.command_kind == "send_voice_note" {
                                "Voice note".to_owned()
                            } else {
                                format!("Media attachment: {payload_filename}")
                            }
                        }),
                    reply_to_provider_message_id: None,
                    forward_origin_chat_id: None,
                    forward_origin_message_id: None,
                    forward_origin_sender_id: None,
                    forward_origin_sender_name: None,
                    forwarded_at: None,
                    message_metadata: json!({}),
                    import_batch_id: import_batch_id.clone(),
                    occurred_at: Utc::now(),
                    delivery_state:
                        crate::integrations::whatsapp::client::WhatsappWebDeliveryState::Sent,
                })
                .await
                .map_err(fixture_error)?;
            fixture_ingest
                .ingest_media(&NewWhatsappWebMedia {
                    account_id: command.account_id.clone(),
                    provider_chat_id: command.provider_chat_id.clone(),
                    provider_message_id: provider_message_id.clone(),
                    provider_attachment_id: provider_attachment_id.clone(),
                    filename: media_blob.filename,
                    content_type: media_blob.content_type,
                    size_bytes: media_blob.size_bytes,
                    sha256: media_blob.sha256,
                    storage_kind: media_blob.storage_kind,
                    storage_path: media_blob.storage_path,
                    import_batch_id,
                    observed_at: Utc::now(),
                })
                .await
                .map_err(fixture_error)?;
            publish_media_execution_progress(
                fixture_ingest,
                command,
                "provider_observed",
                100,
                Some(&provider_message_id),
                Some(&provider_attachment_id),
                None,
            )
            .await?;
            publish_media_execution_completed(
                fixture_ingest,
                command,
                Some(&provider_message_id),
                Some(&provider_attachment_id),
                None,
            )
            .await?;
        }
        "download_media" => {
            let provider_attachment_id = command
                .payload
                .get("provider_attachment_id")
                .and_then(Value::as_str)
                .or_else(|| {
                    command
                        .target_ref
                        .get("provider_attachment_id")
                        .and_then(Value::as_str)
                })
                .unwrap_or(command.command_id.as_str())
                .to_owned();
            let provider_message_id = command
                .provider_message_id
                .clone()
                .ok_or_else(|| "provider_message_id is required".to_owned())?;
            fixture_ingest
                .ingest_media(&NewWhatsappWebMedia {
                    account_id: command.account_id.clone(),
                    provider_chat_id: command.provider_chat_id.clone(),
                    provider_message_id: provider_message_id.clone(),
                    provider_attachment_id: provider_attachment_id.clone(),
                    filename: command
                        .payload
                        .get("filename")
                        .and_then(Value::as_str)
                        .map(str::to_owned),
                    content_type: command
                        .payload
                        .get("content_type")
                        .and_then(Value::as_str)
                        .unwrap_or("application/octet-stream")
                        .to_owned(),
                    size_bytes: 0,
                    sha256: fixture_media_sha256(&command.command_id),
                    storage_kind: "local_fs".to_owned(),
                    storage_path: format!("whatsapp/fixture/downloads/{}.bin", command.command_id),
                    import_batch_id,
                    observed_at: Utc::now(),
                })
                .await
                .map_err(fixture_error)?;
            publish_media_execution_progress(
                fixture_ingest,
                command,
                "provider_observed",
                100,
                Some(&provider_message_id),
                Some(&provider_attachment_id),
                None,
            )
            .await?;
            publish_media_execution_completed(
                fixture_ingest,
                command,
                Some(&provider_message_id),
                Some(&provider_attachment_id),
                None,
            )
            .await?;
        }
        "delete" => {
            fixture_ingest
                .ingest_message_delete(&NewWhatsappWebMessageDelete {
                    account_id: command.account_id.clone(),
                    provider_chat_id: command.provider_chat_id.clone(),
                    provider_message_id: command
                        .provider_message_id
                        .clone()
                        .ok_or_else(|| "provider_message_id is required".to_owned())?,
                    reason_class: "provider_command".to_owned(),
                    actor_class: "self".to_owned(),
                    import_batch_id,
                    observed_at: Utc::now(),
                })
                .await
                .map_err(fixture_error)?;
        }
        "react" | "unreact" => {
            fixture_ingest
                .ingest_reaction(&NewWhatsappWebReaction {
                    account_id: command.account_id.clone(),
                    provider_chat_id: command.provider_chat_id.clone(),
                    provider_message_id: command
                        .provider_message_id
                        .clone()
                        .ok_or_else(|| "provider_message_id is required".to_owned())?,
                    provider_actor_id: command.account_id.clone(),
                    sender_display_name: "Hermes Owner".to_owned(),
                    reaction: command
                        .payload
                        .get("reaction_emoji")
                        .and_then(Value::as_str)
                        .ok_or_else(|| "payload.reaction_emoji is required".to_owned())?
                        .to_owned(),
                    is_active: command.command_kind == "react",
                    import_batch_id,
                    observed_at: Utc::now(),
                })
                .await
                .map_err(fixture_error)?;
        }
        "archive" | "unarchive" | "pin" | "unpin" | "mute" | "unmute" | "mark_read"
        | "mark_unread" => {
            fixture_ingest
                .ingest_dialog(&NewWhatsappWebDialog {
                    account_id: command.account_id.clone(),
                    provider_chat_id: command.provider_chat_id.clone(),
                    chat_title: command
                        .target_ref
                        .get("chat_title")
                        .and_then(Value::as_str)
                        .unwrap_or(command.provider_chat_id.as_str())
                        .to_owned(),
                    chat_kind: command
                        .target_ref
                        .get("chat_kind")
                        .and_then(Value::as_str)
                        .unwrap_or("private")
                        .to_owned(),
                    is_archived: matches!(command.command_kind.as_str(), "archive"),
                    is_pinned: matches!(command.command_kind.as_str(), "pin"),
                    is_muted: match command.command_kind.as_str() {
                        "mute" => Some(true),
                        "unmute" => Some(false),
                        _ => None,
                    },
                    is_unread: match command.command_kind.as_str() {
                        "mark_unread" => Some(true),
                        "mark_read" => Some(false),
                        _ => None,
                    },
                    unread_count: None,
                    participant_count: None,
                    community_parent_chat_id: None,
                    community_parent_title: None,
                    invite_link: None,
                    is_community_root: None,
                    is_broadcast: None,
                    is_newsletter: None,
                    avatar_metadata: json!({}),
                    provider_labels: Vec::new(),
                    import_batch_id,
                    observed_at: Utc::now(),
                })
                .await
                .map_err(fixture_error)?;
        }
        "join_group" | "leave_group" => {
            fixture_ingest
                .ingest_participant(
                    &crate::integrations::whatsapp::client::NewWhatsappWebParticipant {
                        account_id: command.account_id.clone(),
                        provider_chat_id: command.provider_chat_id.clone(),
                        chat_title: command
                            .target_ref
                            .get("chat_title")
                            .and_then(Value::as_str)
                            .unwrap_or(command.provider_chat_id.as_str())
                            .to_owned(),
                        chat_kind: command
                            .target_ref
                            .get("chat_kind")
                            .and_then(Value::as_str)
                            .unwrap_or("group")
                            .to_owned(),
                        provider_member_id: format!("self-member:{}", command.account_id),
                        provider_identity_id: format!("self-identity:{}", command.account_id),
                        identity_kind: "whatsapp_self".to_owned(),
                        display_name: "Hermes Owner".to_owned(),
                        push_name: Some("Hermes Owner".to_owned()),
                        address: None,
                        business_profile: json!({}),
                        profile_photo_ref: json!({}),
                        role: "member".to_owned(),
                        status: match command.command_kind.as_str() {
                            "join_group" => "member".to_owned(),
                            "leave_group" => "left".to_owned(),
                            _ => unreachable!("unsupported join/leave command kind"),
                        },
                        is_self: true,
                        is_admin: false,
                        is_owner: false,
                        import_batch_id,
                        observed_at: Utc::now(),
                    },
                )
                .await
                .map_err(fixture_error)?;
        }
        "publish_status" => {
            publish_status_execution_started(fixture_ingest, command)
                .await
                .map_err(fixture_error)?;
            fixture_ingest
                .ingest_status(&NewWhatsappWebStatus {
                    account_id: command.account_id.clone(),
                    provider_status_id: format!("provider-status:{}", command.command_id),
                    sender_id: command.account_id.clone(),
                    sender_display_name: "Hermes Owner".to_owned(),
                    sender_identity_kind: None,
                    sender_address: None,
                    sender_push_name: None,
                    sender_business_profile: json!({}),
                    sender_profile_photo_ref: json!({}),
                    text: command
                        .payload
                        .get("text")
                        .and_then(Value::as_str)
                        .ok_or_else(|| "payload.text is required".to_owned())?
                        .to_owned(),
                    import_batch_id,
                    occurred_at: Utc::now(),
                })
                .await
                .map_err(fixture_error)?;
        }
        unsupported => {
            return Err(format!(
                "unsupported fixture command executor kind `{unsupported}`"
            ));
        }
    }

    Ok(())
}

async fn publish_command_event(
    event_store: &EventStore,
    event_bus: &EventBus,
    event_type: &str,
    command: &WhatsAppProviderWriteCommand,
    extra_payload: Value,
) -> Result<(), crate::platform::events::EventStoreError> {
    let now = Utc::now();
    let source = extra_payload
        .get("source")
        .and_then(Value::as_str)
        .unwrap_or("command_executor");
    let phase = extra_payload
        .get("phase")
        .and_then(Value::as_str)
        .unwrap_or("state_change");
    let source_id = format!(
        "{}:{}:{}:{}:{}:{}",
        command.command_id,
        command.command_kind,
        command.status,
        source,
        phase,
        now.timestamp_micros()
    );
    let event = NewEventEnvelope::builder(
        whatsapp_command_executor_event_id("command", &command.command_id, now),
        event_type.to_owned(),
        now,
        json!({
            "channel": "whatsapp",
            "account_id": command.account_id,
            "actor_id": "hermes-frontend",
            "kind": "whatsapp_provider_commands",
            "source_id": source_id,
        }),
        json!({
            "id": command.command_id,
            "entity_id": command.command_id,
            "kind": "whatsapp_provider_command",
        }),
    )
    .payload(json!({
        "account_id": command.account_id,
        "command_id": command.command_id,
        "idempotency_key": command.idempotency_key,
        "command_kind": command.command_kind,
        "provider_chat_id": command.provider_chat_id,
        "provider_message_id": command.provider_message_id,
        "status": command.status,
        "retry_count": command.retry_count,
        "max_retries": command.max_retries,
        "last_error": command.last_error,
        "reconciliation_status": command.reconciliation_status,
        "provider_observed_at": command.provider_observed_at,
        "reconciled_at": command.reconciled_at,
        "completed_at": command.completed_at,
        "payload": extra_payload,
    }))
    .build()
    .expect("WhatsApp command executor event envelope must be valid");
    event_store.append(&event).await?;
    let _ = event_bus.broadcast(event);
    Ok(())
}

fn fixture_error(error: CommunicationFixtureIngestError) -> String {
    error.to_string()
}

async fn publish_media_execution_started_event(
    fixture_ingest: &WhatsappFixtureIngestApplicationService,
    command: &WhatsAppProviderWriteCommand,
) -> Result<(), crate::platform::events::EventStoreError> {
    match command.command_kind.as_str() {
        "send_media" | "send_voice_note" => {
            publish_media_event(
                fixture_ingest,
                whatsapp_event_types::MEDIA_UPLOAD_STARTED,
                command,
                json!({
                    "status": "started",
                    "phase": "dispatching_to_provider",
                    "progress_percent": 0,
                    "attachment_id": command.payload.get("attachment_id").cloned(),
                    "blob_id": command.payload.get("blob_id").cloned(),
                    "media_type": command.payload.get("media_type").cloned(),
                    "filename": command.payload.get("filename").cloned(),
                }),
            )
            .await
        }
        "download_media" => publish_media_event(
            fixture_ingest,
            whatsapp_event_types::MEDIA_DOWNLOAD_STARTED,
            command,
            json!({
                "status": "started",
                "phase": "dispatching_to_provider",
                "progress_percent": 0,
                "provider_attachment_id": command.payload.get("provider_attachment_id").cloned(),
                "provider_media_id": command.payload.get("provider_media_id").cloned(),
                "filename": command.payload.get("filename").cloned(),
            }),
        )
        .await,
        _ => Ok(()),
    }
}

async fn publish_media_execution_failed_event(
    fixture_ingest: &WhatsappFixtureIngestApplicationService,
    command: &WhatsAppProviderWriteCommand,
    error: &str,
) -> Result<(), crate::platform::events::EventStoreError> {
    match command.command_kind.as_str() {
        "send_media" | "send_voice_note" => {
            publish_media_event(
                fixture_ingest,
                whatsapp_event_types::MEDIA_UPLOAD_FAILED,
                command,
                json!({
                    "status": "failed",
                    "error": error,
                    "attachment_id": command.payload.get("attachment_id").cloned(),
                    "blob_id": command.payload.get("blob_id").cloned(),
                    "media_type": command.payload.get("media_type").cloned(),
                }),
            )
            .await
        }
        "download_media" => publish_media_event(
            fixture_ingest,
            whatsapp_event_types::MEDIA_DOWNLOAD_FAILED,
            command,
            json!({
                "status": "failed",
                "error": error,
                "provider_attachment_id": command.payload.get("provider_attachment_id").cloned(),
                "provider_media_id": command.payload.get("provider_media_id").cloned(),
            }),
        )
        .await,
        _ => Ok(()),
    }
}

async fn publish_media_execution_progress(
    fixture_ingest: &WhatsappFixtureIngestApplicationService,
    command: &WhatsAppProviderWriteCommand,
    phase: &str,
    progress_percent: u8,
    provider_message_id: Option<&str>,
    provider_attachment_id: Option<&str>,
    message_id: Option<&str>,
) -> Result<(), String> {
    let event_type = match command.command_kind.as_str() {
        "send_media" | "send_voice_note" => whatsapp_event_types::MEDIA_UPLOAD_PROGRESS,
        "download_media" => whatsapp_event_types::MEDIA_DOWNLOAD_PROGRESS,
        _ => return Ok(()),
    };
    publish_media_event(
        fixture_ingest,
        event_type,
        command,
        json!({
            "status": "in_progress",
            "phase": phase,
            "progress_percent": progress_percent,
            "provider_message_id": provider_message_id,
            "provider_attachment_id": provider_attachment_id,
            "message_id": message_id,
        }),
    )
    .await
    .map_err(|error| error.to_string())
}

async fn publish_media_execution_completed(
    fixture_ingest: &WhatsappFixtureIngestApplicationService,
    command: &WhatsAppProviderWriteCommand,
    provider_message_id: Option<&str>,
    provider_attachment_id: Option<&str>,
    message_id: Option<&str>,
) -> Result<(), String> {
    let event_type = match command.command_kind.as_str() {
        "send_media" | "send_voice_note" => whatsapp_event_types::MEDIA_UPLOAD_COMPLETED,
        "download_media" => whatsapp_event_types::MEDIA_DOWNLOAD_COMPLETED,
        _ => return Ok(()),
    };
    publish_media_event(
        fixture_ingest,
        event_type,
        command,
        json!({
            "status": "completed",
            "phase": "provider_observed",
            "progress_percent": 100,
            "provider_message_id": provider_message_id,
            "provider_attachment_id": provider_attachment_id,
            "message_id": message_id,
        }),
    )
    .await
    .map_err(|error| error.to_string())
}

async fn publish_media_event(
    fixture_ingest: &WhatsappFixtureIngestApplicationService,
    event_type: &str,
    command: &WhatsAppProviderWriteCommand,
    extra_payload: Value,
) -> Result<(), crate::platform::events::EventStoreError> {
    let now = Utc::now();
    let source_id = format!(
        "{}:{}:{}",
        command.command_id,
        event_type,
        now.timestamp_micros()
    );
    let payload = json!({
        "account_id": command.account_id,
        "command_id": command.command_id,
        "command_kind": command.command_kind,
        "provider_chat_id": command.provider_chat_id,
        "provider_message_id": command.provider_message_id,
        "payload": extra_payload,
    });
    if let Err(error) = fixture_ingest
        .capture_media_lifecycle_event(
            &command.account_id,
            &command.command_id,
            event_type,
            payload.clone(),
            "command_executor_media",
            now,
        )
        .await
    {
        return Err(crate::platform::events::EventStoreError::Sqlx(
            sqlx::Error::Protocol(format!(
                "failed to capture whatsapp media runtime event: {error}"
            )),
        ));
    }
    let event = NewEventEnvelope::builder(
        whatsapp_command_executor_event_id("media", &command.command_id, now),
        event_type.to_owned(),
        now,
        json!({
            "channel": "whatsapp",
            "account_id": command.account_id,
            "actor_id": "hermes-frontend",
            "kind": "whatsapp_provider_commands",
            "source_id": source_id,
        }),
        json!({
            "id": command.command_id,
            "entity_id": command.command_id,
            "kind": "whatsapp_media_command",
        }),
    )
    .payload(payload)
    .build()
    .expect("WhatsApp media executor event envelope must be valid");
    fixture_ingest.event_store().append(&event).await?;
    let _ = fixture_ingest.event_bus().broadcast(event);
    Ok(())
}

fn whatsapp_command_executor_event_id(
    scope: &str,
    subject: &str,
    now: chrono::DateTime<chrono::Utc>,
) -> String {
    let seq = WHATSAPP_COMMAND_EXECUTOR_EVENT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!(
        "evt_whatsapp_executor_{}_{}_{}_{}",
        scope,
        subject.replace(|c: char| !c.is_ascii_alphanumeric(), "_"),
        now.timestamp_nanos_opt().unwrap_or_default(),
        seq
    )
}

async fn publish_status_execution_started(
    fixture_ingest: &WhatsappFixtureIngestApplicationService,
    command: &WhatsAppProviderWriteCommand,
) -> Result<(), CommunicationFixtureIngestError> {
    publish_status_runtime_event(
        fixture_ingest,
        command,
        "started",
        json!({
            "command_id": command.command_id,
            "command_kind": command.command_kind,
            "phase": "dispatching_to_provider",
        }),
    )
    .await
}

async fn publish_status_runtime_event(
    fixture_ingest: &WhatsappFixtureIngestApplicationService,
    command: &WhatsAppProviderWriteCommand,
    phase: &str,
    metadata: Value,
) -> Result<(), CommunicationFixtureIngestError> {
    let observed_at = Utc::now();
    fixture_ingest
        .capture_runtime_lifecycle_event(
            &command.account_id,
            &format!(
                "{}:status.publish:{}:{}",
                command.command_id,
                phase,
                observed_at.timestamp_micros()
            ),
            &format!("status.publish.{phase}"),
            None,
            Some(phase),
            Some("info"),
            metadata,
            "command_executor_status_publish",
            observed_at,
        )
        .await?;
    Ok(())
}

struct UploadMediaBlobDescriptor {
    filename: Option<String>,
    content_type: String,
    size_bytes: i64,
    sha256: String,
    storage_kind: String,
    storage_path: String,
}

async fn resolve_fixture_upload_media_blob(
    fixture_ingest: &WhatsappFixtureIngestApplicationService,
    command: &WhatsAppProviderWriteCommand,
) -> Result<UploadMediaBlobDescriptor, String> {
    resolve_upload_media_blob_descriptor(fixture_ingest.pool(), command).await
}

async fn resolve_upload_media_blob_descriptor(
    pool: &PgPool,
    command: &WhatsAppProviderWriteCommand,
) -> Result<UploadMediaBlobDescriptor, String> {
    let storage = CommunicationStorageStore::new(pool.clone());

    if let Some(attachment_id) = command.payload.get("attachment_id").and_then(Value::as_str) {
        let attachment = storage
            .imported_attachment_by_id(attachment_id)
            .await
            .map_err(|error| error.to_string())?
            .ok_or_else(|| format!("attachment `{attachment_id}` is not available"))?;
        return Ok(UploadMediaBlobDescriptor {
            filename: attachment.filename,
            content_type: attachment.content_type,
            size_bytes: attachment.size_bytes,
            sha256: attachment.sha256,
            storage_kind: attachment.storage_kind,
            storage_path: attachment.storage_path,
        });
    }

    let blob_id = command
        .payload
        .get("blob_id")
        .and_then(Value::as_str)
        .ok_or_else(|| "payload.blob_id is required".to_owned())?;
    let blob = storage
        .blob_by_id(blob_id)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("blob `{blob_id}` is not available"))?;

    Ok(UploadMediaBlobDescriptor {
        filename: command
            .payload
            .get("filename")
            .and_then(Value::as_str)
            .map(str::to_owned),
        content_type: blob.content_type.unwrap_or_else(|| {
            command
                .payload
                .get("content_type")
                .and_then(Value::as_str)
                .unwrap_or("application/octet-stream")
                .to_owned()
        }),
        size_bytes: blob.size_bytes,
        sha256: blob.sha256,
        storage_kind: blob.storage_kind,
        storage_path: blob.storage_path,
    })
}

fn upload_media_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{:x}", hasher.finalize())
}

fn fixture_media_sha256(seed: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(seed.as_bytes());
    format!("sha256:{:x}", hasher.finalize())
}
