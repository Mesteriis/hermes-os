use std::collections::BTreeSet;

use crate::error::{LegacyProviderRecoveryErrorV1, LegacyProviderRecoveryResultV1};
use crate::model::{
    GoogleOauthClientV1, LegacyProviderCandidateKindV1, LegacyProviderConfigurationV1,
    LegacyProviderRecoverySourceV1, RECOVERY_SCHEMA_REVISION, RecoveryCatalogCandidateV1,
    RecoveryCatalogConfigurationV1, RecoveryCatalogV1,
};
use crate::private_files::{is_generation, is_sha256};

const MAX_IDENTIFIER_BYTES: usize = 256;
const MAX_LABEL_BYTES: usize = 512;
const MAX_HOST_BYTES: usize = 253;
const MAX_OAUTH_VALUE_BYTES: usize = 4096;

pub(crate) fn validate_catalog(catalog: &RecoveryCatalogV1) -> LegacyProviderRecoveryResultV1<()> {
    if catalog.schema_revision != RECOVERY_SCHEMA_REVISION
        || !is_generation(&catalog.source_generation)
        || !catalog.counts.is_exact()
        || catalog.candidates.len() != 3
    {
        return Err(LegacyProviderRecoveryErrorV1::InvalidCatalog);
    }
    let mut kinds = BTreeSet::new();
    let mut handles = BTreeSet::new();
    for candidate in &catalog.candidates {
        if !kinds.insert(candidate.kind)
            || !handles.insert(candidate.source_account_digest_sha256.as_str())
            || !is_sha256(&candidate.source_account_digest_sha256)
        {
            return Err(LegacyProviderRecoveryErrorV1::InvalidCatalog);
        }
        require_bounded(&candidate.account_id, MAX_IDENTIFIER_BYTES)?;
        require_bounded(&candidate.display_name, MAX_LABEL_BYTES)?;
        require_bounded(&candidate.external_account_id, MAX_LABEL_BYTES)?;
        validate_candidate(candidate)?;
    }
    let expected = [
        LegacyProviderCandidateKindV1::Gmail,
        LegacyProviderCandidateKindV1::Icloud,
        LegacyProviderCandidateKindV1::TelegramUser,
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    if kinds != expected {
        return Err(LegacyProviderRecoveryErrorV1::InvalidCatalog);
    }
    Ok(())
}

pub(crate) fn recovery_source(
    candidate: &RecoveryCatalogCandidateV1,
    legacy: &LegacyProviderConfigurationV1,
    google: &GoogleOauthClientV1,
) -> LegacyProviderRecoveryResultV1<LegacyProviderRecoverySourceV1> {
    let handle = candidate.source_account_digest_sha256.clone();
    match (&candidate.kind, &candidate.configuration) {
        (
            LegacyProviderCandidateKindV1::Gmail,
            RecoveryCatalogConfigurationV1::Gmail { oauth_client_id },
        ) => {
            if oauth_client_id != &google.client_id {
                return Err(LegacyProviderRecoveryErrorV1::InvalidConfiguration);
            }
            let redirect = google
                .redirect_uris
                .iter()
                .find(|uri| is_loopback_redirect(uri))
                .cloned()
                .ok_or(LegacyProviderRecoveryErrorV1::InvalidConfiguration)?;
            Ok(LegacyProviderRecoverySourceV1::Gmail {
                handle,
                account_id: candidate.account_id.clone(),
                display_name: candidate.display_name.clone(),
                email: candidate.external_account_id.clone(),
                oauth_client_id: oauth_client_id.clone(),
                oauth_redirect_uri: redirect,
            })
        }
        (
            LegacyProviderCandidateKindV1::Icloud,
            RecoveryCatalogConfigurationV1::Icloud {
                imap_host,
                imap_port,
                tls,
                username,
            },
        ) if *tls => Ok(LegacyProviderRecoverySourceV1::Icloud {
            handle,
            account_id: candidate.account_id.clone(),
            display_name: candidate.display_name.clone(),
            email: candidate.external_account_id.clone(),
            imap_host: imap_host.clone(),
            imap_port: *imap_port,
            username: username.clone(),
        }),
        (
            LegacyProviderCandidateKindV1::TelegramUser,
            RecoveryCatalogConfigurationV1::TelegramUser,
        ) if legacy.telegram_api_id > 0 => Ok(LegacyProviderRecoverySourceV1::TelegramUser {
            handle,
            account_id: candidate.account_id.clone(),
            display_name: candidate.display_name.clone(),
            external_account_id: candidate.external_account_id.clone(),
            api_id: legacy.telegram_api_id,
        }),
        _ => Err(LegacyProviderRecoveryErrorV1::InvalidCatalog),
    }
}

fn validate_candidate(
    candidate: &RecoveryCatalogCandidateV1,
) -> LegacyProviderRecoveryResultV1<()> {
    let secret = &candidate.legacy_secret;
    require_bounded(&secret.secret_ref, MAX_LABEL_BYTES)?;
    require_bounded(&secret.secret_kind, 64)?;
    if secret.store_kind != "host_vault" {
        return Err(LegacyProviderRecoveryErrorV1::InvalidCatalog);
    }
    match (&candidate.kind, &candidate.configuration) {
        (
            LegacyProviderCandidateKindV1::Gmail,
            RecoveryCatalogConfigurationV1::Gmail { oauth_client_id },
        ) if secret.purpose == "oauth_token" => {
            require_ascii(oauth_client_id, MAX_OAUTH_VALUE_BYTES)
        }
        (
            LegacyProviderCandidateKindV1::Icloud,
            RecoveryCatalogConfigurationV1::Icloud {
                imap_host,
                imap_port,
                tls,
                username,
            },
        ) if secret.purpose == "imap_password" && *tls && *imap_port > 0 => {
            require_host(imap_host)?;
            require_bounded(username, MAX_LABEL_BYTES)
        }
        (
            LegacyProviderCandidateKindV1::TelegramUser,
            RecoveryCatalogConfigurationV1::TelegramUser,
        ) if secret.purpose == "telegram_session_key" => Ok(()),
        _ => Err(LegacyProviderRecoveryErrorV1::InvalidCatalog),
    }
}

pub(crate) fn validate_legacy_configuration(
    configuration: &LegacyProviderConfigurationV1,
) -> LegacyProviderRecoveryResultV1<()> {
    if configuration.schema_revision != RECOVERY_SCHEMA_REVISION
        || configuration.telegram_api_id <= 0
        || configuration.telegram_api_hash.is_empty()
        || configuration.telegram_api_hash.len() > MAX_OAUTH_VALUE_BYTES
        || !configuration.telegram_api_hash.is_ascii()
    {
        return Err(LegacyProviderRecoveryErrorV1::InvalidConfiguration);
    }
    Ok(())
}

pub(crate) fn validate_google_oauth(
    configuration: &GoogleOauthClientV1,
) -> LegacyProviderRecoveryResultV1<()> {
    if configuration.schema_revision != RECOVERY_SCHEMA_REVISION
        || require_ascii(&configuration.client_id, MAX_OAUTH_VALUE_BYTES).is_err()
        || configuration.redirect_uris.is_empty()
        || configuration.redirect_uris.len() > 16
        || configuration
            .redirect_uris
            .iter()
            .any(|value| require_ascii(value, MAX_OAUTH_VALUE_BYTES).is_err())
    {
        return Err(LegacyProviderRecoveryErrorV1::InvalidConfiguration);
    }
    Ok(())
}

fn require_host(value: &str) -> LegacyProviderRecoveryResultV1<()> {
    if value.is_empty()
        || value.len() > MAX_HOST_BYTES
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
    {
        return Err(LegacyProviderRecoveryErrorV1::InvalidCatalog);
    }
    Ok(())
}

fn require_ascii(value: &str, maximum: usize) -> LegacyProviderRecoveryResultV1<()> {
    if value.is_empty() || value.len() > maximum || !value.is_ascii() {
        return Err(LegacyProviderRecoveryErrorV1::InvalidConfiguration);
    }
    Ok(())
}

fn require_bounded(value: &str, maximum: usize) -> LegacyProviderRecoveryResultV1<()> {
    if value.trim().is_empty() || value.len() > maximum || value.contains(['\0', '\r', '\n']) {
        return Err(LegacyProviderRecoveryErrorV1::InvalidCatalog);
    }
    Ok(())
}

fn is_loopback_redirect(value: &str) -> bool {
    (value.starts_with("http://127.0.0.1") || value.starts_with("http://localhost"))
        && !value.contains(['\0', '\r', '\n', '#'])
}
