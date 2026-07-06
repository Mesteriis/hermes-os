use super::super::errors::AiControlCenterError;
use super::super::models::{AiProviderCommandKind, AiProviderCommandResponse};
use super::super::provider_auth::local_provider_auth_probe;
use super::super::store::AiControlCenterStore;

impl AiControlCenterStore {
    pub async fn provider_command(
        &self,
        provider_id: &str,
        command: AiProviderCommandKind,
    ) -> Result<AiProviderCommandResponse, AiControlCenterError> {
        let provider = self
            .provider(provider_id)
            .await?
            .ok_or(AiControlCenterError::ProviderNotFound)?;
        let (status, message) = match command {
            AiProviderCommandKind::Test => match provider.provider_kind.as_str() {
                "built_in" => (
                    "ok".to_owned(),
                    "Built-in runtime metadata is configured".to_owned(),
                ),
                "cli" => {
                    let probe =
                        local_provider_auth_probe(&provider.provider_kind, &provider.provider_key)
                            .await?;
                    if probe.authenticated {
                        (
                            "ok".to_owned(),
                            "CLI provider is installed and authenticated".to_owned(),
                        )
                    } else {
                        ("needs_setup".to_owned(), probe.message)
                    }
                }
                "api" => {
                    if provider.status == "disabled" {
                        ("disabled".to_owned(), "API provider is disabled".to_owned())
                    } else if provider.consent_state != "granted" {
                        (
                            "needs_consent".to_owned(),
                            "API provider requires remote-context consent".to_owned(),
                        )
                    } else if !self
                        .api_key_secret_configured(&provider.provider_id)
                        .await?
                    {
                        (
                            "needs_setup".to_owned(),
                            "API provider requires a host-vault API key".to_owned(),
                        )
                    } else if provider.status != "ready" {
                        (
                            "needs_setup".to_owned(),
                            "API provider setup is incomplete".to_owned(),
                        )
                    } else {
                        (
                            "ok".to_owned(),
                            "API provider consent and host-vault key reference are configured; live network check is deferred".to_owned(),
                        )
                    }
                }
                _ => ("error".to_owned(), "Unsupported provider kind".to_owned()),
            },
            AiProviderCommandKind::SyncModels => {
                self.seed_models_for_provider(
                    &provider,
                    "ai_control_center.provider_command.sync_models",
                )
                .await?;
                (
                    "synced".to_owned(),
                    "Curated model catalog synchronized".to_owned(),
                )
            }
        };

        Ok(AiProviderCommandResponse {
            provider_id: provider.provider_id,
            command: command.as_str().to_owned(),
            status,
            message,
        })
    }
}
