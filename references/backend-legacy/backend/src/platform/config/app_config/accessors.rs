use std::net::SocketAddr;
use std::path::Path;

use crate::platform::secrets::models::ResolvedSecret;

use super::super::ai::AiRuntimeProvider;
use super::super::google::GoogleOAuthClientConfig;
use super::AppConfig;

impl AppConfig {
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

    pub fn nats_server_url(&self) -> Option<&str> {
        self.nats_server_url.as_deref()
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

    pub fn zoom_token_maintenance_scheduler_enabled(&self) -> bool {
        self.zoom_token_maintenance_scheduler_enabled
    }

    pub fn zoom_recording_sync_scheduler_enabled(&self) -> bool {
        self.zoom_recording_sync_scheduler_enabled
    }

    pub fn zoom_retention_cleanup_scheduler_enabled(&self) -> bool {
        self.zoom_retention_cleanup_scheduler_enabled
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
