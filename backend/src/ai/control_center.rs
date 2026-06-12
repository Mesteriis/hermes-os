use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Row, Transaction};
use thiserror::Error;

use crate::ai::core::AI_EMBEDDING_DIMENSION;
use crate::platform::secrets::{
    NewSecretReference, SecretKind, SecretReferenceError, SecretReferenceStore, SecretStoreKind,
};
use crate::vault::{HostVault, HostVaultError, SecretEntryContext};

pub const BUILT_IN_OLLAMA_PROVIDER_ID: &str = "provider:built_in:ollama";
pub const OLLAMA_CHAT_MODEL: &str = "qwen3:4b";
pub const OLLAMA_EMBEDDING_MODEL: &str = "qwen3-embedding:4b";

const SECRET_PURPOSE_API_KEY: &str = "api_key";

const CAPABILITY_SLOTS: &[&str] = &[
    "default_chat",
    "reasoning",
    "summarization",
    "mail_intelligence",
    "reply_draft",
    "extraction",
    "embeddings",
    "meeting_prep",
];

const ENTITY_SCOPES: &[&str] = &[
    "global",
    "person",
    "organization",
    "project",
    "document",
    "task",
    "meeting",
    "communication",
    "conversation",
];

#[derive(Clone)]
pub struct AiControlCenterStore {
    pool: PgPool,
}

