use serde_json::{Value, json};

use super::errors::AiControlCenterError;
use super::evidence::capture_model_catalog_item_observation;
use super::models::{AiModelAvailabilityUpdateRequest, AiModelCatalogItem, AiProviderAccount};
use super::presets::curated_models_for;
use super::rows::row_to_model;
use super::store::AiControlCenterStore;
use super::validation::validate_non_empty;

#[derive(Debug)]
pub(super) struct DiscoveredModel {
    pub(super) model_key: String,
    pub(super) display_name: String,
    pub(super) category: String,
    pub(super) privacy: String,
    pub(super) capabilities: Vec<String>,
    pub(super) context_window: Option<i32>,
    pub(super) embedding_dimension: Option<i32>,
    pub(super) metadata: Value,
}

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

    pub async fn update_model_availability(
        &self,
        request: &AiModelAvailabilityUpdateRequest,
        actor: &str,
    ) -> Result<AiModelCatalogItem, AiControlCenterError> {
        validate_non_empty("provider_id", &request.provider_id)?;
        validate_non_empty("model_key", &request.model_key)?;

        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE ai_model_catalog
            SET is_available = $3,
                updated_at = now()
            WHERE provider_id = $1 AND model_key = $2
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
        .bind(request.provider_id.trim())
        .bind(request.model_key.trim())
        .bind(request.is_available)
        .fetch_optional(&mut *transaction)
        .await?;

        let row = row.ok_or(AiControlCenterError::ModelNotFound)?;
        if !request.is_available {
            sqlx::query(
                r#"
                DELETE FROM ai_model_routes
                WHERE provider_id = $1 AND model_key = $2
                "#,
            )
            .bind(request.provider_id.trim())
            .bind(request.model_key.trim())
            .execute(&mut *transaction)
            .await?;
        }

        let model = row_to_model(row)?;
        capture_model_catalog_item_observation(
            &mut transaction,
            &model,
            if request.is_available {
                "availability_enabled"
            } else {
                "availability_disabled"
            },
            actor,
        )
        .await?;
        transaction.commit().await?;
        Ok(model)
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

    pub(super) async fn upsert_discovered_models_for_provider(
        &self,
        provider: &AiProviderAccount,
        models: &[DiscoveredModel],
        actor: &str,
    ) -> Result<usize, AiControlCenterError> {
        let mut transaction = self.pool.begin().await?;
        let mut synced = 0usize;
        for model in models {
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
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, false, $9, now(), now())
                ON CONFLICT (provider_id, model_key)
                DO UPDATE SET
                    display_name = EXCLUDED.display_name,
                    category = EXCLUDED.category,
                    privacy = EXCLUDED.privacy,
                    capabilities = EXCLUDED.capabilities,
                    context_window = EXCLUDED.context_window,
                    embedding_dimension = EXCLUDED.embedding_dimension,
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
            .bind(&model.model_key)
            .bind(&model.display_name)
            .bind(&model.category)
            .bind(&model.privacy)
            .bind(json!(model.capabilities))
            .bind(model.context_window)
            .bind(model.embedding_dimension)
            .bind(&model.metadata)
            .fetch_one(&mut *transaction)
            .await?;
            let model = row_to_model(row)?;
            capture_model_catalog_item_observation(&mut transaction, &model, "sync", actor).await?;
            synced += 1;
        }
        transaction.commit().await?;
        Ok(synced)
    }
}
