use reqwest::header::{ACCEPT, CONTENT_TYPE};
use url::form_urlencoded::byte_serialize;

use super::super::models::ZoomWebhookSubscription;
use super::api_models::{
    ZoomApiEventSubscription, ZoomApiEventSubscriptionListResponse, ZoomApiRecordingListResponse,
};
use super::{ZoomError, ZoomStore};

impl ZoomStore {
    #[allow(clippy::too_many_arguments)]
    pub(super) async fn fetch_zoom_recordings_page(
        &self,
        api_base_url: &str,
        user_id: &str,
        from: &str,
        to: &str,
        page_size: usize,
        next_page_token: Option<&str>,
        access_token: &str,
    ) -> Result<ZoomApiRecordingListResponse, ZoomError> {
        let user_id = byte_serialize(user_id.trim().as_bytes()).collect::<String>();
        let endpoint = format!("{api_base_url}/users/{user_id}/recordings");
        let mut query = vec![
            ("from", from.trim().to_owned()),
            ("to", to.trim().to_owned()),
            ("page_size", page_size.to_string()),
        ];
        if let Some(token) = next_page_token
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            query.push(("next_page_token", token.to_owned()));
        }
        Ok(self
            .http
            .get(endpoint)
            .bearer_auth(access_token.trim())
            .query(&query)
            .send()
            .await?
            .error_for_status()?
            .json::<ZoomApiRecordingListResponse>()
            .await?)
    }

    pub(super) async fn fetch_zoom_webhook_subscriptions(
        &self,
        api_base_url: &str,
        access_token: &str,
    ) -> Result<Vec<ZoomWebhookSubscription>, ZoomError> {
        let response = self
            .http
            .get(format!("{api_base_url}/marketplace/app/event_subscription"))
            .bearer_auth(access_token.trim())
            .header(ACCEPT, "application/json")
            .send()
            .await?
            .error_for_status()?
            .json::<ZoomApiEventSubscriptionListResponse>()
            .await?;
        Ok(response
            .event_subscriptions
            .into_iter()
            .filter_map(|subscription| subscription.into_public())
            .collect())
    }

    pub(super) async fn create_zoom_webhook_subscription(
        &self,
        api_base_url: &str,
        subscription_name: &str,
        endpoint_url: &str,
        event_types: &[String],
        access_token: &str,
    ) -> Result<ZoomWebhookSubscription, ZoomError> {
        let response = self.http.post(format!("{api_base_url}/marketplace/app/event_subscription")).bearer_auth(access_token.trim()).header(ACCEPT, "application/json").header(CONTENT_TYPE, "application/json").json(&serde_json::json!({"subscription_name": subscription_name.trim(), "event_webhook_url": endpoint_url.trim(), "event_types": event_types})).send().await?.error_for_status()?.json::<ZoomApiEventSubscription>().await?;
        response.into_public().ok_or_else(|| {
            ZoomError::InvalidRequest(
                "Zoom webhook subscription create response was missing required fields".to_owned(),
            )
        })
    }

    pub(super) async fn delete_zoom_webhook_subscription(
        &self,
        api_base_url: &str,
        subscription_id: &str,
        access_token: &str,
    ) -> Result<(), ZoomError> {
        let subscription_id = byte_serialize(subscription_id.trim().as_bytes()).collect::<String>();
        self.http
            .delete(format!(
                "{api_base_url}/marketplace/app/event_subscription/{subscription_id}"
            ))
            .bearer_auth(access_token.trim())
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}
