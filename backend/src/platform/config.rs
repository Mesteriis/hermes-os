use std::env;
use std::net::{AddrParseError, SocketAddr};
use std::num::ParseIntError;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::platform::secrets::{ResolvedSecret, SecretResolutionError};

const DEFAULT_HTTP_ADDR: &str = "127.0.0.1:8080";
const DEFAULT_SERVICE_NAME: &str = "hermes-hub-backend";
const DEFAULT_OLLAMA_BASE_URL: &str = "http://127.0.0.1:11434";
const DEFAULT_OLLAMA_CHAT_MODEL: &str = "qwen3:4b";
const DEFAULT_OLLAMA_EMBED_MODEL: &str = "qwen3-embedding:4b";
const DEFAULT_OLLAMA_TIMEOUT_SECONDS: u64 = 120;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppConfig {
    service_name: String,
    http_addr: SocketAddr,
    database_url: Option<String>,
    local_api_secret: Option<String>,
    secret_vault_path: Option<PathBuf>,
    secret_vault_key: Option<ResolvedSecret>,
    ollama_base_url: String,
    ollama_chat_model: String,
    ollama_embed_model: String,
    ollama_timeout_seconds: u64,
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
                "HERMES_LOCAL_API_SECRET" => {
                    let raw_secret = value.as_ref().trim();
                    if raw_secret.is_empty() {
                        return Err(ConfigError::EmptyLocalApiSecret);
                    }
                    config.local_api_secret = Some(raw_secret.to_owned());
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
                "HERMES_OLLAMA_BASE_URL" => {
                    let raw_url = value.as_ref().trim();
                    if raw_url.is_empty() {
                        return Err(ConfigError::EmptyOllamaBaseUrl);
                    }
                    config.ollama_base_url = raw_url.trim_end_matches('/').to_owned();
                }
                "HERMES_OLLAMA_CHAT_MODEL" => {
                    let raw_model = value.as_ref().trim();
                    if raw_model.is_empty() {
                        return Err(ConfigError::EmptyOllamaChatModel);
                    }
                    config.ollama_chat_model = raw_model.to_owned();
                }
                "HERMES_OLLAMA_EMBED_MODEL" => {
                    let raw_model = value.as_ref().trim();
                    if raw_model.is_empty() {
                        return Err(ConfigError::EmptyOllamaEmbedModel);
                    }
                    config.ollama_embed_model = raw_model.to_owned();
                }
                "HERMES_OLLAMA_TIMEOUT_SECONDS" => {
                    let raw_timeout = value.as_ref().trim();
                    let timeout = raw_timeout.parse::<u64>().map_err(|source| {
                        ConfigError::InvalidOllamaTimeout {
                            value: raw_timeout.to_owned(),
                            reason: "must be a positive integer",
                            source: Some(source),
                        }
                    })?;
                    if timeout == 0 {
                        return Err(ConfigError::InvalidOllamaTimeout {
                            value: raw_timeout.to_owned(),
                            reason: "must be greater than zero",
                            source: None,
                        });
                    }
                    config.ollama_timeout_seconds = timeout;
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

    pub fn local_api_secret(&self) -> Option<&str> {
        self.local_api_secret.as_deref()
    }

    pub fn secret_vault_path(&self) -> Option<&Path> {
        self.secret_vault_path.as_deref()
    }

    pub fn secret_vault_key(&self) -> Option<&ResolvedSecret> {
        self.secret_vault_key.as_ref()
    }

    pub fn ollama_base_url(&self) -> &str {
        &self.ollama_base_url
    }

    pub fn ollama_chat_model(&self) -> &str {
        &self.ollama_chat_model
    }

    pub fn ollama_embed_model(&self) -> &str {
        &self.ollama_embed_model
    }

    pub fn ollama_timeout_seconds(&self) -> u64 {
        self.ollama_timeout_seconds
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
            local_api_secret: None,
            secret_vault_path: None,
            secret_vault_key: None,
            ollama_base_url: DEFAULT_OLLAMA_BASE_URL.to_owned(),
            ollama_chat_model: DEFAULT_OLLAMA_CHAT_MODEL.to_owned(),
            ollama_embed_model: DEFAULT_OLLAMA_EMBED_MODEL.to_owned(),
            ollama_timeout_seconds: DEFAULT_OLLAMA_TIMEOUT_SECONDS,
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

    #[error("HERMES_LOCAL_API_SECRET is set but empty")]
    EmptyLocalApiSecret,

    #[error("HERMES_SECRET_VAULT_PATH is set but empty")]
    EmptySecretVaultPath,

    #[error("HERMES_SECRET_VAULT_KEY is set but empty")]
    EmptySecretVaultKey,

    #[error("HERMES_OLLAMA_BASE_URL is set but empty")]
    EmptyOllamaBaseUrl,

    #[error("HERMES_OLLAMA_CHAT_MODEL is set but empty")]
    EmptyOllamaChatModel,

    #[error("HERMES_OLLAMA_EMBED_MODEL is set but empty")]
    EmptyOllamaEmbedModel,

    #[error("invalid HERMES_OLLAMA_TIMEOUT_SECONDS `{value}`: {reason}")]
    InvalidOllamaTimeout {
        value: String,
        reason: &'static str,
        #[source]
        source: Option<ParseIntError>,
    },

    #[error(transparent)]
    SecretResolution(#[from] SecretResolutionError),
}
