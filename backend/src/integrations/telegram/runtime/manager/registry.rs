use std::sync::mpsc::Sender;

use crate::integrations::telegram::client::errors::TelegramError;

use super::super::state::{
    TelegramRuntimeActorHandle, TelegramRuntimeActorState, TelegramRuntimeCommand,
};
use super::super::validation::validate_non_empty;
use super::TelegramRuntimeManager;

impl TelegramRuntimeManager {
    pub(in crate::integrations::telegram::runtime::manager) fn actor_state(
        &self,
        account_id: &str,
    ) -> Result<Option<TelegramRuntimeActorState>, TelegramError> {
        let actors = self.actors.lock().map_err(|_| {
            TelegramError::TdlibRuntime("Telegram runtime state lock poisoned".into())
        })?;
        Ok(actors.get(account_id).map(|handle| handle.state.clone()))
    }

    pub fn stop_account(&self, account_id: &str) -> Result<bool, TelegramError> {
        let account_id = validate_non_empty("account_id", account_id)?;
        let mut actors = self.actors.lock().map_err(|_| {
            TelegramError::TdlibRuntime("Telegram runtime state lock poisoned".into())
        })?;
        Ok(actors.remove(&account_id).is_some())
    }

    pub(in crate::integrations::telegram::runtime::manager) fn set_actor_handle(
        &self,
        account_id: String,
        actor_handle: TelegramRuntimeActorHandle,
    ) -> Result<(), TelegramError> {
        let mut actors = self.actors.lock().map_err(|_| {
            TelegramError::TdlibRuntime("Telegram runtime state lock poisoned".into())
        })?;
        actors.insert(account_id, actor_handle);
        Ok(())
    }

    pub(in crate::integrations::telegram::runtime::manager) fn actor_command_tx(
        &self,
        account_id: &str,
    ) -> Result<Option<Sender<TelegramRuntimeCommand>>, TelegramError> {
        let actors = self.actors.lock().map_err(|_| {
            TelegramError::TdlibRuntime("Telegram runtime state lock poisoned".into())
        })?;
        Ok(actors
            .get(account_id)
            .and_then(|handle| handle.command_tx.clone()))
    }

    pub(crate) fn active_account_ids(&self) -> Result<Vec<String>, TelegramError> {
        let actors = self.actors.lock().map_err(|_| {
            TelegramError::TdlibRuntime("Telegram runtime state lock poisoned".into())
        })?;
        Ok(actors
            .iter()
            .filter(|(_, handle)| handle.command_tx.is_some())
            .map(|(id, _)| id.clone())
            .collect())
    }
}
