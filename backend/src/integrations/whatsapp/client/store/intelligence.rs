use super::WhatsappWebStore;
use crate::integrations::whatsapp::client::errors::WhatsappWebError;

impl WhatsappWebStore {
    pub(in crate::integrations::whatsapp::client::store) async fn refresh_message_intelligence_candidates(
        &self,
        _message_id: &str,
    ) -> Result<(), WhatsappWebError> {
        Ok(())
    }
}
