use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde::Serialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::platform::communications::{
    NewProviderAccountSecretBinding, ProviderAccount, ProviderAccountCommandPort,
    ProviderAccountSecretPurpose, ProviderSecretBindingCommandPort,
};
use crate::platform::events::bus::yandex_telemost_event_types;
use crate::platform::events::{EventBus, EventLogQuery, EventStore, NewEventEnvelope};
use crate::platform::secrets::{
    NewSecretReference, SecretKind, SecretReferenceStore, SecretStoreKind,
};
use crate::platform::settings::ApplicationSettingsStore;
use crate::vault::{HostVault, SecretEntryContext};

use super::{
    TelemostCohost, YANDEX_TELEMOST_API_BASE_URL, YANDEX_TELEMOST_LIVE_RUNTIME_KIND,
    YANDEX_TELEMOST_PROVIDER_KIND_STR, YANDEX_TELEMOST_RUNTIME_KIND, YandexTelemostAccount,
    YandexTelemostAccountListResponse, YandexTelemostAccountSetupRequest,
    YandexTelemostAccountSetupResponse, YandexTelemostCapabilityState, YandexTelemostCohostPage,
    YandexTelemostConference, YandexTelemostConferencePatchRequest,
    YandexTelemostConferenceRequest, YandexTelemostError, YandexTelemostRetentionCleanupItem,
    YandexTelemostRetentionCleanupRequest, YandexTelemostRetentionCleanupResponse,
    YandexTelemostRuntimeStatus, sanitize_yandex_telemost_payload, telemost_provider_kind,
    validate_api_base_url, validate_json_object, validate_required, yandex_telemost_capabilities,
    yandex_telemost_default_config, yandex_telemost_oauth_token_secret_ref,
};

const YANDEX_TELEMOST_RECORDING_RETENTION_DAYS_SETTING_KEY: &str =
    "privacy.yandex_telemost_recording_retention_days";
const YANDEX_TELEMOST_SPEAKER_TIMELINE_RETENTION_DAYS_SETTING_KEY: &str =
    "privacy.yandex_telemost_speaker_timeline_retention_days";

#[derive(Clone)]
pub struct YandexTelemostHttpClient {
    http: reqwest::Client,
    base_url: String,
}

impl YandexTelemostHttpClient {
    pub fn new(base_url: Option<&str>) -> Result<Self, YandexTelemostError> {
        Ok(Self {
            http: reqwest::Client::new(),
            base_url: validate_api_base_url(base_url)?,
        })
    }

    pub async fn create_conference(
        &self,
        oauth_token: &str,
        request: &YandexTelemostConferenceRequest,
    ) -> Result<YandexTelemostConference, YandexTelemostError> {
        validate_conference_request(request)?;
        let payload = provider_payload(request)?;
        let response = self
            .http
            .post(format!("{}/conferences", self.base_url))
            .header(AUTHORIZATION, format!("OAuth {}", oauth_token.trim()))
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;
        Ok(response.json::<YandexTelemostConference>().await?)
    }

    pub async fn get_conference(
        &self,
        oauth_token: &str,
        conference_id: &str,
    ) -> Result<YandexTelemostConference, YandexTelemostError> {
        let conference_id = validate_required("conference_id", conference_id)?;
        let response = self
            .http
            .get(format!("{}/conferences/{}", self.base_url, conference_id))
            .header(AUTHORIZATION, format!("OAuth {}", oauth_token.trim()))
            .header(ACCEPT, "application/json")
            .send()
            .await?
            .error_for_status()?;
        Ok(response.json::<YandexTelemostConference>().await?)
    }

    pub async fn update_conference(
        &self,
        oauth_token: &str,
        conference_id: &str,
        request: &YandexTelemostConferencePatchRequest,
    ) -> Result<YandexTelemostConference, YandexTelemostError> {
        let conference_id = validate_required("conference_id", conference_id)?;
        validate_conference_patch_request(request)?;
        let payload = provider_payload(request)?;
        let response = self
            .http
            .patch(format!("{}/conferences/{}", self.base_url, conference_id))
            .header(AUTHORIZATION, format!("OAuth {}", oauth_token.trim()))
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;
        Ok(response.json::<YandexTelemostConference>().await?)
    }

    pub async fn list_cohosts(
        &self,
        oauth_token: &str,
        conference_id: &str,
        offset: Option<u32>,
        limit: Option<u16>,
    ) -> Result<YandexTelemostCohostPage, YandexTelemostError> {
        let conference_id = validate_required("conference_id", conference_id)?;
        let mut request = self
            .http
            .get(format!(
                "{}/conferences/{}/cohosts",
                self.base_url, conference_id
            ))
            .header(AUTHORIZATION, format!("OAuth {}", oauth_token.trim()))
            .header(ACCEPT, "application/json");
        if let Some(offset) = offset {
            request = request.query(&[("offset", offset)]);
        }
        if let Some(limit) = limit {
            request = request.query(&[("limit", limit)]);
        }
        let response = request.send().await?.error_for_status()?;
        Ok(response.json::<YandexTelemostCohostPage>().await?)
    }
}

