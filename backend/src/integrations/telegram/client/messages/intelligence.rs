use crate::workflows::review_inbox::{
    refresh_message_decisions_into_review, refresh_message_task_candidates_into_review,
};

use super::super::errors::TelegramError;
use super::super::store::TelegramStore;

impl TelegramStore {
    pub(in crate::integrations::telegram::client::messages) async fn refresh_message_intelligence_candidates(
        &self,
        message_id: &str,
    ) -> Result<(), TelegramError> {
        let message_ids = vec![message_id.to_owned()];
        refresh_message_decisions_into_review(&self.pool, &message_ids).await?;
        refresh_message_task_candidates_into_review(&self.pool, &message_ids).await?;
        Ok(())
    }
}
