use std::sync::mpsc::Sender;

use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::qr_login::{
    TelegramQrLoginStartRequest, TelegramQrLoginStatus, TelegramQrLoginStatusResponse,
};

use super::types::{
    PendingQrLoginMap, QrLoginWorkerCompletion, TelegramQrLoginCommand, TelegramQrLoginIdentity,
    TelegramQrLoginSession,
};

pub(in crate::integrations::telegram::tdjson) fn upsert_pending_response(
    pending_logins: &PendingQrLoginMap,
    request: TelegramQrLoginStartRequest,
    response: TelegramQrLoginStatusResponse,
    command_tx: Sender<TelegramQrLoginCommand>,
    worker_completion: QrLoginWorkerCompletion,
) -> Result<(), TelegramError> {
    let mut pending_logins = pending_logins.lock().map_err(|_| {
        TelegramError::TdlibRuntime("Telegram QR login state lock was poisoned".to_owned())
    })?;
    pending_logins.insert(
        response.setup_id.clone(),
        TelegramQrLoginSession {
            request,
            response,
            command_tx,
            worker_completion,
        },
    );
    Ok(())
}

pub(in crate::integrations::telegram::tdjson) fn mark_pending_status(
    pending_logins: &PendingQrLoginMap,
    setup_id: &str,
    status: TelegramQrLoginStatus,
    message: &str,
    poll_after_ms: u64,
) -> Result<(), TelegramError> {
    let mut pending_logins = pending_logins.lock().map_err(|_| {
        TelegramError::TdlibRuntime("Telegram QR login state lock was poisoned".to_owned())
    })?;
    if let Some(session) = pending_logins.get_mut(setup_id) {
        let response = &mut session.response;
        response.status = status;
        response.poll_after_ms = poll_after_ms;
        response.message = Some(message.to_owned());
        if !matches!(
            status,
            TelegramQrLoginStatus::WaitingQrScan | TelegramQrLoginStatus::WaitingPassword
        ) {
            response.expires_at = None;
        }
    }
    Ok(())
}

pub(in crate::integrations::telegram::tdjson) fn mark_pending_ready_status(
    pending_logins: &PendingQrLoginMap,
    setup_id: &str,
    message: &str,
    identity: Option<&TelegramQrLoginIdentity>,
) -> Result<(), TelegramError> {
    let mut pending_logins = pending_logins.lock().map_err(|_| {
        TelegramError::TdlibRuntime("Telegram QR login state lock was poisoned".to_owned())
    })?;
    if let Some(session) = pending_logins.get_mut(setup_id) {
        let response = &mut session.response;
        response.status = TelegramQrLoginStatus::Ready;
        response.poll_after_ms = 0;
        response.message = Some(message.to_owned());
        response.expires_at = None;
        if let Some(identity) = identity {
            response.telegram_user_id = Some(identity.user_id.clone());
            response.telegram_username = identity.username.clone();
            response.suggested_account_id = Some(identity.suggested_account_id.clone());
            response.suggested_display_name = Some(identity.suggested_display_name.clone());
            response.suggested_external_account_id =
                Some(identity.suggested_external_account_id.clone());
        }
    }
    Ok(())
}