#[derive(Clone)]
pub struct YandexTelemostStore {
    provider_account_store: Arc<dyn ProviderAccountCommandPort>,
    provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
    event_store: EventStore,
    event_bus: EventBus,
}

impl YandexTelemostStore {
    pub fn new(
        provider_account_store: Arc<dyn ProviderAccountCommandPort>,
        provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
        event_store: EventStore,
        event_bus: EventBus,
    ) -> Self {
        Self {
            provider_account_store,
            provider_secret_binding_store,
            event_store,
            event_bus,
        }
    }

    pub async fn setup_account(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &YandexTelemostAccountSetupRequest,
    ) -> Result<YandexTelemostAccountSetupResponse, YandexTelemostError> {
        validate_account_setup_request(request)?;
        let account_id = validate_required("account_id", &request.account_id)?;
        let display_name = validate_required("display_name", &request.display_name)?;
        let external_account_id =
            validate_required("external_account_id", &request.external_account_id)?;
        let api_base_url = validate_api_base_url(request.api_base_url.as_deref())?;
        let token_secret_ref = self
            .store_or_register_oauth_token(secret_store, vault, &account_id, request)
            .await?;
        let config = merge_metadata(
            yandex_telemost_default_config(Some(&token_secret_ref), &api_base_url),
            &request.metadata,
        );
        let account = self
            .provider_account_store
            .upsert_runtime_account(
                account_id.clone(),
                telemost_provider_kind().as_str().to_owned(),
                display_name,
                external_account_id,
                config,
            )
            .await?;
        self.provider_secret_binding_store
            .bind(&NewProviderAccountSecretBinding::new(
                &account_id,
                ProviderAccountSecretPurpose::YandexTelemostOauthToken,
                &token_secret_ref,
            ))
            .await?;
        self.publish_account_configured_event(&account, &token_secret_ref)
            .await?;
        self.publish_authorization_event(&account, &token_secret_ref)
            .await?;
        Ok(YandexTelemostAccountSetupResponse {
            account: account.into(),
        })
    }

    pub async fn list_accounts(
        &self,
        include_removed: bool,
    ) -> Result<YandexTelemostAccountListResponse, YandexTelemostError> {
        let mut items = self
            .provider_account_store
            .list()
            .await?
            .into_iter()
            .filter(|account| account.provider_kind.is_yandex_telemost())
            .map(YandexTelemostAccount::from)
            .filter(|account| include_removed || account.lifecycle_state != "removed")
            .collect::<Vec<_>>();
        items.sort_by(|left, right| left.display_name.cmp(&right.display_name));
        Ok(YandexTelemostAccountListResponse { items })
    }

    pub async fn runtime_status(
        &self,
        account_id: &str,
    ) -> Result<YandexTelemostRuntimeStatus, YandexTelemostError> {
        let account = self.telemost_account(account_id).await?;
        let authorized = self
            .provider_secret_binding_store
            .get_for_account(
                &account.account_id,
                ProviderAccountSecretPurpose::YandexTelemostOauthToken,
            )
            .await?
            .is_some();
        let status = runtime_status_from_account(account.into(), authorized);
        self.publish_runtime_status_event(&status).await?;
        Ok(status)
    }

