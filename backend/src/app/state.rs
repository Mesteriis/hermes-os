use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::ai::control_center::AiProviderAuthPendingGrant;
use crate::app::runtime::RuntimeLease;
use crate::integrations::mail::accounts::models::GmailOAuthPendingGrant;
use crate::integrations::telegram::runtime::TelegramRuntimeManager;
use crate::integrations::telegram::tdjson::PendingQrLoginMap;
use crate::integrations::zoom::client::models::ZoomOAuthPendingGrant;
use crate::platform::config::AppConfig;
use crate::platform::events::bus::InMemoryEventBus;
use crate::platform::storage::Database;
use crate::vault::HostVault;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) config: AppConfig,
    pub(crate) database: Database,
    pub(crate) vault: HostVault,
    pub(crate) account_setup: AccountSetupState,
    pub(crate) telegram_runtime: TelegramRuntimeManager,
    pub(crate) event_bus: InMemoryEventBus,
    pub(crate) runtime_lease: Option<RuntimeLease>,
}

#[derive(Clone, Default)]
pub(crate) struct AccountSetupState {
    pub(crate) pending_gmail_oauth: Arc<Mutex<HashMap<String, GmailOAuthPendingGrant>>>,
    pub(crate) pending_ai_provider_auth: Arc<Mutex<HashMap<String, AiProviderAuthPendingGrant>>>,
    pub(crate) pending_zoom_oauth: Arc<Mutex<HashMap<String, ZoomOAuthPendingGrant>>>,
    pub(crate) pending_telegram_qr_login: PendingQrLoginMap,
}
