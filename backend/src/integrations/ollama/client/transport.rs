use serde::Deserialize;
use serde_json::Value;

use super::OllamaClient;
use super::error::OllamaError;

impl OllamaClient {
    pub(in crate::integrations::ollama::client) fn endpoint(
        &self,
        path: &str,
    ) -> Result<reqwest::Url, OllamaError> {
        self.base_url
            .join(path.trim_start_matches('/'))
            .map_err(|error| OllamaError::InvalidConfig(error.to_string()))
    }

    pub(in crate::integrations::ollama::client) async fn get_json<T>(
        &self,
        path: &str,
    ) -> Result<T, OllamaError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let response = self.http.get(self.endpoint(path)?).send().await?;
        decode_response(response).await
    }

    pub(in crate::integrations::ollama::client) async fn post_json<T>(
        &self,
        path: &str,
        body: &Value,
    ) -> Result<T, OllamaError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let response = self
            .http
            .post(self.endpoint(path)?)
            .json(body)
            .send()
            .await?;
        decode_response(response).await
    }
}

async fn decode_response<T>(response: reqwest::Response) -> Result<T, OllamaError>
where
    T: for<'de> Deserialize<'de>,
{
    let status = response.status();
    if !status.is_success() {
        return Err(OllamaError::Endpoint {
            status: status.as_u16(),
        });
    }

    response
        .json::<T>()
        .await
        .map_err(|error| OllamaError::Protocol(error.to_string()))
}
