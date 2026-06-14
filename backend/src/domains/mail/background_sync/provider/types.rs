use serde_json::Value;

use crate::domains::mail::core::{CommunicationIngestionStore, ProviderAccount};

use super::super::models::MailSyncSettings;
use super::super::store::MailSyncStore;

pub(in crate::domains::mail::background_sync) struct ProviderSyncContext<'a> {
    pub(in crate::domains::mail::background_sync) store: &'a MailSyncStore,
    pub(in crate::domains::mail::background_sync) communication_store:
        &'a CommunicationIngestionStore,
    pub(in crate::domains::mail::background_sync) account: &'a ProviderAccount,
    pub(in crate::domains::mail::background_sync) run_id: &'a str,
    pub(in crate::domains::mail::background_sync) settings: &'a MailSyncSettings,
    pub(in crate::domains::mail::background_sync) checkpoint_before: Option<Value>,
}

#[derive(Clone, Copy)]
pub(in crate::domains::mail::background_sync::provider) struct ImapAccountConfig<'a> {
    pub(in crate::domains::mail::background_sync::provider) host: &'a str,
    pub(in crate::domains::mail::background_sync::provider) port: u16,
    pub(in crate::domains::mail::background_sync::provider) tls: bool,
    pub(in crate::domains::mail::background_sync::provider) mailbox: &'a str,
}
