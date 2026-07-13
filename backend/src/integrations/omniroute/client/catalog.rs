use serde::Deserialize;

use super::OmniRouteClient;
use super::error::OmniRouteError;

impl OmniRouteClient {
    pub async fn models(&self) -> Result<Vec<String>, OmniRouteError> {
        let response: ModelsResponse = self.get_json("models").await?;
        Ok(response
            .data
            .into_iter()
            .map(|model| model.id)
            .filter(|id| !id.trim().is_empty())
            .collect())
    }

    pub async fn validate_required_models(&self) -> Result<(), OmniRouteError> {
        let models = self.models().await?;
        for model in [&self.chat_model, &self.embed_model] {
            if !models.iter().any(|candidate| candidate == model) {
                return Err(OmniRouteError::MissingModel {
                    model: model.to_owned(),
                });
            }
        }
        Ok(())
    }
}

#[derive(Deserialize)]
struct ModelsResponse {
    data: Vec<ModelItem>,
}

#[derive(Deserialize)]
struct ModelItem {
    id: String,
}
