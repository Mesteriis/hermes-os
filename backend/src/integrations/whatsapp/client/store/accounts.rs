use serde_json::json;

use crate::domains::mail::core::{CommunicationProviderKind, NewProviderAccount};
use crate::vault::CommunicationProviderAccountStore;

use super::WhatsappWebStore;
use crate::integrations::whatsapp::client::errors::WhatsappWebError;
use crate::integrations::whatsapp::client::ids::whatsapp_web_session_id;
use crate::integrations::whatsapp::client::models::{
    NewWhatsappWebSession, WhatsappWebAccountSetupRequest, WhatsappWebAccountSetupResponse,
    WhatsappWebCompanionRuntime, WhatsappWebLinkState,
};

impl WhatsappWebStore {
    pub async fn setup_fixture_account(
        &self,
        request: &WhatsappWebAccountSetupRequest,
    ) -> Result<WhatsappWebAccountSetupResponse, WhatsappWebError> {
        request.validate()?;
        if request.provider_kind != CommunicationProviderKind::WhatsappWeb {
            return Err(WhatsappWebError::InvalidRequest(
                "provider_kind must be whatsapp_web".to_owned(),
            ));
        }

        let account = NewProviderAccount::new(
            &request.account_id,
            request.provider_kind,
            &request.display_name,
            &request.external_account_id,
        )
        .config(json!({
            "runtime": "fixture",
            "local_state_path": request.local_state_path,
            "device_name": request.device_name,
        }));
        let stored_account = CommunicationProviderAccountStore::new(self.pool.clone())
            .upsert(&account)
            .await?;

        let session = self
            .upsert_session(&NewWhatsappWebSession {
                session_id: whatsapp_web_session_id(&request.account_id),
                account_id: stored_account.account_id.clone(),
                device_name: request.device_name.clone(),
                companion_runtime: WhatsappWebCompanionRuntime::Fixture,
                link_state: WhatsappWebLinkState::Fixture,
                local_state_path: request.local_state_path.clone(),
                last_sync_at: None,
                metadata: json!({"runtime": "fixture"}),
            })
            .await?;

        Ok(WhatsappWebAccountSetupResponse {
            account_id: stored_account.account_id,
            provider_kind: stored_account.provider_kind.as_str().to_owned(),
            runtime: "fixture".to_owned(),
            session,
        })
    }
}