    pub async fn cleanup_retention(
        &self,
        account_id: &str,
        request: &YandexTelemostRetentionCleanupRequest,
    ) -> Result<YandexTelemostRetentionCleanupResponse, YandexTelemostError> {
        let account = self.telemost_account(account_id).await?;
        request.validate()?;
        let checked_at = Utc::now();
        let mut response = YandexTelemostRetentionCleanupResponse {
            account_id: account.account_id.clone(),
            checked_at,
            audio_files_removed: 0,
            speaker_hint_files_removed: 0,
            bundles_cleaned: 0,
            items: Vec::new(),
        };
        let events = self
            .event_store
            .list_matching(
                EventLogQuery::default()
                    .event_type(yandex_telemost_event_types::LOCAL_RECORDING_COMPLETED)
                    .limit(1000),
            )
            .await?;

        for event in events {
            if response.items.len() >= request.limit() as usize {
                break;
            }
            let Some(candidate) = retention_cleanup_candidate_from_event(
                &event.event.payload,
                event.event.occurred_at,
            ) else {
                continue;
            };
            if candidate.account_id != account.account_id {
                continue;
            }
            let policy = self
                .resolved_local_recording_retention_policy(
                    &candidate.manifest_path,
                    event.event.occurred_at,
                )
                .await?;
            let now = Utc::now();
            let audio_expired = request.remove_audio
                && policy
                    .audio_expires_at
                    .is_some_and(|expires| expires <= now);
            let speaker_expired = request.remove_speaker_hints
                && policy
                    .speaker_hints_expires_at
                    .is_some_and(|expires| expires <= now);
            if !audio_expired && !speaker_expired {
                continue;
            }

            let removed_at = Utc::now();
            let audio_removed = if audio_expired {
                remove_local_file_if_exists(&candidate.audio_path)?
            } else {
                false
            };
            let speaker_jsonl_removed = if speaker_expired {
                remove_local_file_if_exists(&candidate.speaker_jsonl_path)?
            } else {
                false
            };
            let speaker_txt_removed = if speaker_expired {
                remove_local_file_if_exists(&candidate.speaker_txt_path)?
            } else {
                false
            };
            let speaker_hints_removed = if speaker_expired {
                remove_local_file_if_exists(&candidate.bundle_root.join("speaker-hints.jsonl"))?
            } else {
                false
            };
            if !audio_removed
                && !speaker_jsonl_removed
                && !speaker_txt_removed
                && !speaker_hints_removed
            {
                continue;
            }

            record_retention_cleanup_in_manifest(
                &candidate.manifest_path,
                &policy,
                audio_removed,
                speaker_jsonl_removed || speaker_txt_removed || speaker_hints_removed,
                removed_at,
            )?;
            if audio_removed {
                response.audio_files_removed += 1;
            }
            response.speaker_hint_files_removed += [
                speaker_jsonl_removed,
                speaker_txt_removed,
                speaker_hints_removed,
            ]
            .into_iter()
            .filter(|removed| *removed)
            .count();
            response.bundles_cleaned += 1;
            response.items.push(YandexTelemostRetentionCleanupItem {
                bundle_id: candidate.bundle_id,
                conference_id: candidate.conference_id,
                bundle_root: candidate.bundle_root.to_string_lossy().into_owned(),
                audio_removed,
                speaker_jsonl_removed,
                speaker_txt_removed,
                speaker_hints_removed,
                audio_expires_at: policy.audio_expires_at,
                speaker_hints_expires_at: policy.speaker_hints_expires_at,
                removed_at,
            });
        }

        self.publish_retention_cleanup_completed_event(&response)
            .await?;
        Ok(response)
    }

    pub async fn create_conference(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        account_id: &str,
        request: &YandexTelemostConferenceRequest,
    ) -> Result<YandexTelemostConference, YandexTelemostError> {
        let account = self.telemost_account(account_id).await?;
        let token = self
            .resolve_oauth_token(secret_store, vault, &account.account_id)
            .await?;
        let conference = client_for_account(&account)?
            .create_conference(&token, request)
            .await?;
        self.publish_conference_event(
            yandex_telemost_event_types::CONFERENCE_CREATED,
            &account,
            &conference,
            request.metadata.clone(),
            "create_conference",
        )
        .await?;
        Ok(conference)
    }

    pub async fn get_conference(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        account_id: &str,
        conference_id: &str,
    ) -> Result<YandexTelemostConference, YandexTelemostError> {
        let account = self.telemost_account(account_id).await?;
        let token = self
            .resolve_oauth_token(secret_store, vault, &account.account_id)
            .await?;
        let conference = client_for_account(&account)?
            .get_conference(&token, conference_id)
            .await?;
        self.publish_conference_event(
            yandex_telemost_event_types::CONFERENCE_OBSERVED,
            &account,
            &conference,
            json!({ "source": "explicit_read" }),
            "get_conference",
        )
        .await?;
        Ok(conference)
    }

    pub async fn update_conference(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        account_id: &str,
        conference_id: &str,
        request: &YandexTelemostConferencePatchRequest,
    ) -> Result<YandexTelemostConference, YandexTelemostError> {
        let account = self.telemost_account(account_id).await?;
        let token = self
            .resolve_oauth_token(secret_store, vault, &account.account_id)
            .await?;
        let conference = client_for_account(&account)?
            .update_conference(&token, conference_id, request)
            .await?;
        self.publish_conference_event(
            yandex_telemost_event_types::CONFERENCE_UPDATED,
            &account,
            &conference,
            request.metadata.clone(),
            "update_conference",
        )
        .await?;
        Ok(conference)
    }

    pub async fn list_cohosts(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        account_id: &str,
        conference_id: &str,
        offset: Option<u32>,
        limit: Option<u16>,
    ) -> Result<YandexTelemostCohostPage, YandexTelemostError> {
        let account = self.telemost_account(account_id).await?;
        let token = self
            .resolve_oauth_token(secret_store, vault, &account.account_id)
            .await?;
        let page = client_for_account(&account)?
            .list_cohosts(&token, conference_id, offset, limit)
            .await?;
        self.publish_cohosts_observed_event(&account, conference_id, &page)
            .await?;
        Ok(page)
    }

