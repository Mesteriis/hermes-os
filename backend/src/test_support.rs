use std::sync::Arc;

use serde_json::Value;
use sqlx::PgPool;

use crate::domains::communications::core::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore,
};
use crate::domains::communications::messages::ProviderChannelMessageStore;
use crate::domains::signal_hub::{SignalHubStore, SignalRuntimeStateUpdate};
use crate::integrations::telegram::client::TelegramStore;
use crate::integrations::whatsapp::client::WhatsappWebStore;
use crate::platform::communications::StoredRawCommunicationRecord;
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
            crate::domains::communications::core::CommunicationIngestionStore::new(pool.clone()),
        ),
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

pub async fn restore_signal_hub_system_sources(pool: &PgPool) {
    SignalHubStore::new(pool.clone())
        .restore_system_sources()
        .await
        .expect("restore Signal Hub system sources");
}

pub async fn set_signal_runtime_state(
    pool: &PgPool,
    source_code: &str,
    runtime_kind: &str,
    state: &str,
    metadata: Value,
) {
    SignalHubStore::new(pool.clone())
        .set_runtime_state(&SignalRuntimeStateUpdate {
            source_code: source_code.to_owned(),
            runtime_kind: runtime_kind.to_owned(),
            state: state.to_owned(),
            metadata,
        })
        .await
        .expect("set Signal Hub runtime state");
}

pub async fn load_communication_raw_record(
    pool: &PgPool,
    raw_record_id: &str,
) -> StoredRawCommunicationRecord {
    crate::domains::communications::core::CommunicationIngestionStore::new(pool.clone())
        .raw_record(raw_record_id)
        .await
        .expect("load communication raw record")
        .expect("stored communication raw record")
}
