use super::*;

pub(crate) async fn post_gmail_oauth_start(
    State(state): State<AppState>,
    Json(request): Json<GmailOAuthStartApiRequest>,
) -> Result<Json<GmailOAuthStartApiResponse>, ApiError> {
    require_unlocked_host_vault(&state)?;
    let service = account_setup_service(&state)?;
    let pending = service.start_gmail_oauth(request.into_setup_request(&state.config)?)?;
    let response = GmailOAuthStartApiResponse {
        setup_id: pending.setup_id.clone(),
        authorization_url: pending.authorization_url.clone(),
        state: pending.state.clone(),
        redirect_uri: pending.request.redirect_uri.clone(),
    };
    let mut pending_map = state
        .account_setup
        .pending_gmail_oauth
        .lock()
        .map_err(|_| ApiError::AccountSetupState)?;
    pending_map.insert(pending.setup_id.clone(), pending);

    Ok(Json(response))
}
pub(crate) async fn post_gmail_oauth_complete(
    State(state): State<AppState>,
    Json(request): Json<GmailOAuthCompleteApiRequest>,
) -> Result<Json<EmailAccountSetupApiResponse>, ApiError> {
    let mut pending = {
        let mut pending_map = state
            .account_setup
            .pending_gmail_oauth
            .lock()
            .map_err(|_| ApiError::AccountSetupState)?;
        pending_map
            .remove(&request.setup_id)
            .ok_or(ApiError::AccountSetupPendingGrantNotFound)?
    };
    if pending.state != request.state {
        return Err(ApiError::AccountSetupStateMismatch);
    }
    if let Some(external_account_id) = trimmed_optional(request.external_account_id) {
        pending.request = pending.request.external_account_id(external_account_id);
    }
    let mail_account_id = pending.account_id.clone();
    let display_name = pending.request.display_name.clone();
    let external_account_id = gmail_pending_external_account_id(&pending);

    let service = account_setup_service(&state)?;
    let result = service
        .complete_gmail_oauth(pending, &request.authorization_code)
        .await?;
    upsert_google_workspace_calendar_account(
        &state,
        &mail_account_id,
        &display_name,
        &external_account_id,
        &result.secret_ref,
    )
    .await?;

    Ok(Json(result.into()))
}

pub(crate) async fn get_gmail_oauth_callback(
    State(state): State<AppState>,
    Query(query): Query<GmailOAuthCallbackQuery>,
) -> (StatusCode, Html<String>) {
    let GmailOAuthCallbackQuery {
        code,
        state: oauth_state,
        error,
        error_description: _,
    } = query;
    if trimmed_optional(error).is_some() {
        return gmail_oauth_callback_error_page(
            StatusCode::BAD_REQUEST,
            "Google authorization failed. Start the mail connection again.",
        );
    }
    let Some(code) = trimmed_optional(code) else {
        return gmail_oauth_callback_error_page(
            StatusCode::BAD_REQUEST,
            "Missing authorization code. Start the mail connection again.",
        );
    };
    let Some(oauth_state) = trimmed_optional(oauth_state) else {
        return gmail_oauth_callback_error_page(
            StatusCode::BAD_REQUEST,
            "Missing OAuth state. Start the mail connection again.",
        );
    };

    let pending = match remove_pending_gmail_oauth_by_state(&state, &oauth_state) {
        Ok(Some(pending)) => pending,
        Ok(None) => {
            return gmail_oauth_callback_error_page(
                StatusCode::BAD_REQUEST,
                "OAuth grant expired or was already used. Start the mail connection again.",
            );
        }
        Err(_error) => {
            tracing::error!("Gmail OAuth callback state lookup failed");
            return gmail_oauth_callback_error_page(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Account setup state is unavailable. Start the mail connection again.",
            );
        }
    };

    let service = match account_setup_service(&state) {
        Ok(service) => service,
        Err(_error) => {
            tracing::error!("Gmail OAuth callback setup service failed");
            return gmail_oauth_callback_error_page(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Account setup is unavailable. Check local backend and vault status.",
            );
        }
    };
    let app_return_url = pending.request.app_return_url.clone();
    let mail_account_id = pending.account_id.clone();
    let display_name = pending.request.display_name.clone();
    let external_account_id = gmail_pending_external_account_id(&pending);
    match service.complete_gmail_oauth(pending, &code).await {
        Ok(result) => {
            if let Err(error) = upsert_google_workspace_calendar_account(
                &state,
                &mail_account_id,
                &display_name,
                &external_account_id,
                &result.secret_ref,
            )
            .await
            {
                tracing::error!("Gmail OAuth callback calendar account setup failed");
                return gmail_oauth_callback_error_page(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    gmail_oauth_callback_api_error_message(&error),
                );
            }
            gmail_oauth_callback_success_page(&result.account_id, app_return_url.as_deref())
        }
        Err(error) => {
            let status = if matches!(
                error,
                EmailAccountSetupError::InvalidRequest { .. }
                    | EmailAccountSetupError::MissingProviderField { .. }
            ) {
                StatusCode::BAD_REQUEST
            } else {
                tracing::error!(error = %error, "Gmail OAuth callback completion failed");
                StatusCode::INTERNAL_SERVER_ERROR
            };
            gmail_oauth_callback_error_page(status, gmail_oauth_callback_error_message(&error))
        }
    }
}

