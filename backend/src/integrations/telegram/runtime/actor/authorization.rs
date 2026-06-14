use std::time::Instant;

use serde_json::{Value, json};

use crate::integrations::telegram::client::{TelegramError, TelegramQrLoginStartRequest};
use crate::integrations::telegram::tdjson::{self, TdJsonClient};

use super::super::{TDJSON_BOOTSTRAP_TIMEOUT, TDJSON_RECEIVE_POLL_SECONDS};

pub(super) fn prepare_tdlib_client(
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

pub(super) fn wait_for_tdlib_ready(
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
