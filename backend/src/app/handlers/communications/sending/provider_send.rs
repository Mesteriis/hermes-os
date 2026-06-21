use super::super::*;
use crate::domains::communications::service::{
    CommunicationCommandService, CommunicationOutboxSendCommand,
};
use crate::platform::communications::OutgoingEmail;

pub(crate) async fn post_v1_send(
    State(state): State<AppState>,
    Json(req): Json<SendRequest>,
) -> Result<Json<SendResponse>, ApiError> {
    if req.confirmed_provider_write != Some(true) {
        return Err(ApiError::ProviderWriteConfirmationRequired);
    }

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let account =
        crate::domains::communications::core::CommunicationProviderAccountStore::new(pool.clone())
            .get(&req.account_id)
            .await?
            .ok_or(ApiError::InvalidCommunicationQuery(
                "provider account was not found",
            ))?;
    let email = OutgoingEmail {
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
        return send_via_gmail_api(&state, &account, email).await;
    }

    let smtp_config =
        crate::domains::communications::outbox::smtp_config_for_provider_account(&account)
            .map_err(outbox_delivery_api_error)?;
    let credential_reader = ProviderCredentialReader::new(
        crate::domains::communications::core::CommunicationProviderSecretBindingStore::new(
            pool.clone(),
        ),
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
    let result = crate::integrations::mail::send::SmtpClient::new()
        .send(&smtp_config, &credential.secret, &email)
        .await?;
    CommunicationCommandService::new(
        state
            .database
            .pool()
            .ok_or(ApiError::DatabaseNotConfigured)?
            .clone(),
    )
    .record_provider_send_sent(&account, &email, "smtp", &result.message_id)
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
    account: &ProviderAccount,
    email: OutgoingEmail,
) -> Result<Json<SendResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let binding =
        crate::domains::communications::core::CommunicationProviderSecretBindingStore::new(
            pool.clone(),
        )
        .get_for_account(
            &account.account_id,
            ProviderAccountSecretPurpose::OauthToken,
        )
        .await?
        .ok_or(ApiError::InvalidCommunicationQuery(
            "Gmail OAuth credential is unavailable for this account",
        ))?;
    let account_setup = EmailAccountSetupService::new_with_host_vault_for_token_refresh(
        pool.clone(),
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
    let result = crate::integrations::mail::gmail::client::GmailApiClient::new(gmail_api_base_url(
        &account.config,
    ))
    .user_id("me")
    .send_message(&access_token, &email)
    .await
    .map_err(gmail_send_api_error)?;
    CommunicationCommandService::new(
        state
            .database
            .pool()
            .ok_or(ApiError::DatabaseNotConfigured)?
            .clone(),
    )
    .record_provider_send_sent(account, &email, "gmail", &result.message_id)
    .await?;

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
    email: OutgoingEmail,
    req: SendRequest,
) -> Result<Json<SendResponse>, ApiError> {
    let recipient_count = email.to.len() + email.cc.len() + email.bcc.len();
    let item = CommunicationCommandService::new(
        state
            .database
            .pool()
            .ok_or(ApiError::DatabaseNotConfigured)?
            .clone(),
    )
    .enqueue_outbox_send(
        account,
        &email,
        &CommunicationOutboxSendCommand {
            draft_id: req.draft_id,
            scheduled_send_at: req.scheduled_send_at,
            undo_send_seconds: req.undo_send_seconds,
        },
    )
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
    error: crate::domains::communications::core::ProviderCredentialError,
) -> ApiError {
    tracing::warn!(error = %error, "SMTP credential lookup failed");
    ApiError::InvalidCommunicationQuery("SMTP credential is unavailable for this account")
}

fn gmail_send_api_error(
    error: crate::integrations::mail::gmail::client::EmailProviderNetworkError,
) -> ApiError {
    tracing::warn!(error = %error, "Gmail API send failed");
    ApiError::InvalidCommunicationQuery("Gmail send failed")
}

fn outbox_delivery_api_error(
    error: crate::domains::communications::outbox::OutboxDeliveryError,
) -> ApiError {
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
