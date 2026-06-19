use crate::workflows::review_inbox::{
    refresh_message_decisions_into_review, refresh_message_task_candidates_into_review,
};

use super::WhatsappWebStore;
use crate::integrations::whatsapp::client::errors::WhatsappWebError;

impl WhatsappWebStore {
    pub(in crate::integrations::whatsapp::client::store) async fn refresh_message_intelligence_candidates(
        &self,
        message_id: &str,
    ) -> Result<(), WhatsappWebError> {
        let message_ids = vec![message_id.to_owned()];
        refresh_message_decisions_into_review(&self.pool, &message_ids).await?;
        refresh_message_task_candidates_into_review(&self.pool, &message_ids).await?;
        Ok(())
    }
}
