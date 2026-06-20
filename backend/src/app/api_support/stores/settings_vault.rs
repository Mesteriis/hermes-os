use super::super::*;
use super::database::database_pool;

pub(crate) fn settings_store(state: &AppState) -> Result<ApplicationSettingsStore, ApiError> {
    Ok(ApplicationSettingsStore::new(database_pool(state)?))
}

pub(crate) fn database_encrypted_vault(
    config: &AppConfig,
    pool: sqlx::postgres::PgPool,
) -> Option<DatabaseEncryptedSecretVault> {
    Some(DatabaseEncryptedSecretVault::new(
        pool,
        config.secret_vault_key()?.clone(),
    ))
}