    async fn store_or_register_oauth_token(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        account_id: &str,
        request: &YandexTelemostAccountSetupRequest,
    ) -> Result<String, YandexTelemostError> {
        if let Some(token) = request
            .oauth_token
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            let secret_ref = request
                .oauth_token_ref
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
                .unwrap_or_else(|| yandex_telemost_oauth_token_secret_ref(account_id));
            store_oauth_token(
                secret_store,
                vault,
                account_id,
                &secret_ref,
                token,
                &request.metadata,
            )
            .await?;
            return Ok(secret_ref);
        }
        let secret_ref = request
            .oauth_token_ref
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| YandexTelemostError::InvalidRequest("oauth_token or oauth_token_ref must be provided for live Yandex Telemost API calls".to_owned()))?;
        let reference = secret_store
            .secret_reference(secret_ref)
            .await?
            .ok_or_else(|| {
                YandexTelemostError::InvalidRequest(format!(
                    "Yandex Telemost token secret reference `{secret_ref}` was not found"
                ))
            })?;
        if reference.secret_kind != SecretKind::OauthToken
            || reference.store_kind != SecretStoreKind::HostVault
        {
            return Err(YandexTelemostError::InvalidRequest(format!(
                "Yandex Telemost token secret reference `{secret_ref}` must be oauth_token in host_vault"
            )));
        }
        let _ = vault.read_secret(secret_ref)?;
        Ok(secret_ref.to_owned())
    }

    async fn telemost_account(
        &self,
        account_id: &str,
    ) -> Result<ProviderAccount, YandexTelemostError> {
        let account_id = validate_required("account_id", account_id)?;
        let account = self
            .provider_account_store
            .get(&account_id)
            .await?
            .ok_or_else(|| {
                YandexTelemostError::InvalidRequest(format!(
                    "Yandex Telemost account `{account_id}` was not found"
                ))
            })?;
        if !account.provider_kind.is_yandex_telemost() {
            return Err(YandexTelemostError::InvalidRequest(format!(
                "provider account `{account_id}` is not a Yandex Telemost account"
            )));
        }
        Ok(account)
    }

    async fn resolve_oauth_token(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        account_id: &str,
    ) -> Result<String, YandexTelemostError> {
        let binding = self
            .provider_secret_binding_store
            .get_for_account(
                account_id,
                ProviderAccountSecretPurpose::YandexTelemostOauthToken,
            )
            .await?
            .ok_or_else(|| {
                YandexTelemostError::InvalidRequest(format!(
                    "Yandex Telemost account `{account_id}` has no oauth token binding"
                ))
            })?;
        let reference = secret_store
            .secret_reference(&binding.secret_ref)
            .await?
            .ok_or_else(|| {
                YandexTelemostError::InvalidRequest(format!(
                    "Yandex Telemost token secret reference `{}` was not found",
                    binding.secret_ref
                ))
            })?;
        if reference.secret_kind != SecretKind::OauthToken
            || reference.store_kind != SecretStoreKind::HostVault
        {
            return Err(YandexTelemostError::InvalidRequest(format!(
                "Yandex Telemost token secret reference `{}` must be oauth_token in host_vault",
                reference.secret_ref
            )));
        }
        Ok(vault.read_secret(&reference.secret_ref)?)
    }

    async fn publish_account_configured_event(
        &self,
        account: &ProviderAccount,
        token_secret_ref: &str,
    ) -> Result<(), YandexTelemostError> {
        let event = NewEventEnvelope::builder(
            format!("yandex-telemost-account-{}-{}", account.account_id, Uuid::new_v4()),
            yandex_telemost_event_types::ACCOUNT_CONFIGURED,
            Utc::now(),
            json!({ "source_code": "integration.yandex_telemost", "provider": YANDEX_TELEMOST_PROVIDER_KIND_STR }),
            json!({ "kind": "provider_account", "entity_id": account.account_id }),
        )
        .payload(json!({
            "account_id": account.account_id,
            "provider_kind": account.provider_kind.as_str(),
            "token_secret_ref": token_secret_ref,
            "secret_material": "excluded"
        }))
        .provenance(json!({ "origin": "local_setup" }))
        .correlation_id(format!("yandex-telemost:{}", account.account_id))
        .build()?;
        self.append_and_broadcast(&event).await
    }

    async fn publish_authorization_event(
        &self,
        account: &ProviderAccount,
        token_secret_ref: &str,
    ) -> Result<(), YandexTelemostError> {
        let event = NewEventEnvelope::builder(
            format!("yandex-telemost-authorization-{}-{}", account.account_id, Uuid::new_v4()),
            yandex_telemost_event_types::AUTHORIZATION_COMPLETED,
            Utc::now(),
            json!({ "source_code": "integration.yandex_telemost", "provider": YANDEX_TELEMOST_PROVIDER_KIND_STR }),
            json!({ "kind": "provider_account", "entity_id": account.account_id }),
        )
        .payload(json!({
            "account_id": account.account_id,
            "provider_kind": account.provider_kind.as_str(),
            "token_secret_ref": token_secret_ref,
            "secret_material": "excluded"
        }))
        .provenance(json!({ "origin": "local_setup" }))
        .correlation_id(format!("yandex-telemost:{}", account.account_id))
        .build()?;
        self.append_and_broadcast(&event).await
    }

    async fn publish_runtime_status_event(
        &self,
        status: &YandexTelemostRuntimeStatus,
    ) -> Result<(), YandexTelemostError> {
        let event = NewEventEnvelope::builder(
            format!("yandex-telemost-runtime-{}-{}", status.account_id, Uuid::new_v4()),
            yandex_telemost_event_types::RUNTIME_STATUS_CHANGED,
            Utc::now(),
            json!({ "source_code": "integration.yandex_telemost", "provider": YANDEX_TELEMOST_PROVIDER_KIND_STR }),
            json!({ "kind": "provider_account", "entity_id": status.account_id }),
        )
        .payload(sanitize_yandex_telemost_payload(json!({ "status": status })))
        .provenance(json!({ "origin": "runtime_status_check" }))
        .correlation_id(format!("yandex-telemost:{}", status.account_id))
        .build()?;
        self.append_and_broadcast(&event).await
    }

    async fn publish_conference_event(
        &self,
        event_type: &str,
        account: &ProviderAccount,
        conference: &YandexTelemostConference,
        metadata: Value,
        operation: &'static str,
    ) -> Result<(), YandexTelemostError> {
        let event = NewEventEnvelope::builder(
            format!(
                "yandex-telemost-conference-{}-{}-{}",
                account.account_id,
                conference.id,
                Uuid::new_v4()
            ),
            event_type,
            Utc::now(),
            json!({ "source_code": "integration.yandex_telemost", "provider": YANDEX_TELEMOST_PROVIDER_KIND_STR }),
            json!({ "kind": "telemost_conference", "entity_id": conference.id }),
        )
        .payload(sanitize_yandex_telemost_payload(json!({
            "account_id": account.account_id,
            "provider_kind": account.provider_kind.as_str(),
            "conference": conference,
            "metadata": metadata,
            "operation": operation,
        })))
        .provenance(json!({
            "origin": "yandex_telemost_api",
            "api_base_url": account.config.get("api_base_url").and_then(Value::as_str).unwrap_or(YANDEX_TELEMOST_API_BASE_URL)
        }))
        .correlation_id(format!("yandex-telemost:{}:{}", account.account_id, conference.id))
        .build()?;
        self.append_and_broadcast(&event).await
    }

    async fn publish_cohosts_observed_event(
        &self,
        account: &ProviderAccount,
        conference_id: &str,
        page: &YandexTelemostCohostPage,
    ) -> Result<(), YandexTelemostError> {
        let event = NewEventEnvelope::builder(
            format!("yandex-telemost-cohosts-{}-{}-{}", account.account_id, conference_id, Uuid::new_v4()),
            yandex_telemost_event_types::COHOSTS_OBSERVED,
            Utc::now(),
            json!({ "source_code": "integration.yandex_telemost", "provider": YANDEX_TELEMOST_PROVIDER_KIND_STR }),
            json!({ "kind": "telemost_conference", "entity_id": conference_id }),
        )
        .payload(sanitize_yandex_telemost_payload(json!({
            "account_id": account.account_id,
            "conference_id": conference_id,
            "cohosts": page.cohosts,
        })))
        .provenance(json!({ "origin": "yandex_telemost_api" }))
        .correlation_id(format!("yandex-telemost:{}:{}", account.account_id, conference_id))
        .build()?;
        self.append_and_broadcast(&event).await
    }

    async fn publish_retention_cleanup_completed_event(
        &self,
        result: &YandexTelemostRetentionCleanupResponse,
    ) -> Result<(), YandexTelemostError> {
        let event = NewEventEnvelope::builder(
            format!(
                "yandex-telemost-retention-cleanup-{}-{}",
                result.account_id,
                Uuid::new_v4()
            ),
            yandex_telemost_event_types::RETENTION_CLEANUP_COMPLETED,
            result.checked_at,
            json!({ "source_code": "integration.yandex_telemost", "provider": YANDEX_TELEMOST_PROVIDER_KIND_STR }),
            json!({ "kind": "provider_account", "entity_id": result.account_id }),
        )
        .payload(serde_json::to_value(result)?)
        .provenance(json!({ "origin": "local_retention_cleanup" }))
        .correlation_id(format!("yandex-telemost-retention:{}", result.account_id))
        .build()?;
        self.append_and_broadcast(&event).await
    }

    async fn resolved_local_recording_retention_policy(
        &self,
        manifest_path: &Path,
        observed_at: DateTime<Utc>,
    ) -> Result<LocalRecordingRetentionPolicy, YandexTelemostError> {
        if let Ok(content) = fs::read_to_string(manifest_path)
            && let Ok(manifest) = serde_json::from_str::<Value>(&content)
            && let Some(policy) = local_recording_retention_policy_from_manifest(&manifest)
        {
            return Ok(policy);
        }

        let settings = ApplicationSettingsStore::new(self.event_store.pool().clone());
        settings.repair_declared_settings().await?;
        let recording_retention_days = settings
            .setting(YANDEX_TELEMOST_RECORDING_RETENTION_DAYS_SETTING_KEY)
            .await?
            .and_then(|setting| setting.value.as_i64())
            .unwrap_or(0)
            .max(0);
        let speaker_hint_retention_days = settings
            .setting(YANDEX_TELEMOST_SPEAKER_TIMELINE_RETENTION_DAYS_SETTING_KEY)
            .await?
            .and_then(|setting| setting.value.as_i64())
            .unwrap_or(0)
            .max(0);
        Ok(local_recording_retention_policy_from_days(
            observed_at,
            recording_retention_days,
            speaker_hint_retention_days,
        ))
    }

    async fn append_and_broadcast(
        &self,
        event: &NewEventEnvelope,
    ) -> Result<(), YandexTelemostError> {
        if self
            .event_store
            .append_for_dispatch_idempotent(event)
            .await?
            .is_some()
        {
            self.event_bus.broadcast(event.clone());
        }
        Ok(())
    }
}

