use std::env;
use std::net::{AddrParseError, SocketAddr};
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::secrets::{ResolvedSecret, SecretResolutionError};

const DEFAULT_HTTP_ADDR: &str = "127.0.0.1:8080";
const DEFAULT_SERVICE_NAME: &str = "hermes-hub-backend";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppConfig {
    service_name: String,
    http_addr: SocketAddr,
    database_url: Option<String>,
    local_api_token: Option<String>,
    secret_vault_path: Option<PathBuf>,
    secret_vault_key: Option<ResolvedSecret>,
}

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

        for (key, value) in pairs {
            match key.as_ref() {
                "HERMES_HTTP_ADDR" => {
                    let raw_addr = value.as_ref().trim();
                    config.http_addr =
                        raw_addr
                            .parse()
                            .map_err(|source| ConfigError::InvalidHttpAddr {
                                value: raw_addr.to_owned(),
                                source,
                            })?;
                }
                "DATABASE_URL" => {
                    let raw_url = value.as_ref().trim();
                    if raw_url.is_empty() {
                        return Err(ConfigError::EmptyDatabaseUrl);
                    }
                    config.database_url = Some(raw_url.to_owned());
                }
                "HERMES_LOCAL_API_TOKEN" => {
                    let raw_token = value.as_ref().trim();
                    if raw_token.is_empty() {
                        return Err(ConfigError::EmptyLocalApiToken);
                    }
                    config.local_api_token = Some(raw_token.to_owned());
                }
                "HERMES_LOCAL_WRITE_TOKEN" => {
                    let raw_token = value.as_ref().trim();
                    if raw_token.is_empty() {
                        return Err(ConfigError::EmptyLocalWriteToken);
                    }
                    if config.local_api_token.is_none() {
                        config.local_api_token = Some(raw_token.to_owned());
                    }
                }
                "HERMES_SECRET_VAULT_PATH" => {
                    let raw_path = value.as_ref().trim();
                    if raw_path.is_empty() {
                        return Err(ConfigError::EmptySecretVaultPath);
                    }
                    config.secret_vault_path = Some(PathBuf::from(raw_path));
                }
                "HERMES_SECRET_VAULT_KEY" => {
                    let raw_key = value.as_ref().trim();
                    if raw_key.is_empty() {
                        return Err(ConfigError::EmptySecretVaultKey);
                    }
                    config.secret_vault_key = Some(ResolvedSecret::new(raw_key)?);
                }
                _ => {}
            }
        }

        Ok(config)
    }

    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    pub fn http_addr(&self) -> SocketAddr {
        self.http_addr
    }

    pub fn database_url(&self) -> Option<&str> {
        self.database_url.as_deref()
    }

    pub fn local_api_token(&self) -> Option<&str> {
        self.local_api_token.as_deref()
    }

    pub fn secret_vault_path(&self) -> Option<&Path> {
        self.secret_vault_path.as_deref()
    }

    pub fn secret_vault_key(&self) -> Option<&ResolvedSecret> {
        self.secret_vault_key.as_ref()
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            service_name: DEFAULT_SERVICE_NAME.to_owned(),
            http_addr: DEFAULT_HTTP_ADDR
                .parse()
                .expect("default HTTP bind address must be valid"),
            database_url: None,
            local_api_token: None,
            secret_vault_path: None,
            secret_vault_key: None,
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid HERMES_HTTP_ADDR `{value}`: {source}")]
    InvalidHttpAddr {
        value: String,
        #[source]
        source: AddrParseError,
    },

    #[error("DATABASE_URL is set but empty")]
    EmptyDatabaseUrl,

    #[error("HERMES_LOCAL_API_TOKEN is set but empty")]
    EmptyLocalApiToken,

    #[error("HERMES_LOCAL_WRITE_TOKEN is set but empty")]
    EmptyLocalWriteToken,

    #[error("HERMES_SECRET_VAULT_PATH is set but empty")]
    EmptySecretVaultPath,

    #[error("HERMES_SECRET_VAULT_KEY is set but empty")]
    EmptySecretVaultKey,

    #[error(transparent)]
    SecretResolution(#[from] SecretResolutionError),
}
