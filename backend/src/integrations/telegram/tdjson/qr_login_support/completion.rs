use std::sync::{Arc, Condvar, Mutex};

use crate::integrations::telegram::client::TelegramError;

use super::constants::QR_CANCEL_WAIT_TIMEOUT;
use super::types::QrLoginWorkerCompletion;

pub(in crate::integrations::telegram::tdjson) fn new_worker_completion() -> QrLoginWorkerCompletion
{
    Arc::new((Mutex::new(false), Condvar::new()))
}

pub(in crate::integrations::telegram::tdjson) fn mark_worker_complete(
    worker_completion: &QrLoginWorkerCompletion,
) {
    let (lock, cvar) = &**worker_completion;
    if let Ok(mut completed) = lock.lock() {
        *completed = true;
        cvar.notify_all();
    }
}

pub(in crate::integrations::telegram::tdjson) fn wait_for_worker_completion(
    worker_completion: &QrLoginWorkerCompletion,
) -> Result<(), TelegramError> {
    let (lock, cvar) = &**worker_completion;
    let completed = lock.lock().map_err(|_| {
        TelegramError::TdlibRuntime("Telegram QR login worker lock was poisoned".to_owned())
    })?;
    if *completed {
        return Ok(());
    }
    let _ = cvar
        .wait_timeout(completed, QR_CANCEL_WAIT_TIMEOUT)
        .map_err(|_| {
            TelegramError::TdlibRuntime("Telegram QR login worker lock was poisoned".to_owned())
        })?;
    Ok(())
}
