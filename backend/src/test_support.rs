use std::sync::Arc;

use sqlx::PgPool;

use crate::domains::communications::core::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore,
};
use crate::domains::communications::messages::ProviderChannelMessageStore;
use crate::integrations::telegram::client::TelegramStore;
use crate::integrations::whatsapp::client::WhatsappWebStore;
use crate::platform::communications::{EmailProviderKind, NewProviderAccount, ProviderAccount};

pub fn communication_provider_account_store(pool: &PgPool) -> CommunicationProviderAccountStore {
    CommunicationProviderAccountStore::new(pool.clone())
}

pub fn communication_provider_secret_binding_store(
    pool: &PgPool,
) -> CommunicationProviderSecretBindingStore {
    CommunicationProviderSecretBindingStore::new(pool.clone())
}

pub fn telegram_store(pool: &PgPool) -> TelegramStore {
    TelegramStore::new(
        pool.clone(),
        Arc::new(communication_provider_account_store(pool)),
        Arc::new(communication_provider_secret_binding_store(pool)),
        Arc::new(ProviderChannelMessageStore::new(pool.clone())),
        Arc::new(
            crate::platform::communications::EventStoreProviderMessageObservationEventPort::new(
                pool.clone(),
            ),
        ),
    )
}

pub fn whatsapp_web_store(pool: &PgPool) -> WhatsappWebStore {
    WhatsappWebStore::new(
        pool.clone(),
        Arc::new(communication_provider_account_store(pool)),
        Arc::new(ProviderChannelMessageStore::new(pool.clone())),
    )
}

pub async fn upsert_telegram_runtime_account(
    pool: &PgPool,
    account_id: &str,
    display_name: &str,
    external_account_id: &str,
) -> ProviderAccount {
    communication_provider_account_store(pool)
        .upsert(&NewProviderAccount::new(
            account_id,
            EmailProviderKind::TelegramUser,
            display_name,
            external_account_id,
        ))
        .await
        .expect("seed Telegram provider account")
}
