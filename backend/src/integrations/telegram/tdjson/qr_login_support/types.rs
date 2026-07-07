use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Condvar, Mutex};

use crate::integrations::telegram::client::{
    TelegramQrLoginStartRequest, TelegramQrLoginStatusResponse,
};

pub(crate) type PendingQrLoginMap = Arc<Mutex<HashMap<String, TelegramQrLoginSession>>>;
pub(in crate::integrations::telegram::tdjson) type QrLoginWorkerCompletion =
    Arc<(Mutex<bool>, Condvar)>;

#[derive(Clone)]
pub(crate) struct TelegramQrLoginSession {
    pub(crate) request: TelegramQrLoginStartRequest,
    pub(crate) response: TelegramQrLoginStatusResponse,
    pub(in crate::integrations::telegram::tdjson) command_tx: Sender<TelegramQrLoginCommand>,
    pub(in crate::integrations::telegram::tdjson) worker_completion: QrLoginWorkerCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub(in crate::integrations::telegram::tdjson) enum TelegramQrLoginCommand {
    CheckPassword(String),
    Cancel,
}

#[derive(Debug, Eq, PartialEq)]
pub(in crate::integrations::telegram::tdjson) enum DrainedQrLoginCommand {
    None,
    PasswordSubmitted,
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::integrations::telegram::tdjson) struct TelegramQrLoginIdentity {
    pub(in crate::integrations::telegram::tdjson) user_id: String,
    pub(in crate::integrations::telegram::tdjson) username: Option<String>,
    pub(in crate::integrations::telegram::tdjson) suggested_account_id: String,
    pub(in crate::integrations::telegram::tdjson) suggested_display_name: String,
    pub(in crate::integrations::telegram::tdjson) suggested_external_account_id: String,
}
