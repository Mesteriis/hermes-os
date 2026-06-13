use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::Instant;

use serde_json::{Value, json};
use tokio::task;

use crate::integrations::telegram::client::{
    TelegramError, TelegramQrLoginPasswordRequest, TelegramQrLoginStartRequest,
    TelegramQrLoginStatus, TelegramQrLoginStatusResponse,
};
use crate::platform::config::AppConfig;

use super::client::{TdJsonClient, TdJsonLibrary};
use super::identifiers::safe_path_segment;
use super::parsing::{
    authorization_state, is_tdlib_database_encryption_key_needed_error,
    is_tdlib_parameters_not_specified_error, tdlib_error_message,
};
use super::qr_login_support::{
    DrainedQrLoginCommand, PendingQrLoginMap, QR_FIRST_LINK_TIMEOUT, QR_POLL_AFTER_MS,
    QR_SESSION_LIFETIME, QrLoginWorkerCompletion, TelegramQrLoginCommand,
    fetch_authorized_user_identity, mark_pending_ready_status, mark_pending_status,
    mark_worker_complete, new_setup_id, new_worker_completion, password_hint,
    qr_preparing_response, qr_waiting_response, short_thread_suffix, state_allows_qr_request,
    upsert_pending_response, wait_for_worker_completion,
};
use super::requests::{
    check_database_encryption_key_request, set_tdlib_parameters_request, tdlib_database_directory,
};

pub(crate) async fn start_qr_login(
    config: AppConfig,
    pending_logins: PendingQrLoginMap,
    request: TelegramQrLoginStartRequest,
) -> Result<TelegramQrLoginStatusResponse, TelegramError> {
    request.validate()?;
    task::spawn_blocking(move || start_qr_login_driver(config, pending_logins, request))
        .await
        .map_err(|error| {
            TelegramError::TdlibRuntime(format!("Telegram QR login worker failed: {error}"))
        })?
}

pub(crate) fn submit_qr_login_password(
    pending_logins: PendingQrLoginMap,
    setup_id: &str,
    request: TelegramQrLoginPasswordRequest,
) -> Result<TelegramQrLoginStatusResponse, TelegramError> {
    let setup_id = setup_id.trim();
    if setup_id.is_empty() {
        return Err(TelegramError::InvalidRequest(
            "setup_id must not be empty".to_owned(),
        ));
    }
    if request.password.is_empty() {
        return Err(TelegramError::InvalidRequest(
            "password must not be empty".to_owned(),
        ));
    }

    let mut pending_logins = pending_logins.lock().map_err(|_| {
        TelegramError::TdlibRuntime("Telegram QR login state lock was poisoned".to_owned())
    })?;
    let session = pending_logins
        .get_mut(setup_id)
        .ok_or(TelegramError::QrLoginNotFound)?;
    if session.response.status != TelegramQrLoginStatus::WaitingPassword {
        return Err(TelegramError::InvalidRequest(
            "Telegram QR login is not waiting for a password".to_owned(),
        ));
    }

    session
        .command_tx
        .send(TelegramQrLoginCommand::CheckPassword(request.password))
        .map_err(|_| {
            TelegramError::TdlibRuntime(
                "Telegram QR login worker is no longer accepting password commands".to_owned(),
            )
        })?;
    session.response.message = Some("Checking Telegram password.".to_owned());
    session.response.poll_after_ms = QR_POLL_AFTER_MS;

    Ok(session.response.clone())
}

