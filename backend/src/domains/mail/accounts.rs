use std::time::Duration;

use aes_gcm::aead::rand_core::RngCore;
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, TimeDelta, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use thiserror::Error;
use url::form_urlencoded;

use crate::domains::mail::core::{
    CommunicationIngestionError, CommunicationIngestionStore, EmailProviderKind,
    NewProviderAccount, NewProviderAccountSecretBinding, ProviderAccountSecretPurpose,
};
use crate::platform::secrets::{DatabaseEncryptedSecretVault, DatabaseEncryptedVaultError};
use crate::platform::secrets::{
    NewSecretReference, ResolvedSecret, SecretKind, SecretReference, SecretReferenceError,
    SecretReferenceStore, SecretResolutionError, SecretResolutionFuture, SecretResolver,
    SecretStoreKind,
};
use crate::vault::{HostVault, HostVaultError, SecretEntryContext};

const DEFAULT_GOOGLE_AUTHORIZATION_ENDPOINT: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const DEFAULT_GOOGLE_TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_GMAIL_READONLY_SCOPE: &str = "https://www.googleapis.com/auth/gmail.readonly";
const GOOGLE_CALENDAR_READONLY_SCOPE: &str = "https://www.googleapis.com/auth/calendar.readonly";
const GOOGLE_CONTACTS_READONLY_SCOPE: &str = "https://www.googleapis.com/auth/contacts.readonly";
const DEFAULT_GOOGLE_WORKSPACE_SCOPES: [&str; 3] = [
    GOOGLE_GMAIL_READONLY_SCOPE,
    GOOGLE_CALENDAR_READONLY_SCOPE,
    GOOGLE_CONTACTS_READONLY_SCOPE,
];

#[derive(Clone)]
enum AccountSecretVault {
    Database(DatabaseEncryptedSecretVault),
    Host(HostVault),
}

impl AccountSecretVault {
    fn store_kind(&self) -> SecretStoreKind {
        match self {
            Self::Database(_) => SecretStoreKind::DatabaseEncryptedVault,
            Self::Host(_) => SecretStoreKind::HostVault,
        }
    }

    async fn store_secret(
        &self,
        secret_ref: &str,
        value: &str,
        context: SecretWriteContext<'_>,
    ) -> Result<(), EmailAccountSetupError> {
        match self {
            Self::Database(vault) => vault.store_secret(secret_ref, value).await?,
            Self::Host(vault) => vault.store_secret(
                secret_ref,
                value,
                SecretEntryContext {
                    entry_kind: context.entry_kind,
                    account_id: context.account_id,
                    purpose: context.purpose,
                    secret_kind: context.secret_kind.as_str(),
                    label: context.label,
                    metadata: context.metadata,
                },
            )?,
        }
        Ok(())
    }

    fn secret_reference(&self, secret_ref: &str, secret_kind: SecretKind) -> SecretReference {
        vault_secret_reference(secret_ref, secret_kind, self.store_kind())
    }
}

struct SecretWriteContext<'a> {
    entry_kind: &'a str,
    account_id: &'a str,
    purpose: &'a str,
    secret_kind: SecretKind,
    label: &'a str,
    metadata: &'a serde_json::Value,
}

impl SecretResolver for AccountSecretVault {
    fn resolve<'a>(&'a self, reference: &'a SecretReference) -> SecretResolutionFuture<'a> {
        match self {
            Self::Database(vault) => vault.resolve(reference),
            Self::Host(vault) => vault.resolve(reference),
        }
    }
}

#[derive(Clone)]
pub struct EmailAccountSetupService {
    communication_store: Option<CommunicationIngestionStore>,
    secret_store: Option<SecretReferenceStore>,
    vault: AccountSecretVault,
    http: Client,
}

impl EmailAccountSetupService {
    pub fn new(
        communication_store: CommunicationIngestionStore,
        secret_store: SecretReferenceStore,
        vault: DatabaseEncryptedSecretVault,
    ) -> Self {
        Self {
            communication_store: Some(communication_store),
            secret_store: Some(secret_store),
            vault: AccountSecretVault::Database(vault),
            http: http_client(),
        }
    }

    pub fn new_for_vault_only(vault: DatabaseEncryptedSecretVault) -> Self {
        Self {
            communication_store: None,
            secret_store: None,
            vault: AccountSecretVault::Database(vault),
            http: http_client(),
        }
    }

