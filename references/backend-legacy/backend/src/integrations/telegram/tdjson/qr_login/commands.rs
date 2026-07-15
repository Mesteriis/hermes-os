use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::qr_login::{
    TelegramQrLoginPasswordRequest, TelegramQrLoginStatus, TelegramQrLoginStatusResponse,
};

use super::super::qr_login_support::completion::wait_for_worker_completion;
use super::super::qr_login_support::constants::QR_POLL_AFTER_MS;
use super::super::qr_login_support::types::{PendingQrLoginMap, TelegramQrLoginCommand};

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

pub(in crate::integrations::telegram::tdjson) fn cancel_existing_qr_logins_for_account(
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
