use hermes_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use serde_json::json;

use super::WhatsappWebStore;
use crate::integrations::whatsapp::client::errors::WhatsappWebError;
use crate::integrations::whatsapp::client::ids::whatsapp_web_session_id;
use crate::integrations::whatsapp::client::models::{
    NewWhatsappWebSession, WhatsappLiveAccountSetupRequest, WhatsappWebAccountSetupRequest,
    WhatsappWebAccountSetupResponse, WhatsappWebCompanionRuntime, WhatsappWebLinkState,
};

impl WhatsappWebStore {
    pub async fn setup_fixture_account(
        &self,
        request: &WhatsappWebAccountSetupRequest,
    ) -> Result<WhatsappWebAccountSetupResponse, WhatsappWebError> {
        request.validate()?;
        if !request.provider_kind.is_whatsapp() {
            return Err(WhatsappWebError::InvalidRequest(
                "provider_kind must be a WhatsApp provider".to_owned(),
            ));
        }
        let provider_shape = normalize_fixture_provider_shape(
            request.provider_kind,
            request.provider_shape.as_deref(),
        )?;
        let session_mode = fixture_session_mode(request.provider_kind);
        let setup_semantics = fixture_setup_semantics(request.provider_kind);

        let account = NewProviderAccount::new(
            &request.account_id,
            request.provider_kind,
            &request.display_name,
            &request.external_account_id,
        )
        .config(json!({
            "runtime": "fixture",
            "provider_shape": provider_shape,
            "local_state_path": request.local_state_path,
            "device_name": request.device_name,
            "lifecycle_state": "created",
            "setup_semantics": setup_semantics,
        }));
        let stored_account = self
            .provider_account_store()
            .upsert(&account)
            .await
            .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?;

        let session = self
            .upsert_session(&NewWhatsappWebSession {
                session_id: whatsapp_web_session_id(&request.account_id),
                account_id: stored_account.account_id.clone(),
                device_name: request.device_name.clone(),
                companion_runtime: fixture_companion_runtime(request.provider_kind),
                link_state: WhatsappWebLinkState::Fixture,
                local_state_path: request.local_state_path.clone(),
                last_sync_at: None,
                metadata: json!({
                    "runtime": "fixture",
                    "provider_shape": provider_shape,
                    "setup_semantics": setup_semantics,
                    "session_mode": session_mode,
                }),
            })
            .await?;

        Ok(WhatsappWebAccountSetupResponse {
            account_id: stored_account.account_id,
            provider_kind: stored_account.provider_kind.as_str().to_owned(),
            runtime: "fixture".to_owned(),
            session,
        })
    }

    pub async fn setup_live_blocked_account(
        &self,
        request: &WhatsappLiveAccountSetupRequest,
    ) -> Result<WhatsappWebAccountSetupResponse, WhatsappWebError> {
        request.validate()?;
        let provider_shape = normalize_provider_shape(&request.provider_shape)?;
        validate_live_provider_kind(request.provider_kind, provider_shape)?;
        let device_name = default_live_device_name(provider_shape, request.device_name.clone());
        let local_state_path = default_live_local_state_path(
            provider_shape,
            &request.account_id,
            request.local_state_path.clone(),
        );

        let account = NewProviderAccount::new(
            &request.account_id,
            request.provider_kind,
            &request.display_name,
            &request.external_account_id,
        )
        .config(json!({
            "runtime": "live_blocked",
            "provider_shape": provider_shape,
            "local_state_path": local_state_path,
            "device_name": device_name,
            "lifecycle_state": "created",
            "setup_semantics": live_setup_semantics(provider_shape),
        }));
        let stored_account = self
            .provider_account_store()
            .upsert(&account)
            .await
            .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?;

        let session = self
            .upsert_session(&NewWhatsappWebSession {
                session_id: whatsapp_web_session_id(&request.account_id),
                account_id: stored_account.account_id.clone(),
                device_name,
                companion_runtime: live_companion_runtime(provider_shape),
                link_state: WhatsappWebLinkState::Blocked,
                local_state_path,
                last_sync_at: None,
                metadata: json!({
                    "runtime": "live_blocked",
                    "provider_shape": provider_shape,
                    "setup_semantics": live_setup_semantics(provider_shape),
                    "session_mode": live_session_mode(provider_shape),
                }),
            })
            .await?;

        Ok(WhatsappWebAccountSetupResponse {
            account_id: stored_account.account_id,
            provider_kind: stored_account.provider_kind.as_str().to_owned(),
            runtime: "live_blocked".to_owned(),
            session,
        })
    }
}

