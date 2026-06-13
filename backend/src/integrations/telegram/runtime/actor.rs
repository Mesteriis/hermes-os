use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use crate::domains::mail::core::{
    CommunicationIngestionStore, ProviderAccount, ProviderAccountSecretPurpose,
    ProviderCredentialError, ProviderCredentialReader,
};
use crate::integrations::telegram::client::{
    TelegramError, TelegramManualSendRequest, TelegramQrLoginStartRequest,
};
use crate::integrations::telegram::tdjson::{
    self, TdJsonClient, TelegramTdlibChatSnapshot, TelegramTdlibFileSnapshot,
    TelegramTdlibMessageSnapshot,
};
use crate::platform::config::AppConfig;
use crate::platform::secrets::{SecretReferenceStore, SecretResolver};

use super::models::TelegramHistorySyncMode;
use super::state::TelegramRuntimeCommand;
use super::{TDJSON_BOOTSTRAP_TIMEOUT, TDJSON_COMMAND_TIMEOUT, TDJSON_RECEIVE_POLL_SECONDS};

pub(super) fn spawn_tdlib_actor(
    config: AppConfig,
    account: ProviderAccount,
    session_encryption_key: Option<String>,
) -> Result<Sender<TelegramRuntimeCommand>, TelegramError> {
    if !tdjson::runtime_available(config.tdjson_path()) {
        return Err(TelegramError::TdlibRuntimeUnavailable(
            "libtdjson is not available for Telegram live runtime".to_owned(),
        ));
    }
    let start_request =
        tdlib_start_request_from_account(&config, &account, session_encryption_key)?;
    let (command_tx, command_rx) = mpsc::channel();
    let thread_name = format!(
        "telegram-tdlib-{}",
        short_thread_suffix(&account.account_id)
    );
    thread::Builder::new()
        .name(thread_name)
        .spawn(move || {
            if let Err(error) = drive_tdlib_actor(config, start_request, command_rx) {
                tracing::warn!(error = %error, "Telegram TDLib actor stopped");
            }
        })
        .map_err(|error| {
            TelegramError::TdlibRuntime(format!("failed to spawn Telegram TDLib actor: {error}"))
        })?;

    Ok(command_tx)
}

fn tdlib_start_request_from_account(
    config: &AppConfig,
    account: &ProviderAccount,
    session_encryption_key: Option<String>,
) -> Result<TelegramQrLoginStartRequest, TelegramError> {
    let api_id = config.telegram_api_id().ok_or_else(|| {
        TelegramError::InvalidRequest(
            "HERMES_TELEGRAM_API_ID is required for Telegram TDLib runtime".to_owned(),
        )
    })?;
    let api_hash = config
        .telegram_api_hash()
        .map(|secret| secret.expose_for_runtime().to_owned())
        .ok_or_else(|| {
            TelegramError::InvalidRequest(
                "HERMES_TELEGRAM_API_HASH is required for Telegram TDLib runtime".to_owned(),
            )
        })?;
    let tdlib_data_path = account
        .config
        .get("tdlib_data_path")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            TelegramError::InvalidRequest(
                "tdlib_data_path is required for Telegram TDLib runtime".to_owned(),
            )
        })?;

    Ok(TelegramQrLoginStartRequest {
        account_id: account.account_id.clone(),
        display_name: account.display_name.clone(),
        external_account_id: account.external_account_id.clone(),
        api_id: Some(api_id),
        api_hash: Some(api_hash),
        session_encryption_key,
        tdlib_data_path: Some(tdlib_data_path),
        transcription_enabled: false,
    })
}

pub(super) async fn optional_telegram_session_key(
    communication_store: &CommunicationIngestionStore,
    secret_store: &SecretReferenceStore,
    secret_resolver: &(impl SecretResolver + Sync + ?Sized),
    account_id: &str,
) -> Result<Option<String>, TelegramError> {
    let credential_reader = ProviderCredentialReader::new(
        communication_store.clone(),
        secret_store.clone(),
        secret_resolver,
    );
    match credential_reader
        .read(account_id, ProviderAccountSecretPurpose::TelegramSessionKey)
        .await
    {
        Ok(credential) => Ok(Some(credential.secret.expose_for_runtime().to_owned())),
        Err(ProviderCredentialError::MissingBinding { .. }) => Ok(None),
        Err(error) => Err(TelegramError::TdlibRuntime(format!(
            "failed to resolve Telegram session encryption key: {error}"
        ))),
    }
}

