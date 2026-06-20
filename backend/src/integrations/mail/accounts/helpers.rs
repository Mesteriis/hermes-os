use std::time::Duration;

use aes_gcm::aead::rand_core::RngCore;
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, TimeDelta, Utc};
use reqwest::Client;
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::platform::communications::EmailProviderKind;
use crate::platform::secrets::{SecretKind, SecretReference, SecretStoreKind};

pub(super) fn http_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("reqwest client configuration must be valid")
}

pub(super) fn expires_at(expires_in: Option<i64>) -> DateTime<Utc> {
    let seconds = expires_in.unwrap_or(3600).saturating_sub(60).max(60);
    Utc::now() + TimeDelta::seconds(seconds)
}

pub(super) fn oauth_secret_ref(account_id: &str) -> String {
    format!("secret:provider-account:{account_id}:oauth_token")
}

pub(super) fn imap_secret_ref(account_id: &str) -> String {
    format!("secret:provider-account:{account_id}:imap_password")
}

pub(super) fn smtp_secret_ref(account_id: &str) -> String {
    format!("secret:provider-account:{account_id}:smtp_password")
}

pub(super) fn email_provider_connected_services(
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

pub(super) fn vault_secret_reference(
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

pub(super) fn random_url_token() -> String {
    let mut bytes = [0_u8; 32];
    aes_gcm::aead::OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

pub(super) fn pkce_challenge(code_verifier: &str) -> String {
    let digest = Sha256::digest(code_verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
}
