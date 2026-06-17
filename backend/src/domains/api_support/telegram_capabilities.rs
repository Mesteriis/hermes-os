use super::*;
use crate::domains::mail::core::ProviderAccount;

// ---------------------------------------------------------------------------

/// Capability states per ADR-0091.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramCapabilityState {
    Available,
    Blocked,
    Degraded,
    Unsupported,
}

impl TelegramCapabilityState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Blocked => "blocked",
            Self::Degraded => "degraded",
            Self::Unsupported => "unsupported",
        }
    }
}

/// Action classes per ADR-0052.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramActionClass {
    Read,
    LocalWrite,
    ProviderWrite,
    Destructive,
    Export,
    SecretAccess,
    Automation,
}

impl TelegramActionClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::LocalWrite => "local_write",
            Self::ProviderWrite => "provider_write",
            Self::Destructive => "destructive",
            Self::Export => "export",
            Self::SecretAccess => "secret_access",
            Self::Automation => "automation",
        }
    }
}

/// A single operation capability entry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TelegramOperationCapability {
    pub operation: String,
    pub category: String,
    pub status: String,
    pub action_class: String,
    pub reason: String,
    pub confirmation_required: bool,
    pub closure_gate: bool,
}

impl TelegramOperationCapability {
    pub(super) fn new(
        operation: &str,
        category: &str,
        state: TelegramCapabilityState,
        action_class: TelegramActionClass,
        reason: &str,
        confirmation_required: bool,
        closure_gate: bool,
    ) -> Self {
        Self {
            operation: operation.to_owned(),
            category: category.to_owned(),
            status: state.as_str().to_owned(),
            action_class: action_class.as_str().to_owned(),
            reason: reason.to_owned(),
            confirmation_required,
            closure_gate,
        }
    }
}

/// Detailed per-operation Telegram capability response per ADR-0091.
#[derive(Serialize)]
pub(crate) struct TelegramCapabilitiesResponse {
    pub(crate) version: &'static str,
    pub(crate) runtime_mode: &'static str,
    pub(crate) account_scope: Option<TelegramCapabilityAccountScope>,
    pub(crate) telegram_app_credentials_configured: bool,
    pub(crate) tdjson_runtime_available: bool,
    pub(crate) qr_login_ready: bool,
    pub(crate) bot_runtime_available: bool,
    pub(crate) capabilities: Vec<TelegramOperationCapability>,
    pub(crate) unsupported_features: Vec<&'static str>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TelegramCapabilityAccountScope {
    pub account_id: String,
    pub provider_kind: String,
    pub runtime_kind: String,
    pub lifecycle_state: String,
}

impl TelegramCapabilitiesResponse {
    pub(crate) fn current(config: &AppConfig) -> Self {
        Self::build(config, None)
    }

    pub(crate) fn current_for_account(config: &AppConfig, account: &ProviderAccount) -> Self {
        Self::build(config, Some(account))
    }

    fn build(config: &AppConfig, account: Option<&ProviderAccount>) -> Self {
        let app_creds = config.telegram_api_id().is_some() && config.telegram_api_hash().is_some();
        let tdjson_ok = tdjson::runtime_available(config.tdjson_path());
        let qr_ready = app_creds && tdjson_ok;
        let bot_ok = false; // Bot API runtime not implemented per ADR-0091
        let account_scope = account.map(TelegramCapabilityAccountScope::from_account);

        let capabilities = super::telegram_capability_catalog::telegram_capability_rows(qr_ready);
        let mut response = Self {
            version: "2.0",
            runtime_mode: if let Some(scope) = account_scope.as_ref() {
                match scope.runtime_kind.as_str() {
                    "tdlib_qr_authorized" => "tdlib_qr_authorized",
                    "live_blocked" => "live_blocked",
                    "fixture" => "fixture",
                    _ => "unknown",
                }
            } else if qr_ready {
                "tdlib_qr"
            } else {
                "fixture"
            },
            account_scope,
            telegram_app_credentials_configured: app_creds,
            tdjson_runtime_available: tdjson_ok,
            qr_login_ready: qr_ready,
            bot_runtime_available: bot_ok,
            capabilities,
            unsupported_features: vec![
                "video_calls",
                "group_calls",
                "screen_sharing",
                "hidden_recording",
                "telegram_data_fine_tuning",
                "third_party_plugin_execution",
                "session_import_export",
                "proxy_profiles",
                "forum_topics",
                "chat_export",
                "bot_live_runtime",
            ],
        };
        response.apply_account_scope_overrides();
        response
    }