fn normalize_provider_shape(input: &str) -> Result<&str, WhatsappWebError> {
    let normalized = input.trim();
    match normalized {
        "whatsapp_web_companion" | "whatsapp_native_md" | "whatsapp_business_cloud" => {
            Ok(normalized)
        }
        _ => Err(WhatsappWebError::InvalidRequest(format!(
            "unsupported WhatsApp provider_shape `{input}`"
        ))),
    }
}

fn normalize_fixture_provider_shape(
    provider_kind: CommunicationProviderKind,
    requested_shape: Option<&str>,
) -> Result<&'static str, WhatsappWebError> {
    match requested_shape {
        Some(input) => {
            let normalized = normalize_provider_shape(input)?;
            validate_live_provider_kind(provider_kind, normalized)?;
            Ok(match normalized {
                "whatsapp_web_companion" => "whatsapp_web_companion",
                "whatsapp_native_md" => "whatsapp_native_md",
                "whatsapp_business_cloud" => "whatsapp_business_cloud",
                _ => unreachable!("normalize_provider_shape returned unsupported value"),
            })
        }
        None => Ok(fixture_provider_shape(provider_kind)),
    }
}

fn validate_live_provider_kind(
    provider_kind: CommunicationProviderKind,
    provider_shape: &str,
) -> Result<(), WhatsappWebError> {
    let expected_kind = match provider_shape {
        "whatsapp_business_cloud" => CommunicationProviderKind::WhatsappBusinessCloud,
        _ => CommunicationProviderKind::WhatsappWeb,
    };
    if provider_kind != expected_kind {
        return Err(WhatsappWebError::InvalidRequest(format!(
            "provider_kind `{}` is invalid for provider_shape `{provider_shape}`; expected `{}`",
            provider_kind.as_str(),
            expected_kind.as_str(),
        )));
    }
    Ok(())
}

fn default_live_device_name(provider_shape: &str, request_value: Option<String>) -> String {
    match provider_shape {
        "whatsapp_business_cloud" => "WhatsApp Business Cloud API".to_owned(),
        _ => request_value.unwrap_or_else(|| format!("{provider_shape} blocked runtime")),
    }
}

fn default_live_local_state_path(
    provider_shape: &str,
    account_id: &str,
    request_value: Option<String>,
) -> String {
    request_value.unwrap_or_else(|| match provider_shape {
        "whatsapp_business_cloud" => {
            format!("docker/data/whatsapp/business-cloud/{account_id}")
        }
        _ => format!("docker/data/whatsapp/blocked/{account_id}"),
    })
}

fn live_setup_semantics(provider_shape: &str) -> &'static str {
    match provider_shape {
        "whatsapp_business_cloud" => "business_cloud",
        _ => "personal_runtime",
    }
}

fn live_session_mode(provider_shape: &str) -> &'static str {
    match provider_shape {
        "whatsapp_business_cloud" => "api_credentials",
        _ => "device_session",
    }
}

fn live_companion_runtime(provider_shape: &str) -> WhatsappWebCompanionRuntime {
    match provider_shape {
        "whatsapp_business_cloud" => WhatsappWebCompanionRuntime::ApiCredentials,
        _ => WhatsappWebCompanionRuntime::Blocked,
    }
}

fn fixture_provider_shape(provider_kind: CommunicationProviderKind) -> &'static str {
    match provider_kind {
        CommunicationProviderKind::WhatsappBusinessCloud => "whatsapp_business_cloud",
        CommunicationProviderKind::WhatsappWeb => "whatsapp_web_companion",
        _ => "whatsapp_web_companion",
    }
}

fn fixture_setup_semantics(provider_kind: CommunicationProviderKind) -> &'static str {
    match provider_kind {
        CommunicationProviderKind::WhatsappBusinessCloud => "business_cloud",
        CommunicationProviderKind::WhatsappWeb => "personal_runtime",
        _ => "personal_runtime",
    }
}

fn fixture_session_mode(provider_kind: CommunicationProviderKind) -> &'static str {
    match provider_kind {
        CommunicationProviderKind::WhatsappBusinessCloud => "api_credentials",
        CommunicationProviderKind::WhatsappWeb => "device_session",
        _ => "device_session",
    }
}

fn fixture_companion_runtime(
    provider_kind: CommunicationProviderKind,
) -> WhatsappWebCompanionRuntime {
    match provider_kind {
        CommunicationProviderKind::WhatsappBusinessCloud => {
            WhatsappWebCompanionRuntime::ApiCredentials
        }
        CommunicationProviderKind::WhatsappWeb => WhatsappWebCompanionRuntime::Fixture,
        _ => WhatsappWebCompanionRuntime::Fixture,
    }
}
