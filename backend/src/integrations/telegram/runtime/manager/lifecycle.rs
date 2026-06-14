use chrono::Utc;

use crate::domains::mail::core::CommunicationIngestionStore;
use crate::integrations::telegram::client::TelegramError;
use crate::platform::config::AppConfig;
use crate::platform::secrets::{SecretReferenceStore, SecretResolver};

use super::super::actor::{optional_telegram_session_key, spawn_tdlib_actor};
use super::super::models::{TelegramRuntimeStartRequest, TelegramRuntimeStatus};
use super::super::state::{
    TelegramRuntimeActorHandle, TelegramRuntimeActorState, TelegramRuntimeState,
};
use super::super::status::{account_runtime_kind, load_telegram_account, status_from_account};
use super::TelegramRuntimeManager;
use super::account::load_active_account;
use super::actor_states::running_actor_state;

impl TelegramRuntimeManager {
    pub async fn status_for_account(
        &self,
        communication_store: &CommunicationIngestionStore,
        config: &AppConfig,
        account_id: &str,
    ) -> Result<TelegramRuntimeStatus, TelegramError> {
        let account = load_telegram_account(communication_store, account_id).await?;
        let actor_state = self.actor_state(&account.account_id)?;

        Ok(status_from_account(config, &account, actor_state))
    }

    pub async fn start_account(
        &self,
        communication_store: &CommunicationIngestionStore,
        secret_store: &SecretReferenceStore,
        secret_resolver: &(impl SecretResolver + Sync + ?Sized),
        config: &AppConfig,
        request: &TelegramRuntimeStartRequest,
    ) -> Result<TelegramRuntimeStatus, TelegramError> {
        request.validate()?;
        let account = load_active_account(communication_store, &request.account_id).await?;
        let session_encryption_key = optional_telegram_session_key(
            communication_store,
            secret_store,
            secret_resolver,
            &account.account_id,
        )
        .await?;
        let runtime_kind = account_runtime_kind(&account);
        let now = Utc::now();
        let (actor_state, command_tx) = match runtime_kind.as_str() {
            "fixture" => running_actor_state(now).without_command(),
            "tdlib_qr_authorized" => {
                match spawn_tdlib_actor(config.clone(), account.clone(), session_encryption_key) {
                    Ok(command_tx) => running_actor_state(now).with_command(command_tx),
                    Err(error) => TelegramRuntimeActorState {
                        status: TelegramRuntimeState::Degraded,
                        last_error: Some(error.to_string()),
                        updated_at: now,
                    }
                    .without_command(),
                }
            }
            "live_blocked" => TelegramRuntimeActorState {
                status: TelegramRuntimeState::Blocked,
                last_error: Some(
                    "account runtime is blocked until live TDLib is enabled".to_owned(),
                ),
                updated_at: now,
            }
            .without_command(),
            other => TelegramRuntimeActorState {
                status: TelegramRuntimeState::Error,
                last_error: Some(format!("unsupported Telegram runtime `{other}`")),
                updated_at: now,
            }
            .without_command(),
        };

        self.set_actor_handle(
            account.account_id.clone(),
            TelegramRuntimeActorHandle {
                state: actor_state.clone(),
                command_tx,
            },
        )?;

        Ok(status_from_account(config, &account, Some(actor_state)))
    }
}