fn drive_tdlib_actor(
    config: AppConfig,
    start_request: TelegramQrLoginStartRequest,
    command_rx: mpsc::Receiver<TelegramRuntimeCommand>,
) -> Result<(), TelegramError> {
    let library = tdjson::TdJsonLibrary::load(config.tdjson_path())?;
    let client = library.create_client()?;
    prepare_tdlib_client(&client, &start_request)?;
    wait_for_tdlib_ready(&client, &start_request)?;

    while let Ok(command) = command_rx.recv() {
        match command {
            TelegramRuntimeCommand::LoadChats { limit, reply_tx } => {
                let _ = reply_tx.send(actor_load_chats(&client, limit));
            }
            TelegramRuntimeCommand::SyncHistory {
                provider_chat_id,
                from_message_id,
                limit,
                mode,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_sync_history(
                    &client,
                    &provider_chat_id,
                    from_message_id,
                    limit,
                    mode,
                ));
            }
            TelegramRuntimeCommand::SendText { request, reply_tx } => {
                let _ = reply_tx.send(actor_send_text(&client, &request));
            }
            TelegramRuntimeCommand::DownloadFile {
                file_id,
                priority,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_download_file(&client, file_id, priority));
            }
        }
    }

    let _ = client.send_json(&json!({ "@type": "close" }));
    Ok(())
}

fn prepare_tdlib_client(
    client: &TdJsonClient,
    start_request: &TelegramQrLoginStartRequest,
) -> Result<(), TelegramError> {
    let database_directory = tdjson::tdlib_database_directory(start_request);
    let files_directory = database_directory.join("files");
    std::fs::create_dir_all(&files_directory).map_err(|error| {
        TelegramError::TdlibRuntime(format!(
            "failed to create TDLib data directory `{}`: {error}",
            files_directory.display()
        ))
    })?;
    let _ = client.execute_json(&json!({
        "@type": "setLogVerbosityLevel",
        "new_verbosity_level": 1
    }));
    client.send_json(&json!({
        "@type": "getAuthorizationState",
        "@extra": "hermes-runtime-initial-authorization-state"
    }))?;
    Ok(())
}

fn wait_for_tdlib_ready(
    client: &TdJsonClient,
    start_request: &TelegramQrLoginStartRequest,
) -> Result<(), TelegramError> {
    let database_directory = tdjson::tdlib_database_directory(start_request);
    let started_at = Instant::now();
    let mut tdlib_parameters_sent = false;

    while started_at.elapsed() < TDJSON_BOOTSTRAP_TIMEOUT {
        let Some(event) = client.receive_json(TDJSON_RECEIVE_POLL_SECONDS)? else {
            continue;
        };

        if tdjson::is_tdlib_parameters_not_specified_error(&event) && !tdlib_parameters_sent {
            client.send_json(&tdjson::set_tdlib_parameters_request(
                start_request,
                &database_directory,
            )?)?;
            tdlib_parameters_sent = true;
            continue;
        }
        if tdjson::is_tdlib_database_encryption_key_needed_error(&event) {
            client.send_json(&tdjson::check_database_encryption_key_request(
                start_request,
            ))?;
            continue;
        }
        if let Some(message) = tdjson::tdlib_error_message(&event) {
            return Err(TelegramError::TdlibRuntime(message));
        }

        let Some(authorization_state) = tdjson::authorization_state(&event) else {
            continue;
        };
        match authorization_state.get("@type").and_then(Value::as_str) {
            Some("authorizationStateWaitTdlibParameters") => {
                client.send_json(&tdjson::set_tdlib_parameters_request(
                    start_request,
                    &database_directory,
                )?)?;
                tdlib_parameters_sent = true;
            }
            Some("authorizationStateWaitEncryptionKey") => {
                client.send_json(&tdjson::check_database_encryption_key_request(
                    start_request,
                ))?;
            }
            Some("authorizationStateReady") => return Ok(()),
            Some("authorizationStateClosed")
            | Some("authorizationStateClosing")
            | Some("authorizationStateLoggingOut") => {
                return Err(TelegramError::TdlibRuntime(
                    "Telegram TDLib authorization session is closed".to_owned(),
                ));
            }
            Some(wait_state) if wait_state.starts_with("authorizationStateWait") => {
                return Err(TelegramError::TdlibRuntime(format!(
                    "Telegram TDLib account is not authorized; current state is `{wait_state}`"
                )));
            }
            _ => {}
        }
    }

    Err(TelegramError::TdlibRuntime(
        "Telegram TDLib authorization did not become ready in time".to_owned(),
    ))
}

