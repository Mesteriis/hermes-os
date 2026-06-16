use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::domains::mail::background_sync::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use crate::domains::mail::core::CommunicationIngestionStore;
use crate::domains::mail::storage::MailStorageStore;
use crate::integrations::telegram::client::TelegramStore;
use crate::platform::config::AppConfig;
use crate::platform::secrets::{SecretReferenceStore, SecretResolver};

use super::state::TelegramRuntimeActorHandle;

mod account;
mod actor_states;
pub(crate) mod command_executor;
mod lifecycle;
mod media_download;
mod registry;
mod send;
mod sync_chats;
mod sync_history;
mod sync_history_tdlib;
mod tdlib_actor;
mod topics;

#[derive(Clone, Default)]
pub struct TelegramRuntimeManager {
    actors: Arc<Mutex<HashMap<String, TelegramRuntimeActorHandle>>>,
}

pub(crate) struct TelegramMediaDownloadContext<'a, S: SecretResolver + Sync + ?Sized> {
    pub(crate) communication_store: &'a CommunicationIngestionStore,
    pub(crate) telegram_store: &'a TelegramStore,
    pub(crate) mail_store: &'a MailStorageStore,
    pub(crate) secret_store: &'a SecretReferenceStore,
    pub(crate) secret_resolver: &'a S,
    pub(crate) config: &'a AppConfig,
}

fn telegram_media_blob_root() -> &'static std::path::Path {
    std::path::Path::new(DEFAULT_MAIL_SYNC_BLOB_ROOT)
}
