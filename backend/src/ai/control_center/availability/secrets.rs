use super::super::errors::AiControlCenterError;
use super::super::store::AiControlCenterStore;
use super::super::validation::validate_non_empty;

const SECRET_PURPOSE_API_KEY: &str = "api_key";
const SECRET_KIND_API_TOKEN: &str = "api_token";
const SECRET_STORE_HOST_VAULT: &str = "host_vault";

impl AiControlCenterStore {
    pub(in crate::ai::control_center) async fn api_key_secret_configured(
        &self,
        provider_id: &str,
    ) -> Result<bool, AiControlCenterError> {
        validate_non_empty("provider_id", provider_id)?;
        let configured = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM ai_provider_secret_refs refs
                JOIN secret_references secrets ON secrets.secret_ref = refs.secret_ref
                WHERE refs.provider_id = $1
                    AND refs.secret_purpose = $2
                    AND secrets.secret_kind = $3
                    AND secrets.store_kind = $4
            )
            "#,
        )
        .bind(provider_id.trim())
        .bind(SECRET_PURPOSE_API_KEY)
        .bind(SECRET_KIND_API_TOKEN)
        .bind(SECRET_STORE_HOST_VAULT)
        .fetch_one(&self.pool)
        .await?;

        Ok(configured)
    }

    pub(in crate::ai::control_center) async fn api_key_secret_reference_is_host_vault(
        &self,
        secret_ref: &str,
    ) -> Result<bool, AiControlCenterError> {
        validate_non_empty("secret_ref", secret_ref)?;
        let configured = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM secret_references
                WHERE secret_ref = $1
                    AND secret_kind = $2
                    AND store_kind = $3
            )
            "#,
        )
        .bind(secret_ref.trim())
        .bind(SECRET_KIND_API_TOKEN)
        .bind(SECRET_STORE_HOST_VAULT)
        .fetch_one(&self.pool)
        .await?;

        Ok(configured)
    }
}
