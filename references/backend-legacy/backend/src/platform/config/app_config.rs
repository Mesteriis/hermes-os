mod accessors;
mod ai_env;
mod core_env;
mod defaults;
mod env;
mod provider_env;
#[cfg(any(test, feature = "test-support"))]
mod test_support;

use std::net::SocketAddr;
use std::path::PathBuf;

use crate::platform::secrets::models::ResolvedSecret;

use super::ai::AiRuntimeProvider;
use super::google::GoogleOAuthClientConfig;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppConfig {
    service_name: String,
    http_addr: SocketAddr,
    database_url: Option<String>,
    local_api_secret: Option<String>,
    nats_server_url: Option<String>,
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
    zoom_token_maintenance_scheduler_enabled: bool,
    zoom_recording_sync_scheduler_enabled: bool,
    zoom_retention_cleanup_scheduler_enabled: bool,
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