fn gmail_pending_external_account_id(pending: &GmailOAuthPendingGrant) -> String {
    trimmed_optional(Some(pending.request.external_account_id.clone()))
        .unwrap_or_else(|| pending.account_id.clone())
}

async fn upsert_google_workspace_calendar_account(
    state: &AppState,
    mail_account_id: &str,
    display_name: &str,
    external_account_id: &str,
    secret_ref: &str,
) -> Result<(), ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CalendarAccountStore::new(pool)
        .upsert_google_workspace_account(
            mail_account_id,
            display_name,
            Some(external_account_id),
            secret_ref,
        )
        .await?;
    Ok(())
}

fn remove_pending_gmail_oauth_by_state(
    state: &AppState,
    oauth_state: &str,
) -> Result<Option<GmailOAuthPendingGrant>, ApiError> {
    let mut pending_map = state
        .account_setup
        .pending_gmail_oauth
        .lock()
        .map_err(|_| ApiError::AccountSetupState)?;
    let setup_id = pending_map
        .iter()
        .find_map(|(setup_id, pending)| (pending.state == oauth_state).then(|| setup_id.clone()));
    Ok(setup_id.and_then(|setup_id| pending_map.remove(&setup_id)))
}

fn gmail_oauth_callback_success_page(
    account_id: &str,
    app_return_url: Option<&str>,
) -> (StatusCode, Html<String>) {
    let account_id = html_escape(account_id);
    let return_url_json = app_return_url
        .map(|url| serde_json::to_string(url).expect("serialize OAuth return URL"))
        .unwrap_or_else(|| "null".to_owned());
    let return_link = app_return_url
        .map(|url| {
            format!(
                r#"<p><a href="{}">Return to Hermes Hub settings</a></p>"#,
                html_escape(url)
            )
        })
        .unwrap_or_default();
    (
        StatusCode::OK,
        Html(format!(
            r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Hermes Hub OAuth</title>
  <style>
    body {{ margin: 0; font-family: system-ui, sans-serif; color: #182033; background: #f5f6f8; }}
    main {{ max-width: 720px; margin: 48px auto; background: #fff; border: 1px solid #d9dee7; border-radius: 8px; padding: 24px; }}
    p {{ line-height: 1.5; }}
    a {{ color: #0f766e; font-weight: 700; }}
    code {{ display: block; overflow-wrap: anywhere; background: #f8fafc; border: 1px solid #d9dee7; border-radius: 6px; padding: 10px; }}
  </style>
  <script>
    window.setTimeout(function () {{
      try {{
        if (window.opener && !window.opener.closed) {{
          window.opener.postMessage({{ type: 'hermes:gmail-oauth-connected' }}, '*');
        }}
      }} catch (_error) {{}}
      try {{
        window.close();
      }} catch (_error) {{}}
    }}, 250);
    window.setTimeout(function () {{
      var returnUrl = {return_url_json};
      if (returnUrl) {{
        window.location.replace(returnUrl);
      }}
    }}, 1400);
  </script>
</head>
<body>
  <main>
    <h1>Google mail connected</h1>
    <p>Hermes Hub saved the Google mail account and encrypted OAuth credential locally.</p>
    <p>Account</p>
    <code>{account_id}</code>
    <p>This tab will close automatically. If it stays open, return to Hermes Hub settings.</p>
    {return_link}
  </main>
</body>
</html>"#
        )),
    )
}

fn gmail_oauth_callback_error_page(
    status: StatusCode,
    message: &str,
) -> (StatusCode, Html<String>) {
    let message = html_escape(message);
    (
        status,
        Html(format!(
            r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Hermes Hub OAuth</title>
  <style>
    body {{ margin: 0; font-family: system-ui, sans-serif; color: #182033; background: #f5f6f8; }}
    main {{ max-width: 720px; margin: 48px auto; background: #fff; border: 1px solid #d9dee7; border-radius: 8px; padding: 24px; }}
    code {{ display: block; overflow-wrap: anywhere; background: #f8fafc; border: 1px solid #d9dee7; border-radius: 6px; padding: 10px; }}
  </style>
</head>
<body>
  <main>
    <h1>Google mail connection failed</h1>
    <p>{message}</p>
    <p>Return to Hermes Hub and start Google mail connection again.</p>
  </main>
</body>
</html>"#
        )),
    )
}

fn gmail_oauth_callback_error_message(error: &EmailAccountSetupError) -> &'static str {
    match error {
        EmailAccountSetupError::HostVault(HostVaultError::Locked) => {
            "Hermes Secure Vault is locked. Unlock the vault in Hermes Hub, then start Google mail connection again."
        }
        EmailAccountSetupError::HostVault(HostVaultError::Uninitialized) => {
            "Hermes Secure Vault is not initialized. Create the vault in Hermes Hub, then start Google mail connection again."
        }
        EmailAccountSetupError::InvalidRequest { field, .. } if *field == "authorization_code" => {
            "Missing authorization code. Start the mail connection again."
        }
        EmailAccountSetupError::MissingProviderField { field } if *field == "refresh_token" => {
            "Google did not return a refresh token. Start the connection again and approve offline access."
        }
        EmailAccountSetupError::InvalidRequest { .. }
        | EmailAccountSetupError::MissingProviderField { .. } => {
            "Google mail authorization response was incomplete. Start the connection again."
        }
        _ => "Google mail account setup failed. Check local backend and vault status.",
    }
}

fn gmail_oauth_callback_api_error_message(error: &ApiError) -> &'static str {
    match error {
        ApiError::DatabaseNotConfigured => {
            "Google mail connected, but calendar account setup could not write to the local database."
        }
        _ => {
            "Google mail connected, but linked calendar account setup failed. Check local backend status."
        }
    }
}

pub(crate) async fn post_imap_account_setup(
    State(state): State<AppState>,
    Json(request): Json<ImapAccountSetupApiRequest>,
) -> Result<Json<EmailAccountSetupApiResponse>, ApiError> {
    let setup_request = request.into_setup_request()?;
    let service = account_setup_service(&state)?;
    require_unlocked_host_vault(&state)?;
    let icloud_calendar_account =
        (setup_request.provider_kind == EmailProviderKind::Icloud).then(|| {
            (
                setup_request.account_id.clone(),
                setup_request.display_name.clone(),
                setup_request.external_account_id.clone(),
            )
        });
    let result = service.setup_imap_account(setup_request).await?;
    if let Some((mail_account_id, display_name, external_account_id)) = icloud_calendar_account {
        upsert_apple_icloud_calendar_account(
            &state,
            &mail_account_id,
            &display_name,
            &external_account_id,
            &result.secret_ref,
        )
        .await?;
    }

    Ok(Json(result.into()))
}

async fn upsert_apple_icloud_calendar_account(
    state: &AppState,
    mail_account_id: &str,
    display_name: &str,
    external_account_id: &str,
    secret_ref: &str,
) -> Result<(), ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CalendarAccountStore::new(pool)
        .upsert_apple_icloud_account(
            mail_account_id,
            display_name,
            Some(external_account_id),
            secret_ref,
        )
        .await?;
    Ok(())
}

#[derive(Deserialize)]
pub(crate) struct GmailOAuthStartApiRequest {
    pub(super) account_id: String,
    pub(super) display_name: String,
    pub(super) external_account_id: Option<String>,
    pub(super) client_id: Option<String>,
    pub(super) client_secret: Option<String>,
    pub(super) redirect_uri: String,
    pub(super) app_return_url: Option<String>,
    pub(super) scopes: Option<Vec<String>>,
    pub(super) authorization_endpoint: Option<String>,
    pub(super) token_endpoint: Option<String>,
}

impl GmailOAuthStartApiRequest {
    fn into_setup_request(
        self,
        config: &crate::platform::config::AppConfig,
    ) -> Result<GmailOAuthSetupRequest, EmailAccountSetupError> {
        let client_id = trimmed_optional(self.client_id)
            .or_else(|| config.google_oauth_client_id().map(str::to_owned))
            .ok_or(EmailAccountSetupError::InvalidRequest {
                field: "client_id",
                message: "must be configured as request client_id or HERMES_GOOGLE_OAUTH_CLIENT_ID",
            })?;
        let mut request = GmailOAuthSetupRequest::new(
            self.account_id,
            self.display_name,
            trimmed_optional(self.external_account_id).unwrap_or_default(),
            client_id,
            self.redirect_uri,
        );
        if let Some(app_return_url) = trimmed_optional(self.app_return_url) {
            request = request.app_return_url(app_return_url);
        }
        if let Some(client) = config.google_oauth_client() {
            request = request
                .authorization_endpoint(client.authorization_endpoint().to_owned())
                .token_endpoint(client.token_endpoint().to_owned());
        }
        if let Some(client_secret) = trimmed_optional(self.client_secret).or_else(|| {
            config
                .google_oauth_client_secret()
                .map(|secret| secret.expose_for_runtime().to_owned())
        }) {
            request = request.client_secret(client_secret);
        }
        if let Some(scopes) = self.scopes {
            request = request.scopes(scopes);
        }
        if let Some(authorization_endpoint) = self.authorization_endpoint {
            request = request.authorization_endpoint(authorization_endpoint);
        }
        if let Some(token_endpoint) = self.token_endpoint {
            request = request.token_endpoint(token_endpoint);
        }

        Ok(request)
    }
}

#[derive(Serialize)]
pub(crate) struct GmailOAuthStartApiResponse {
    pub(super) setup_id: String,
    pub(super) authorization_url: String,
    pub(super) state: String,
    pub(super) redirect_uri: String,
}

#[derive(Deserialize)]
pub(crate) struct GmailOAuthCompleteApiRequest {
    pub(super) setup_id: String,
    pub(super) state: String,
    pub(super) authorization_code: String,
    pub(super) external_account_id: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct GmailOAuthCallbackQuery {
    pub(super) code: Option<String>,
    pub(super) state: Option<String>,
    pub(super) error: Option<String>,
    pub(super) error_description: Option<String>,
}

fn trimmed_optional(value: Option<String>) -> Option<String> {
    value
        .map(|raw| raw.trim().to_owned())
        .filter(|trimmed| !trimmed.is_empty())
}

#[derive(Deserialize)]
pub(crate) struct ImapAccountSetupApiRequest {
    pub(super) account_id: String,
    pub(super) provider_kind: String,
    pub(super) display_name: String,
    pub(super) external_account_id: String,
    pub(super) host: String,
    pub(super) port: u16,
    pub(super) tls: bool,
    pub(super) mailbox: String,
    pub(super) username: String,
    pub(super) password: String,
    pub(super) secret_kind: Option<String>,
    pub(super) smtp_host: Option<String>,
    pub(super) smtp_port: Option<u16>,
    pub(super) smtp_tls: Option<bool>,
    pub(super) smtp_starttls: Option<bool>,
    pub(super) smtp_username: Option<String>,
}

impl ImapAccountSetupApiRequest {
    fn into_setup_request(self) -> Result<ImapAccountSetupRequest, ApiError> {
        let Self {
            account_id,
            provider_kind,
            display_name,
            external_account_id,
            host,
            port,
            tls,
            mailbox,
            username,
            password,
            secret_kind,
            smtp_host,
            smtp_port,
            smtp_tls,
            smtp_starttls,
            smtp_username,
        } = self;
        let provider_kind = match provider_kind.trim() {
            "icloud" => EmailProviderKind::Icloud,
            "imap" => EmailProviderKind::Imap,
            _ => {
                return Err(EmailAccountSetupError::InvalidRequest {
                    field: "provider_kind",
                    message: "must be icloud or imap",
                }
                .into());
            }
        };
        let secret_kind = match secret_kind.as_deref().unwrap_or("password").trim() {
            "app_password" => SecretKind::AppPassword,
            "password" => SecretKind::Password,
            _ => {
                return Err(EmailAccountSetupError::InvalidRequest {
                    field: "secret_kind",
                    message: "must be app_password or password",
                }
                .into());
            }
        };

        let mut request = ImapAccountSetupRequest::new(
            account_id,
            provider_kind,
            display_name,
            external_account_id,
            host,
            port,
            tls,
            mailbox,
            username,
            password,
        )
        .secret_kind(secret_kind);
        if let Some(smtp_host) = trimmed_optional(smtp_host) {
            request = request.smtp_host(smtp_host);
        }
        if let Some(smtp_port) = smtp_port {
            request = request.smtp_port(smtp_port);
        }
        if let Some(smtp_tls) = smtp_tls {
            request = request.smtp_tls(smtp_tls);
        }
        if let Some(smtp_starttls) = smtp_starttls {
            request = request.smtp_starttls(smtp_starttls);
        }
        if let Some(smtp_username) = trimmed_optional(smtp_username) {
            request = request.smtp_username(smtp_username);
        }

        Ok(request)
    }
}

#[derive(Serialize)]
pub(crate) struct EmailAccountSetupApiResponse {
    pub(super) account_id: String,
    pub(super) secret_ref: String,
    pub(super) secret_kind: SecretKind,
    pub(super) store_kind: crate::platform::secrets::SecretStoreKind,
}

impl From<crate::domains::mail::accounts::EmailAccountSetupResult>
    for EmailAccountSetupApiResponse
{
    fn from(result: crate::domains::mail::accounts::EmailAccountSetupResult) -> Self {
        Self {
            account_id: result.account_id,
            secret_ref: result.secret_ref,
            secret_kind: result.secret_kind,
            store_kind: result.store_kind,
        }
    }
}
