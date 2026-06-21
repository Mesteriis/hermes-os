use sqlx::postgres::PgPool;

use crate::domains::communications::core::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore,
};
use crate::domains::communications::messages::ProviderChannelMessageStore;

#[derive(Clone)]
pub struct TelegramStore {
    pub(super) pool: PgPool,
}

impl TelegramStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub(super) fn provider_account_store(&self) -> CommunicationProviderAccountStore {
        CommunicationProviderAccountStore::new(self.pool.clone())
    }

    pub(super) fn provider_secret_binding_store(&self) -> CommunicationProviderSecretBindingStore {
        CommunicationProviderSecretBindingStore::new(self.pool.clone())
    }

    pub(super) fn provider_channel_message_store(&self) -> ProviderChannelMessageStore {
        ProviderChannelMessageStore::new(self.pool.clone())
    }
}
