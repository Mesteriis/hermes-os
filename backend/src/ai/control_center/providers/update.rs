use serde_json::{Value, json};

use super::super::errors::AiControlCenterError;
use super::super::models::{AiProviderAccount, AiProviderPatchRequest};
use super::super::rows::row_to_provider;
use super::super::store::AiControlCenterStore;
use super::super::validation::{
    non_empty_optional, object_value, reject_secret_like_json, validate_non_empty,
};

impl AiControlCenterStore {
    pub async fn update_provider(
        &self,
        provider_id: &str,
        request: &AiProviderPatchRequest,
    ) -> Result<AiProviderAccount, AiControlCenterError> {
        validate_non_empty("provider_id", provider_id)?;
        let current = self
            .provider(provider_id)
            .await?
            .ok_or(AiControlCenterError::ProviderNotFound)?;
        if current.provider_kind != "api"
            && request
                .api_key
                .as_deref()
                .map(str::trim)
                .is_some_and(|value| !value.is_empty())
        {
            return Err(AiControlCenterError::InvalidRequest(
                "API keys can only be configured for API providers".to_owned(),
            ));
        }
        let display_name = non_empty_optional(&request.display_name)
            .unwrap_or_else(|| current.display_name.clone());
        let api_key_configured = if current.provider_kind == "api" {
            self.api_key_secret_configured(provider_id).await?
        } else {
            true
        };
        let status = match request.enabled {
            Some(true) if current.provider_kind == "api" && !api_key_configured => {
                "needs_setup".to_owned()
            }
            Some(true) if current.status == "needs_setup" => "needs_setup".to_owned(),
            Some(true) => "ready".to_owned(),
            Some(false) => "disabled".to_owned(),
            None => current.status.clone(),
        };
        let mut config = object_value(current.config.clone(), "config")?;
        if let Some(config_patch) = &request.config {
            let patch = object_value(config_patch.clone(), "config")?;
            for (key, value) in patch {
                config.insert(key, value);
            }
        }
        if let Some(base_url) = non_empty_optional(&request.base_url) {
            config.insert("base_url".to_owned(), json!(base_url));
        }
        reject_secret_like_json(&Value::Object(config.clone()))?;

        let row = sqlx::query(
            r#"
            UPDATE ai_provider_accounts
            SET
                display_name = $2,
                status = $3,
                config = $4,
                updated_at = now()
            WHERE provider_id = $1
            RETURNING
                provider_id,
                provider_kind,
                provider_key,
                display_name,
                status,
                consent_state,
                consented_at,
                config,
                capabilities,
                created_at,
                updated_at
            "#,
        )
        .bind(provider_id.trim())
        .bind(display_name)
        .bind(status)
        .bind(Value::Object(config))
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AiControlCenterError::ProviderNotFound)?;

        row_to_provider(row)
    }
}
