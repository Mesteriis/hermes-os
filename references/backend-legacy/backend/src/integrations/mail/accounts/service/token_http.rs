use super::super::errors::EmailAccountSetupError;
use super::super::models::{GmailOAuthPendingGrant, GmailOAuthTokenBundle, OAuthTokenResponse};
use super::EmailAccountSetupService;

impl EmailAccountSetupService {
    pub(in crate::integrations::mail::accounts::service) async fn exchange_authorization_code(
        &self,
        pending: &GmailOAuthPendingGrant,
        authorization_code: &str,
    ) -> Result<OAuthTokenResponse, EmailAccountSetupError> {
        let mut form = vec![
            ("grant_type", "authorization_code".to_owned()),
            ("code", authorization_code.trim().to_owned()),
            ("client_id", pending.request.client_id.clone()),
            ("redirect_uri", pending.request.redirect_uri.clone()),
            ("code_verifier", pending.code_verifier.clone()),
        ];
        if let Some(client_secret) = &pending.request.client_secret {
            form.push(("client_secret", client_secret.clone()));
        }

        Ok(self
            .http
            .post(&pending.request.token_endpoint)
            .form(&form)
            .send()
            .await?
            .error_for_status()?
            .json::<OAuthTokenResponse>()
            .await?)
    }

    pub(in crate::integrations::mail::accounts::service) async fn refresh_token(
        &self,
        bundle: &GmailOAuthTokenBundle,
    ) -> Result<OAuthTokenResponse, EmailAccountSetupError> {
        let mut form = vec![
            ("grant_type", "refresh_token".to_owned()),
            ("refresh_token", bundle.refresh_token.clone()),
            ("client_id", bundle.client_id.clone()),
        ];
        if let Some(client_secret) = &bundle.client_secret {
            form.push(("client_secret", client_secret.clone()));
        }

        Ok(self
            .http
            .post(&bundle.token_url)
            .form(&form)
            .send()
            .await?
            .error_for_status()?
            .json::<OAuthTokenResponse>()
            .await?)
    }
}
