use serde::Deserialize;
use serde_json::json;

use super::models::OmniRouteEmbedResult;
use super::{OmniRouteClient, OmniRouteError};

impl OmniRouteClient {
    pub async fn embed(&self, input: &str) -> Result<OmniRouteEmbedResult, OmniRouteError> {
        self.embed_with_model(input, &self.embed_model).await
    }

    pub async fn embed_with_model(
        &self,
        input: &str,
        model: &str,
    ) -> Result<OmniRouteEmbedResult, OmniRouteError> {
        if model.trim().is_empty() {
            return Err(OmniRouteError::InvalidConfig(
                "embedding model is empty".to_owned(),
            ));
        }
        let body = json!({
            "model": model,
            "input": input,
        });
        let response: EmbeddingsResponse = self.post_json("embeddings", &body).await?;
        let embedding = response
            .data
            .into_iter()
            .next()
            .map(|item| item.embedding)
            .ok_or_else(|| {
                OmniRouteError::Protocol("OmniRoute embeddings response omitted data".to_owned())
            })?;
        if embedding.is_empty() {
            return Err(OmniRouteError::Protocol(
                "OmniRoute embeddings response returned an empty vector".to_owned(),
            ));
        }

        Ok(OmniRouteEmbedResult {
            model: response.model.unwrap_or_else(|| model.to_owned()),
            embedding,
        })
    }
}

#[derive(Deserialize)]
struct EmbeddingsResponse {
    model: Option<String>,
    data: Vec<EmbeddingItem>,
}

#[derive(Deserialize)]
struct EmbeddingItem {
    embedding: Vec<f32>,
}
