use super::super::*;

pub(crate) async fn post_v1_send(
    State(state): State<AppState>,
    Json(req): Json<SendRequest>,
) -> Result<Json<SendResponse>, ApiError> {
    if req.confirmed_provider_write != Some(true) {
        return Err(ApiError::ProviderWriteConfirmationRequired);
    }

    let communication_store = communication_ingestion_store(&state)?;
    let account = communication_store
        .provider_account(&req.account_id)
        .await?
        .ok_or(ApiError::InvalidCommunicationQuery(
            "provider account was not found",
        ))?;
    let email = crate::domains::mail::send::OutgoingEmail {
        from: account.external_account_id.clone(),
        to: req.to.clone(),
        cc: req.cc.clone().unwrap_or_default(),
        bcc: req.bcc.clone().unwrap_or_default(),
        subject: req.subject.clone(),
        body_text: req.body_text.clone(),
        body_html: req.body_html.clone(),
        in_reply_to: req.in_reply_to.clone(),
        references: req.references.clone().unwrap_or_default(),
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

    if req.scheduled_send_at.is_some() || req.undo_send_seconds.unwrap_or(0) > 0 {
        return enqueue_outbox_send(&state, &account, email, req).await;
    }

    require_unlocked_host_vault(&state)?;

    if matches!(account.provider_kind, EmailProviderKind::Gmail)
        && gmail_send_enabled(&account.config)
    {
        return send_via_gmail_api(&state, communication_store, &account, email).await;
    }

    let smtp_config = crate::domains::mail::outbox::smtp_config_for_provider_account(&account)
        .map_err(outbox_delivery_api_error)?;
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
        outbox_id: None,
        accepted: result.accepted_recipients.clone(),
        accepted_recipients: result.accepted_recipients,
        transport: "smtp".to_owned(),
        status: "sent".to_owned(),
        scheduled_send_at: None,
        undo_deadline_at: None,
        failure_reason: None,
    }))
}

async fn send_via_gmail_api(
    state: &AppState,
    communication_store: CommunicationIngestionStore,
    account: &ProviderAccount,
    email: crate::domains::mail::send::OutgoingEmail,
) -> Result<Json<SendResponse>, ApiError> {
    let binding = communication_store
        .provider_account_secret_binding(
            &account.account_id,
            ProviderAccountSecretPurpose::OauthToken,
        )
        .await?
        .ok_or(ApiError::InvalidCommunicationQuery(
            "Gmail OAuth credential is unavailable for this account",
        ))?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let account_setup = EmailAccountSetupService::new_with_host_vault(
        communication_store,
        SecretReferenceStore::new(pool),
        state.vault.clone(),
    );
    let access_token = account_setup
        .refresh_gmail_access_token(&binding.secret_ref)
        .await?;

    api_audit_log(state)?
        .record(&NewApiAuditRecord::communication_email_send(
            "hermes-frontend",
            &account.account_id,
            email.to.len() + email.cc.len() + email.bcc.len(),
        ))
        .await?;

    let result = crate::integrations::gmail::client::GmailApiClient::new(gmail_api_base_url(
        &account.config,
    ))
    .user_id("me")
    .send_message(&access_token, &email)
    .await
    .map_err(gmail_send_api_error)?;

    Ok(Json(SendResponse {
        message_id: result.message_id,
        outbox_id: None,
        accepted: result.accepted_recipients.clone(),
        accepted_recipients: result.accepted_recipients,
        transport: "gmail".to_owned(),
        status: "sent".to_owned(),
        scheduled_send_at: None,
        undo_deadline_at: None,
        failure_reason: None,
    }))
}

async fn enqueue_outbox_send(
    state: &AppState,
    account: &ProviderAccount,
    email: crate::domains::mail::send::OutgoingEmail,
    req: SendRequest,
) -> Result<Json<SendResponse>, ApiError> {
    let now = Utc::now();
    let undo_deadline_at = req
        .undo_send_seconds
        .filter(|seconds| *seconds > 0)
        .map(|seconds| now + chrono::Duration::seconds(seconds.clamp(1, 300)));
    let status = match req.scheduled_send_at {
        Some(scheduled_send_at) if scheduled_send_at > now => {
            crate::domains::mail::outbox::EmailOutboxStatus::Scheduled
        }
        _ => crate::domains::mail::outbox::EmailOutboxStatus::Queued,
    };
    let outbox_id = format!(
        "outbox:{}:{}",
        account.account_id,
        now.timestamp_nanos_opt().unwrap_or_default()
    );
    let recipient_count = email.to.len() + email.cc.len() + email.bcc.len();
    let item = crate::domains::mail::outbox::EmailOutboxStore::new(
        state
            .database
            .pool()
            .ok_or(ApiError::DatabaseNotConfigured)?
            .clone(),
    )
    .enqueue(&crate::domains::mail::outbox::NewEmailOutboxItem {
        outbox_id,
        account_id: account.account_id.clone(),
        draft_id: req.draft_id,
        to_recipients: email.to.clone(),
        cc_recipients: email.cc.clone(),
        bcc_recipients: email.bcc.clone(),
        subject: email.subject,
        body_text: email.body_text,
        body_html: email.body_html,
        status,
        scheduled_send_at: req.scheduled_send_at,
        undo_deadline_at,
        metadata: json!({
            "from": email.from,
            "in_reply_to": email.in_reply_to,
            "references": email.references
        }),
    })
    .await?;

    api_audit_log(state)?
        .record(&NewApiAuditRecord::communication_email_send(
            "hermes-frontend",
            &account.account_id,
            recipient_count,
        ))
        .await?;

    Ok(Json(SendResponse {
        message_id: item.outbox_id.clone(),
        outbox_id: Some(item.outbox_id),
        accepted: item.to_recipients.clone(),
        accepted_recipients: item.to_recipients,
        transport: "outbox".to_owned(),
        status: item.status.as_str().to_owned(),
        scheduled_send_at: item.scheduled_send_at,
        undo_deadline_at: item.undo_deadline_at,
        failure_reason: None,
    }))
}

fn gmail_send_enabled(config: &Value) -> bool {
    config
        .get("gmail_send_enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn gmail_api_base_url(config: &Value) -> &str {
    config
        .get("gmail_api_base_url")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("https://www.googleapis.com")
}

fn provider_credential_api_error(
    error: crate::domains::mail::core::ProviderCredentialError,
) -> ApiError {
    tracing::warn!(error = %error, "SMTP credential lookup failed");
    ApiError::InvalidCommunicationQuery("SMTP credential is unavailable for this account")
}

fn gmail_send_api_error(
    error: crate::integrations::gmail::client::EmailProviderNetworkError,
) -> ApiError {
    tracing::warn!(error = %error, "Gmail API send failed");
    ApiError::InvalidCommunicationQuery("Gmail send failed")
}

fn outbox_delivery_api_error(error: crate::domains::mail::outbox::OutboxDeliveryError) -> ApiError {
    match error.public_message() {
        "Gmail send is unavailable until OAuth send scopes are configured" => {
            ApiError::InvalidCommunicationQuery(
                "Gmail send is unavailable until OAuth send scopes are configured",
            )
        }
        "provider does not support SMTP send" => {
            ApiError::InvalidCommunicationQuery("provider does not support SMTP send")
        }
        "provider account config must be a JSON object" => {
            ApiError::InvalidCommunicationQuery("provider account config must be a JSON object")
        }
        "SMTP port is unavailable for this account" => {
            ApiError::InvalidCommunicationQuery("SMTP port is unavailable for this account")
        }
        _ => ApiError::InvalidCommunicationQuery("SMTP config is unavailable for this account"),
    }
}
