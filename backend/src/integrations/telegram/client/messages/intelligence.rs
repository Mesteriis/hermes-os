use super::super::errors::TelegramError;
use super::super::store::TelegramStore;

impl TelegramStore {
    pub(in crate::integrations::telegram::client::messages) async fn refresh_message_intelligence_candidates(
        &self,
        _message_id: &str,
    ) -> Result<(), TelegramError> {
        Ok(())
    }
}
