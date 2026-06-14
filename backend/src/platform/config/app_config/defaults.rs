use std::env;
use std::path::PathBuf;

use crate::vault::{default_dev_key_path, default_vault_home};

use super::super::ai::AiRuntimeProvider;
use super::super::constants::{
    DEFAULT_HTTP_ADDR, DEFAULT_OLLAMA_BASE_URL, DEFAULT_OLLAMA_CHAT_MODEL,
    DEFAULT_OLLAMA_EMBED_MODEL, DEFAULT_OLLAMA_TIMEOUT_SECONDS, DEFAULT_OMNIROUTE_BASE_URL,
    DEFAULT_OMNIROUTE_CHAT_MODEL, DEFAULT_OMNIROUTE_EMBED_MODEL, DEFAULT_OMNIROUTE_TIMEOUT_SECONDS,
    DEFAULT_SERVICE_NAME,
};
use super::AppConfig;

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