fn actor_load_chats(
    client: &TdJsonClient,
    limit: i32,
) -> Result<Vec<TelegramTdlibChatSnapshot>, TelegramError> {
    let load_extra = "hermes-runtime-load-chats";
    client.send_json(&tdjson::tdlib_load_chats_request(limit, load_extra))?;
    let load_response = receive_tdlib_extra(client, load_extra, TDJSON_COMMAND_TIMEOUT)?;
    if tdjson::tdlib_error_message(&load_response).is_some() && !is_tdlib_not_found(&load_response)
    {
        return Err(TelegramError::TdlibRuntime(
            tdjson::tdlib_error_message(&load_response)
                .unwrap_or_else(|| "TDLib loadChats failed".to_owned()),
        ));
    }

    let chats_extra = "hermes-runtime-get-chats";
    client.send_json(&tdjson::tdlib_get_chats_request(limit, chats_extra))?;
    let chats_response = receive_tdlib_extra(client, chats_extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&chats_response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    let chat_ids = tdjson::parse_tdlib_chat_ids(&chats_response)?;
    let mut snapshots = Vec::with_capacity(chat_ids.len());
    for chat_id in chat_ids {
        let extra = format!("hermes-runtime-get-chat-{chat_id}");
        client.send_json(&tdjson::tdlib_get_chat_request(chat_id, &extra))?;
        let chat_response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
        if let Some(message) = tdjson::tdlib_error_message(&chat_response) {
            return Err(TelegramError::TdlibRuntime(message));
        }
        snapshots.push(tdjson::parse_tdlib_chat_snapshot(&chat_response)?);
    }
    Ok(snapshots)
}

fn actor_sync_history(
    client: &TdJsonClient,
    provider_chat_id: &str,
    from_message_id: Option<i64>,
    limit: i32,
    mode: TelegramHistorySyncMode,
) -> Result<Vec<TelegramTdlibMessageSnapshot>, TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let page_limit = limit.clamp(1, 100);
    let mut cursor = from_message_id;
    let mut snapshots = Vec::new();
    let mut page_index = 0;

    loop {
        let extra = format!(
            "hermes-runtime-history-{chat_id}-{}-{page_index}",
            cursor.unwrap_or(0)
        );
        client.send_json(&tdjson::tdlib_get_chat_history_request(
            chat_id, cursor, page_limit, false, &extra,
        ))?;
        let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
        if let Some(message) = tdjson::tdlib_error_message(&response) {
            return Err(TelegramError::TdlibRuntime(message));
        }
        let page = tdjson::parse_tdlib_message_list(&response)?;
        if page.is_empty() {
            break;
        }

        let page_len = page.len();
        let next_cursor = oldest_tdlib_message_id(&page);
        snapshots.extend(page);
        if mode != TelegramHistorySyncMode::Full || page_len < page_limit as usize {
            break;
        }
        if next_cursor.is_none() || next_cursor == cursor {
            break;
        }
        cursor = next_cursor;
        page_index += 1;
    }

    Ok(snapshots)
}

fn actor_send_text(
    client: &TdJsonClient,
    request: &TelegramManualSendRequest,
) -> Result<TelegramTdlibMessageSnapshot, TelegramError> {
    let chat_id = tdlib_provider_chat_id(&request.provider_chat_id)?;
    let extra = format!("hermes-runtime-send-{}", request.command_id.trim());
    client.send_json(&tdjson::tdlib_send_text_message_request(
        chat_id,
        &request.text,
        &extra,
    )?)?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    tdjson::parse_tdlib_message_snapshot(&response)
}

fn actor_download_file(
    client: &TdJsonClient,
    file_id: i64,
    priority: i32,
) -> Result<TelegramTdlibFileSnapshot, TelegramError> {
    let extra = format!("hermes-runtime-download-file-{file_id}");
    client.send_json(&tdjson::tdlib_download_file_request(
        file_id, priority, &extra,
    ))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    tdjson::parse_tdlib_file_snapshot(&response)
}

fn receive_tdlib_extra(
    client: &TdJsonClient,
    expected_extra: &str,
    timeout: Duration,
) -> Result<Value, TelegramError> {
    let started_at = Instant::now();
    while started_at.elapsed() < timeout {
        let Some(event) = client.receive_json(TDJSON_RECEIVE_POLL_SECONDS)? else {
            continue;
        };
        if event.get("@extra").and_then(Value::as_str) == Some(expected_extra) {
            return Ok(event);
        }
        if let Some(message) = tdjson::tdlib_error_message(&event) {
            tracing::debug!(error = %message, "ignored unrelated TDLib error while waiting for correlated response");
        }
    }
    Err(TelegramError::TdlibRuntime(format!(
        "TDLib request `{expected_extra}` timed out"
    )))
}

pub(super) fn oldest_tdlib_message_id(snapshots: &[TelegramTdlibMessageSnapshot]) -> Option<i64> {
    snapshots
        .iter()
        .filter_map(|snapshot| snapshot.provider_message_id.trim().parse::<i64>().ok())
        .min()
}

fn tdlib_provider_chat_id(provider_chat_id: &str) -> Result<i64, TelegramError> {
    provider_chat_id.trim().parse::<i64>().map_err(|_| {
        TelegramError::InvalidRequest(format!(
            "TDLib provider_chat_id `{}` must be a Telegram numeric chat id",
            provider_chat_id.trim()
        ))
    })
}

fn is_tdlib_not_found(event: &Value) -> bool {
    event.get("@type").and_then(Value::as_str) == Some("error")
        && event.get("code").and_then(Value::as_i64) == Some(404)
}

fn short_thread_suffix(account_id: &str) -> String {
    let sanitized = account_id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_owned();
    if sanitized.is_empty() {
        "account".to_owned()
    } else {
        sanitized.chars().take(32).collect()
    }
}
