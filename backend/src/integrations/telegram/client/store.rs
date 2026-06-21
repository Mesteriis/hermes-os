use std::sync::Arc;

use sqlx::postgres::PgPool;

use crate::platform::communications::{
    ProviderAccountCommandPort, ProviderChannelMessageLookupPort,
    ProviderMessageObservationEventPort, ProviderSecretBindingCommandPort,
};

#[derive(Clone)]
pub struct TelegramStore {
    pub(super) pool: PgPool,
    provider_account_store: Arc<dyn ProviderAccountCommandPort>,
    provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
    provider_channel_message_store: Arc<dyn ProviderChannelMessageLookupPort>,
    provider_observation_events: Arc<dyn ProviderMessageObservationEventPort>,
}

impl TelegramStore {
    pub fn new(
        pool: PgPool,
        provider_account_store: Arc<dyn ProviderAccountCommandPort>,
        provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
        provider_channel_message_store: Arc<dyn ProviderChannelMessageLookupPort>,
        provider_observation_events: Arc<dyn ProviderMessageObservationEventPort>,
    ) -> Self {
        Self {
            pool,
            provider_account_store,
            provider_secret_binding_store,
            provider_channel_message_store,
            provider_observation_events,
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

    pub(in crate::integrations::telegram) fn provider_observation_events(
        &self,
    ) -> &dyn ProviderMessageObservationEventPort {
        self.provider_observation_events.as_ref()
    }
}
