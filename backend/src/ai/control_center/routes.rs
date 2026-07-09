use crate::ai::core::AI_EMBEDDING_DIMENSION;

use super::errors::AiControlCenterError;
use super::evidence::capture_model_route_observation;
use super::models::{AiModelRoute, AiModelRouteUpdateRequest};
use super::rows::row_to_route;
use super::store::AiControlCenterStore;
use super::validation::{validate_capability_slot, validate_non_empty};

impl AiControlCenterStore {
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
            .ensure_model_ready_for_private_context(&request.provider_id, &request.model_key)
            .await?;
        if slot == "embeddings" && model.embedding_dimension != Some(AI_EMBEDDING_DIMENSION as i32)
        {
            return Err(AiControlCenterError::InvalidRequest(
                "embedding route requires a 2560-dimension model".to_owned(),
            ));
        }
        let mut transaction = self.pool.begin().await?;
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
        .fetch_one(&mut *transaction)
        .await?;

        let route = row_to_route(row)?;
        capture_model_route_observation(
            &mut transaction,
            &route,
            "ai_control_center.put_model_route",
        )
        .await?;
        transaction.commit().await?;
        Ok(route)
    }

    pub async fn delete_model_route(&self, slot: &str) -> Result<(), AiControlCenterError> {
        validate_capability_slot(slot)?;
        sqlx::query(
            r#"
            DELETE FROM ai_model_routes
            WHERE capability_slot = $1
            "#,
        )
        .bind(slot.trim())
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
