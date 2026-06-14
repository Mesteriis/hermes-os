use crate::domains::decisions::DecisionStore;
use crate::domains::tasks::candidates::TaskCandidateStore;

use super::super::errors::TelegramError;
use super::super::store::TelegramStore;

impl TelegramStore {
    pub(in crate::integrations::telegram::client::messages) async fn refresh_message_intelligence_candidates(
        &self,
        message_id: &str,
    ) -> Result<(), TelegramError> {
        let message_ids = vec![message_id.to_owned()];
        DecisionStore::new(self.pool.clone())
            .refresh_message_candidates_for_ids(&message_ids)
            .await?;
        TaskCandidateStore::new(self.pool.clone())
            .refresh_message_candidates_for_ids(&message_ids)
            .await?;
        Ok(())
    }
}