impl AiControlCenterStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn overview(&self) -> Result<AiSettingsOverviewResponse, AiControlCenterError> {
        Ok(AiSettingsOverviewResponse {
            providers: self.list_providers().await?,
            models: self.list_models().await?,
            routes: self.list_model_routes().await?,
            prompts: self.list_prompts().await?,
            eval_runs: self.list_prompt_eval_runs(25).await?,
            capability_slots: capability_slots(),
            provider_presets: provider_presets(),
        })
    }

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

    pub async fn list_models(&self) -> Result<Vec<AiModelCatalogItem>, AiControlCenterError> {
        let rows = sqlx::query(
            r#"
            SELECT
                provider_id,
                model_key,
                display_name,
                category,
                privacy,
                capabilities,
                context_window,
                embedding_dimension,
                is_available,
                metadata,
                created_at,
                updated_at
            FROM ai_model_catalog
            ORDER BY category ASC, privacy ASC, display_name ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_model).collect()
    }

    pub async fn list_model_routes(&self) -> Result<Vec<AiModelRoute>, AiControlCenterError> {
        let rows = sqlx::query(
            r#"
            SELECT
                capability_slot,
                provider_id,
                model_key,
                created_at,
                updated_at
            FROM ai_model_routes
            ORDER BY capability_slot ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_route).collect()
    }

    pub async fn route_for_slot(
        &self,
        slot: &str,
    ) -> Result<Option<AiModelRoute>, AiControlCenterError> {
        validate_capability_slot(slot)?;
        let row = sqlx::query(
            r#"
            SELECT
                capability_slot,
                provider_id,
                model_key,
                created_at,
                updated_at
            FROM ai_model_routes
            WHERE capability_slot = $1
            "#,
        )
        .bind(slot.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_route).transpose()
    }

    pub async fn put_model_route(
        &self,
        slot: &str,
        request: &AiModelRouteUpdateRequest,
    ) -> Result<AiModelRoute, AiControlCenterError> {
        validate_capability_slot(slot)?;
        validate_non_empty("provider_id", &request.provider_id)?;
        validate_non_empty("model_key", &request.model_key)?;
        let model = self
            .model(&request.provider_id, &request.model_key)
            .await?
            .ok_or(AiControlCenterError::ModelNotFound)?;
        if slot == "embeddings" && model.embedding_dimension != Some(AI_EMBEDDING_DIMENSION as i32)
        {
            return Err(AiControlCenterError::InvalidRequest(
                "embedding route requires a 2560-dimension model".to_owned(),
            ));
        }
        let row = sqlx::query(
            r#"
            INSERT INTO ai_model_routes (capability_slot, provider_id, model_key, updated_at)
            VALUES ($1, $2, $3, now())
            ON CONFLICT (capability_slot)
            DO UPDATE SET
                provider_id = EXCLUDED.provider_id,
                model_key = EXCLUDED.model_key,
                updated_at = now()
            RETURNING
                capability_slot,
                provider_id,
                model_key,
                created_at,
                updated_at
            "#,
        )
        .bind(slot.trim())
        .bind(request.provider_id.trim())
        .bind(request.model_key.trim())
        .fetch_one(&self.pool)
        .await?;

        row_to_route(row)
    }

    pub async fn model(
        &self,
        provider_id: &str,
        model_key: &str,
    ) -> Result<Option<AiModelCatalogItem>, AiControlCenterError> {
        validate_non_empty("provider_id", provider_id)?;
        validate_non_empty("model_key", model_key)?;
        let row = sqlx::query(
            r#"
            SELECT
                provider_id,
                model_key,
                display_name,
                category,
                privacy,
                capabilities,
                context_window,
                embedding_dimension,
                is_available,
                metadata,
                created_at,
                updated_at
            FROM ai_model_catalog
            WHERE provider_id = $1 AND model_key = $2
            "#,
        )
        .bind(provider_id.trim())
        .bind(model_key.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_model).transpose()
    }

    pub async fn list_prompts(&self) -> Result<Vec<AiPromptTemplate>, AiControlCenterError> {
        let rows = sqlx::query(
            r#"
            SELECT
                prompt_id,
                name,
                entity_scope,
                capability_slot,
                description,
                is_system,
                active_version_id,
                metadata,
                created_at,
                updated_at
            FROM ai_prompt_templates
            ORDER BY entity_scope ASC, capability_slot ASC, name ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_prompt).collect()
    }

    pub async fn create_prompt(
        &self,
        request: &AiPromptCreateRequest,
        actor_id: &str,
    ) -> Result<AiPromptTemplate, AiControlCenterError> {
        validate_non_empty("actor_id", actor_id)?;
        request.validate()?;
        let prompt_id = request
            .prompt_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| {
                format!(
                    "prompt:user:{}:{}",
                    request.entity_scope.trim(),
                    slug_id(request.name.trim())
                )
            });
        let metadata = object_value(
            request.metadata.clone().unwrap_or_else(|| json!({})),
            "metadata",
        )?;
        reject_secret_like_json(&Value::Object(metadata.clone()))?;
        let row = sqlx::query(
            r#"
            INSERT INTO ai_prompt_templates (
                prompt_id,
                name,
                entity_scope,
                capability_slot,
                description,
                is_system,
                metadata
            )
            VALUES ($1, $2, $3, $4, $5, false, $6)
            RETURNING
                prompt_id,
                name,
                entity_scope,
                capability_slot,
                description,
                is_system,
                active_version_id,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(prompt_id)
        .bind(request.name.trim())
        .bind(request.entity_scope.trim())
        .bind(request.capability_slot.trim())
        .bind(request.description.as_deref().map(str::trim))
        .bind(Value::Object(metadata))
        .fetch_one(&self.pool)
        .await?;

        row_to_prompt(row)
    }

    pub async fn create_prompt_version(
        &self,
        prompt_id: &str,
        request: &AiPromptVersionCreateRequest,
        actor_id: &str,
    ) -> Result<AiPromptVersion, AiControlCenterError> {
        validate_non_empty("prompt_id", prompt_id)?;
        validate_non_empty("actor_id", actor_id)?;
        request.validate()?;
        let prompt = self
            .prompt(prompt_id)
            .await?
            .ok_or(AiControlCenterError::PromptNotFound)?;
        if prompt.is_system {
            return Err(AiControlCenterError::InvalidRequest(
                "system prompts are read-only".to_owned(),
            ));
        }
        let variables = string_array_value(request.variables.clone(), "variables")?;
        let version_label = request
            .version_label
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| Utc::now().format("v%Y%m%d%H%M%S").to_string());
        let prompt_version_id = request
            .prompt_version_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| {
                format!(
                    "prompt-version:{}:{}",
                    prompt_id.trim(),
                    slug_id(&version_label)
                )
            });
        let row = sqlx::query(
            r#"
            INSERT INTO ai_prompt_template_versions (
                prompt_version_id,
                prompt_id,
                version_label,
                body_template,
                variables,
                status,
                created_by_actor_id
            )
            VALUES ($1, $2, $3, $4, $5, 'draft', $6)
            RETURNING
                prompt_version_id,
                prompt_id,
                version_label,
                body_template,
                variables,
                status,
                created_by_actor_id,
                created_at,
                updated_at
            "#,
        )
        .bind(prompt_version_id)
        .bind(prompt_id.trim())
        .bind(version_label)
        .bind(request.body_template.trim())
        .bind(json!(variables))
        .bind(actor_id.trim())
        .fetch_one(&self.pool)
        .await?;

        row_to_prompt_version(row)
    }

    pub async fn activate_prompt_version(
        &self,
        prompt_id: &str,
        request: &AiPromptActivateRequest,
    ) -> Result<AiPromptTemplate, AiControlCenterError> {
        validate_non_empty("prompt_id", prompt_id)?;
        validate_non_empty("prompt_version_id", &request.prompt_version_id)?;
        let prompt = self
            .prompt(prompt_id)
            .await?
            .ok_or(AiControlCenterError::PromptNotFound)?;
        if prompt.is_system {
            return Err(AiControlCenterError::InvalidRequest(
                "system prompts are read-only".to_owned(),
            ));
        }
        let mut tx = self.pool.begin().await?;
        let version_exists: Option<String> = sqlx::query_scalar(
            "SELECT prompt_version_id FROM ai_prompt_template_versions WHERE prompt_id = $1 AND prompt_version_id = $2",
        )
        .bind(prompt_id.trim())
        .bind(request.prompt_version_id.trim())
        .fetch_optional(&mut *tx)
        .await?;
        if version_exists.is_none() {
            return Err(AiControlCenterError::PromptVersionNotFound);
        }
        sqlx::query(
            "UPDATE ai_prompt_template_versions SET status = 'draft', updated_at = now() WHERE prompt_id = $1 AND status = 'active'",
        )
        .bind(prompt_id.trim())
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            "UPDATE ai_prompt_template_versions SET status = 'active', updated_at = now() WHERE prompt_version_id = $1",
        )
        .bind(request.prompt_version_id.trim())
        .execute(&mut *tx)
        .await?;
        let row = sqlx::query(
            r#"
            UPDATE ai_prompt_templates
            SET
                active_version_id = $2,
                updated_at = now()
            WHERE prompt_id = $1
            RETURNING
                prompt_id,
                name,
                entity_scope,
                capability_slot,
                description,
                is_system,
                active_version_id,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(prompt_id.trim())
        .bind(request.prompt_version_id.trim())
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;

        row_to_prompt(row)
    }

    pub async fn test_prompt(
        &self,
        prompt_id: &str,
        request: &AiPromptTestRequest,
        actor_id: &str,
    ) -> Result<AiPromptEvalRun, AiControlCenterError> {
        validate_non_empty("prompt_id", prompt_id)?;
        validate_non_empty("actor_id", actor_id)?;
        validate_non_empty("prompt_version_id", &request.prompt_version_id)?;
        validate_non_empty("provider_id", &request.provider_id)?;
        validate_non_empty("model_key", &request.model_key)?;
        let version = self
            .prompt_version(prompt_id, &request.prompt_version_id)
            .await?
            .ok_or(AiControlCenterError::PromptVersionNotFound)?;
        let _model = self
            .model(&request.provider_id, &request.model_key)
            .await?
            .ok_or(AiControlCenterError::ModelNotFound)?;
        let variables = object_value(request.variables.clone(), "variables")?;
        reject_secret_like_json(&Value::Object(variables.clone()))?;
        let source_refs = request.source_refs.clone().unwrap_or_default();
        let rendered = render_prompt(&version.body_template, &variables);
        let output_text = format!("Prompt studio preview\n\n{rendered}");
        let eval_run_id = format!(
            "prompt-eval:{}:{}",
            prompt_id.trim(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );
        let row = sqlx::query(
            r#"
            INSERT INTO ai_prompt_eval_runs (
                eval_run_id,
                prompt_id,
                prompt_version_id,
                provider_id,
                model_key,
                source_refs,
                variables,
                output_text,
                score,
                notes,
                actor_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING
                eval_run_id,
                prompt_id,
                prompt_version_id,
                provider_id,
                model_key,
                source_refs,
                variables,
                output_text,
                score,
                notes,
                actor_id,
                created_at
            "#,
        )
        .bind(eval_run_id)
        .bind(prompt_id.trim())
        .bind(request.prompt_version_id.trim())
        .bind(request.provider_id.trim())
        .bind(request.model_key.trim())
        .bind(json!(source_refs))
        .bind(Value::Object(variables))
        .bind(output_text)
        .bind(request.score)
        .bind(request.notes.as_deref().map(str::trim))
        .bind(actor_id.trim())
        .fetch_one(&self.pool)
        .await?;

        row_to_eval_run(row)
    }

    async fn prompt(
        &self,
        prompt_id: &str,
    ) -> Result<Option<AiPromptTemplate>, AiControlCenterError> {
        let row = sqlx::query(
            r#"
            SELECT
                prompt_id,
                name,
                entity_scope,
                capability_slot,
                description,
                is_system,
                active_version_id,
                metadata,
                created_at,
                updated_at
            FROM ai_prompt_templates
            WHERE prompt_id = $1
            "#,
        )
        .bind(prompt_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_prompt).transpose()
    }

    async fn prompt_version(
        &self,
        prompt_id: &str,
        prompt_version_id: &str,
    ) -> Result<Option<AiPromptVersion>, AiControlCenterError> {
        let row = sqlx::query(
            r#"
            SELECT
                prompt_version_id,
                prompt_id,
                version_label,
                body_template,
                variables,
                status,
                created_by_actor_id,
                created_at,
                updated_at
            FROM ai_prompt_template_versions
            WHERE prompt_id = $1 AND prompt_version_id = $2
            "#,
        )
        .bind(prompt_id.trim())
        .bind(prompt_version_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_prompt_version).transpose()
    }

    async fn list_prompt_eval_runs(
        &self,
        limit: i64,
    ) -> Result<Vec<AiPromptEvalRun>, AiControlCenterError> {
        let rows = sqlx::query(
            r#"
            SELECT
                eval_run_id,
                prompt_id,
                prompt_version_id,
                provider_id,
                model_key,
                source_refs,
                variables,
                output_text,
                score,
                notes,
                actor_id,
                created_at
            FROM ai_prompt_eval_runs
            ORDER BY created_at DESC, eval_run_id ASC
            LIMIT $1
            "#,
        )
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_eval_run).collect()
    }

    async fn seed_models_for_provider(
        &self,
        provider: &AiProviderAccount,
    ) -> Result<(), AiControlCenterError> {
        for model in curated_models_for(provider) {
            sqlx::query(
                r#"
                INSERT INTO ai_model_catalog (
                    provider_id,
                    model_key,
                    display_name,
                    category,
                    privacy,
                    capabilities,
                    context_window,
                    embedding_dimension,
                    is_available,
                    metadata,
                    updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true, $9, now())
                ON CONFLICT (provider_id, model_key)
                DO UPDATE SET
                    display_name = EXCLUDED.display_name,
                    category = EXCLUDED.category,
                    privacy = EXCLUDED.privacy,
                    capabilities = EXCLUDED.capabilities,
                    context_window = EXCLUDED.context_window,
                    embedding_dimension = EXCLUDED.embedding_dimension,
                    is_available = true,
                    metadata = EXCLUDED.metadata,
                    updated_at = now()
                "#,
            )
            .bind(&provider.provider_id)
            .bind(model.model_key)
            .bind(model.display_name)
            .bind(model.category)
            .bind(model.privacy)
            .bind(json!(model.capabilities))
            .bind(model.context_window)
            .bind(model.embedding_dimension)
            .bind(model.metadata)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }
}

pub async fn store_api_key_in_host_vault(
    pool: &PgPool,
    vault: &HostVault,
    provider_id: &str,
    api_key: &str,
) -> Result<String, AiControlCenterError> {
    validate_non_empty("provider_id", provider_id)?;
    validate_non_empty("api_key", api_key)?;
    let secret_ref = format!("secret:ai-provider:{provider_id}:{SECRET_PURPOSE_API_KEY}");
    let metadata = json!({
        "provider_id": provider_id,
        "secret_purpose": SECRET_PURPOSE_API_KEY
    });
    let reference = NewSecretReference::new(
        &secret_ref,
        SecretKind::ApiToken,
        SecretStoreKind::HostVault,
        "AI provider API key",
    )
    .metadata(metadata.clone());

    SecretReferenceStore::new(pool.clone())
        .upsert_secret_reference(&reference)
        .await?;
    vault.store_secret(
        &secret_ref,
        api_key.trim(),
        SecretEntryContext {
            entry_kind: "ai_provider",
            account_id: provider_id,
            purpose: SECRET_PURPOSE_API_KEY,
            secret_kind: SecretKind::ApiToken.as_str(),
            label: "AI provider API key",
            metadata: &metadata,
        },
    )?;
    AiControlCenterStore::new(pool.clone())
        .bind_api_key_secret(provider_id, &secret_ref)
        .await?;

    Ok(secret_ref)
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiSettingsOverviewResponse {
    pub providers: Vec<AiProviderAccount>,
    pub models: Vec<AiModelCatalogItem>,
    pub routes: Vec<AiModelRoute>,
    pub prompts: Vec<AiPromptTemplate>,
    pub eval_runs: Vec<AiPromptEvalRun>,
    pub capability_slots: Vec<AiCapabilitySlot>,
    pub provider_presets: Vec<AiProviderPreset>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiCapabilitySlot {
    pub slot: String,
    pub label: String,
    pub description: String,
    pub requires_embedding_dimension: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiProviderPreset {
    pub provider_kind: String,
    pub provider_key: String,
    pub display_name: String,
    pub privacy: String,
    pub base_url: Option<String>,
    pub command_preset: Option<String>,
    pub capabilities: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiProviderAccount {
    pub provider_id: String,
    pub provider_kind: String,
    pub provider_key: String,
    pub display_name: String,
    pub status: String,
    pub consent_state: String,
    pub consented_at: Option<DateTime<Utc>>,
    pub config: Value,
    pub capabilities: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiProviderCreateRequest {
    pub provider_id: Option<String>,
    pub provider_kind: String,
    pub provider_key: String,
    pub display_name: String,
    pub base_url: Option<String>,
    pub command_preset: Option<String>,
    pub config: Option<Value>,
    pub capabilities: Option<Vec<String>>,
    pub enabled: Option<bool>,
    pub remote_context_consent: Option<bool>,
    #[serde(default, skip_serializing)]
    pub api_key: Option<String>,
}

impl AiProviderCreateRequest {
    fn validate(&self) -> Result<(), AiControlCenterError> {
        validate_provider_kind(&self.provider_kind)?;
        validate_non_empty("provider_key", &self.provider_key)?;
        validate_non_empty("display_name", &self.display_name)?;
        if self.provider_kind == "cli" {
            let preset = self.command_preset.as_deref().ok_or_else(|| {
                AiControlCenterError::InvalidRequest(
                    "CLI provider requires command_preset".to_owned(),
                )
            })?;
            validate_cli_preset(preset)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiProviderPatchRequest {
    pub display_name: Option<String>,
    pub base_url: Option<String>,
    pub config: Option<Value>,
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing)]
    pub api_key: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiProviderConsentRequest {
    pub consented: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiProviderCommandResponse {
    pub provider_id: String,
    pub command: String,
    pub status: String,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiProviderCommandKind {
    Test,
    SyncModels,
}

impl AiProviderCommandKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Test => "test",
            Self::SyncModels => "sync_models",
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiModelCatalogItem {
    pub provider_id: String,
    pub model_key: String,
    pub display_name: String,
    pub category: String,
    pub privacy: String,
    pub capabilities: Vec<String>,
    pub context_window: Option<i32>,
    pub embedding_dimension: Option<i32>,
    pub is_available: bool,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiModelRoute {
    pub capability_slot: String,
    pub provider_id: String,
    pub model_key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiModelRouteUpdateRequest {
    pub provider_id: String,
    pub model_key: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiPromptTemplate {
    pub prompt_id: String,
    pub name: String,
    pub entity_scope: String,
    pub capability_slot: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub active_version_id: Option<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiPromptCreateRequest {
    pub prompt_id: Option<String>,
    pub name: String,
    pub entity_scope: String,
    pub capability_slot: String,
    pub description: Option<String>,
    pub metadata: Option<Value>,
}

impl AiPromptCreateRequest {
    fn validate(&self) -> Result<(), AiControlCenterError> {
        validate_non_empty("name", &self.name)?;
        validate_entity_scope(&self.entity_scope)?;
        validate_capability_slot(&self.capability_slot)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiPromptVersion {
    pub prompt_version_id: String,
    pub prompt_id: String,
    pub version_label: String,
    pub body_template: String,
    pub variables: Vec<String>,
    pub status: String,
    pub created_by_actor_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiPromptVersionCreateRequest {
    pub prompt_version_id: Option<String>,
    pub version_label: Option<String>,
    pub body_template: String,
    pub variables: Vec<String>,
}

impl AiPromptVersionCreateRequest {
    fn validate(&self) -> Result<(), AiControlCenterError> {
        validate_non_empty("body_template", &self.body_template)?;
        let _ = string_array_value(self.variables.clone(), "variables")?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiPromptActivateRequest {
    pub prompt_version_id: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiPromptTestRequest {
    pub prompt_version_id: String,
    pub provider_id: String,
    pub model_key: String,
    pub variables: Value,
    pub source_refs: Option<Vec<Value>>,
    pub score: Option<i32>,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiPromptEvalRun {
    pub eval_run_id: String,
    pub prompt_id: String,
    pub prompt_version_id: String,
    pub provider_id: String,
    pub model_key: String,
    pub source_refs: Vec<Value>,
    pub variables: Value,
    pub output_text: String,
    pub score: Option<i32>,
    pub notes: Option<String>,
    pub actor_id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum AiControlCenterError {
    #[error("AI provider was not found")]
    ProviderNotFound,

    #[error("AI model was not found")]
    ModelNotFound,

    #[error("AI prompt was not found")]
    PromptNotFound,

    #[error("AI prompt version was not found")]
    PromptVersionNotFound,

    #[error("invalid AI control center request: {0}")]
    InvalidRequest(String),

    #[error("invalid AI control center field `{field}`")]
    EmptyField { field: &'static str },

    #[error("AI control center payload contains secret-like data")]
    SecretLikePayload,

    #[error(transparent)]
    SecretReference(#[from] SecretReferenceError),

    #[error(transparent)]
    HostVault(#[from] HostVaultError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

impl AiControlCenterError {
    pub fn is_invalid_request(&self) -> bool {
        matches!(
            self,
            Self::InvalidRequest(_)
                | Self::EmptyField { .. }
                | Self::SecretLikePayload
                | Self::ModelNotFound
                | Self::PromptNotFound
                | Self::PromptVersionNotFound
        )
    }
}

fn row_to_provider(row: PgRow) -> Result<AiProviderAccount, AiControlCenterError> {
    Ok(AiProviderAccount {
        provider_id: row.try_get("provider_id")?,
        provider_kind: row.try_get("provider_kind")?,
        provider_key: row.try_get("provider_key")?,
        display_name: row.try_get("display_name")?,
        status: row.try_get("status")?,
        consent_state: row.try_get("consent_state")?,
        consented_at: row.try_get("consented_at")?,
        config: row.try_get("config")?,
        capabilities: json_string_array(row.try_get("capabilities")?)?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_model(row: PgRow) -> Result<AiModelCatalogItem, AiControlCenterError> {
    Ok(AiModelCatalogItem {
        provider_id: row.try_get("provider_id")?,
        model_key: row.try_get("model_key")?,
        display_name: row.try_get("display_name")?,
        category: row.try_get("category")?,
        privacy: row.try_get("privacy")?,
        capabilities: json_string_array(row.try_get("capabilities")?)?,
        context_window: row.try_get("context_window")?,
        embedding_dimension: row.try_get("embedding_dimension")?,
        is_available: row.try_get("is_available")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_route(row: PgRow) -> Result<AiModelRoute, AiControlCenterError> {
    Ok(AiModelRoute {
        capability_slot: row.try_get("capability_slot")?,
        provider_id: row.try_get("provider_id")?,
        model_key: row.try_get("model_key")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_prompt(row: PgRow) -> Result<AiPromptTemplate, AiControlCenterError> {
    Ok(AiPromptTemplate {
        prompt_id: row.try_get("prompt_id")?,
        name: row.try_get("name")?,
        entity_scope: row.try_get("entity_scope")?,
        capability_slot: row.try_get("capability_slot")?,
        description: row.try_get("description")?,
        is_system: row.try_get("is_system")?,
        active_version_id: row.try_get("active_version_id")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_prompt_version(row: PgRow) -> Result<AiPromptVersion, AiControlCenterError> {
    Ok(AiPromptVersion {
        prompt_version_id: row.try_get("prompt_version_id")?,
        prompt_id: row.try_get("prompt_id")?,
        version_label: row.try_get("version_label")?,
        body_template: row.try_get("body_template")?,
        variables: json_string_array(row.try_get("variables")?)?,
        status: row.try_get("status")?,
        created_by_actor_id: row.try_get("created_by_actor_id")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_eval_run(row: PgRow) -> Result<AiPromptEvalRun, AiControlCenterError> {
    Ok(AiPromptEvalRun {
        eval_run_id: row.try_get("eval_run_id")?,
        prompt_id: row.try_get("prompt_id")?,
        prompt_version_id: row.try_get("prompt_version_id")?,
        provider_id: row.try_get("provider_id")?,
        model_key: row.try_get("model_key")?,
        source_refs: json_array(row.try_get("source_refs")?)?,
        variables: row.try_get("variables")?,
        output_text: row.try_get("output_text")?,
        score: row.try_get("score")?,
        notes: row.try_get("notes")?,
        actor_id: row.try_get("actor_id")?,
        created_at: row.try_get("created_at")?,
    })
}

fn capability_slots() -> Vec<AiCapabilitySlot> {
    CAPABILITY_SLOTS
        .iter()
        .map(|slot| AiCapabilitySlot {
            slot: (*slot).to_owned(),
            label: settings_label(slot),
            description: capability_description(slot),
            requires_embedding_dimension: if *slot == "embeddings" {
                Some(AI_EMBEDDING_DIMENSION as i32)
            } else {
                None
            },
        })
        .collect()
}

fn provider_presets() -> Vec<AiProviderPreset> {
    vec![
        AiProviderPreset {
            provider_kind: "built_in".to_owned(),
            provider_key: "ollama".to_owned(),
            display_name: "Built-in Ollama".to_owned(),
            privacy: "local".to_owned(),
            base_url: Some("http://127.0.0.1:11434".to_owned()),
            command_preset: None,
            capabilities: vec![
                "chat".to_owned(),
                "embeddings".to_owned(),
                "local_runtime".to_owned(),
            ],
        },
        AiProviderPreset {
            provider_kind: "cli".to_owned(),
            provider_key: "codex".to_owned(),
            display_name: "Codex CLI".to_owned(),
            privacy: "cli".to_owned(),
            base_url: None,
            command_preset: Some("codex".to_owned()),
            capabilities: vec!["chat".to_owned(), "reasoning".to_owned()],
        },
        AiProviderPreset {
            provider_kind: "cli".to_owned(),
            provider_key: "claude".to_owned(),
            display_name: "Claude CLI".to_owned(),
            privacy: "cli".to_owned(),
            base_url: None,
            command_preset: Some("claude".to_owned()),
            capabilities: vec!["chat".to_owned(), "reasoning".to_owned()],
        },
        AiProviderPreset {
            provider_kind: "api".to_owned(),
            provider_key: "openai".to_owned(),
            display_name: "OpenAI".to_owned(),
            privacy: "remote".to_owned(),
            base_url: Some("https://api.openai.com/v1".to_owned()),
            command_preset: None,
            capabilities: vec![
                "chat".to_owned(),
                "reasoning".to_owned(),
                "embeddings".to_owned(),
            ],
        },
        AiProviderPreset {
            provider_kind: "api".to_owned(),
            provider_key: "deepseek".to_owned(),
            display_name: "DeepSeek".to_owned(),
            privacy: "remote".to_owned(),
            base_url: Some("https://api.deepseek.com/v1".to_owned()),
            command_preset: None,
            capabilities: vec!["chat".to_owned(), "reasoning".to_owned()],
        },
        AiProviderPreset {
            provider_kind: "api".to_owned(),
            provider_key: "omniroute".to_owned(),
            display_name: "OmniRoute".to_owned(),
            privacy: "remote".to_owned(),
            base_url: Some("https://ai.sh-inc.ru/v1".to_owned()),
            command_preset: None,
            capabilities: vec![
                "chat".to_owned(),
                "embeddings".to_owned(),
                "routing".to_owned(),
            ],
        },
    ]
}

struct CuratedModel {
    model_key: &'static str,
    display_name: &'static str,
    category: &'static str,
    privacy: &'static str,
    capabilities: Vec<&'static str>,
    context_window: Option<i32>,
    embedding_dimension: Option<i32>,
    metadata: Value,
}

fn curated_models_for(provider: &AiProviderAccount) -> Vec<CuratedModel> {
    match (
        provider.provider_kind.as_str(),
        provider.provider_key.as_str(),
    ) {
        ("built_in", "ollama") => vec![
            CuratedModel {
                model_key: OLLAMA_CHAT_MODEL,
                display_name: "Qwen3 4B",
                category: "chat",
                privacy: "local",
                capabilities: vec!["chat", "reasoning", "summarization", "extraction"],
                context_window: Some(32768),
                embedding_dimension: None,
                metadata: json!({"curated": true, "pull_required": true}),
            },
            CuratedModel {
                model_key: OLLAMA_EMBEDDING_MODEL,
                display_name: "Qwen3 Embedding 4B",
                category: "embeddings",
                privacy: "local",
                capabilities: vec!["embeddings"],
                context_window: Some(8192),
                embedding_dimension: Some(AI_EMBEDDING_DIMENSION as i32),
                metadata: json!({"curated": true, "pull_required": true}),
            },
        ],
        ("api", "openai") => vec![
            CuratedModel {
                model_key: "gpt-5.5",
                display_name: "GPT-5.5",
                category: "reasoning",
                privacy: "remote",
                capabilities: vec!["chat", "reasoning", "summarization"],
                context_window: Some(128000),
                embedding_dimension: None,
                metadata: json!({"curated": true}),
            },
            CuratedModel {
                model_key: "text-embedding-3-large",
                display_name: "Text Embedding 3 Large",
                category: "embeddings",
                privacy: "remote",
                capabilities: vec!["embeddings"],
                context_window: Some(8192),
                embedding_dimension: Some(3072),
                metadata: json!({"curated": true, "embedding_route_supported": false}),
            },
        ],
        ("api", "deepseek") => vec![CuratedModel {
            model_key: "deepseek-chat",
            display_name: "DeepSeek Chat",
            category: "chat",
            privacy: "remote",
            capabilities: vec!["chat", "reasoning", "summarization"],
            context_window: Some(64000),
            embedding_dimension: None,
            metadata: json!({"curated": true}),
        }],
        ("api", "omniroute") => vec![
            CuratedModel {
                model_key: "codex/gpt-5.5",
                display_name: "Codex GPT-5.5",
                category: "reasoning",
                privacy: "remote",
                capabilities: vec!["chat", "reasoning", "summarization"],
                context_window: Some(128000),
                embedding_dimension: None,
                metadata: json!({"curated": true}),
            },
            CuratedModel {
                model_key: "openai-compatible-chat-ollama-pve/qwen3-embedding:4b",
                display_name: "Qwen3 Embedding via OmniRoute",
                category: "embeddings",
                privacy: "remote",
                capabilities: vec!["embeddings"],
                context_window: Some(8192),
                embedding_dimension: Some(AI_EMBEDDING_DIMENSION as i32),
                metadata: json!({"curated": true}),
            },
        ],
        ("cli", "codex") => vec![CuratedModel {
            model_key: "codex-cli/default",
            display_name: "Codex CLI Default",
            category: "reasoning",
            privacy: "cli",
            capabilities: vec!["chat", "reasoning"],
            context_window: None,
            embedding_dimension: None,
            metadata: json!({"curated": true, "command_preset": "codex"}),
        }],
        ("cli", "claude") => vec![CuratedModel {
            model_key: "claude-cli/default",
            display_name: "Claude CLI Default",
            category: "reasoning",
            privacy: "cli",
            capabilities: vec!["chat", "reasoning"],
            context_window: None,
            embedding_dimension: None,
            metadata: json!({"curated": true, "command_preset": "claude"}),
        }],
        _ => vec![CuratedModel {
            model_key: "custom/default",
            display_name: "Custom default",
            category: "chat",
            privacy: if provider.provider_kind == "api" {
                "remote"
            } else {
                "cli"
            },
            capabilities: vec!["chat"],
            context_window: None,
            embedding_dimension: None,
            metadata: json!({"curated": false}),
        }],
    }
}

fn default_capabilities(provider_kind: &str, provider_key: &str) -> Vec<String> {
    match provider_kind {
        "built_in" => vec!["chat", "embeddings", "local_runtime"],
        "cli" => vec!["chat", "reasoning"],
        "api" if provider_key == "omniroute" => vec!["chat", "embeddings", "routing"],
        "api" => vec!["chat", "reasoning"],
        _ => vec!["chat"],
    }
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn validate_provider_kind(value: &str) -> Result<(), AiControlCenterError> {
    match value.trim() {
        "built_in" | "cli" | "api" => Ok(()),
        other => Err(AiControlCenterError::InvalidRequest(format!(
            "unsupported provider_kind `{other}`"
        ))),
    }
}

fn validate_cli_preset(value: &str) -> Result<(), AiControlCenterError> {
    match value.trim() {
        "codex" | "claude" | "hermes" => Ok(()),
        other => Err(AiControlCenterError::InvalidRequest(format!(
            "unsupported CLI command preset `{other}`"
        ))),
    }
}

fn validate_capability_slot(value: &str) -> Result<(), AiControlCenterError> {
    if CAPABILITY_SLOTS.contains(&value.trim()) {
        Ok(())
    } else {
        Err(AiControlCenterError::InvalidRequest(format!(
            "unsupported capability slot `{}`",
            value.trim()
        )))
    }
}

fn validate_entity_scope(value: &str) -> Result<(), AiControlCenterError> {
    if ENTITY_SCOPES.contains(&value.trim()) {
        Ok(())
    } else {
        Err(AiControlCenterError::InvalidRequest(format!(
            "unsupported entity scope `{}`",
            value.trim()
        )))
    }
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), AiControlCenterError> {
    if value.trim().is_empty() {
        return Err(AiControlCenterError::EmptyField { field });
    }
    Ok(())
}

fn non_empty_optional(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn object_value(
    value: Value,
    field: &'static str,
) -> Result<Map<String, Value>, AiControlCenterError> {
    value
        .as_object()
        .cloned()
        .ok_or_else(|| AiControlCenterError::InvalidRequest(format!("{field} must be an object")))
}

fn string_array_value(
    values: Vec<String>,
    field: &'static str,
) -> Result<Vec<String>, AiControlCenterError> {
    let mut cleaned = Vec::new();
    for value in values {
        validate_non_empty(field, &value)?;
        let value = value.trim().to_owned();
        if !cleaned.contains(&value) {
            cleaned.push(value);
        }
    }
    Ok(cleaned)
}

fn json_string_array(value: Value) -> Result<Vec<String>, AiControlCenterError> {
    let Some(items) = value.as_array() else {
        return Err(AiControlCenterError::InvalidRequest(
            "value must be an array".to_owned(),
        ));
    };
    items
        .iter()
        .map(|item| {
            item.as_str().map(str::to_owned).ok_or_else(|| {
                AiControlCenterError::InvalidRequest("array item must be a string".to_owned())
            })
        })
        .collect()
}

fn json_array(value: Value) -> Result<Vec<Value>, AiControlCenterError> {
    value
        .as_array()
        .cloned()
        .ok_or_else(|| AiControlCenterError::InvalidRequest("value must be an array".to_owned()))
}

fn reject_secret_like_json(value: &Value) -> Result<(), AiControlCenterError> {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                let normalized = key.to_ascii_lowercase();
                if normalized.contains("secret")
                    || normalized.contains("password")
                    || normalized.contains("token")
                    || normalized.contains("credential")
                    || normalized.contains("private_key")
                    || normalized == "body"
                    || normalized == "html"
                    || normalized == "raw"
                {
                    return Err(AiControlCenterError::SecretLikePayload);
                }
                reject_secret_like_json(child)?;
            }
        }
        Value::Array(items) => {
            for item in items {
                reject_secret_like_json(item)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn render_prompt(template: &str, variables: &Map<String, Value>) -> String {
    let mut rendered = template.to_owned();
    for (key, value) in variables {
        let replacement = value
            .as_str()
            .map(str::to_owned)
            .unwrap_or_else(|| value.to_string());
        rendered = rendered.replace(&format!("{{{{{key}}}}}"), &replacement);
    }
    rendered
}

fn slug_id(value: &str) -> String {
    let mut slug = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if slug.is_empty() {
        slug = Utc::now()
            .timestamp_nanos_opt()
            .unwrap_or_default()
            .to_string();
    }
    slug
}

fn settings_label(value: &str) -> String {
    value
        .split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn capability_description(slot: &str) -> String {
    match slot {
        "default_chat" => "General source-backed answers and chat.".to_owned(),
        "reasoning" => "Higher-effort planning and synthesis.".to_owned(),
        "summarization" => "Short summaries over local context.".to_owned(),
        "mail_intelligence" => "Communication analysis and operational context.".to_owned(),
        "reply_draft" => "Drafting replies without sending provider messages.".to_owned(),
        "extraction" => "Structured extraction from untrusted source text.".to_owned(),
        "embeddings" => "Semantic index embeddings; dimension constrained.".to_owned(),
        "meeting_prep" => "Meeting brief generation from local context.".to_owned(),
        _ => "AI capability.".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_like_provider_payloads_are_rejected() {
        let payload = json!({
            "headers": {
                "authorization_token": "sk-test"
            }
        });

        let error = reject_secret_like_json(&payload).expect_err("secret-like keys must fail");

        assert!(matches!(error, AiControlCenterError::SecretLikePayload));
    }

    #[test]
    fn cli_provider_presets_are_allowlisted() {
        assert!(validate_cli_preset("codex").is_ok());
        assert!(validate_cli_preset("claude").is_ok());
        assert!(validate_cli_preset("hermes").is_ok());

        let error = validate_cli_preset("bash -lc env").expect_err("shell-like presets must fail");

        assert!(matches!(error, AiControlCenterError::InvalidRequest(_)));
    }

    #[test]
    fn provider_presets_include_remote_consent_targets() {
        let presets = provider_presets();

        assert!(presets.iter().any(|preset| preset.provider_key == "openai"));
        assert!(
            presets
                .iter()
                .any(|preset| preset.provider_key == "deepseek")
        );
        assert!(
            presets
                .iter()
                .any(|preset| preset.provider_key == "omniroute")
        );
        assert!(
            presets
                .iter()
                .any(|preset| preset.provider_key == "ollama" && preset.privacy == "local")
        );
    }

    #[test]
    fn capability_slots_preserve_embedding_dimension_constraint() {
        let slots = capability_slots();
        let embeddings = slots
            .iter()
            .find(|slot| slot.slot == "embeddings")
            .expect("embeddings capability exists");

        assert_eq!(
            embeddings.requires_embedding_dimension,
            Some(AI_EMBEDDING_DIMENSION as i32)
        );
    }

    #[test]
    fn prompt_rendering_never_needs_source_text_in_events() {
        let mut variables = Map::new();
        variables.insert("entity".to_owned(), json!("Communication"));
        variables.insert("summary".to_owned(), json!("Needs reply"));

        assert_eq!(
            render_prompt("Review {{entity}}: {{summary}}", &variables),
            "Review Communication: Needs reply"
        );
    }
}
