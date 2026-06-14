use super::super::*;

pub(crate) async fn post_v1_send(
    State(state): State<AppState>,
    Json(req): Json<SendRequest>,
) -> Result<Json<SendResponse>, ApiError> {
    if req.confirmed_provider_write != Some(true) {
        return Err(ApiError::ProviderWriteConfirmationRequired);
    }

    require_unlocked_host_vault(&state)?;

    let communication_store = communication_ingestion_store(&state)?;
    let account = communication_store
        .provider_account(&req.account_id)
        .await?
        .ok_or(ApiError::InvalidCommunicationQuery(
            "provider account was not found",
        ))?;
    let smtp_config = smtp_config_for_provider_account(&account)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let credential_reader = ProviderCredentialReader::new(
        communication_store,
        SecretReferenceStore::new(pool),
        &state.vault,
    );
    let credential = credential_reader
        .read(
            &account.account_id,
            ProviderAccountSecretPurpose::SmtpPassword,
        )
        .await
        .map_err(provider_credential_api_error)?;
    let email = crate::domains::mail::send::OutgoingEmail {
        from: account.external_account_id.clone(),
        to: req.to,
        cc: req.cc.unwrap_or_default(),
        bcc: req.bcc.unwrap_or_default(),
        subject: req.subject,
        body_text: req.body_text,
        body_html: req.body_html,
        in_reply_to: req.in_reply_to,
        references: req.references.unwrap_or_default(),
    };

    if email
        .to
        .iter()
        .chain(email.cc.iter())
        .chain(email.bcc.iter())
        .all(|recipient| recipient.trim().is_empty())
    {
        return Err(ApiError::InvalidCommunicationQuery(
            "at least one recipient is required",
        ));
    }

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::communication_email_send(
            "hermes-frontend",
            &account.account_id,
            email.to.len() + email.cc.len() + email.bcc.len(),
        ))
        .await?;

    let result = crate::domains::mail::send::SmtpClient::new()
        .send(&smtp_config, &credential.secret, &email)
        .await?;

    Ok(Json(SendResponse {
        message_id: result.message_id,
        accepted: result.accepted_recipients.clone(),
        accepted_recipients: result.accepted_recipients,
        transport: "smtp".to_owned(),
        status: "sent".to_owned(),
        failure_reason: None,
    }))
}

fn smtp_config_for_provider_account(
    account: &ProviderAccount,
) -> Result<crate::domains::mail::send::SmtpConfig, ApiError> {
    match account.provider_kind {
        EmailProviderKind::Icloud | EmailProviderKind::Imap => {}
        EmailProviderKind::Gmail => {
            return Err(ApiError::InvalidCommunicationQuery(
                "Gmail send is unavailable until OAuth send scopes are configured",
            ));
        }
        _ => {
            return Err(ApiError::InvalidCommunicationQuery(
                "provider does not support SMTP send",
            ));
        }
    }

    let config = account
        .config
        .as_object()
        .ok_or(ApiError::InvalidCommunicationQuery(
            "provider account config must be a JSON object",
        ))?;
    let host = config
        .get("smtp_host")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(ApiError::InvalidCommunicationQuery(
            "SMTP config is unavailable for this account",
        ))?;
    let port = config
        .get("smtp_port")
        .and_then(Value::as_u64)
        .filter(|value| *value > 0 && *value <= u64::from(u16::MAX))
        .ok_or(ApiError::InvalidCommunicationQuery(
            "SMTP port is unavailable for this account",
        ))? as u16;
    let username = config
        .get("smtp_username")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(account.external_account_id.as_str());
    let tls = config
        .get("smtp_tls")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let starttls = config
        .get("smtp_starttls")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    Ok(crate::domains::mail::send::SmtpConfig::new(host, port, tls, username).starttls(starttls))
}

fn provider_credential_api_error(
    error: crate::domains::mail::core::ProviderCredentialError,
) -> ApiError {
    tracing::warn!(error = %error, "SMTP credential lookup failed");
    ApiError::InvalidCommunicationQuery("SMTP credential is unavailable for this account")
}
