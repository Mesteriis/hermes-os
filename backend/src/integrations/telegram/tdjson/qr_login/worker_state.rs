use std::path::Path;
use std::sync::mpsc::Sender;

use crate::integrations::telegram::client::TelegramQrLoginStartRequest;

use super::super::client::TdJsonClient;
use super::super::qr_login_support::{
    PendingQrLoginMap, QrLoginWorkerCompletion, TelegramQrLoginCommand,
};

pub(super) struct QrLoginWorkerContext<'a> {
    pub(super) client: &'a TdJsonClient,
    pub(super) pending_logins: &'a PendingQrLoginMap,
    pub(super) setup_id: &'a str,
    pub(super) request: &'a TelegramQrLoginStartRequest,
    pub(super) command_tx: &'a Sender<TelegramQrLoginCommand>,
    pub(super) worker_completion: &'a QrLoginWorkerCompletion,
    pub(super) database_directory: &'a Path,
}

#[derive(Default)]
pub(super) struct QrLoginRuntimeState {
    pub(super) tdlib_parameters_sent: bool,
    pub(super) database_encryption_key_checked: bool,
    pub(super) qr_requested: bool,
    pub(super) qr_link_issued: bool,
    pub(super) password_check_in_flight: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum QrLoginEventOutcome {
    Continue,
    Complete,
}