pub(crate) fn cancel_qr_login(
    pending_logins: PendingQrLoginMap,
    setup_id: &str,
) -> Result<(), TelegramError> {
    let setup_id = setup_id.trim();
    if setup_id.is_empty() {
        return Err(TelegramError::InvalidRequest(
            "setup_id must not be empty".to_owned(),
        ));
    }

    let session = {
        let mut pending_logins = pending_logins.lock().map_err(|_| {
            TelegramError::TdlibRuntime("Telegram QR login state lock was poisoned".to_owned())
        })?;
        pending_logins
            .remove(setup_id)
            .ok_or(TelegramError::QrLoginNotFound)?
    };
    let _ = session.command_tx.send(TelegramQrLoginCommand::Cancel);
    wait_for_worker_completion(&session.worker_completion)?;
    let mut pending_logins = pending_logins.lock().map_err(|_| {
        TelegramError::TdlibRuntime("Telegram QR login state lock was poisoned".to_owned())
    })?;
    pending_logins.remove(setup_id);
    Ok(())
}

pub(super) fn cancel_existing_qr_logins_for_account(
    pending_logins: &PendingQrLoginMap,
    account_id: &str,
) -> Result<(), TelegramError> {
    let sessions = {
        let mut pending = pending_logins.lock().map_err(|_| {
            TelegramError::TdlibRuntime("Telegram QR login state lock was poisoned".to_owned())
        })?;
        let setup_ids = pending
            .iter()
            .filter(|(_, session)| session.response.account_id == account_id)
            .map(|(setup_id, _)| setup_id.clone())
            .collect::<Vec<_>>();
        setup_ids
            .into_iter()
            .filter_map(|setup_id| pending.remove(&setup_id).map(|session| (setup_id, session)))
            .collect::<Vec<_>>()
    };

    for (setup_id, session) in sessions {
        let _ = session.command_tx.send(TelegramQrLoginCommand::Cancel);
        wait_for_worker_completion(&session.worker_completion)?;
        let mut pending = pending_logins.lock().map_err(|_| {
            TelegramError::TdlibRuntime("Telegram QR login state lock was poisoned".to_owned())
        })?;
        pending.remove(&setup_id);
    }

    Ok(())
}

fn start_qr_login_driver(
    config: AppConfig,
    pending_logins: PendingQrLoginMap,
    request: TelegramQrLoginStartRequest,
) -> Result<TelegramQrLoginStatusResponse, TelegramError> {
    let _runtime_probe = TdJsonLibrary::load(config.tdjson_path())?;
    cancel_existing_qr_logins_for_account(&pending_logins, &request.account_id)?;
    let (command_tx, command_rx) = mpsc::channel();
    let worker_completion = new_worker_completion();
    let setup_id = new_setup_id(&request.account_id);
    let response = qr_preparing_response(&setup_id, &request.account_id);
    let thread_name = format!(
        "telegram-qr-login-{}",
        short_thread_suffix(&request.account_id)
    );

    upsert_pending_response(
        &pending_logins,
        response.clone(),
        command_tx.clone(),
        Arc::clone(&worker_completion),
    )?;

    thread::Builder::new()
        .name(thread_name)
        .spawn({
            let setup_id = setup_id.clone();
            let pending_logins = Arc::clone(&pending_logins);
            let worker_completion = Arc::clone(&worker_completion);
            move || {
                let result = drive_qr_login(
                    config,
                    pending_logins.clone(),
                    request,
                    setup_id.clone(),
                    command_tx,
                    command_rx,
                    Arc::clone(&worker_completion),
                );
                if let Err(error) = result {
                    let _ = mark_pending_status(
                        &pending_logins,
                        &setup_id,
                        TelegramQrLoginStatus::Failed,
                        "Telegram QR login failed before the QR code was issued.",
                        0,
                    );
                    tracing::warn!(error = %error, "Telegram QR login worker failed");
                }
                mark_worker_complete(&worker_completion);
            }
        })
        .map_err(|error| {
            let _ = pending_logins.lock().map(|mut pending| {
                pending.remove(&setup_id);
            });
            TelegramError::TdlibRuntime(format!(
                "failed to spawn Telegram QR login worker: {error}"
            ))
        })?;

    Ok(response)
}

