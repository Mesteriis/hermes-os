mod accounts;
mod evidence;
mod ingestion;
mod intelligence;
mod queries;
mod sessions;

use sqlx::postgres::PgPool;

use crate::domains::communications::core::CommunicationProviderAccountStore;
use crate::domains::communications::messages::ProviderChannelMessageStore;

#[derive(Clone)]
pub struct WhatsappWebStore {
    pub(in crate::integrations::whatsapp::client::store) pool: PgPool,
}

impl WhatsappWebStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub(in crate::integrations::whatsapp::client) fn provider_account_store(
        &self,
    ) -> CommunicationProviderAccountStore {
        CommunicationProviderAccountStore::new(self.pool.clone())
    }

    pub(in crate::integrations::whatsapp::client) fn provider_channel_message_store(
        &self,
    ) -> ProviderChannelMessageStore {
        ProviderChannelMessageStore::new(self.pool.clone())
    }
}
