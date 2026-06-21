use serde_json::Value;

use crate::domains::communications::core::CommunicationIngestionPort;
use crate::platform::communications::ProviderAccount;

use super::super::models::MailSyncSettings;
use super::super::store::MailSyncStatePort;

pub(in crate::workflows::mail_background_sync) struct ProviderSyncContext<'a> {
    pub(in crate::workflows::mail_background_sync) store: &'a MailSyncStatePort,
    pub(in crate::workflows::mail_background_sync) communication_store:
        &'a CommunicationIngestionPort,
    pub(in crate::workflows::mail_background_sync) account: &'a ProviderAccount,
    pub(in crate::workflows::mail_background_sync) run_id: &'a str,
    pub(in crate::workflows::mail_background_sync) settings: &'a MailSyncSettings,
    pub(in crate::workflows::mail_background_sync) checkpoint_before: Option<Value>,
}

#[derive(Clone, Copy)]
pub(in crate::workflows::mail_background_sync::provider) struct ImapAccountConfig<'a> {
    pub(in crate::workflows::mail_background_sync::provider) host: &'a str,
    pub(in crate::workflows::mail_background_sync::provider) port: u16,
    pub(in crate::workflows::mail_background_sync::provider) tls: bool,
    pub(in crate::workflows::mail_background_sync::provider) mailbox: &'a str,
}
