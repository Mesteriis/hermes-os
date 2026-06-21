use sqlx::Row;

use crate::domains::decisions::{
    DecisionEngine, DecisionExtractionInput, DecisionExtractionResult,
};

use super::errors::DecisionStoreError;
use super::models::DecisionEntityKind;
use super::store::DecisionStore;
use super::validation::{preserve_existing_review_state, validate_refresh_limit};

impl DecisionStore {
    pub async fn refresh_deterministic_candidates(
        &self,
        limit: i64,
    ) -> Result<usize, DecisionStoreError> {
        let limit = validate_refresh_limit(limit)?;
        let message_count = self.refresh_message_candidates(limit).await?;
        let document_count = self.refresh_document_candidates(limit).await?;

        Ok(message_count + document_count)
    }

    pub async fn refresh_message_candidates_for_ids(
        &self,
        message_ids: &[String],
    ) -> Result<usize, DecisionStoreError> {
        if message_ids.is_empty() {
            return Ok(0);
        }

        let rows = sqlx::query(
            r#"
            SELECT
                message_id,
                observation_id,
                subject,
                body_text
            FROM communication_messages
            WHERE message_id = ANY($1)
            ORDER BY COALESCE(occurred_at, projected_at) DESC, message_id
            "#,
        )
        .bind(message_ids.to_vec())
        .fetch_all(&self.pool)
        .await?;

        let mut count = 0usize;
        for row in rows {
            let source_id = row.try_get::<String, _>("message_id")?;
            let observation_id = row.try_get::<Option<String>, _>("observation_id")?;
            let source_text = format!(
                "{}\n{}",
                row.try_get::<String, _>("subject")?,
                row.try_get::<String, _>("body_text")?,
            );
            count += self
                .refresh_communication_decision_candidates(&source_id, observation_id, &source_text)
                .await?;
        }

        Ok(count)
    }

    async fn refresh_message_candidates(&self, limit: i64) -> Result<usize, DecisionStoreError> {
        let rows = sqlx::query(
            r#"
            SELECT
                message_id,
                observation_id,
                subject,
                body_text
            FROM communication_messages
            ORDER BY COALESCE(occurred_at, projected_at) DESC, message_id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut count = 0usize;
        for row in rows {
            let source_id = row.try_get::<String, _>("message_id")?;
            let observation_id = row.try_get::<Option<String>, _>("observation_id")?;
            let source_text = format!(
                "{}\n{}",
                row.try_get::<String, _>("subject")?,
                row.try_get::<String, _>("body_text")?,
            );
            count += self
                .refresh_communication_decision_candidates(&source_id, observation_id, &source_text)
                .await?;
        }

        Ok(count)
    }

    async fn refresh_communication_decision_candidates(
        &self,
        source_id: &str,
        observation_id: Option<String>,
        source_text: &str,
    ) -> Result<usize, DecisionStoreError> {
        let input = DecisionExtractionInput::communication(
            source_id,
            source_text,
            DecisionEntityKind::Communication,
            source_id,
        )
        .with_observation_id(observation_id);
        let extraction = DecisionEngine::detect_candidates(&input)?;
        self.persist_decision_extraction(extraction).await
    }

    async fn refresh_document_candidates(&self, limit: i64) -> Result<usize, DecisionStoreError> {
        let rows = sqlx::query(
            r#"
            SELECT
                document_id,
                observation_id,
                title,
                extracted_text
            FROM documents
            ORDER BY imported_at DESC, document_id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut count = 0usize;
        for row in rows {
            let source_id = row.try_get::<String, _>("document_id")?;
            let observation_id = row.try_get::<Option<String>, _>("observation_id")?;
            let source_text = format!(
                "{}\n{}",
                row.try_get::<String, _>("title")?,
                row.try_get::<String, _>("extracted_text")?,
            );
            let input = DecisionExtractionInput::document(
                &source_id,
                &source_text,
                DecisionEntityKind::Document,
                &source_id,
            )
            .with_observation_id(observation_id);
            let extraction = DecisionEngine::detect_candidates(&input)?;
            count += self.persist_decision_extraction(extraction).await?;
        }

        Ok(count)
    }

    async fn persist_decision_extraction(
        &self,
        extraction: DecisionExtractionResult,
    ) -> Result<usize, DecisionStoreError> {
        let mut count = 0usize;
        for candidate in extraction.decisions {
            let (mut decision, evidence, impacted_entities) = candidate.to_decision_draft();
            preserve_existing_review_state(&self.pool, &mut decision).await?;
            self.upsert_with_evidence(&decision, &[evidence], &impacted_entities)
                .await?;
            count += 1;
        }

        Ok(count)
    }
}
