use std::env;

use super::super::errors::ConfigError;
use super::AppConfig;
use super::ai_env::apply_ai_env;
use super::core_env::apply_core_env;
use super::provider_env::{apply_bundled_google_oauth_client, apply_provider_env};

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_pairs(env::vars())
    }

    pub fn from_pairs<I, K, V>(pairs: I) -> Result<Self, ConfigError>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let mut config = Self::default();
        apply_bundled_google_oauth_client(&mut config)?;

        for (key, value) in pairs {
            apply_config_pair(&mut config, key.as_ref(), value.as_ref())?;
        }

        Ok(config)
    }
}

pub(super) fn apply_config_pair(
    config: &mut AppConfig,
    key: &str,
    value: &str,
) -> Result<(), ConfigError> {
    if apply_core_env(config, key, value)?
        || apply_provider_env(config, key, value)?
        || apply_ai_env(config, key, value)?
    {
        return Ok(());
    }

    Ok(())
}