fn drive_qr_login(
    config: AppConfig,
    pending_logins: PendingQrLoginMap,
    request: TelegramQrLoginStartRequest,
    setup_id: String,
    command_tx: Sender<TelegramQrLoginCommand>,
    command_rx: Receiver<TelegramQrLoginCommand>,
    worker_completion: QrLoginWorkerCompletion,
) -> Result<(), TelegramError> {
    let library = TdJsonLibrary::load(config.tdjson_path())?;
    let client = library.create_client()?;
    let database_directory = tdlib_database_directory(&request);
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
        "@extra": "hermes-initial-authorization-state"
    }))?;

    let started_at = Instant::now();
    let mut tdlib_parameters_sent = false;
    let mut database_encryption_key_checked = false;
    let mut qr_requested = false;
    let mut qr_link_issued = false;
    let mut password_check_in_flight = false;

    loop {
        match drain_qr_login_commands(&client, &command_rx)? {
            DrainedQrLoginCommand::Cancelled => return Ok(()),
            DrainedQrLoginCommand::PasswordSubmitted => {
                password_check_in_flight = true;
                mark_pending_status(
                    &pending_logins,
                    &setup_id,
                    TelegramQrLoginStatus::WaitingPassword,
                    "Checking Telegram password.",
                    QR_POLL_AFTER_MS,
                )?;
            }
            DrainedQrLoginCommand::None => {}
        }

        if !qr_link_issued && started_at.elapsed() > QR_FIRST_LINK_TIMEOUT {
            mark_pending_status(
                &pending_logins,
                &setup_id,
                TelegramQrLoginStatus::Failed,
                "Telegram TDLib did not return a QR confirmation link in time.",
                0,
            )?;
            let _ = client.send_json(&json!({ "@type": "close" }));
            return Ok(());
        }
        if started_at.elapsed() > QR_SESSION_LIFETIME {
            mark_pending_status(
                &pending_logins,
                &setup_id,
                TelegramQrLoginStatus::Expired,
                "Telegram QR login session expired; start a new QR login.",
                0,
            )?;
            let _ = client.send_json(&json!({ "@type": "close" }));
            return Ok(());
        }

        let Some(event) = client.receive_json(1.0)? else {
            continue;
        };

        if is_tdlib_parameters_not_specified_error(&event) {
            if !tdlib_parameters_sent {
                send_tdlib_parameters(&client, &request, &database_directory)?;
                tdlib_parameters_sent = true;
            }
            continue;
        }
        if is_tdlib_database_encryption_key_needed_error(&event) {
            if !database_encryption_key_checked {
                client.send_json(&check_database_encryption_key_request(&request))?;
                database_encryption_key_checked = true;
            }
            continue;
        }

        if let Some(message) = tdlib_error_message(&event) {
            if password_check_in_flight {
                password_check_in_flight = false;
                mark_pending_status(
                    &pending_logins,
                    &setup_id,
                    TelegramQrLoginStatus::WaitingPassword,
                    "Telegram password was rejected. Try again.",
                    QR_POLL_AFTER_MS,
                )?;
                continue;
            }
            return Err(TelegramError::TdlibRuntime(message));
        }

        let Some(authorization_state) = authorization_state(&event) else {
            continue;
        };
        let Some(state_type) = authorization_state.get("@type").and_then(Value::as_str) else {
            continue;
        };

        match state_type {
            "authorizationStateWaitTdlibParameters" => {
                send_tdlib_parameters(&client, &request, &database_directory)?;
                tdlib_parameters_sent = true;
            }
            "authorizationStateWaitEncryptionKey" => {
                client.send_json(&check_database_encryption_key_request(&request))?;
                database_encryption_key_checked = true;
            }
            state if state_allows_qr_request(state) && !qr_requested => {
                client.send_json(&json!({
                    "@type": "requestQrCodeAuthentication",
                    "other_user_ids": [],
                    "@extra": "hermes-request-qr-code-authentication"
                }))?;
                qr_requested = true;
            }
            "authorizationStateWaitOtherDeviceConfirmation" => {
                qr_link_issued = true;
                let link = authorization_state
                    .get("link")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| {
                        TelegramError::TdlibRuntime(
                            "TDLib QR authorization state did not include a link".to_owned(),
                        )
                    })?;
                let response = qr_waiting_response(&setup_id, &request.account_id, link)?;
                upsert_pending_response(
                    &pending_logins,
                    response.clone(),
                    command_tx.clone(),
                    Arc::clone(&worker_completion),
                )?;
            }
            "authorizationStateWaitPassword" => {
                password_check_in_flight = false;
                let password_hint = password_hint(authorization_state);
                let message = password_hint
                    .as_deref()
                    .map(|hint| {
                        format!("Telegram requires your 2-step verification password. Hint: {hint}")
                    })
                    .unwrap_or_else(|| {
                        "Telegram requires your 2-step verification password.".to_owned()
                    });
                mark_pending_status(
                    &pending_logins,
                    &setup_id,
                    TelegramQrLoginStatus::WaitingPassword,
                    &message,
                    QR_POLL_AFTER_MS,
                )?;
            }
            "authorizationStateReady" => {
                let identity = match fetch_authorized_user_identity(&client) {
                    Ok(identity) => identity,
                    Err(error) => {
                        tracing::warn!(
                            error = %error,
                            "Telegram QR login completed, but TDLib user identity lookup failed"
                        );
                        None
                    }
                };
                let message = if qr_requested {
                    "Telegram QR login confirmed on the other device."
                } else {
                    "Telegram TDLib session is already authorized."
                };
                mark_pending_ready_status(&pending_logins, &setup_id, message, identity.as_ref())?;
                let _ = client.send_json(&json!({ "@type": "close" }));
                return Ok(());
            }
            "authorizationStateClosed" => {
                mark_pending_status(
                    &pending_logins,
                    &setup_id,
                    TelegramQrLoginStatus::Failed,
                    "Telegram TDLib authorization session closed before QR login completed.",
                    0,
                )?;
                return Ok(());
            }
            "authorizationStateClosing" | "authorizationStateLoggingOut" => {
                mark_pending_status(
                    &pending_logins,
                    &setup_id,
                    TelegramQrLoginStatus::Failed,
                    "Telegram TDLib authorization session is closing.",
                    0,
                )?;
                return Ok(());
            }
            unsupported if qr_requested => {
                mark_pending_status(
                    &pending_logins,
                    &setup_id,
                    TelegramQrLoginStatus::Failed,
                    &format!(
                        "Telegram QR login requires unsupported authorization state `{unsupported}`."
                    ),
                    0,
                )?;
                return Ok(());
            }
            _ => {}
        }
    }
}

