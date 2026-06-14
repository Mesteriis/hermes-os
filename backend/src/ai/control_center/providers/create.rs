use serde_json::{Value, json};

use super::super::errors::AiControlCenterError;
use super::super::models::{AiProviderAccount, AiProviderCreateRequest};
use super::super::presets::default_capabilities;
use super::super::rows::row_to_provider;
use super::super::store::AiControlCenterStore;
use super::super::validation::{
    non_empty_optional, object_value, reject_secret_like_json, slug_id, string_array_value,
    validate_non_empty,
};

impl AiControlCenterStore {
    pub async fn create_provider(
        &self,
        request: &AiProviderCreateRequest,
    ) -> Result<AiProviderAccount, AiControlCenterError> {
        request.validate()?;
        let provider_kind = request.provider_kind.trim();
        let provider_key = request.provider_key.trim();
        let provider_id = request
            .provider_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| {
                format!(
                    "provider:{provider_kind}:{}",
                    slug_id(&format!("{}-{}", provider_key, request.display_name.trim()))
                )
            });
        validate_non_empty("provider_id", &provider_id)?;

        let mut config = object_value(
            request.config.clone().unwrap_or_else(|| json!({})),
            "config",
        )?;
        if let Some(base_url) = non_empty_optional(&request.base_url) {
            config.insert("base_url".to_owned(), json!(base_url));
        }
        if let Some(command_preset) = non_empty_optional(&request.command_preset) {
            config.insert("command_preset".to_owned(), json!(command_preset));
        }
        reject_secret_like_json(&Value::Object(config.clone()))?;

        let capabilities = string_array_value(
            request
                .capabilities
                .clone()
                .unwrap_or_else(|| default_capabilities(provider_kind, provider_key)),
            "capabilities",
        )?;
        let status = if request.enabled.unwrap_or(true) {
            match provider_kind {
                "api" => "needs_setup",
                _ => "ready",
            }
        } else {
            "disabled"
        };
        let consent_state = match provider_kind {
            "api" if request.remote_context_consent == Some(true) => "granted",
            "api" => "required",
            _ => "not_required",
        };

        let row = sqlx::query(
            r#"
            INSERT INTO ai_provider_accounts (
                provider_id,
                provider_kind,
                provider_key,
                display_name,
                status,
                consent_state,
                consented_at,
                config,
                capabilities
            )
            VALUES ($1, $2, $3, $4, $5, $6, CASE WHEN $6 = 'granted' THEN now() ELSE NULL END, $7, $8)
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
        .bind(provider_id)
        .bind(provider_kind)
        .bind(provider_key)
        .bind(request.display_name.trim())
        .bind(status)
        .bind(consent_state)
        .bind(Value::Object(config))
        .bind(json!(capabilities))
        .fetch_one(&self.pool)
        .await?;

        let provider = row_to_provider(row)?;
        self.seed_models_for_provider(&provider).await?;
        Ok(provider)
    }
}