    pub fn new_with_host_vault(
        communication_store: CommunicationIngestionStore,
        secret_store: SecretReferenceStore,
        vault: HostVault,
    ) -> Self {
        Self {
            communication_store: Some(communication_store),
            secret_store: Some(secret_store),
            vault: AccountSecretVault::Host(vault),
            http: http_client(),
        }
    }

    pub fn start_gmail_oauth(
        &self,
        request: GmailOAuthSetupRequest,
    ) -> Result<GmailOAuthPendingGrant, EmailAccountSetupError> {
        request.validate()?;
        let setup_id = random_url_token();
        let state = random_url_token();
        let code_verifier = random_url_token();
        let code_challenge = pkce_challenge(&code_verifier);
        let scope = request.scopes.join(" ");
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("response_type", "code")
            .append_pair("client_id", &request.client_id)
            .append_pair("redirect_uri", &request.redirect_uri)
            .append_pair("scope", &scope)
            .append_pair("state", &state)
            .append_pair("code_challenge", &code_challenge)
            .append_pair("code_challenge_method", "S256")
            .append_pair("access_type", "offline")
            .append_pair("prompt", "consent")
            .finish();
        let authorization_url = format!("{}?{query}", request.authorization_endpoint);

        Ok(GmailOAuthPendingGrant {
            setup_id,
            account_id: request.account_id.clone(),
            authorization_url,
            state,
            code_verifier,
            request,
        })
    }