fn drain_qr_login_commands(
    client: &TdJsonClient,
    command_rx: &Receiver<TelegramQrLoginCommand>,
) -> Result<DrainedQrLoginCommand, TelegramError> {
    let mut password_submitted = false;
    loop {
        match command_rx.try_recv() {
            Ok(TelegramQrLoginCommand::CheckPassword(password)) => {
                client.send_json(&json!({
                    "@type": "checkAuthenticationPassword",
                    "password": password,
                    "@extra": "hermes-check-authentication-password"
                }))?;
                password_submitted = true;
            }
            Ok(TelegramQrLoginCommand::Cancel) => {
                client.send_json(&json!({ "@type": "close" }))?;
                return Ok(DrainedQrLoginCommand::Cancelled);
            }
            Err(TryRecvError::Empty | TryRecvError::Disconnected) => {
                return Ok(if password_submitted {
                    DrainedQrLoginCommand::PasswordSubmitted
                } else {
                    DrainedQrLoginCommand::None
                });
            }
        }
    }
}

fn send_tdlib_parameters(
    client: &TdJsonClient,
    request: &TelegramQrLoginStartRequest,
    database_directory: &Path,
) -> Result<(), TelegramError> {
    client.send_json(&set_tdlib_parameters_request(request, database_directory)?)
}
