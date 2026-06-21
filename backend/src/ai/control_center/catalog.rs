use serde_json::json;

use super::errors::AiControlCenterError;
use super::evidence::capture_model_catalog_item_observation;
use super::models::{AiModelCatalogItem, AiProviderAccount};
use super::presets::curated_models_for;
use super::rows::row_to_model;
use super::store::AiControlCenterStore;
use super::validation::validate_non_empty;

impl AiControlCenterStore {
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

    pub(super) async fn seed_models_for_provider(
        &self,
        provider: &AiProviderAccount,
        actor: &str,
    ) -> Result<(), AiControlCenterError> {
        let mut transaction = self.pool.begin().await?;
        for model in curated_models_for(provider) {
            let row = sqlx::query(
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
                    created_at,
                    updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true, $9, now(), now())
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
                RETURNING
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
            .fetch_one(&mut *transaction)
            .await?;
            let model = row_to_model(row)?;
            capture_model_catalog_item_observation(&mut transaction, &model, "seed", actor).await?;
        }
        transaction.commit().await?;
        Ok(())
    }
}