async fn store_oauth_token(
    secret_store: &SecretReferenceStore,
    vault: &HostVault,
    account_id: &str,
    secret_ref: &str,
    token: &str,
    metadata: &Value,
) -> Result<(), YandexTelemostError> {
    validate_required("oauth_token", token)?;
    validate_json_object("metadata", metadata)?;
    let reference = NewSecretReference::new(
        secret_ref,
        SecretKind::OauthToken,
        SecretStoreKind::HostVault,
        "Yandex Telemost OAuth token",
    )
    .metadata(json!({
        "provider": YANDEX_TELEMOST_PROVIDER_KIND_STR,
        "account_id": account_id,
        "secret_material": "excluded",
        "metadata": sanitize_yandex_telemost_payload(metadata.clone()),
    }));
    secret_store.upsert_secret_reference(&reference).await?;
    let vault_metadata = json!({
        "provider": YANDEX_TELEMOST_PROVIDER_KIND_STR,
        "provider_kind": YANDEX_TELEMOST_PROVIDER_KIND_STR,
        "account_id": account_id,
        "secret_purpose": ProviderAccountSecretPurpose::YandexTelemostOauthToken.as_str(),
        "metadata": sanitize_yandex_telemost_payload(metadata.clone()),
    });
    vault.store_secret(
        secret_ref,
        token.trim(),
        SecretEntryContext {
            entry_kind: "provider_credential",
            account_id,
            purpose: ProviderAccountSecretPurpose::YandexTelemostOauthToken.as_str(),
            secret_kind: SecretKind::OauthToken.as_str(),
            label: "Yandex Telemost OAuth token",
            metadata: &vault_metadata,
        },
    )?;
    Ok(())
}

