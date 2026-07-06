use serde_json::Value;

use crate::ai::control_center::AiProviderVaultRestore;
use crate::platform::secrets::{SecretKind, SecretStoreKind};
use crate::vault::HostVaultManifestEntry;

use super::metadata::{metadata_string, non_empty};

const SECRET_PURPOSE_API_KEY: &str = "api_key";

pub(super) struct RecoverableAiProviderSecret {
    pub(super) restore: AiProviderVaultRestore,
    pub(super) secret_kind: SecretKind,
    pub(super) store_kind: SecretStoreKind,
}

impl RecoverableAiProviderSecret {
    pub(super) fn from_manifest(entry: HostVaultManifestEntry) -> Option<Self> {
        if entry.entry_kind != "ai_provider" {
            return None;
        }
        if entry.purpose != SECRET_PURPOSE_API_KEY {
            return None;
        }
        let secret_kind = SecretKind::try_from(entry.secret_kind.as_str()).ok()?;
        if secret_kind != SecretKind::ApiToken {
            return None;
        }
        let store_kind = SecretStoreKind::try_from(entry.store_kind.as_str()).ok()?;
        if store_kind != SecretStoreKind::HostVault {
            return None;
        }

        let provider_id =
            non_empty(metadata_string(&entry.metadata, "provider_id")).unwrap_or(entry.account_id);
        let provider_kind = metadata_string(&entry.metadata, "provider_kind")
            .or_else(|| provider_kind_from_provider_id(&provider_id))
            .unwrap_or_else(|| "api".to_owned());
        if provider_kind != "api" {
            return None;
        }
        let provider_key = metadata_string(&entry.metadata, "provider_key")
            .or_else(|| provider_key_from_provider_id(&provider_id))
            .unwrap_or_else(|| provider_id.clone());
        let display_name = metadata_string(&entry.metadata, "display_name")
            .unwrap_or_else(|| fallback_display_name(&provider_key));
        let status =
            metadata_string(&entry.metadata, "status").unwrap_or_else(|| "ready".to_owned());
        let consent_state = metadata_string(&entry.metadata, "consent_state")
            .unwrap_or_else(|| "required".to_owned());
        let config = object_metadata(&entry.metadata, "config").unwrap_or_else(|| {
            object_metadata(&entry.metadata, "provider_config").unwrap_or_else(|| {
                if let Some(base_url) = metadata_string(&entry.metadata, "base_url") {
                    serde_json::json!({ "base_url": base_url })
                } else {
                    serde_json::json!({})
                }
            })
        });
        let capabilities = string_array_metadata(&entry.metadata, "capabilities");

        Some(Self {
            restore: AiProviderVaultRestore {
                provider_id,
                provider_kind,
                provider_key,
                display_name,
                status,
                consent_state,
                config,
                capabilities,
                secret_ref: entry.secret_ref,
                secret_purpose: entry.purpose,
                secret_metadata: entry.metadata,
                secret_label: entry.label,
            },
            secret_kind,
            store_kind,
        })
    }
}

fn object_metadata(metadata: &Value, key: &str) -> Option<Value> {
    metadata.get(key).filter(|value| value.is_object()).cloned()
}

fn string_array_metadata(metadata: &Value, key: &str) -> Vec<String> {
    metadata
        .get(key)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn provider_kind_from_provider_id(provider_id: &str) -> Option<String> {
    provider_id
        .strip_prefix("provider:")
        .and_then(|rest| rest.split(':').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn provider_key_from_provider_id(provider_id: &str) -> Option<String> {
    provider_id
        .strip_prefix("provider:api:")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn fallback_display_name(provider_key: &str) -> String {
    provider_key
        .split(['-', '_'])
        .filter(|part| !part.trim().is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
