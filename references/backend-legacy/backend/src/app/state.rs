use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::ai::control_center::models::AiProviderAuthPendingGrant;
use crate::integrations::mail::accounts::models::GmailOAuthPendingGrant;
use crate::integrations::telegram::runtime::manager::TelegramRuntimeManager;
use crate::integrations::telegram::tdjson::qr_login_support::types::PendingQrLoginMap;
use crate::integrations::zoom::client::models::oauth_models::ZoomOAuthPendingGrant;
use crate::platform::config::app_config::AppConfig;
use crate::platform::events::bus::InMemoryEventBus;
use crate::platform::storage::database::Database;
use crate::vault::HostVault;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) config: AppConfig,
    pub(crate) database: Database,
    pub(crate) vault: HostVault,
    pub(crate) account_setup: AccountSetupState,
    pub(crate) telegram_runtime: TelegramRuntimeManager,
    pub(crate) event_bus: InMemoryEventBus,
}

#[derive(Clone, Default)]
pub(crate) struct AccountSetupState {
    pub(crate) pending_gmail_oauth: Arc<Mutex<HashMap<String, GmailOAuthPendingGrant>>>,
    pub(crate) pending_ai_provider_auth: Arc<Mutex<HashMap<String, AiProviderAuthPendingGrant>>>,
    pub(crate) pending_zoom_oauth: Arc<Mutex<HashMap<String, ZoomOAuthPendingGrant>>>,
    pub(crate) pending_telegram_qr_login: PendingQrLoginMap,
}
