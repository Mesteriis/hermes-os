use serde_json::json;

use crate::domains::mail::core::{CommunicationIngestionStore, NewRawCommunicationRecord};
use crate::domains::mail::messages::MessageProjectionStore;
use crate::vault::CommunicationProviderAccountStore;

use super::WhatsappWebStore;
use crate::integrations::whatsapp::client::constants::WHATSAPP_WEB_MESSAGE_RECORD_KIND;
use crate::integrations::whatsapp::client::errors::WhatsappWebError;
use crate::integrations::whatsapp::client::ids::whatsapp_web_raw_record_id;
use crate::integrations::whatsapp::client::models::{
    NewWhatsappWebMessage, WhatsappWebLinkState, WhatsappWebMessageIngestResult,
};
use crate::integrations::whatsapp::client::projection::project_raw_whatsapp_web_message;

impl WhatsappWebStore {
    pub async fn ingest_fixture_message(
        &self,
        message: &NewWhatsappWebMessage,
    ) -> Result<WhatsappWebMessageIngestResult, WhatsappWebError> {
        message.validate()?;
        let communication_store = CommunicationIngestionStore::new(self.pool.clone());
        let provider_account = CommunicationProviderAccountStore::new(self.pool.clone())
            .get(&message.account_id)
            .await?
            .ok_or_else(|| {
                WhatsappWebError::InvalidRequest(format!(
                    "WhatsApp Web account `{}` is not configured",
                    message.account_id
                ))
            })?;
        if !provider_account.provider_kind.is_whatsapp() {
            return Err(WhatsappWebError::InvalidRequest(format!(
                "account `{}` is not a WhatsApp Web provider account",
                message.account_id
            )));
        }

        let session = self
            .list_sessions(Some(&message.account_id), 1)
            .await?
            .into_iter()
            .next()
            .ok_or_else(|| {
                WhatsappWebError::InvalidRequest(format!(
                    "WhatsApp Web account `{}` has no session metadata",
                    message.account_id
                ))
            })?;
        if session.link_state == WhatsappWebLinkState::Blocked.as_str() {
            return Err(WhatsappWebError::InvalidRequest(
                "blocked WhatsApp Web sessions cannot ingest fixture messages".to_owned(),
            ));
        }

        let raw_record_id = whatsapp_web_raw_record_id(
            &message.account_id,
            WHATSAPP_WEB_MESSAGE_RECORD_KIND,
            &message.provider_message_id,
        );
        let raw = NewRawCommunicationRecord::new(
            &raw_record_id,
            &message.account_id,
            WHATSAPP_WEB_MESSAGE_RECORD_KIND,
            &message.provider_message_id,
            message.source_fingerprint(),
            &message.import_batch_id,
            json!({
                "provider_chat_id": message.provider_chat_id,
                "chat_title": message.chat_title,
                "sender_id": message.sender_id,
                "sender_display_name": message.sender_display_name,
                "text": message.text,
                "delivery_state": message.delivery_state.as_str(),
            }),
        )
        .occurred_at(message.occurred_at)
        .provenance(json!({
            "provider": "whatsapp_web",
            "provider_kind": provider_account.provider_kind.as_str(),
            "runtime": session.companion_runtime,
            "account_id": message.account_id,
            "provider_chat_id": message.provider_chat_id,
        }));
        let raw = communication_store.record_raw_source(&raw).await?;
        let projected =
            project_raw_whatsapp_web_message(&MessageProjectionStore::new(self.pool.clone()), &raw)
                .await?;
        self.refresh_message_intelligence_candidates(&projected.message_id)
            .await?;

        self.update_session_last_sync(&message.account_id, message.occurred_at)
            .await?;

        Ok(WhatsappWebMessageIngestResult {
            raw_record_id: raw.raw_record_id,
            message_id: projected.message_id,
        })
    }
}
