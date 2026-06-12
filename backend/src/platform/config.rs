use std::env;
use std::fs;
use std::io;
use std::net::{AddrParseError, SocketAddr};
use std::num::ParseIntError;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

use crate::platform::secrets::{ResolvedSecret, SecretResolutionError};
use crate::vault::{default_dev_key_path, default_vault_home};

const DEFAULT_HTTP_ADDR: &str = "127.0.0.1:8080";
const DEFAULT_SERVICE_NAME: &str = "hermes-hub-backend";
const DEFAULT_OLLAMA_BASE_URL: &str = "http://127.0.0.1:11434";
const DEFAULT_OLLAMA_CHAT_MODEL: &str = "qwen3:4b";
const DEFAULT_OLLAMA_EMBED_MODEL: &str = "qwen3-embedding:4b";
const DEFAULT_OLLAMA_TIMEOUT_SECONDS: u64 = 120;
const DEFAULT_OMNIROUTE_BASE_URL: &str = "https://ai.sh-inc.ru/v1";
const DEFAULT_OMNIROUTE_CHAT_MODEL: &str = "codex/gpt-5.5";
const DEFAULT_OMNIROUTE_EMBED_MODEL: &str = "openai-compatible-chat-ollama-pve/qwen3-embedding:4b";
const DEFAULT_OMNIROUTE_TIMEOUT_SECONDS: u64 = 120;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiRuntimeProvider {
    Ollama,
    OmniRoute,
}

impl AiRuntimeProvider {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ollama => "ollama",
            Self::OmniRoute => "omniroute",
        }
    }
}

