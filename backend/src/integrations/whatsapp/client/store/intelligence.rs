use crate::domains::decisions::DecisionStore;
use crate::domains::tasks::candidates::TaskCandidateStore;

use super::WhatsappWebStore;
use crate::integrations::whatsapp::client::errors::WhatsappWebError;

impl WhatsappWebStore {
    pub(in crate::integrations::whatsapp::client::store) async fn refresh_message_intelligence_candidates(
        &self,
        message_id: &str,
    ) -> Result<(), WhatsappWebError> {
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
