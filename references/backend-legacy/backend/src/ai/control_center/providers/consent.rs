use super::super::errors::AiControlCenterError;
use super::super::evidence::capture_provider_account_observation;
use super::super::models::{AiProviderAccount, AiProviderConsentRequest};
use super::super::rows::row_to_provider;
use super::super::store::AiControlCenterStore;

impl AiControlCenterStore {
    pub async fn record_consent(
        &self,
        provider_id: &str,
        request: &AiProviderConsentRequest,
    ) -> Result<AiProviderAccount, AiControlCenterError> {
        let provider = self
            .provider(provider_id)
            .await?
            .ok_or(AiControlCenterError::ProviderNotFound)?;
        if provider.provider_kind != "api" {
            return Err(AiControlCenterError::InvalidRequest(
                "Remote-context consent applies only to API providers".to_owned(),
            ));
        }
        let consent_state = if request.consented {
            "granted"
        } else {
            "revoked"
        };
        let mut transaction = self.pool.begin().await?;
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
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(AiControlCenterError::ProviderNotFound)?;

        let provider = row_to_provider(row)?;
        capture_provider_account_observation(
            &mut transaction,
            &provider,
            "consent_recorded",
            "ai_control_center.record_consent",
        )
        .await?;
        transaction.commit().await?;
        Ok(provider)
    }
}
