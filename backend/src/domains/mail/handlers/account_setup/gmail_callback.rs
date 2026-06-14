use super::super::*;
use super::calendar::upsert_google_workspace_calendar_account;
use super::helpers::{gmail_pending_external_account_id, trimmed_optional};
use super::models::GmailOAuthCallbackQuery;

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
