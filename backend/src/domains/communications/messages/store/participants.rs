use serde_json::json;
use sqlx::Row;

use crate::domains::communications::evidence::link_mail_entity_in_transaction;
use crate::domains::communications::messages::{
    MessageProjectionError, MessageProjectionStore, ProjectedMessage,
};

impl MessageProjectionStore {
    pub async fn upsert_email_participant(
        &self,
        message: &ProjectedMessage,
        person_id: &str,
        email_address: &str,
        display_name: Option<&str>,
        role: &str,
    ) -> Result<bool, MessageProjectionError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            INSERT INTO communication_message_participants (
                message_id, person_id, email_address, display_name, role, source, confidence
            )
            VALUES ($1, $2, $3, $4, $5, 'email_sync', 1.0)
            ON CONFLICT (message_id, person_id, role, email_address)
            DO UPDATE SET
                display_name = EXCLUDED.display_name,
                source = EXCLUDED.source,
                confidence = EXCLUDED.confidence,
                updated_at = now()
            RETURNING id::text AS participant_id, (xmax = 0) AS inserted
            "#,
        )
        .bind(&message.message_id)
        .bind(person_id)
        .bind(email_address)
        .bind(display_name)
        .bind(role)
        .fetch_one(&mut *transaction)
        .await?;
        let participant_id: String = row.try_get("participant_id")?;
        let inserted: bool = row.try_get("inserted")?;

        link_mail_entity_in_transaction(
            &mut transaction,
            &message.observation_id,
            "message_participant",
            participant_id,
            "email_sync_participant",
            json!({
                "message_id": message.message_id,
                "person_id": person_id,
                "email_address": email_address,
                "display_name": display_name,
                "role": role,
                "source": "email_sync",
            }),
            None,
        )
        .await?;

        transaction.commit().await?;
        Ok(inserted)
    }
}
