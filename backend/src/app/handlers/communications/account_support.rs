use super::*;

#[derive(Serialize)]
pub(crate) struct MailSyncStatusListResponse {
    pub(super) items: Vec<MailSyncStatus>,
}

#[derive(Serialize)]
pub(crate) struct EmailAccountListResponse {
    pub(super) items: Vec<EmailAccountView>,
}

#[derive(Serialize)]
pub(crate) struct EmailAccountView {
    pub(super) account: ProviderAccount,
    pub(super) capabilities: EmailAccountCapabilities,
}

#[derive(Serialize)]
pub(crate) struct EmailAccountCapabilities {
    pub(super) read: bool,
    pub(super) sync: bool,
    pub(super) send: bool,
    pub(super) oauth: bool,
    pub(super) imap: bool,
    pub(super) smtp: bool,
    pub(super) mutate_flags: bool,
    pub(super) mutate_mailboxes: bool,
    pub(super) server_delete: bool,
    pub(super) provider_folders: bool,
    pub(super) local_trash: bool,
}

#[derive(Serialize)]
pub(crate) struct EmailAccountExportResponse {
    pub(super) exported_at: DateTime<Utc>,
    pub(super) account: ProviderAccount,
    pub(super) capabilities: EmailAccountCapabilities,
    pub(super) sync_settings: MailSyncSettings,
}

#[derive(Serialize)]
pub(crate) struct EmailAccountLogoutResponse {
    pub(super) account: ProviderAccount,
    pub(super) capabilities: EmailAccountCapabilities,
    pub(super) sync_settings: MailSyncSettings,
}

#[derive(Serialize)]
pub(crate) struct EmailAccountDeleteResponse {
    pub(super) account_id: String,
    pub(super) deleted: bool,
    pub(super) unbound_secret_refs: Vec<String>,
}

#[derive(Deserialize)]
pub(super) struct EmailAccountImportRequest {
    pub(super) account: EmailAccountImportAccount,
    pub(super) sync_settings: Option<EmailAccountImportSyncSettings>,
}

#[derive(Deserialize)]
pub(super) struct EmailAccountImportAccount {
    pub(super) account_id: String,
    pub(super) provider_kind: String,
    pub(super) display_name: String,
    pub(super) external_account_id: String,
    #[serde(default)]
    pub(super) config: Value,
}

#[derive(Deserialize)]
pub(super) struct EmailAccountImportSyncSettings {
    pub(super) sync_enabled: Option<bool>,
    pub(super) batch_size: Option<i32>,
    pub(super) poll_interval_seconds: Option<i32>,
}
pub(super) async fn email_account_or_not_found(
    state: &AppState,
    account_id: &str,
) -> Result<ProviderAccount, ApiError> {
    let Some(pool) = state.database.pool().cloned() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let Some(account) = CommunicationProviderAccountStore::new(pool)
        .get(account_id)
        .await?
    else {
        return Err(ApiError::NotFound);
    };
    if !account.provider_kind.is_email() {
        return Err(ApiError::NotFound);
    }

    Ok(account)
}

pub(super) fn email_account_view(account: ProviderAccount) -> EmailAccountView {
    EmailAccountView {
        capabilities: email_account_capabilities(&account),
        account,
    }
}

pub(super) fn email_account_capabilities(account: &ProviderAccount) -> EmailAccountCapabilities {
    let logged_out = account
        .config
        .get("auth_state")
        .and_then(Value::as_str)
        .is_some_and(|state| state == "logged_out");
    let smtp = smtp_configured(&account.config);
    let imap = matches!(
        account.provider_kind,
        EmailProviderKind::Icloud | EmailProviderKind::Imap
    );
    let oauth = matches!(account.provider_kind, EmailProviderKind::Gmail)
        || account
            .config
            .get("auth")
            .and_then(Value::as_str)
            .is_some_and(|auth| auth == "oauth");
    let gmail_send_enabled = account
        .config
        .get("gmail_send_enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    EmailAccountCapabilities {
        read: !logged_out,
        sync: !logged_out,
        send: !logged_out && (smtp || gmail_send_enabled),
        oauth,
        imap,
        smtp,
        mutate_flags: !logged_out && imap,
        mutate_mailboxes: false,
        server_delete: false,
        provider_folders: false,
        local_trash: true,
    }
}

