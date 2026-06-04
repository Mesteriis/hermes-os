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

use crate::communications::{
    CommunicationIngestionError, CommunicationIngestionStore, EmailProviderKind,
    NewProviderAccount, NewProviderAccountSecretBinding, ProviderAccountSecretPurpose,
};
use crate::secret_vault::{EncryptedSecretVault, EncryptedVaultError};
use crate::secrets::{
    NewSecretReference, ResolvedSecret, SecretKind, SecretReference, SecretReferenceError,
    SecretReferenceStore, SecretResolutionError, SecretResolver, SecretStoreKind,
};

const DEFAULT_GOOGLE_AUTHORIZATION_ENDPOINT: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const DEFAULT_GOOGLE_TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
const GMAIL_READONLY_SCOPE: &str = "https://www.googleapis.com/auth/gmail.readonly";

#[derive(Clone)]
pub struct EmailAccountSetupService {
    communication_store: Option<CommunicationIngestionStore>,
    secret_store: Option<SecretReferenceStore>,
    vault: EncryptedSecretVault,
    http: Client,
}

impl EmailAccountSetupService {
    pub fn new(
        communication_store: CommunicationIngestionStore,
        secret_store: SecretReferenceStore,
        vault: EncryptedSecretVault,
    ) -> Self {
        Self {
            communication_store: Some(communication_store),
            secret_store: Some(secret_store),
            vault,
            http: http_client(),
        }
    }

    pub fn new_for_vault_only(vault: EncryptedSecretVault) -> Self {
        Self {
            communication_store: None,
            secret_store: None,
            vault,
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
        self.vault
            .store_secret(&secret_ref, &serde_json::to_string(&token_bundle)?)
            .map_err(EmailAccountSetupError::Vault)?;

        let secret_store = self.secret_store()?;
        let communication_store = self.communication_store()?;
        secret_store
            .upsert_secret_reference(
                &NewSecretReference::new(
                    &secret_ref,
                    SecretKind::OauthToken,
                    SecretStoreKind::EncryptedVault,
                    format!(
                        "Gmail OAuth credential for {}",
                        pending.request.display_name
                    ),
                )
                .metadata(json!({
                    "provider": "gmail",
                    "account_id": pending.account_id
                })),
            )
            .await?;
        communication_store
            .upsert_provider_account(
                &NewProviderAccount::new(
                    &pending.account_id,
                    EmailProviderKind::Gmail,
                    &pending.request.display_name,
                    &pending.request.external_account_id,
                )
                .config(json!({
                    "auth": "oauth",
                    "api": "gmail",
                    "oauth_client_id": pending.request.client_id,
                    "history_stream_id": "gmail:history"
                })),
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
            store_kind: SecretStoreKind::EncryptedVault,
        })
    }

    pub async fn refresh_gmail_access_token(
        &self,
        secret_ref: &str,
    ) -> Result<ResolvedSecret, EmailAccountSetupError> {
        validate_non_empty("secret_ref", secret_ref)?;
        let reference = vault_secret_reference(secret_ref, SecretKind::OauthToken);
        let resolved = self.vault.resolve(&reference)?;
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
            .store_secret(secret_ref, &serde_json::to_string(&bundle)?)
            .map_err(EmailAccountSetupError::Vault)?;
        ResolvedSecret::new(bundle.access_token).map_err(EmailAccountSetupError::Secret)
    }

    pub async fn setup_imap_account(
        &self,
        request: ImapAccountSetupRequest,
    ) -> Result<EmailAccountSetupResult, EmailAccountSetupError> {
        request.validate()?;
        let secret_ref = imap_secret_ref(&request.account_id);
        self.vault
            .store_secret(&secret_ref, &request.password)
            .map_err(EmailAccountSetupError::Vault)?;

        let secret_store = self.secret_store()?;
        let communication_store = self.communication_store()?;
        secret_store
            .upsert_secret_reference(
                &NewSecretReference::new(
                    &secret_ref,
                    request.secret_kind,
                    SecretStoreKind::EncryptedVault,
                    format!("IMAP credential for {}", request.display_name),
                )
                .metadata(json!({
                    "provider": request.provider_kind.as_str(),
                    "account_id": request.account_id
                })),
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
                .config(json!({
                    "host": request.host,
                    "port": request.port,
                    "tls": request.tls,
                    "mailbox": request.mailbox,
                    "username": request.username
                })),
            )
            .await?;
        communication_store
            .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
                &request.account_id,
                ProviderAccountSecretPurpose::ImapPassword,
                &secret_ref,
            ))
            .await?;

        Ok(EmailAccountSetupResult {
            account_id: request.account_id,
            secret_ref,
            secret_kind: request.secret_kind,
            store_kind: SecretStoreKind::EncryptedVault,
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
            authorization_endpoint: DEFAULT_GOOGLE_AUTHORIZATION_ENDPOINT.to_owned(),
            token_endpoint: DEFAULT_GOOGLE_TOKEN_ENDPOINT.to_owned(),
            scopes: vec![GMAIL_READONLY_SCOPE.to_owned()],
        }
    }

    pub fn client_secret(mut self, client_secret: impl Into<String>) -> Self {
        self.client_secret = Some(client_secret.into());
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

    fn validate(&self) -> Result<(), EmailAccountSetupError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("external_account_id", &self.external_account_id)?;
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
        }
    }

    pub fn secret_kind(mut self, secret_kind: SecretKind) -> Self {
        self.secret_kind = secret_kind;
        self
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

        Ok(())
    }
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

fn vault_secret_reference(secret_ref: &str, secret_kind: SecretKind) -> SecretReference {
    let now = Utc::now();

    SecretReference {
        secret_ref: secret_ref.to_owned(),
        secret_kind,
        store_kind: SecretStoreKind::EncryptedVault,
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
    Vault(#[from] EncryptedVaultError),

    #[error(transparent)]
    SecretReference(#[from] SecretReferenceError),

    #[error(transparent)]
    Secret(#[from] SecretResolutionError),

    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),
}