fn merge_metadata(mut config: Value, metadata: &Value) -> Value {
    if let Some(object) = config.as_object_mut() {
        object.insert(
            "metadata".to_owned(),
            sanitize_yandex_telemost_payload(metadata.clone()),
        );
    }
    config
}

fn runtime_status_from_account(
    account: YandexTelemostAccount,
    authorized: bool,
) -> YandexTelemostRuntimeStatus {
    let blockers = if authorized {
        vec![]
    } else {
        vec!["yandex_telemost_oauth_token_missing".to_owned()]
    };
    let mut capabilities = yandex_telemost_capabilities(authorized);
    if !authorized {
        capabilities.push(YandexTelemostCapabilityState {
            capability: "telemost.oauth_token.required".to_owned(),
            status: "blocked".to_owned(),
            source: "provider_secret_binding_store".to_owned(),
            confidence: 0.95,
            evidence: json!({
                "missing_secret_purpose": ProviderAccountSecretPurpose::YandexTelemostOauthToken.as_str()
            }),
        });
    }
    YandexTelemostRuntimeStatus {
        account_id: account.account_id,
        provider_kind: account.provider_kind,
        lifecycle_state: if authorized {
            "authorized".to_owned()
        } else {
            account.lifecycle_state
        },
        runtime_kind: if authorized {
            YANDEX_TELEMOST_LIVE_RUNTIME_KIND.to_owned()
        } else {
            YANDEX_TELEMOST_RUNTIME_KIND.to_owned()
        },
        checked_at: Utc::now(),
        api_base_url: account.api_base_url,
        authorized,
        blockers,
        capabilities,
    }
}

