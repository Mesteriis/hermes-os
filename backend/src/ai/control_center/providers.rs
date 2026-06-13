use serde_json::{Value, json};

use super::errors::AiControlCenterError;
use super::models::{
    AiProviderAccount, AiProviderCommandKind, AiProviderCommandResponse, AiProviderConsentRequest,
    AiProviderCreateRequest, AiProviderPatchRequest,
};
use super::presets::default_capabilities;
use super::rows::row_to_provider;
use super::store::AiControlCenterStore;
use super::validation::{
    non_empty_optional, object_value, reject_secret_like_json, slug_id, string_array_value,
    validate_non_empty,
};

impl AiControlCenterStore {
    pub async fn list_providers(&self) -> Result<Vec<AiProviderAccount>, AiControlCenterError> {
        let rows = sqlx::query(
            r#"
            SELECT
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
            FROM ai_provider_accounts
            ORDER BY provider_kind ASC, display_name ASC, provider_id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_provider).collect()
    }

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
                "api" if request.api_key.as_deref().unwrap_or("").trim().is_empty() => {
                    "needs_setup"
                }
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
        let display_name = non_empty_optional(&request.display_name)
            .unwrap_or_else(|| current.display_name.clone());
        let status = match request.enabled {
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

    pub async fn provider(
        &self,
        provider_id: &str,
    ) -> Result<Option<AiProviderAccount>, AiControlCenterError> {
        validate_non_empty("provider_id", provider_id)?;
        let row = sqlx::query(
            r#"
            SELECT
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
            FROM ai_provider_accounts
            WHERE provider_id = $1
            "#,
        )
        .bind(provider_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_provider).transpose()
    }

    pub async fn provider_command(
        &self,
        provider_id: &str,
        command: AiProviderCommandKind,
    ) -> Result<AiProviderCommandResponse, AiControlCenterError> {
        let provider = self
            .provider(provider_id)
            .await?
            .ok_or(AiControlCenterError::ProviderNotFound)?;
        let (status, message) = match command {
            AiProviderCommandKind::Test => match provider.provider_kind.as_str() {
                "built_in" => ("ok", "Built-in runtime metadata is configured"),
                "cli" => ("ok", "CLI provider preset is allowlisted"),
                "api" if provider.consent_state == "granted" => (
                    "ok",
                    "API provider consent is granted; live network check is deferred",
                ),
                "api" => (
                    "needs_consent",
                    "API provider requires remote-context consent",
                ),
                _ => ("error", "Unsupported provider kind"),
            },
            AiProviderCommandKind::SyncModels => {
                self.seed_models_for_provider(&provider).await?;
                ("synced", "Curated model catalog synchronized")
            }
        };

        Ok(AiProviderCommandResponse {
            provider_id: provider.provider_id,
            command: command.as_str().to_owned(),
            status: status.to_owned(),
            message: message.to_owned(),
        })
    }

    pub async fn record_consent(
        &self,
        provider_id: &str,
        request: &AiProviderConsentRequest,
    ) -> Result<AiProviderAccount, AiControlCenterError> {
        let consent_state = if request.consented {
            "granted"
        } else {
            "revoked"
        };
        let row = sqlx::query(
            r#"
            UPDATE ai_provider_accounts
            SET
                consent_state = $2,
                consented_at = CASE WHEN $2 = 'granted' THEN now() ELSE NULL END,
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
        .bind(consent_state)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AiControlCenterError::ProviderNotFound)?;

        row_to_provider(row)
    }

    pub async fn bind_api_key_secret(
        &self,
        provider_id: &str,
        secret_ref: &str,
    ) -> Result<(), AiControlCenterError> {
        validate_non_empty("provider_id", provider_id)?;
        validate_non_empty("secret_ref", secret_ref)?;
        sqlx::query(
            r#"
            INSERT INTO ai_provider_secret_refs (provider_id, secret_purpose, secret_ref, updated_at)
            VALUES ($1, 'api_key', $2, now())
            ON CONFLICT (provider_id, secret_purpose)
            DO UPDATE SET
                secret_ref = EXCLUDED.secret_ref,
                updated_at = now()
            "#,
        )
        .bind(provider_id.trim())
        .bind(secret_ref.trim())
        .execute(&self.pool)
        .await?;
        sqlx::query(
            r#"
            UPDATE ai_provider_accounts
            SET
                status = CASE WHEN status = 'needs_setup' THEN 'ready' ELSE status END,
                updated_at = now()
            WHERE provider_id = $1
            "#,
        )
        .bind(provider_id.trim())
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
