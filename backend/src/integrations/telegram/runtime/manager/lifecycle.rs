use chrono::Utc;

use crate::integrations::telegram::client::TelegramError;
use crate::platform::config::AppConfig;

use super::super::actor::{optional_telegram_session_key, spawn_tdlib_actor};
use super::super::models::{
    TelegramRuntimeRestartRequest, TelegramRuntimeStartRequest, TelegramRuntimeStatus,
    TelegramRuntimeStopRequest,
};
use super::super::state::{
    TelegramRuntimeActorHandle, TelegramRuntimeActorState, TelegramRuntimeState,
};
use super::super::status::{account_runtime_kind, load_telegram_account, status_from_account};
use super::account::load_active_account;
use super::actor_states::running_actor_state;
use super::realtime_events::spawn_telegram_runtime_event_bridge;
use super::{TelegramRuntimeManager, TelegramRuntimeStartContext};
use crate::domains::mail::core::CommunicationIngestionStore;

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

    pub(crate) async fn start_account<S>(
        &self,
        context: &TelegramRuntimeStartContext<'_, S>,
        request: &TelegramRuntimeStartRequest,
    ) -> Result<TelegramRuntimeStatus, TelegramError>
    where
        S: crate::platform::secrets::SecretResolver + Sync + ?Sized,
    {
        request.validate()?;
        let account = load_active_account(context.communication_store, &request.account_id).await?;
        let session_encryption_key = optional_telegram_session_key(
            context.communication_store,
            context.secret_store,
            context.secret_resolver,
            &account.account_id,
        )
        .await?;
        let runtime_kind = account_runtime_kind(&account);
        let now = Utc::now();
        let (actor_state, command_tx) = match runtime_kind.as_str() {
            "fixture" => running_actor_state(now).without_command(),
            "tdlib_qr_authorized" => {
                let (runtime_event_tx, runtime_event_rx) = tokio::sync::mpsc::unbounded_channel();
                let result = match spawn_tdlib_actor(
                    context.config.clone(),
                    account.clone(),
                    session_encryption_key,
                    Some(runtime_event_tx),
                ) {
                    Ok(command_tx) => running_actor_state(now).with_command(command_tx),
                    Err(error) => TelegramRuntimeActorState {
                        status: TelegramRuntimeState::Degraded,
                        last_error: Some(error.to_string()),
                        updated_at: now,
                    }
                    .without_command(),
                };
                if result.1.is_some() {
                    spawn_telegram_runtime_event_bridge(
                        context.event_store_pool.clone(),
                        context.event_bus.clone(),
                        account.account_id.clone(),
                        runtime_event_rx,
                    );
                }
                result
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

        Ok(status_from_account(
            context.config,
            &account,
            Some(actor_state),
        ))
    }

    pub async fn stop_account_runtime(
        &self,
        communication_store: &CommunicationIngestionStore,
        config: &AppConfig,
        request: &TelegramRuntimeStopRequest,
    ) -> Result<TelegramRuntimeStatus, TelegramError> {
        request.validate()?;
        let account = load_telegram_account(communication_store, &request.account_id).await?;
        self.stop_account(&account.account_id)?;

        Ok(status_from_account(config, &account, None))
    }

    pub(crate) async fn restart_account_runtime<S>(
        &self,
        context: &TelegramRuntimeStartContext<'_, S>,
        request: &TelegramRuntimeRestartRequest,
    ) -> Result<TelegramRuntimeStatus, TelegramError>
    where
        S: crate::platform::secrets::SecretResolver + Sync + ?Sized,
    {
        request.validate()?;
        self.stop_account(&request.account_id)?;
        self.start_account(
            context,
            &TelegramRuntimeStartRequest {
                account_id: request.account_id.clone(),
            },
        )
        .await
    }
}
