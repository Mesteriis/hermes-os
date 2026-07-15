use super::super::models::ZoomOAuthTokenResponse;
use super::{ZoomError, ZoomStore};

impl ZoomStore {
    pub(super) async fn exchange_oauth_authorization_code(
        &self,
        token_endpoint: String,
        client_id: &str,
        client_secret: &str,
        authorization_code: &str,
        redirect_uri: &str,
    ) -> Result<ZoomOAuthTokenResponse, ZoomError> {
        Ok(self
            .http
            .post(token_endpoint)
            .basic_auth(client_id.trim(), Some(client_secret.trim()))
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", authorization_code.trim()),
                ("redirect_uri", redirect_uri.trim()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json::<ZoomOAuthTokenResponse>()
            .await?)
    }

    pub(super) async fn exchange_oauth_refresh_token(
        &self,
        token_endpoint: String,
        client_id: &str,
        client_secret: &str,
        refresh_token: &str,
    ) -> Result<ZoomOAuthTokenResponse, ZoomError> {
        Ok(self
            .http
            .post(token_endpoint)
            .basic_auth(client_id.trim(), Some(client_secret.trim()))
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token.trim()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json::<ZoomOAuthTokenResponse>()
            .await?)
    }

    pub(super) async fn exchange_server_to_server_token(
        &self,
        token_endpoint: String,
        client_id: &str,
        client_secret: &str,
        zoom_account_id: &str,
    ) -> Result<ZoomOAuthTokenResponse, ZoomError> {
        Ok(self
            .http
            .post(token_endpoint)
            .basic_auth(client_id.trim(), Some(client_secret.trim()))
            .query(&[
                ("grant_type", "account_credentials"),
                ("account_id", zoom_account_id.trim()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json::<ZoomOAuthTokenResponse>()
            .await?)
    }

    pub(super) async fn exchange_client_credentials_token(
        &self,
        token_endpoint: String,
        client_id: &str,
        client_secret: &str,
    ) -> Result<ZoomOAuthTokenResponse, ZoomError> {
        Ok(self
            .http
            .post(token_endpoint)
            .basic_auth(client_id.trim(), Some(client_secret.trim()))
            .form(&[("grant_type", "client_credentials")])
            .send()
            .await?
            .error_for_status()?
            .json::<ZoomOAuthTokenResponse>()
            .await?)
    }
}
