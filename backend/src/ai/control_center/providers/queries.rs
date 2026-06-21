use super::super::errors::AiControlCenterError;
use super::super::models::AiProviderAccount;
use super::super::rows::row_to_provider;
use super::super::store::AiControlCenterStore;
use super::super::validation::validate_non_empty;

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
}
