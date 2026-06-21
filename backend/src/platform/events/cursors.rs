use sqlx::postgres::PgPool;

use super::errors::EventStoreError;
use super::validation::validate_non_empty;

#[derive(Clone)]
pub struct ProjectionCursorStore {
    pool: PgPool,
}

impl ProjectionCursorStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn last_processed_position(
        &self,
        projection_name: &str,
    ) -> Result<i64, EventStoreError> {
        validate_non_empty("projection_name", projection_name)?;

        let position = sqlx::query_scalar::<_, Option<i64>>(
            r#"
            SELECT last_processed_position
            FROM projection_cursors
            WHERE projection_name = $1
            "#,
        )
        .bind(projection_name.trim())
        .fetch_optional(&self.pool)
        .await?;

        Ok(position.flatten().unwrap_or(0))
    }

    pub async fn save_position(
        &self,
        projection_name: &str,
        position: i64,
    ) -> Result<i64, EventStoreError> {
        validate_non_empty("projection_name", projection_name)?;
        if position < 0 {
            return Err(EventStoreError::InvalidReplayPosition(position));
        }

        let saved_position = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO projection_cursors (
                projection_name,
                last_processed_position,
                updated_at
            )
            VALUES ($1, $2, now())
            ON CONFLICT (projection_name)
            DO UPDATE SET
                last_processed_position = GREATEST(
                    projection_cursors.last_processed_position,
                    EXCLUDED.last_processed_position
                ),
                updated_at = now()
            RETURNING last_processed_position
            "#,
        )
        .bind(projection_name.trim())
        .bind(position)
        .fetch_one(&self.pool)
        .await?;

        Ok(saved_position)
    }
}
