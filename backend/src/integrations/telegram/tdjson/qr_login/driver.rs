use std::sync::Arc;
use std::sync::mpsc;
use std::thread;

use tokio::task;

use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::qr_login::{
    TelegramQrLoginStartRequest, TelegramQrLoginStatusResponse,
};
use crate::platform::config::app_config::AppConfig;

use super::super::client::TdJsonLibrary;
use super::super::qr_login_support::completion::{mark_worker_complete, new_worker_completion};
use super::super::qr_login_support::identifiers::{new_setup_id, short_thread_suffix};
use super::super::qr_login_support::pending::{mark_pending_status, upsert_pending_response};
use super::super::qr_login_support::responses::qr_preparing_response;
use super::super::qr_login_support::types::PendingQrLoginMap;
use super::commands::cancel_existing_qr_logins_for_account;
use super::worker::drive_qr_login;

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
        request.clone(),
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
                        crate::integrations::telegram::client::models::qr_login::TelegramQrLoginStatus::Failed,
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
