use std::path::PathBuf;

use crate::platform::config::ConfigError;

use super::AppConfig;
use super::env::apply_config_pair;

impl AppConfig {
    pub fn test_with_api_secret(api_secret: impl Into<String>) -> Self {
        Self {
            local_api_secret: Some(api_secret.into()),
            zoom_token_maintenance_scheduler_enabled: false,
            zoom_recording_sync_scheduler_enabled: false,
            zoom_retention_cleanup_scheduler_enabled: false,
            ..Self::default()
        }
    }

    pub fn test_with_api_secret_and_database_url(
        api_secret: impl Into<String>,
        database_url: impl Into<String>,
    ) -> Self {
        let mut config = Self::test_with_api_secret(api_secret);
        config.database_url = Some(database_url.into());
        config
    }

    pub fn with_test_database_url(mut self, database_url: impl Into<String>) -> Self {
        self.database_url = Some(database_url.into());
        self
    }

    pub fn with_test_api_secret(mut self, api_secret: impl Into<String>) -> Self {
        self.local_api_secret = Some(api_secret.into());
        self
    }

    pub fn with_test_pairs<I, K, V>(mut self, pairs: I) -> Result<Self, ConfigError>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        for (key, value) in pairs {
            apply_config_pair(&mut self, key.as_ref(), value.as_ref())?;
        }

        Ok(self)
    }

    pub fn with_test_dev_mode(mut self) -> Self {
        self.dev_mode = true;
        self
    }

    pub fn with_test_dev_vault_paths(
        mut self,
        vault_home: impl Into<PathBuf>,
        dev_key_path: impl Into<PathBuf>,
    ) -> Self {
        self.dev_mode = true;
        self.vault_home = vault_home.into();
        self.dev_key_path = dev_key_path.into();
        self
    }

    pub fn with_test_tdjson_path(mut self, tdjson_path: impl Into<PathBuf>) -> Self {
        self.tdjson_path = Some(tdjson_path.into());
        self
    }

    pub fn with_test_telegram_app_credentials(
        mut self,
        api_id: i64,
        api_hash: impl AsRef<str>,
    ) -> Self {
        self.telegram_api_id = Some(api_id);
        self.telegram_api_hash = Some(
            crate::platform::secrets::ResolvedSecret::new(api_hash.as_ref())
                .expect("test Telegram API hash must be valid"),
        );
        self
    }
}