fn client_for_account(
    account: &ProviderAccount,
) -> Result<YandexTelemostHttpClient, YandexTelemostError> {
    YandexTelemostHttpClient::new(account.config.get("api_base_url").and_then(Value::as_str))
}

fn provider_payload<T: Serialize>(request: &T) -> Result<Value, YandexTelemostError> {
    let mut value = serde_json::to_value(request)?;
    if let Value::Object(ref mut object) = value {
        object.remove("metadata");
    }
    Ok(value)
}

fn validate_account_setup_request(
    request: &YandexTelemostAccountSetupRequest,
) -> Result<(), YandexTelemostError> {
    validate_required("account_id", &request.account_id)?;
    validate_required("display_name", &request.display_name)?;
    validate_required("external_account_id", &request.external_account_id)?;
    validate_json_object("metadata", &request.metadata)?;
    validate_api_base_url(request.api_base_url.as_deref())?;
    if request
        .oauth_token
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_none()
        && request
            .oauth_token_ref
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .is_none()
    {
        return Err(YandexTelemostError::InvalidRequest(
            "oauth_token or oauth_token_ref must be provided for live Yandex Telemost API calls"
                .to_owned(),
        ));
    }
    Ok(())
}

fn validate_conference_request(
    request: &YandexTelemostConferenceRequest,
) -> Result<(), YandexTelemostError> {
    validate_json_object("metadata", &request.metadata)?;
    validate_cohosts(&request.cohosts)
}

fn validate_conference_patch_request(
    request: &YandexTelemostConferencePatchRequest,
) -> Result<(), YandexTelemostError> {
    validate_json_object("metadata", &request.metadata)?;
    validate_cohosts(&request.cohosts)
}

