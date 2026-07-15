use serde_json::{Value, json};

use super::super::errors::AiControlCenterError;
use super::super::evidence::capture_provider_account_observation;
use super::super::models::{AiProviderAccount, AiProviderAuthPendingGrant};
use super::super::presets::{BUILT_IN_OLLAMA_PROVIDER_ID, default_capabilities};
use super::super::rows::row_to_provider;
use super::super::store::AiControlCenterStore;
use super::super::validation::reject_secret_like_json;

impl AiControlCenterStore {
    pub async fn connect_local_or_cli_provider(
        &self,
        pending: &AiProviderAuthPendingGrant,
    ) -> Result<AiProviderAccount, AiControlCenterError> {
        let capabilities = default_capabilities(&pending.provider_kind, &pending.provider_key);
        let config = provider_auth_config(pending);
        reject_secret_like_json(&config)?;

        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            INSERT INTO ai_provider_accounts (
                provider_id,
                provider_kind,
                provider_key,
                display_name,
                status,
                consent_state,
                config,
                capabilities
            )
            VALUES ($1, $2, $3, $4, 'ready', 'not_required', $5, $6)
            ON CONFLICT (provider_kind, provider_key)
            DO UPDATE SET
                display_name = EXCLUDED.display_name,
                status = 'ready',
                consent_state = 'not_required',
                config = EXCLUDED.config,
                capabilities = EXCLUDED.capabilities,
                updated_at = now()
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
        .bind(provider_id_for_local_auth(pending))
        .bind(pending.provider_kind.trim())
        .bind(pending.provider_key.trim())
        .bind(pending.display_name.trim())
        .bind(config)
        .bind(json!(capabilities))
        .fetch_one(&mut *transaction)
        .await?;

        let provider = row_to_provider(row)?;
        capture_provider_account_observation(
            &mut transaction,
            &provider,
            "local_callback_connect",
            "ai_control_center.connect_local_or_cli_provider",
        )
        .await?;
        transaction.commit().await?;
        self.seed_models_for_provider(&provider, "ai_control_center.local_callback_connect")
            .await?;
        Ok(provider)
    }
}

fn provider_id_for_local_auth(pending: &AiProviderAuthPendingGrant) -> String {
    if pending.provider_kind == "built_in" && pending.provider_key == "ollama" {
        return BUILT_IN_OLLAMA_PROVIDER_ID.to_owned();
    }
    pending.provider_id.clone()
}

fn provider_auth_config(pending: &AiProviderAuthPendingGrant) -> Value {
    let mut config = json!({
        "local_callback": {
            "setup_id": pending.setup_id,
            "completed_at": chrono::Utc::now(),
        }
    });
    if pending.provider_kind == "built_in" && pending.provider_key == "ollama" {
        config["base_url"] = json!("http://192.168.1.2:11434");
        config["manager"] = json!("ollama");
    }
    if let Some(command_preset) = &pending.login_command {
        config["command_preset"] = json!(pending.provider_key);
        config["login_command_hint"] = json!(command_preset);
    }
    config
}
