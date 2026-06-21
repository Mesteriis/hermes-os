use super::super::errors::AiControlCenterError;
use super::super::evidence::capture_provider_secret_binding_observation;
use super::super::store::AiControlCenterStore;
use super::super::validation::validate_non_empty;

const SECRET_PURPOSE_API_KEY: &str = "api_key";

impl AiControlCenterStore {
    pub async fn bind_api_key_secret(
        &self,
        provider_id: &str,
        secret_ref: &str,
    ) -> Result<(), AiControlCenterError> {
        validate_non_empty("provider_id", provider_id)?;
        validate_non_empty("secret_ref", secret_ref)?;
        let provider = self
            .provider(provider_id)
            .await?
            .ok_or(AiControlCenterError::ProviderNotFound)?;
        if provider.provider_kind != "api" {
            return Err(AiControlCenterError::InvalidRequest(
                "API keys can only be bound to API providers".to_owned(),
            ));
        }
        if !self
            .api_key_secret_reference_is_host_vault(secret_ref)
            .await?
        {
            return Err(AiControlCenterError::InvalidRequest(
                "API provider API key must reference a host-vault api_token secret".to_owned(),
            ));
        }
        let mut transaction = self.pool.begin().await?;
        sqlx::query(
            r#"
            INSERT INTO ai_provider_secret_refs (provider_id, secret_purpose, secret_ref, updated_at)
            VALUES ($1, $2, $3, now())
            ON CONFLICT (provider_id, secret_purpose)
            DO UPDATE SET
                secret_ref = EXCLUDED.secret_ref,
                updated_at = now()
            "#,
        )
        .bind(provider_id.trim())
        .bind(SECRET_PURPOSE_API_KEY)
        .bind(secret_ref.trim())
        .execute(&mut *transaction)
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
        .execute(&mut *transaction)
        .await?;
        capture_provider_secret_binding_observation(
            &mut transaction,
            provider_id.trim(),
            SECRET_PURPOSE_API_KEY,
            secret_ref.trim(),
            "ai_control_center.bind_api_key_secret",
        )
        .await?;
        transaction.commit().await?;
        Ok(())
    }
}
