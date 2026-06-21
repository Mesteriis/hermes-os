use std::sync::Arc;

use sqlx::postgres::PgPool;

use crate::platform::communications::{
    ProviderAccountCommandPort, ProviderChannelMessageLookupPort, ProviderSecretBindingCommandPort,
};

#[derive(Clone)]
pub struct TelegramStore {
    pub(super) pool: PgPool,
    provider_account_store: Arc<dyn ProviderAccountCommandPort>,
    provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
    provider_channel_message_store: Arc<dyn ProviderChannelMessageLookupPort>,
}

impl TelegramStore {
    pub fn new(
        pool: PgPool,
        provider_account_store: Arc<dyn ProviderAccountCommandPort>,
        provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
        provider_channel_message_store: Arc<dyn ProviderChannelMessageLookupPort>,
    ) -> Self {
        Self {
            pool,
            provider_account_store,
            provider_secret_binding_store,
            provider_channel_message_store,
        }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub(super) fn provider_account_store(&self) -> &dyn ProviderAccountCommandPort {
        self.provider_account_store.as_ref()
    }

    pub(super) fn provider_secret_binding_store(&self) -> &dyn ProviderSecretBindingCommandPort {
        self.provider_secret_binding_store.as_ref()
    }

    pub(super) fn provider_channel_message_store(&self) -> &dyn ProviderChannelMessageLookupPort {
        self.provider_channel_message_store.as_ref()
    }
}