    pub async fn complete_gmail_oauth(
        &self,
        pending: GmailOAuthPendingGrant,
        authorization_code: &str,
    ) -> Result<EmailAccountSetupResult, EmailAccountSetupError> {
        validate_non_empty("authorization_code", authorization_code)?;
        let external_account_id = if pending.request.external_account_id.trim().is_empty() {
            pending.account_id.clone()
        } else {
            pending.request.external_account_id.clone()
        };
        let token = self
            .exchange_authorization_code(&pending, authorization_code)
            .await?;
        let refresh_token =
            token
                .refresh_token
                .clone()
                .ok_or(EmailAccountSetupError::MissingProviderField {
                    field: "refresh_token",
                })?;
        let expires_at = expires_at(token.expires_in);
        let secret_ref = oauth_secret_ref(&pending.account_id);
        let token_bundle = GmailOAuthTokenBundle {
            token_url: pending.request.token_endpoint.clone(),
            client_id: pending.request.client_id.clone(),
            client_secret: pending.request.client_secret.clone(),
            access_token: token.access_token,
            refresh_token,
            expires_at,
            token_type: token.token_type,
            scope: token.scope,
        };
        let secret_store = self.secret_store()?;
        let communication_store = self.communication_store()?;
        let account_config = json!({
            "auth": "oauth",
            "api": "gmail",
            "oauth_client_id": pending.request.client_id,
            "requested_scopes": pending.request.scopes,
            "connected_services": ["mail", "calendar", "contacts"],
            "history_stream_id": "gmail:history"
        });
        let secret_metadata = json!({
            "provider": "gmail",
            "account_id": pending.account_id,
            "display_name": pending.request.display_name,
            "external_account_id": external_account_id,
            "connected_services": ["mail", "calendar", "contacts"],
            "provider_account_config": account_config.clone()
        });
        secret_store
            .upsert_secret_reference(
                &NewSecretReference::new(
                    &secret_ref,
                    SecretKind::OauthToken,
                    self.vault.store_kind(),
                    format!(
                        "Gmail OAuth credential for {}",
                        pending.request.display_name
                    ),
                )
                .metadata(secret_metadata.clone()),
            )
            .await?;
        self.vault
            .store_secret(
                &secret_ref,
                &serde_json::to_string(&token_bundle)?,
                SecretWriteContext {
                    entry_kind: "provider_credential",
                    account_id: &pending.account_id,
                    purpose: ProviderAccountSecretPurpose::OauthToken.as_str(),
                    secret_kind: SecretKind::OauthToken,
                    label: "Gmail OAuth credential",
                    metadata: &secret_metadata,
                },
            )
            .await?;
        communication_store
            .upsert_provider_account(
                &NewProviderAccount::new(
                    &pending.account_id,
                    EmailProviderKind::Gmail,
                    &pending.request.display_name,
                    &external_account_id,
                )
                .config(account_config),
            )
            .await?;
        communication_store
            .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
                &pending.account_id,
                ProviderAccountSecretPurpose::OauthToken,
                &secret_ref,
            ))
            .await?;

        Ok(EmailAccountSetupResult {
            account_id: pending.account_id,
            secret_ref,
            secret_kind: SecretKind::OauthToken,
            store_kind: self.vault.store_kind(),
        })
    }

    pub async fn refresh_gmail_access_token(
        &self,
        secret_ref: &str,
    ) -> Result<ResolvedSecret, EmailAccountSetupError> {
        validate_non_empty("secret_ref", secret_ref)?;
        let reference = self
            .vault
            .secret_reference(secret_ref, SecretKind::OauthToken);
        let resolved = self.vault.resolve(&reference).await?;
        let mut bundle: GmailOAuthTokenBundle =
            serde_json::from_str(resolved.expose_for_runtime())?;

        if bundle.expires_at > Utc::now() + TimeDelta::seconds(60) {
            return ResolvedSecret::new(bundle.access_token)
                .map_err(EmailAccountSetupError::Secret);
        }

        let refreshed = self.refresh_token(&bundle).await?;
        bundle.access_token = refreshed.access_token;
        if let Some(refresh_token) = refreshed.refresh_token {
            bundle.refresh_token = refresh_token;
        }
        bundle.expires_at = expires_at(refreshed.expires_in);
        bundle.token_type = refreshed.token_type;
        if refreshed.scope.is_some() {
            bundle.scope = refreshed.scope;
        }

        self.vault
            .store_secret(
                secret_ref,
                &serde_json::to_string(&bundle)?,
                SecretWriteContext {
                    entry_kind: "provider_credential",
                    account_id: secret_ref,
                    purpose: ProviderAccountSecretPurpose::OauthToken.as_str(),
                    secret_kind: SecretKind::OauthToken,
                    label: "OAuth credential",
                    metadata: &json!({}),
                },
            )
            .await?;
        ResolvedSecret::new(bundle.access_token).map_err(EmailAccountSetupError::Secret)
    }

    pub async fn setup_imap_account(
        &self,
        request: ImapAccountSetupRequest,
    ) -> Result<EmailAccountSetupResult, EmailAccountSetupError> {
        request.validate()?;
        let secret_ref = imap_secret_ref(&request.account_id);
        let smtp_secret_ref = smtp_secret_ref(&request.account_id);
        let connected_services = email_provider_connected_services(request.provider_kind);
        let smtp_config = request.smtp_config();
        let mut account_config = json!({
            "host": request.host,
            "port": request.port,
            "tls": request.tls,
            "mailbox": request.mailbox,
            "username": request.username,
            "smtp_host": smtp_config.host,
            "smtp_port": smtp_config.port,
            "smtp_tls": smtp_config.tls,
            "smtp_starttls": smtp_config.starttls,
            "smtp_username": smtp_config.username
        });
        if let Some(services) = connected_services {
            account_config["connected_services"] = json!(services);
        }
        let mut secret_metadata = json!({
            "provider": request.provider_kind.as_str(),
            "account_id": request.account_id,
            "display_name": request.display_name,
            "external_account_id": request.external_account_id,
            "provider_account_config": account_config.clone()
        });
        if let Some(services) = connected_services {
            secret_metadata["connected_services"] = json!(services);
        }

        let secret_store = self.secret_store()?;
        let communication_store = self.communication_store()?;
        secret_store
            .upsert_secret_reference(
                &NewSecretReference::new(
                    &secret_ref,
                    request.secret_kind,
                    self.vault.store_kind(),
                    format!("IMAP credential for {}", request.display_name),
                )
                .metadata(secret_metadata.clone()),
            )
            .await?;
        self.vault
            .store_secret(
                &secret_ref,
                &request.password,
                SecretWriteContext {
                    entry_kind: "provider_credential",
                    account_id: &request.account_id,
                    purpose: ProviderAccountSecretPurpose::ImapPassword.as_str(),
                    secret_kind: request.secret_kind,
                    label: "IMAP password",
                    metadata: &secret_metadata,
                },
            )
            .await?;
        secret_store
            .upsert_secret_reference(
                &NewSecretReference::new(
                    &smtp_secret_ref,
                    request.secret_kind,
                    self.vault.store_kind(),
                    format!("SMTP credential for {}", request.display_name),
                )
                .metadata(secret_metadata.clone()),
            )
            .await?;
        communication_store
            .upsert_provider_account(
                &NewProviderAccount::new(
                    &request.account_id,
                    request.provider_kind,
                    &request.display_name,
                    &request.external_account_id,
                )
                .config(account_config),
            )
            .await?;
        communication_store
            .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
                &request.account_id,
                ProviderAccountSecretPurpose::ImapPassword,
                &secret_ref,
            ))
            .await?;
        self.vault
            .store_secret(
                &smtp_secret_ref,
                &request.password,
                SecretWriteContext {
                    entry_kind: "provider_credential",
                    account_id: &request.account_id,
                    purpose: ProviderAccountSecretPurpose::SmtpPassword.as_str(),
                    secret_kind: request.secret_kind,
                    label: "SMTP password",
                    metadata: &secret_metadata,
                },
            )
            .await?;
        communication_store
            .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
                &request.account_id,
                ProviderAccountSecretPurpose::SmtpPassword,
                &smtp_secret_ref,
            ))
            .await?;

        Ok(EmailAccountSetupResult {
            account_id: request.account_id,
            secret_ref,
            secret_kind: request.secret_kind,
            store_kind: self.vault.store_kind(),
        })
    }

    async fn exchange_authorization_code(
        &self,
        pending: &GmailOAuthPendingGrant,
        authorization_code: &str,
    ) -> Result<OAuthTokenResponse, EmailAccountSetupError> {
        let mut form = vec![
            ("grant_type", "authorization_code".to_owned()),
            ("code", authorization_code.trim().to_owned()),
            ("client_id", pending.request.client_id.clone()),
            ("redirect_uri", pending.request.redirect_uri.clone()),
            ("code_verifier", pending.code_verifier.clone()),
        ];
        if let Some(client_secret) = &pending.request.client_secret {
            form.push(("client_secret", client_secret.clone()));
        }

        Ok(self
            .http
            .post(&pending.request.token_endpoint)
            .form(&form)
            .send()
            .await?
            .error_for_status()?
            .json::<OAuthTokenResponse>()
            .await?)
    }

    async fn refresh_token(
        &self,
        bundle: &GmailOAuthTokenBundle,
    ) -> Result<OAuthTokenResponse, EmailAccountSetupError> {
        let mut form = vec![
            ("grant_type", "refresh_token".to_owned()),
            ("refresh_token", bundle.refresh_token.clone()),
            ("client_id", bundle.client_id.clone()),
        ];
        if let Some(client_secret) = &bundle.client_secret {
            form.push(("client_secret", client_secret.clone()));
        }

        Ok(self
            .http
            .post(&bundle.token_url)
            .form(&form)
            .send()
            .await?
            .error_for_status()?
            .json::<OAuthTokenResponse>()
            .await?)
    }

    fn communication_store(&self) -> Result<&CommunicationIngestionStore, EmailAccountSetupError> {
        self.communication_store
            .as_ref()
            .ok_or(EmailAccountSetupError::StoresNotConfigured)
    }

    fn secret_store(&self) -> Result<&SecretReferenceStore, EmailAccountSetupError> {
        self.secret_store
            .as_ref()
            .ok_or(EmailAccountSetupError::StoresNotConfigured)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailOAuthSetupRequest {
    pub account_id: String,
    pub display_name: String,
    pub external_account_id: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub redirect_uri: String,
    pub app_return_url: Option<String>,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub scopes: Vec<String>,
}

impl GmailOAuthSetupRequest {
    pub fn new(
        account_id: impl Into<String>,
        display_name: impl Into<String>,
        external_account_id: impl Into<String>,
        client_id: impl Into<String>,
        redirect_uri: impl Into<String>,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            display_name: display_name.into(),
            external_account_id: external_account_id.into(),
            client_id: client_id.into(),
            client_secret: None,
            redirect_uri: redirect_uri.into(),
            app_return_url: None,
            authorization_endpoint: DEFAULT_GOOGLE_AUTHORIZATION_ENDPOINT.to_owned(),
            token_endpoint: DEFAULT_GOOGLE_TOKEN_ENDPOINT.to_owned(),
            scopes: DEFAULT_GOOGLE_WORKSPACE_SCOPES
                .iter()
                .map(|scope| (*scope).to_owned())
                .collect(),
        }
    }

    pub fn external_account_id(mut self, external_account_id: impl Into<String>) -> Self {
        self.external_account_id = external_account_id.into();
        self
    }

    pub fn client_secret(mut self, client_secret: impl Into<String>) -> Self {
        self.client_secret = Some(client_secret.into());
        self
    }

    pub fn app_return_url(mut self, app_return_url: impl Into<String>) -> Self {
        self.app_return_url = Some(app_return_url.into());
        self
    }

    pub fn authorization_endpoint(mut self, authorization_endpoint: impl Into<String>) -> Self {
        self.authorization_endpoint = authorization_endpoint.into();
        self
    }

    pub fn token_endpoint(mut self, token_endpoint: impl Into<String>) -> Self {
        self.token_endpoint = token_endpoint.into();
        self
    }

    pub fn scopes(mut self, scopes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.scopes = scopes.into_iter().map(Into::into).collect();
        self
    }

    fn validate(&self) -> Result<(), EmailAccountSetupError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("client_id", &self.client_id)?;
        validate_non_empty("redirect_uri", &self.redirect_uri)?;
        validate_non_empty("authorization_endpoint", &self.authorization_endpoint)?;
        validate_non_empty("token_endpoint", &self.token_endpoint)?;
        if self.scopes.is_empty() {
            return Err(EmailAccountSetupError::InvalidRequest {
                field: "scopes",
                message: "must not be empty",
            });
        }
        for scope in &self.scopes {
            validate_non_empty("scope", scope)?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailOAuthPendingGrant {
    pub setup_id: String,
    pub account_id: String,
    pub authorization_url: String,
    pub state: String,
    pub code_verifier: String,
    pub request: GmailOAuthSetupRequest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImapAccountSetupRequest {
    pub account_id: String,
    pub provider_kind: EmailProviderKind,
    pub display_name: String,
    pub external_account_id: String,
    pub host: String,
    pub port: u16,
    pub tls: bool,
    pub mailbox: String,
    pub username: String,
    pub password: String,
    pub secret_kind: SecretKind,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub smtp_tls: Option<bool>,
    pub smtp_starttls: Option<bool>,
    pub smtp_username: Option<String>,
}

impl ImapAccountSetupRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        account_id: impl Into<String>,
        provider_kind: EmailProviderKind,
        display_name: impl Into<String>,
        external_account_id: impl Into<String>,
        host: impl Into<String>,
        port: u16,
        tls: bool,
        mailbox: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            provider_kind,
            display_name: display_name.into(),
            external_account_id: external_account_id.into(),
            host: host.into(),
            port,
            tls,
            mailbox: mailbox.into(),
            username: username.into(),
            password: password.into(),
            secret_kind: SecretKind::Password,
            smtp_host: None,
            smtp_port: None,
            smtp_tls: None,
            smtp_starttls: None,
            smtp_username: None,
        }
    }

    pub fn secret_kind(mut self, secret_kind: SecretKind) -> Self {
        self.secret_kind = secret_kind;
        self
    }

    pub fn smtp_host(mut self, smtp_host: impl Into<String>) -> Self {
        self.smtp_host = Some(smtp_host.into());
        self
    }

    pub fn smtp_port(mut self, smtp_port: u16) -> Self {
        self.smtp_port = Some(smtp_port);
        self
    }

    pub fn smtp_tls(mut self, smtp_tls: bool) -> Self {
        self.smtp_tls = Some(smtp_tls);
        self
    }

    pub fn smtp_starttls(mut self, smtp_starttls: bool) -> Self {
        self.smtp_starttls = Some(smtp_starttls);
        self
    }

    pub fn smtp_username(mut self, smtp_username: impl Into<String>) -> Self {
        self.smtp_username = Some(smtp_username.into());
        self
    }

    fn smtp_config(&self) -> ImapAccountSmtpConfig {
        let default_host = match self.provider_kind {
            EmailProviderKind::Icloud => "smtp.mail.me.com".to_owned(),
            _ => self.host.clone(),
        };
        ImapAccountSmtpConfig {
            host: self.smtp_host.clone().unwrap_or(default_host),
            port: self.smtp_port.unwrap_or(587),
            tls: self.smtp_tls.unwrap_or(true),
            starttls: self.smtp_starttls.unwrap_or(true),
            username: self
                .smtp_username
                .clone()
                .unwrap_or_else(|| self.external_account_id.clone()),
        }
    }

    fn validate(&self) -> Result<(), EmailAccountSetupError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("external_account_id", &self.external_account_id)?;
        validate_non_empty("host", &self.host)?;
        validate_non_empty("mailbox", &self.mailbox)?;
        validate_non_empty("username", &self.username)?;
        validate_non_empty("password", &self.password)?;
        if self.provider_kind == EmailProviderKind::Gmail {
            return Err(EmailAccountSetupError::InvalidRequest {
                field: "provider_kind",
                message: "must be icloud or imap",
            });
        }
        if self.port == 0 {
            return Err(EmailAccountSetupError::InvalidRequest {
                field: "port",
                message: "must be greater than zero",
            });
        }
        if !matches!(
            self.secret_kind,
            SecretKind::AppPassword | SecretKind::Password
        ) {
            return Err(EmailAccountSetupError::InvalidRequest {
                field: "secret_kind",
                message: "must be app_password or password",
            });
        }
        let smtp_config = self.smtp_config();
        validate_non_empty("smtp_host", &smtp_config.host)?;
        validate_non_empty("smtp_username", &smtp_config.username)?;
        if smtp_config.port == 0 {
            return Err(EmailAccountSetupError::InvalidRequest {
                field: "smtp_port",
                message: "must be greater than zero",
            });
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ImapAccountSmtpConfig {
    host: String,
    port: u16,
    tls: bool,
    starttls: bool,
    username: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EmailAccountSetupResult {
    pub account_id: String,
    pub secret_ref: String,
    pub secret_kind: SecretKind,
    pub store_kind: SecretStoreKind,
}

#[derive(Debug, Deserialize, Serialize)]
struct GmailOAuthTokenBundle {
    token_url: String,
    client_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_secret: Option<String>,
    access_token: String,
    refresh_token: String,
    expires_at: DateTime<Utc>,
    token_type: Option<String>,
    scope: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OAuthTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<i64>,
    token_type: Option<String>,
    scope: Option<String>,
}

fn http_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("reqwest client configuration must be valid")
}

fn expires_at(expires_in: Option<i64>) -> DateTime<Utc> {
    let seconds = expires_in.unwrap_or(3600).saturating_sub(60).max(60);
    Utc::now() + TimeDelta::seconds(seconds)
}

fn oauth_secret_ref(account_id: &str) -> String {
    format!("secret:provider-account:{account_id}:oauth_token")
}

fn imap_secret_ref(account_id: &str) -> String {
    format!("secret:provider-account:{account_id}:imap_password")
}

fn smtp_secret_ref(account_id: &str) -> String {
    format!("secret:provider-account:{account_id}:smtp_password")
}

fn email_provider_connected_services(
    provider_kind: EmailProviderKind,
) -> Option<&'static [&'static str]> {
    match provider_kind {
        EmailProviderKind::Gmail | EmailProviderKind::Icloud => {
            Some(&["mail", "calendar", "contacts"])
        }
        EmailProviderKind::Imap
        | EmailProviderKind::TelegramUser
        | EmailProviderKind::TelegramBot
        | EmailProviderKind::WhatsappWeb => None,
    }
}

fn vault_secret_reference(
    secret_ref: &str,
    secret_kind: SecretKind,
    store_kind: SecretStoreKind,
) -> SecretReference {
    let now = Utc::now();

    SecretReference {
        secret_ref: secret_ref.to_owned(),
        secret_kind,
        store_kind,
        label: "encrypted vault secret".to_owned(),
        metadata: json!({}),
        created_at: now,
        updated_at: now,
    }
}

fn random_url_token() -> String {
    let mut bytes = [0_u8; 32];
    aes_gcm::aead::OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn pkce_challenge(code_verifier: &str) -> String {
    let digest = Sha256::digest(code_verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), EmailAccountSetupError> {
    if value.trim().is_empty() {
        return Err(EmailAccountSetupError::InvalidRequest {
            field,
            message: "must not be empty",
        });
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum EmailAccountSetupError {
    #[error("invalid account setup request field {field}: {message}")]
    InvalidRequest {
        field: &'static str,
        message: &'static str,
    },

    #[error("provider response is missing required field: {field}")]
    MissingProviderField { field: &'static str },

    #[error("account setup stores are not configured")]
    StoresNotConfigured,

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    DatabaseVault(#[from] DatabaseEncryptedVaultError),

    #[error(transparent)]
    HostVault(#[from] HostVaultError),

    #[error(transparent)]
    SecretReference(#[from] SecretReferenceError),

    #[error(transparent)]
    Secret(#[from] SecretResolutionError),

    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),
}