fn validate_cohosts(cohosts: &[TelemostCohost]) -> Result<(), YandexTelemostError> {
    if cohosts.len() > 30 {
        return Err(YandexTelemostError::InvalidRequest(
            "Yandex Telemost supports at most 30 cohosts".to_owned(),
        ));
    }
    for cohost in cohosts {
        validate_required("cohost.email", &cohost.email)?;
        if !cohost.email.contains('@') {
            return Err(YandexTelemostError::InvalidRequest(format!(
                "cohost `{}` must be an email address",
                cohost.email
            )));
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TelemostRetentionCleanupCandidate {
    account_id: String,
    bundle_id: String,
    conference_id: Option<String>,
    bundle_root: PathBuf,
    manifest_path: PathBuf,
    audio_path: PathBuf,
    speaker_jsonl_path: PathBuf,
    speaker_txt_path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LocalRecordingRetentionPolicy {
    recording_retention_days: i64,
    speaker_hint_retention_days: i64,
    audio_expires_at: Option<DateTime<Utc>>,
    speaker_hints_expires_at: Option<DateTime<Utc>>,
}

fn retention_cleanup_candidate_from_event(
    payload: &Value,
    occurred_at: DateTime<Utc>,
) -> Option<TelemostRetentionCleanupCandidate> {
    let account_id = payload.get("account_id")?.as_str()?.trim();
    let bundle_id = payload
        .get("bundle_id")
        .or_else(|| payload.get("recording_session_id"))?
        .as_str()?
        .trim();
    let bundle_root = PathBuf::from(payload.get("bundle_root")?.as_str()?.trim());
    let manifest_path = PathBuf::from(payload.get("manifest_path")?.as_str()?.trim());
    let audio_path = PathBuf::from(payload.get("audio_path")?.as_str()?.trim());
    let speaker_jsonl_path = PathBuf::from(payload.get("speaker_jsonl_path")?.as_str()?.trim());
    let speaker_txt_path = PathBuf::from(payload.get("speaker_txt_path")?.as_str()?.trim());
    if account_id.is_empty()
        || bundle_id.is_empty()
        || !bundle_root.is_absolute()
        || !manifest_path.is_absolute()
        || !audio_path.is_absolute()
        || !speaker_jsonl_path.is_absolute()
        || !speaker_txt_path.is_absolute()
        || occurred_at.timestamp() <= 0
    {
        return None;
    }
    if !audio_path.starts_with(&bundle_root)
        || !speaker_jsonl_path.starts_with(&bundle_root)
        || !speaker_txt_path.starts_with(&bundle_root)
        || !manifest_path.starts_with(&bundle_root)
    {
        return None;
    }

    Some(TelemostRetentionCleanupCandidate {
        account_id: account_id.to_owned(),
        bundle_id: bundle_id.to_owned(),
        conference_id: payload
            .get("conference_id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned),
        bundle_root,
        manifest_path,
        audio_path,
        speaker_jsonl_path,
        speaker_txt_path,
    })
}

fn local_recording_retention_policy_from_manifest(
    manifest: &Value,
) -> Option<LocalRecordingRetentionPolicy> {
    let retention = manifest
        .get("provenance")?
        .get("retention_policy")?
        .get("local_recording")?;
    Some(LocalRecordingRetentionPolicy {
        recording_retention_days: retention
            .get("recording_retention_days")
            .and_then(Value::as_i64)
            .unwrap_or(0)
            .max(0),
        speaker_hint_retention_days: retention
            .get("speaker_hint_retention_days")
            .and_then(Value::as_i64)
            .unwrap_or(0)
            .max(0),
        audio_expires_at: parse_optional_datetime(retention.get("audio_expires_at")),
        speaker_hints_expires_at: parse_optional_datetime(
            retention.get("speaker_hints_expires_at"),
        ),
    })
}

fn local_recording_retention_policy_from_days(
    observed_at: DateTime<Utc>,
    recording_retention_days: i64,
    speaker_hint_retention_days: i64,
) -> LocalRecordingRetentionPolicy {
    LocalRecordingRetentionPolicy {
        recording_retention_days,
        speaker_hint_retention_days,
        audio_expires_at: if recording_retention_days > 0 {
            Some(observed_at + chrono::TimeDelta::days(recording_retention_days))
        } else {
            None
        },
        speaker_hints_expires_at: if speaker_hint_retention_days > 0 {
            Some(observed_at + chrono::TimeDelta::days(speaker_hint_retention_days))
        } else {
            None
        },
    }
}

fn parse_optional_datetime(value: Option<&Value>) -> Option<DateTime<Utc>> {
    value
        .and_then(Value::as_str)
        .and_then(|raw| chrono::DateTime::parse_from_rfc3339(raw).ok())
        .map(|value| value.with_timezone(&Utc))
}

fn remove_local_file_if_exists(path: &Path) -> Result<bool, YandexTelemostError> {
    if !path.exists() {
        return Ok(false);
    }
    if !path.is_file() {
        return Ok(false);
    }
    fs::remove_file(path)?;
    Ok(true)
}

fn record_retention_cleanup_in_manifest(
    manifest_path: &Path,
    policy: &LocalRecordingRetentionPolicy,
    audio_removed: bool,
    speaker_removed: bool,
    removed_at: DateTime<Utc>,
) -> Result<(), YandexTelemostError> {
    if !manifest_path.exists() {
        return Ok(());
    }
    let mut manifest: Value = serde_json::from_str(&fs::read_to_string(manifest_path)?)?;
    let Some(provenance) = manifest
        .get_mut("provenance")
        .and_then(Value::as_object_mut)
    else {
        return Ok(());
    };
    provenance.insert(
        "retention_cleanup".to_owned(),
        json!({
            "audio_removed": audio_removed,
            "speaker_hints_removed": speaker_removed,
            "removed_at": removed_at,
            "recording_retention_days": policy.recording_retention_days,
            "speaker_hint_retention_days": policy.speaker_hint_retention_days,
        }),
    );
    fs::write(manifest_path, serde_json::to_string_pretty(&manifest)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_payload_drops_local_metadata() {
        let request = YandexTelemostConferenceRequest {
            waiting_room_level: Some("all".to_owned()),
            live_stream: None,
            cohosts: vec![],
            is_auto_summarization_enabled: Some(true),
            metadata: json!({ "local": true }),
        };
        let payload = provider_payload(&request).unwrap();
        assert!(payload.get("metadata").is_none());
        assert_eq!(payload["waiting_room_level"], "all");
    }

    #[test]
    fn retention_cleanup_candidate_rejects_relative_paths() {
        let candidate = retention_cleanup_candidate_from_event(
            &json!({
                "account_id": "telemost-main",
                "bundle_id": "bundle-1",
                "bundle_root": "relative/root",
                "manifest_path": "/tmp/manifest.json",
                "audio_path": "/tmp/audio.mp3",
                "speaker_jsonl_path": "/tmp/speaker.jsonl",
                "speaker_txt_path": "/tmp/speaker.txt"
            }),
            Utc::now(),
        );

        assert!(candidate.is_none());
    }

    #[test]
    fn local_recording_retention_policy_from_days_sets_expiry_for_positive_values() {
        let observed_at = Utc::now();
        let policy = local_recording_retention_policy_from_days(observed_at, 7, 3);

        assert_eq!(policy.recording_retention_days, 7);
        assert_eq!(policy.speaker_hint_retention_days, 3);
        assert!(policy.audio_expires_at.is_some());
        assert!(policy.speaker_hints_expires_at.is_some());
    }
}