pub(super) fn smtp_configured(config: &Value) -> bool {
    let Some(object) = config.as_object() else {
        return false;
    };
    object
        .get("smtp_host")
        .and_then(Value::as_str)
        .is_some_and(|host| !host.trim().is_empty())
        && object.get("smtp_port").and_then(Value::as_i64).is_some()
}

pub(super) fn sanitize_account_config(value: &Value) -> Value {
    match value {
        Value::Object(object) => Value::Object(
            object
                .iter()
                .filter_map(|(key, value)| {
                    if is_secret_config_key(key) {
                        None
                    } else {
                        Some((key.clone(), sanitize_account_config(value)))
                    }
                })
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(sanitize_account_config).collect()),
        other => other.clone(),
    }
}

pub(super) fn contains_secret_material(value: &Value) -> bool {
    match value {
        Value::Object(object) => object
            .iter()
            .any(|(key, value)| is_secret_config_key(key) || contains_secret_material(value)),
        Value::Array(items) => items.iter().any(contains_secret_material),
        _ => false,
    }
}

pub(super) fn is_secret_config_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    [
        "password",
        "secret",
        "secret_ref",
        "token",
        "credential",
        "api_key",
        "private_key",
        "client_secret",
        "refresh_token",
        "access_token",
    ]
    .iter()
    .any(|marker| key.contains(marker))
}

pub(super) fn require_unlocked_host_vault(state: &AppState) -> Result<(), ApiError> {
    match state.vault.status()?.state {
        VaultMode::Unlocked => Ok(()),
        VaultMode::Locked => Err(ApiError::HostVault(HostVaultError::Locked)),
        VaultMode::Uninitialized => Err(ApiError::HostVault(HostVaultError::Uninitialized)),
    }
}

pub(super) fn mail_sync_store(state: &AppState) -> Result<MailSyncStore, MailSyncError> {
    let Some(pool) = state.database.pool() else {
        return Err(MailSyncError::InvalidSetting {
            field: "database",
            message: "DATABASE_URL is not configured",
        });
    };

    Ok(MailSyncStore::new(pool.clone()))
}

pub(super) fn mail_sync_service(
    state: &AppState,
) -> Result<MailBackgroundSyncService, MailSyncError> {
    let Some(pool) = state.database.pool() else {
        return Err(MailSyncError::InvalidSetting {
            field: "database",
            message: "DATABASE_URL is not configured",
        });
    };

    Ok(MailBackgroundSyncService::new(
        pool.clone(),
        state.vault.clone(),
        DEFAULT_MAIL_SYNC_BLOB_ROOT,
        std::sync::Arc::new(
            crate::integrations::mail::sync_provider::LiveEmailProviderSyncPort::new(
                pool.clone(),
                state.vault.clone(),
                std::sync::Arc::new(
                    crate::domains::communications::core::CommunicationProviderSecretBindingStore::new(pool.clone()),
                ),
                crate::app::workflow_services::mail_background_sync::DEFAULT_GMAIL_API_BASE_URL,
            ),
        ),
    ))
}

pub(super) fn mail_sync_api_error(error: MailSyncError) -> ApiError {
    match error {
        MailSyncError::AccountNotFound => ApiError::NotFound,
        MailSyncError::RunAlreadyActive | MailSyncError::RunNotFound => {
            ApiError::InvalidCommunicationQuery("mail sync run is already active")
        }
        MailSyncError::InvalidSetting {
            field: "database", ..
        } => ApiError::DatabaseNotConfigured,
        MailSyncError::InvalidSetting { .. } => {
            ApiError::InvalidCommunicationQuery("invalid mail sync settings")
        }
        MailSyncError::Sqlx(error) => {
            tracing::error!(error = %error, "mail sync database operation failed");
            ApiError::InvalidCommunicationQuery("mail sync operation failed")
        }
        MailSyncError::Communication(error) => {
            tracing::error!(error = %error, "mail sync communication store failed");
            ApiError::InvalidCommunicationQuery("mail sync operation failed")
        }
        MailSyncError::EventEnvelope(error) => ApiError::InvalidEnvelope(error),
        MailSyncError::EventLogPort(error) => ApiError::Store(error),
        MailSyncError::ObservationPort(error) => {
            tracing::error!(error = %error, "mail sync observation store failed");
            ApiError::InvalidCommunicationQuery("mail sync operation failed")
        }
    }
}
