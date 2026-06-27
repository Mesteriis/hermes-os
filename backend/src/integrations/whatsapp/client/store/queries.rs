use super::WhatsappWebStore;
use crate::integrations::whatsapp::client::errors::WhatsappWebError;
use crate::integrations::whatsapp::client::models::WhatsappWebMessage;
use crate::integrations::whatsapp::client::rows::provider_channel_message_to_whatsapp_web_message;
use crate::integrations::whatsapp::client::validation::validate_limit;

const WHATSAPP_WEB_CHANNEL_KINDS: &[&str] = &["whatsapp_web", "whatsapp_business_cloud"];

impl WhatsappWebStore {
    pub async fn recent_messages(
        &self,
        account_id: Option<&str>,
        provider_chat_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<WhatsappWebMessage>, WhatsappWebError> {
        let limit = validate_limit(limit)?;
        let account_id = account_id.map(str::trim).filter(|value| !value.is_empty());
        let provider_chat_id = provider_chat_id
            .map(str::trim)
            .filter(|value| !value.is_empty());
        Ok(self
            .provider_channel_message_store()
            .recent_messages(
                account_id,
                provider_chat_id,
                WHATSAPP_WEB_CHANNEL_KINDS,
                limit,
            )
            .await?
            .into_iter()
            .map(provider_channel_message_to_whatsapp_web_message)
            .collect())
    }
}
