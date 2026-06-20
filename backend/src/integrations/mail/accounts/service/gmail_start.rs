use url::form_urlencoded;

use super::super::errors::EmailAccountSetupError;
use super::super::helpers::{pkce_challenge, random_url_token};
use super::super::models::{GmailOAuthPendingGrant, GmailOAuthSetupRequest};
use super::EmailAccountSetupService;

impl EmailAccountSetupService {
    pub fn start_gmail_oauth(
        &self,
        request: GmailOAuthSetupRequest,
    ) -> Result<GmailOAuthPendingGrant, EmailAccountSetupError> {
        request.validate()?;
        let setup_id = random_url_token();
        let state = random_url_token();
        let code_verifier = random_url_token();
        let code_challenge = pkce_challenge(&code_verifier);
        let scope = request.scopes.join(" ");
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("response_type", "code")
            .append_pair("client_id", &request.client_id)
            .append_pair("redirect_uri", &request.redirect_uri)
            .append_pair("scope", &scope)
            .append_pair("state", &state)
            .append_pair("code_challenge", &code_challenge)
            .append_pair("code_challenge_method", "S256")
            .append_pair("access_type", "offline")
            .append_pair("prompt", "consent")
            .finish();
        let authorization_url = format!("{}?{query}", request.authorization_endpoint);

        Ok(GmailOAuthPendingGrant {
            setup_id,
            account_id: request.account_id.clone(),
            authorization_url,
            state,
            code_verifier,
            request,
        })
    }
}
