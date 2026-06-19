use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::domains::mail::background_sync::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use crate::domains::mail::storage::MailStorageStore;
use crate::integrations::telegram::client::TelegramStore;
use crate::platform::config::AppConfig;
use crate::platform::events::EventBus;
use crate::platform::secrets::{SecretReferenceStore, SecretResolver};
use crate::vault::{CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore};
use sqlx::PgPool;

use super::state::TelegramRuntimeActorHandle;

mod account;
mod actor_states;
mod chat_event_payloads;
mod chat_events;
pub(crate) mod command_executor;
mod command_executor_dispatch;
mod command_executor_media;
mod lifecycle;
mod media_download;
mod message_events;
mod participant_events;
mod participants;
mod realtime_events;
mod registry;
mod search;
mod send;
mod sync_chats;
mod sync_history;
mod sync_history_tdlib;
mod tdlib_actor;
mod topic_events;
mod topics;

pub(crate) use self::realtime_events::TelegramRuntimeEventBridgeContext;
pub(crate) use self::search::TelegramProviderSearchRequest;

#[derive(Clone, Default)]
pub struct TelegramRuntimeManager {
    actors: Arc<Mutex<HashMap<String, TelegramRuntimeActorHandle>>>,
}

pub(crate) struct TelegramMediaDownloadContext<'a, S: SecretResolver + Sync + ?Sized> {
    pub(crate) provider_account_store: &'a CommunicationProviderAccountStore,
    pub(crate) provider_secret_binding_store: &'a CommunicationProviderSecretBindingStore,
    pub(crate) telegram_store: &'a TelegramStore,
    pub(crate) mail_store: &'a MailStorageStore,
    pub(crate) secret_store: &'a SecretReferenceStore,
    pub(crate) secret_resolver: &'a S,
    pub(crate) config: &'a AppConfig,
    pub(crate) event_bridge: Option<TelegramRuntimeEventBridgeContext>,
}

pub(crate) struct TelegramMemberSyncContext<'a, S: SecretResolver + Sync + ?Sized> {
    pub(crate) provider_account_store: &'a CommunicationProviderAccountStore,
    pub(crate) provider_secret_binding_store: &'a CommunicationProviderSecretBindingStore,
    pub(crate) telegram_store: &'a TelegramStore,
    pub(crate) secret_store: &'a SecretReferenceStore,
    pub(crate) secret_resolver: &'a S,
    pub(crate) config: &'a AppConfig,
    pub(crate) event_bridge: Option<TelegramRuntimeEventBridgeContext>,
}

pub(crate) struct TelegramRuntimeOperationContext<'a, S: SecretResolver + Sync + ?Sized> {
    pub(crate) provider_account_store: &'a CommunicationProviderAccountStore,
    pub(crate) provider_secret_binding_store: &'a CommunicationProviderSecretBindingStore,
    pub(crate) telegram_store: &'a TelegramStore,
    pub(crate) secret_store: &'a SecretReferenceStore,
    pub(crate) secret_resolver: &'a S,
    pub(crate) config: &'a AppConfig,
    pub(crate) event_bridge: Option<TelegramRuntimeEventBridgeContext>,
}

pub(crate) struct TelegramRuntimeStartContext<'a, S: SecretResolver + Sync + ?Sized> {
    pub(crate) provider_account_store: &'a CommunicationProviderAccountStore,
    pub(crate) provider_secret_binding_store: &'a CommunicationProviderSecretBindingStore,
    pub(crate) secret_store: &'a SecretReferenceStore,
    pub(crate) secret_resolver: &'a S,
    pub(crate) config: &'a AppConfig,
    pub(crate) event_bus: &'a EventBus,
    pub(crate) event_store_pool: Option<PgPool>,
}

fn telegram_media_blob_root() -> &'static std::path::Path {
    std::path::Path::new(DEFAULT_MAIL_SYNC_BLOB_ROOT)
}
