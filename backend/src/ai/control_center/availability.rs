use super::errors::AiControlCenterError;
use super::models::{AiModelCatalogItem, AiProviderAccount};
use super::store::AiControlCenterStore;
use super::validation::validate_non_empty;

mod secrets;

impl AiControlCenterStore {
    pub async fn ensure_model_ready_for_private_context(
        &self,
        provider_id: &str,
        model_key: &str,
    ) -> Result<AiModelCatalogItem, AiControlCenterError> {
        validate_non_empty("provider_id", provider_id)?;
        validate_non_empty("model_key", model_key)?;
        let model = self
            .model(provider_id, model_key)
            .await?
            .ok_or(AiControlCenterError::ModelNotFound)?;
        if !model.is_available {
            return Err(AiControlCenterError::InvalidRequest(
                "AI model is unavailable".to_owned(),
            ));
        }
        let provider = self
            .provider(provider_id)
            .await?
            .ok_or(AiControlCenterError::ProviderNotFound)?;
        self.ensure_provider_ready_for_private_context(&provider)
            .await?;
        Ok(model)
    }

    pub async fn model_ready_for_private_context(
        &self,
        provider_id: &str,
        model_key: &str,
    ) -> Result<bool, AiControlCenterError> {
        match self
            .ensure_model_ready_for_private_context(provider_id, model_key)
            .await
        {
            Ok(_) => Ok(true),
            Err(
                AiControlCenterError::InvalidRequest(_)
                | AiControlCenterError::ModelNotFound
                | AiControlCenterError::ProviderNotFound,
            ) => Ok(false),
            Err(error) => Err(error),
        }
    }

    async fn ensure_provider_ready_for_private_context(
        &self,
        provider: &AiProviderAccount,
    ) -> Result<(), AiControlCenterError> {
        if provider.status == "disabled" {
            return Err(AiControlCenterError::InvalidRequest(
                "AI provider is disabled".to_owned(),
            ));
        }
        if provider.provider_kind != "api" {
            return provider_ready_status(&provider.status);
        }
        if provider.consent_state != "granted" {
            return Err(AiControlCenterError::InvalidRequest(
                "API provider requires remote-context consent before private-context use"
                    .to_owned(),
            ));
        }
        if !self
            .api_key_secret_configured(&provider.provider_id)
            .await?
        {
            return Err(AiControlCenterError::InvalidRequest(
                "API provider requires a host-vault API key before private-context use".to_owned(),
            ));
        }
        provider_ready_status(&provider.status)
    }
}

fn provider_ready_status(status: &str) -> Result<(), AiControlCenterError> {
    if status == "ready" {
        return Ok(());
    }
    Err(AiControlCenterError::InvalidRequest(
        "AI provider setup is incomplete".to_owned(),
    ))
}
