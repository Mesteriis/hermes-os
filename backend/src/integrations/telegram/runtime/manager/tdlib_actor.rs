use std::sync::mpsc::Sender;

use chrono::Utc;

use crate::domains::mail::core::{CommunicationIngestionStore, ProviderAccount};
use crate::integrations::telegram::client::TelegramError;
use crate::platform::config::AppConfig;
use crate::platform::secrets::{SecretReferenceStore, SecretResolver};

use super::super::actor::{optional_telegram_session_key, spawn_tdlib_actor};
use super::super::state::{TelegramRuntimeActorHandle, TelegramRuntimeCommand};
use super::TelegramRuntimeManager;
use super::actor_states::running_actor_state;
use super::realtime_events::{
    TelegramRuntimeEventBridgeContext, spawn_telegram_runtime_event_bridge,
};

impl TelegramRuntimeManager {
    pub(in crate::integrations::telegram::runtime::manager) async fn ensure_tdlib_actor(
        &self,
        communication_store: &CommunicationIngestionStore,
        secret_store: &SecretReferenceStore,
        secret_resolver: &(impl SecretResolver + Sync + ?Sized),
        config: &AppConfig,
        account: &ProviderAccount,
        event_bridge: Option<TelegramRuntimeEventBridgeContext>,
    ) -> Result<Sender<TelegramRuntimeCommand>, TelegramError> {
        if let Some(command_tx) = self.actor_command_tx(&account.account_id)? {
            return Ok(command_tx);
        }

        let session_encryption_key = optional_telegram_session_key(
            communication_store,
            secret_store,
            secret_resolver,
            &account.account_id,
        )
        .await?;
        let (runtime_event_tx, runtime_event_rx) = tokio::sync::mpsc::unbounded_channel();
        let runtime_event_tx = event_bridge.as_ref().map(|_| runtime_event_tx);
        let command_tx = spawn_tdlib_actor(
            config.clone(),
            account.clone(),
            session_encryption_key,
            runtime_event_tx,
        )?;
        if let Some(event_bridge) = event_bridge {
            spawn_telegram_runtime_event_bridge(
                event_bridge.pool,
                event_bridge.event_bus,
                account.account_id.clone(),
                runtime_event_rx,
            );
        }
        self.set_actor_handle(
            account.account_id.clone(),
            TelegramRuntimeActorHandle {
                state: running_actor_state(Utc::now()),
                command_tx: Some(command_tx.clone()),
            },
        )?;
        Ok(command_tx)
    }
}