impl TryFrom<&str> for AiRuntimeProvider {
    type Error = ConfigError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim().to_lowercase().as_str() {
            "ollama" => Ok(Self::Ollama),
            "omniroute" | "omni_route" | "omni-route" => Ok(Self::OmniRoute),
            _ => Err(ConfigError::InvalidAiProvider {
                value: value.to_owned(),
            }),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppConfig {
    service_name: String,
    http_addr: SocketAddr,
    database_url: Option<String>,
    local_api_secret: Option<String>,
    secret_vault_path: Option<PathBuf>,
    secret_vault_key: Option<ResolvedSecret>,
    vault_home: PathBuf,
    dev_mode: bool,
    dev_key_path: PathBuf,
    tdjson_path: Option<PathBuf>,
    telegram_api_id: Option<i64>,
    telegram_api_hash: Option<ResolvedSecret>,
    google_oauth_client: Option<GoogleOAuthClientConfig>,
    google_oauth_client_id: Option<String>,
    google_oauth_client_secret: Option<ResolvedSecret>,
    ai_provider: AiRuntimeProvider,
    ollama_base_url: String,
    ollama_chat_model: String,
    ollama_embed_model: String,
    ollama_timeout_seconds: u64,
    omniroute_base_url: String,
    omniroute_chat_model: String,
    omniroute_embed_model: String,
    omniroute_timeout_seconds: u64,
    omniroute_api_key: Option<ResolvedSecret>,
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
        if let Some(raw_json) = option_env!("HERMES_BUNDLED_GOOGLE_OAUTH_CLIENT_JSON") {
            config.google_oauth_client =
                Some(GoogleOAuthClientConfig::from_client_secret_json(raw_json)?);
        }

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
                "HERMES_VAULT_HOME" => {
                    let raw_path = value.as_ref().trim();
                    if raw_path.is_empty() {
                        return Err(ConfigError::EmptyVaultHome);
                    }
                    config.vault_home = PathBuf::from(raw_path);
                }
                "HERMES_DEV_MODE" => {
                    let raw_value = value.as_ref().trim();
                    config.dev_mode = parse_bool_env("HERMES_DEV_MODE", raw_value)?;
                }
                "HERMES_DEV_KEY_PATH" => {
                    let raw_path = value.as_ref().trim();
                    if raw_path.is_empty() {
                        return Err(ConfigError::EmptyDevKeyPath);
                    }
                    config.dev_key_path = PathBuf::from(raw_path);
                }
                "HERMES_TDJSON_PATH" => {
                    let raw_path = value.as_ref().trim();
                    if raw_path.is_empty() {
                        return Err(ConfigError::EmptyTdjsonPath);
                    }
                    config.tdjson_path = Some(PathBuf::from(raw_path));
                }
                "HERMES_TELEGRAM_API_ID" => {
                    let raw_id = value.as_ref().trim();
                    let api_id = raw_id.parse::<i64>().map_err(|source| {
                        ConfigError::InvalidTelegramApiId {
                            value: raw_id.to_owned(),
                            reason: "must be a positive integer",
                            source: Some(source),
                        }
                    })?;
                    if api_id <= 0 {
                        return Err(ConfigError::InvalidTelegramApiId {
                            value: raw_id.to_owned(),
                            reason: "must be greater than zero",
                            source: None,
                        });
                    }
                    config.telegram_api_id = Some(api_id);
                }
                "HERMES_TELEGRAM_API_HASH" => {
                    let raw_hash = value.as_ref().trim();
                    if raw_hash.is_empty() {
                        return Err(ConfigError::EmptyTelegramApiHash);
                    }
                    config.telegram_api_hash = Some(ResolvedSecret::new(raw_hash)?);
                }
                "HERMES_GOOGLE_OAUTH_CLIENT_ID" => {
                    let raw_client_id = value.as_ref().trim();
                    if raw_client_id.is_empty() {
                        return Err(ConfigError::EmptyGoogleOAuthClientId);
                    }
                    config.google_oauth_client_id = Some(raw_client_id.to_owned());
                }
                "HERMES_GOOGLE_OAUTH_CLIENT_SECRET" => {
                    let raw_client_secret = value.as_ref().trim();
                    if raw_client_secret.is_empty() {
                        return Err(ConfigError::EmptyGoogleOAuthClientSecret);
                    }
                    config.google_oauth_client_secret =
                        Some(ResolvedSecret::new(raw_client_secret)?);
                }
                "HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_JSON" => {
                    let raw_json = value.as_ref().trim();
                    if raw_json.is_empty() {
                        return Err(ConfigError::EmptyGoogleOAuthClientConfigJson);
                    }
                    config.google_oauth_client =
                        Some(GoogleOAuthClientConfig::from_client_secret_json(raw_json)?);
                }
                "HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_PATH" => {
                    let raw_path = value.as_ref().trim();
                    if raw_path.is_empty() {
                        return Err(ConfigError::EmptyGoogleOAuthClientConfigPath);
                    }
                    let path = PathBuf::from(raw_path);
                    let raw_json = fs::read_to_string(&path).map_err(|source| {
                        ConfigError::GoogleOAuthClientConfigRead { path, source }
                    })?;
                    config.google_oauth_client =
                        Some(GoogleOAuthClientConfig::from_client_secret_json(&raw_json)?);
                }
                "HERMES_AI_PROVIDER" => {
                    config.ai_provider = AiRuntimeProvider::try_from(value.as_ref())?;
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
                "HERMES_OMNIROUTE_BASE_URL" => {
                    let raw_url = value.as_ref().trim();
                    if raw_url.is_empty() {
                        return Err(ConfigError::EmptyOmniRouteBaseUrl);
                    }
                    config.omniroute_base_url = raw_url.trim_end_matches('/').to_owned();
                }
                "HERMES_OMNIROUTE_CHAT_MODEL" => {
                    let raw_model = value.as_ref().trim();
                    if raw_model.is_empty() {
                        return Err(ConfigError::EmptyOmniRouteChatModel);
                    }
                    config.omniroute_chat_model = raw_model.to_owned();
                }
                "HERMES_OMNIROUTE_EMBED_MODEL" => {
                    let raw_model = value.as_ref().trim();
                    if raw_model.is_empty() {
                        return Err(ConfigError::EmptyOmniRouteEmbedModel);
                    }
                    config.omniroute_embed_model = raw_model.to_owned();
                }
                "HERMES_OMNIROUTE_TIMEOUT_SECONDS" => {
                    let raw_timeout = value.as_ref().trim();
                    let timeout = raw_timeout.parse::<u64>().map_err(|source| {
                        ConfigError::InvalidOmniRouteTimeout {
                            value: raw_timeout.to_owned(),
                            reason: "must be a positive integer",
                            source: Some(source),
                        }
                    })?;
                    if timeout == 0 {
                        return Err(ConfigError::InvalidOmniRouteTimeout {
                            value: raw_timeout.to_owned(),
                            reason: "must be greater than zero",
                            source: None,
                        });
                    }
                    config.omniroute_timeout_seconds = timeout;
                }
                "HERMES_OMNIROUTE_API_KEY" => {
                    let raw_key = value.as_ref().trim();
                    if raw_key.is_empty() {
                        return Err(ConfigError::EmptyOmniRouteApiKey);
                    }
                    config.omniroute_api_key = Some(ResolvedSecret::new(raw_key)?);
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

    pub fn vault_home(&self) -> &Path {
        &self.vault_home
    }

    pub fn dev_mode(&self) -> bool {
        self.dev_mode
    }

    pub fn dev_key_path(&self) -> &Path {
        &self.dev_key_path
    }

    pub fn tdjson_path(&self) -> Option<&Path> {
        self.tdjson_path.as_deref()
    }

    pub fn telegram_api_id(&self) -> Option<i64> {
        self.telegram_api_id
    }

    pub fn telegram_api_hash(&self) -> Option<&ResolvedSecret> {
        self.telegram_api_hash.as_ref()
    }

    pub fn google_oauth_client_id(&self) -> Option<&str> {
        self.google_oauth_client_id.as_deref().or_else(|| {
            self.google_oauth_client
                .as_ref()
                .map(GoogleOAuthClientConfig::client_id)
        })
    }

    pub fn google_oauth_client_secret(&self) -> Option<&ResolvedSecret> {
        self.google_oauth_client_secret.as_ref().or_else(|| {
            self.google_oauth_client
                .as_ref()
                .and_then(GoogleOAuthClientConfig::client_secret)
        })
    }

    pub fn google_oauth_client(&self) -> Option<&GoogleOAuthClientConfig> {
        self.google_oauth_client.as_ref()
    }

    pub fn ai_provider(&self) -> AiRuntimeProvider {
        self.ai_provider
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

    pub fn omniroute_base_url(&self) -> &str {
        &self.omniroute_base_url
    }

    pub fn omniroute_chat_model(&self) -> &str {
        &self.omniroute_chat_model
    }

    pub fn omniroute_embed_model(&self) -> &str {
        &self.omniroute_embed_model
    }

    pub fn omniroute_timeout_seconds(&self) -> u64 {
        self.omniroute_timeout_seconds
    }

    pub fn omniroute_api_key(&self) -> Option<&ResolvedSecret> {
        self.omniroute_api_key.as_ref()
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        let home_dir = env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        Self {
            service_name: DEFAULT_SERVICE_NAME.to_owned(),
            http_addr: DEFAULT_HTTP_ADDR
                .parse()
                .expect("default HTTP bind address must be valid"),
            database_url: None,
            local_api_secret: None,
            secret_vault_path: None,
            secret_vault_key: None,
            vault_home: default_vault_home(&home_dir),
            dev_mode: false,
            dev_key_path: default_dev_key_path(&home_dir),
            tdjson_path: None,
            telegram_api_id: None,
            telegram_api_hash: None,
            google_oauth_client: None,
            google_oauth_client_id: None,
            google_oauth_client_secret: None,
            ai_provider: AiRuntimeProvider::Ollama,
            ollama_base_url: DEFAULT_OLLAMA_BASE_URL.to_owned(),
            ollama_chat_model: DEFAULT_OLLAMA_CHAT_MODEL.to_owned(),
            ollama_embed_model: DEFAULT_OLLAMA_EMBED_MODEL.to_owned(),
            ollama_timeout_seconds: DEFAULT_OLLAMA_TIMEOUT_SECONDS,
            omniroute_base_url: DEFAULT_OMNIROUTE_BASE_URL.to_owned(),
            omniroute_chat_model: DEFAULT_OMNIROUTE_CHAT_MODEL.to_owned(),
            omniroute_embed_model: DEFAULT_OMNIROUTE_EMBED_MODEL.to_owned(),
            omniroute_timeout_seconds: DEFAULT_OMNIROUTE_TIMEOUT_SECONDS,
            omniroute_api_key: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoogleOAuthClientType {
    Installed,
    Web,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoogleOAuthClientConfig {
    client_type: GoogleOAuthClientType,
    client_id: String,
    client_secret: Option<ResolvedSecret>,
    authorization_endpoint: String,
    token_endpoint: String,
    redirect_uris: Vec<String>,
}

impl GoogleOAuthClientConfig {
    fn from_client_secret_json(raw_json: &str) -> Result<Self, ConfigError> {
        let file: GoogleOAuthClientSecretsFile =
            serde_json::from_str(raw_json).map_err(ConfigError::GoogleOAuthClientConfigJson)?;
        if let Some(installed) = file.installed {
            return Self::from_payload(GoogleOAuthClientType::Installed, installed);
        }
        if let Some(web) = file.web {
            return Self::from_payload(GoogleOAuthClientType::Web, web);
        }

        Err(ConfigError::InvalidGoogleOAuthClientConfig {
            field: "client_type",
            message: "must contain installed or web client credentials",
        })
    }

    fn from_payload(
        client_type: GoogleOAuthClientType,
        payload: GoogleOAuthClientSecretsPayload,
    ) -> Result<Self, ConfigError> {
        let client_id = required_trimmed("client_id", payload.client_id)?;
        let authorization_endpoint = required_trimmed("auth_uri", payload.auth_uri)?;
        let token_endpoint = required_trimmed("token_uri", payload.token_uri)?;
        let client_secret = payload
            .client_secret
            .map(|secret| required_trimmed("client_secret", Some(secret)))
            .transpose()?
            .map(ResolvedSecret::new)
            .transpose()?;
        let redirect_uris = payload
            .redirect_uris
            .unwrap_or_default()
            .into_iter()
            .map(|uri| required_trimmed("redirect_uris", Some(uri)))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            client_type,
            client_id,
            client_secret,
            authorization_endpoint,
            token_endpoint,
            redirect_uris,
        })
    }

    pub fn client_type(&self) -> GoogleOAuthClientType {
        self.client_type
    }

    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    pub fn client_secret(&self) -> Option<&ResolvedSecret> {
        self.client_secret.as_ref()
    }

    pub fn authorization_endpoint(&self) -> &str {
        &self.authorization_endpoint
    }

    pub fn token_endpoint(&self) -> &str {
        &self.token_endpoint
    }

    pub fn redirect_uris(&self) -> &[String] {
        &self.redirect_uris
    }
}

#[derive(Debug, Deserialize)]
struct GoogleOAuthClientSecretsFile {
    installed: Option<GoogleOAuthClientSecretsPayload>,
    web: Option<GoogleOAuthClientSecretsPayload>,
}

#[derive(Debug, Deserialize)]
struct GoogleOAuthClientSecretsPayload {
    client_id: Option<String>,
    client_secret: Option<String>,
    auth_uri: Option<String>,
    token_uri: Option<String>,
    redirect_uris: Option<Vec<String>>,
}

fn required_trimmed(field: &'static str, value: Option<String>) -> Result<String, ConfigError> {
    let Some(value) = value else {
        return Err(ConfigError::InvalidGoogleOAuthClientConfig {
            field,
            message: "must be present",
        });
    };
    let value = value.trim();
    if value.is_empty() {
        return Err(ConfigError::InvalidGoogleOAuthClientConfig {
            field,
            message: "must not be empty",
        });
    }
    Ok(value.to_owned())
}

fn parse_bool_env(name: &'static str, value: &str) -> Result<bool, ConfigError> {
    match value {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        other => Err(ConfigError::InvalidBoolEnv {
            name,
            value: other.to_owned(),
        }),
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

    #[error("invalid HERMES_AI_PROVIDER `{value}`: expected ollama or omniroute")]
    InvalidAiProvider { value: String },

    #[error("DATABASE_URL is set but empty")]
    EmptyDatabaseUrl,

    #[error("HERMES_LOCAL_API_SECRET is set but empty")]
    EmptyLocalApiSecret,

    #[error("HERMES_SECRET_VAULT_PATH is set but empty")]
    EmptySecretVaultPath,

    #[error("HERMES_SECRET_VAULT_KEY is set but empty")]
    EmptySecretVaultKey,

    #[error("HERMES_VAULT_HOME is set but empty")]
    EmptyVaultHome,

    #[error("HERMES_DEV_KEY_PATH is set but empty")]
    EmptyDevKeyPath,

    #[error("invalid {name} `{value}`: expected true or false")]
    InvalidBoolEnv { name: &'static str, value: String },

    #[error("HERMES_TDJSON_PATH is set but empty")]
    EmptyTdjsonPath,

    #[error("invalid HERMES_TELEGRAM_API_ID `{value}`: {reason}")]
    InvalidTelegramApiId {
        value: String,
        reason: &'static str,
        #[source]
        source: Option<ParseIntError>,
    },

    #[error("HERMES_TELEGRAM_API_HASH is set but empty")]
    EmptyTelegramApiHash,

    #[error("HERMES_GOOGLE_OAUTH_CLIENT_ID is set but empty")]
    EmptyGoogleOAuthClientId,

    #[error("HERMES_GOOGLE_OAUTH_CLIENT_SECRET is set but empty")]
    EmptyGoogleOAuthClientSecret,

    #[error("HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_JSON is set but empty")]
    EmptyGoogleOAuthClientConfigJson,

    #[error("HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_PATH is set but empty")]
    EmptyGoogleOAuthClientConfigPath,

    #[error("failed to read HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_PATH `{}`: {source}", path.display())]
    GoogleOAuthClientConfigRead {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("invalid Google OAuth client credentials JSON: {0}")]
    GoogleOAuthClientConfigJson(serde_json::Error),

    #[error("invalid Google OAuth client config field {field}: {message}")]
    InvalidGoogleOAuthClientConfig {
        field: &'static str,
        message: &'static str,
    },

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

    #[error("HERMES_OMNIROUTE_BASE_URL is set but empty")]
    EmptyOmniRouteBaseUrl,

    #[error("HERMES_OMNIROUTE_CHAT_MODEL is set but empty")]
    EmptyOmniRouteChatModel,

    #[error("HERMES_OMNIROUTE_EMBED_MODEL is set but empty")]
    EmptyOmniRouteEmbedModel,

    #[error("HERMES_OMNIROUTE_API_KEY is set but empty")]
    EmptyOmniRouteApiKey,

    #[error("invalid HERMES_OMNIROUTE_TIMEOUT_SECONDS `{value}`: {reason}")]
    InvalidOmniRouteTimeout {
        value: String,
        reason: &'static str,
        #[source]
        source: Option<ParseIntError>,
    },

    #[error(transparent)]
    SecretResolution(#[from] SecretResolutionError),
}
