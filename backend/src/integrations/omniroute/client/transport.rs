use reqwest::Url;
use serde::Deserialize;
use serde_json::Value;

use super::{OmniRouteClient, OmniRouteError};

impl OmniRouteClient {
    fn endpoint(&self, path: &str) -> Result<Url, OmniRouteError> {
        self.base_url
            .join(path.trim_start_matches('/'))
            .map_err(|error| OmniRouteError::InvalidConfig(error.to_string()))
    }

    pub(super) async fn get_json<T>(&self, path: &str) -> Result<T, OmniRouteError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let response = self
            .http
            .get(self.endpoint(path)?)
            .bearer_auth(self.api_key.expose_for_runtime())
            .send()
            .await?;
        decode_response(response).await
    }

    pub(super) async fn post_json<T>(&self, path: &str, body: &Value) -> Result<T, OmniRouteError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let response = self
            .http
            .post(self.endpoint(path)?)
            .bearer_auth(self.api_key.expose_for_runtime())
            .json(body)
            .send()
            .await?;
        decode_response(response).await
    }
}

async fn decode_response<T>(response: reqwest::Response) -> Result<T, OmniRouteError>
where
    T: for<'de> Deserialize<'de>,
{
    let status = response.status();
    if !status.is_success() {
        return Err(OmniRouteError::Endpoint {
            status: status.as_u16(),
        });
    }

    response
        .json::<T>()
        .await
        .map_err(|error| OmniRouteError::Protocol(error.to_string()))
}