    fn apply_account_scope_overrides(&mut self) {
        let Some(scope) = self.account_scope.as_ref() else {
            return;
        };
        let provider_kind = scope.provider_kind.as_str();
        let lifecycle_state = scope.lifecycle_state.as_str();
        let runtime_kind = scope.runtime_kind.as_str();
        let is_bot = provider_kind == "telegram_bot";
        let is_removed = lifecycle_state == "removed";
        let is_logged_out = lifecycle_state == "logged_out";

        for capability in &mut self.capabilities {
            match capability.operation.as_str() {
                "auth.qr_start" | "auth.qr_status" | "auth.qr_password" | "auth.qr_cancel"
                    if is_bot =>
                {
                    capability.status = TelegramCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason =
                        "Bot accounts do not use TDLib QR authorization.".to_owned();
                }
                "runtime.tdlib_live" if is_bot => {
                    capability.status = TelegramCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason =
                        "Bot accounts do not use the TDLib user runtime.".to_owned();
                }
                "runtime.bot_live" if !is_bot => {
                    capability.status = TelegramCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason = "User accounts do not use the Bot API runtime.".to_owned();
                }
                "messages.send_text" if is_bot && runtime_kind == "fixture" => {
                    capability.status = TelegramCapabilityState::Degraded.as_str().to_owned();
                    capability.reason = "Fixture bot accounts can validate local command flow, but the live Bot API runtime is still missing.".to_owned();
                }
                "messages.send_media" | "media.upload_send" if is_bot => {
                    capability.status = TelegramCapabilityState::Blocked.as_str().to_owned();
                    capability.reason =
                        "Bot media upload/send requires the separate Bot API runtime.".to_owned();
                }
                "messages.send_media" | "media.upload_send"
                    if runtime_kind != "tdlib_qr_authorized" =>
                {
                    capability.status = TelegramCapabilityState::Blocked.as_str().to_owned();
                    capability.reason = format!(
                        "Account `{}` must use tdlib_qr_authorized runtime before media upload/send is available.",
                        scope.account_id
                    );
                }
                "participants.sync" | "participants.join" | "participants.leave" if is_bot => {
                    capability.status = TelegramCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason =
                        "Bot accounts do not use the TDLib participant lifecycle runtime."
                            .to_owned();
                }
                "participants.sync" | "participants.join" | "participants.leave"
                    if runtime_kind != "tdlib_qr_authorized" =>
                {
                    capability.status = TelegramCapabilityState::Blocked.as_str().to_owned();
                    capability.reason = format!(
                        "Account `{}` must use tdlib_qr_authorized runtime before participant lifecycle commands are available.",
                        scope.account_id
                    );
                }
                _ => {}
            }

            if is_removed {
                if matches!(
                    capability.action_class.as_str(),
                    "local_write"
                        | "provider_write"
                        | "destructive"
                        | "secret_access"
                        | "automation"
                ) || capability.operation.starts_with("sync.")
                    || capability.operation.starts_with("runtime.")
                {
                    capability.status = TelegramCapabilityState::Blocked.as_str().to_owned();
                    capability.reason = format!(
                        "Account `{}` is removed; this operation is no longer available.",
                        scope.account_id
                    );
                }
            } else if is_logged_out
                && (capability.operation.starts_with("sync.")
                    || capability.operation == "messages.send_text"
                    || capability.operation == "messages.send_media"
                    || capability.operation == "media.upload_send"
                    || capability.operation == "media.download")
            {
                capability.status = TelegramCapabilityState::Blocked.as_str().to_owned();
                capability.reason = format!(
                    "Account `{}` is logged out; re-authorize the runtime before using this operation.",
                    scope.account_id
                );
            }
        }
    }
}

impl TelegramCapabilityAccountScope {
    fn from_account(account: &ProviderAccount) -> Self {
        Self {
            account_id: account.account_id.clone(),
            provider_kind: account.provider_kind.as_str().to_owned(),
            runtime_kind: account
                .config
                .get("runtime")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("unknown")
                .to_owned(),
            lifecycle_state: account
                .config
                .get("lifecycle_state")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("active")
                .to_owned(),
        }
    }
}
