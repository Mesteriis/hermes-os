use crate::app::{ApiError, AppState};
use crate::platform::config::AppConfig;
use crate::platform::secrets::SecretReferenceStore;

pub(super) const AUDIT_ACTOR_ID: &str = "hermes-frontend";

pub(super) fn telegram_api_hash_from_config(config: &AppConfig) -> Option<String> {
    config
        .telegram_api_hash()
        .map(|secret| secret.expose_for_runtime().to_owned())
}

pub(super) fn telegram_secret_store(state: &AppState) -> Result<SecretReferenceStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    Ok(SecretReferenceStore::new(pool.clone()))
}
