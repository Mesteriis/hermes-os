use super::super::*;
use super::database::database_pool;
use crate::platform::settings::store::ApplicationSettingsStore;

pub(crate) fn settings_store(state: &AppState) -> Result<ApplicationSettingsStore, ApiError> {
    Ok(ApplicationSettingsStore::new(database_pool(state)?))
}
