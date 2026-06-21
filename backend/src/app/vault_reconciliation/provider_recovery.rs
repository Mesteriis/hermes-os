use serde_json::Value;

use crate::domains::communications::core::{EmailProviderKind, ProviderAccountSecretPurpose};
use crate::platform::secrets::{SecretKind, SecretStoreKind};
use crate::vault::HostVaultManifestEntry;

use super::metadata::{
    fallback_display_name, fallback_provider_account_config, metadata_string, non_empty,
};

pub(super) struct RecoverableProviderSecret {
    pub(super) account_id: String,
    pub(super) provider_kind: EmailProviderKind,
    pub(super) display_name: String,
    pub(super) external_account_id: String,
    pub(super) secret_ref: String,
    pub(super) secret_kind: SecretKind,
    pub(super) store_kind: SecretStoreKind,
    pub(super) secret_purpose: ProviderAccountSecretPurpose,
    pub(super) label: String,
    pub(super) secret_metadata: Value,
    pub(super) provider_account_config: Value,
}

impl RecoverableProviderSecret {
    pub(super) fn from_manifest(entry: HostVaultManifestEntry) -> Option<Self> {
        if entry.entry_kind != "provider_credential" {
            return None;
        }
        let provider = metadata_string(&entry.metadata, "provider")?;
        let provider_kind = EmailProviderKind::try_from(provider.as_str()).ok()?;
        if !matches!(
            provider_kind,
            EmailProviderKind::Gmail | EmailProviderKind::Icloud | EmailProviderKind::Imap
        ) {
            return None;
        }

        let secret_kind = SecretKind::try_from(entry.secret_kind.as_str()).ok()?;
        let store_kind = SecretStoreKind::try_from(entry.store_kind.as_str()).ok()?;
        let secret_purpose = ProviderAccountSecretPurpose::try_from(entry.purpose.as_str()).ok()?;
        if !secret_purpose.accepts_secret_kind(secret_kind) {
            return None;
        }

        let account_id =
            non_empty(metadata_string(&entry.metadata, "account_id")).unwrap_or(entry.account_id);
        let display_name = non_empty(metadata_string(&entry.metadata, "display_name"))
            .unwrap_or_else(|| fallback_display_name(provider_kind, &entry.label, &account_id));
        let external_account_id =
            non_empty(metadata_string(&entry.metadata, "external_account_id"))
                .unwrap_or_else(|| account_id.clone());
        let provider_account_config = entry
            .metadata
            .get("provider_account_config")
            .filter(|value| value.is_object())
            .cloned()
            .unwrap_or_else(|| {
                fallback_provider_account_config(
                    provider_kind,
                    &entry.metadata,
                    &external_account_id,
                )
            });

        Some(Self {
            account_id,
            provider_kind,
            display_name,
            external_account_id,
            secret_ref: entry.secret_ref,
            secret_kind,
            store_kind,
            secret_purpose,
            label: entry.label,
            secret_metadata: entry.metadata,
            provider_account_config,
        })
    }
}
