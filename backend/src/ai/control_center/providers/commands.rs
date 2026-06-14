use super::super::errors::AiControlCenterError;
use super::super::models::{AiProviderCommandKind, AiProviderCommandResponse};
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
                "built_in" => ("ok", "Built-in runtime metadata is configured"),
                "cli" => ("ok", "CLI provider preset is allowlisted"),
                "api" => {
                    if provider.status == "disabled" {
                        ("disabled", "API provider is disabled")
                    } else if provider.consent_state != "granted" {
                        (
                            "needs_consent",
                            "API provider requires remote-context consent",
                        )
                    } else if !self
                        .api_key_secret_configured(&provider.provider_id)
                        .await?
                    {
                        ("needs_setup", "API provider requires a host-vault API key")
                    } else if provider.status != "ready" {
                        ("needs_setup", "API provider setup is incomplete")
                    } else {
                        (
                            "ok",
                            "API provider consent and host-vault key reference are configured; live network check is deferred",
                        )
                    }
                }
                _ => ("error", "Unsupported provider kind"),
            },
            AiProviderCommandKind::SyncModels => {
                self.seed_models_for_provider(&provider).await?;
                ("synced", "Curated model catalog synchronized")
            }
        };

        Ok(AiProviderCommandResponse {
            provider_id: provider.provider_id,
            command: command.as_str().to_owned(),
            status: status.to_owned(),
            message: message.to_owned(),
        })
    }
}
