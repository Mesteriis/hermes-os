use super::*;

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

pub(crate) async fn post_v1_reply(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<SendRequest>,
) -> Result<Json<SendResponse>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let quoted = msg
        .body_text
        .lines()
        .map(|l| format!("> {l}"))
        .collect::<Vec<_>>()
        .join("\n");
    let _body = format!(
        "{}\n\nOn {}, {} wrote:\n{}",
        req.body_text,
        msg.occurred_at.map(|d| d.to_rfc2822()).unwrap_or_default(),
        msg.sender,
        quoted
    );
    Ok(Json(SendResponse {
        message_id: format!(
            "reply-{}",
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ),
        accepted: req.to.clone(),
        accepted_recipients: req.to.clone(),
        transport: "local".to_owned(),
        status: "queued".to_owned(),
        failure_reason: None,
    }))
}

pub(crate) async fn post_v1_imap_mark_read(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    store
        .transition_workflow_state(&message_id, WorkflowState::Reviewed)
        .await?;
    Ok(Json(serde_json::json!({"marked_read": true})))
}

pub(crate) async fn post_v1_imap_delete(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let updated = store
        .move_to_local_trash(&message_id, "imap-delete-alias")
        .await?;
    Ok(Json(serde_json::json!({
        "deleted": true,
        "provider_deleted": false,
        "local_state": updated.local_state.as_str()
    })))
}

pub(crate) async fn post_v1_message_trash(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let updated = message_store(&state)?
        .move_to_local_trash(&message_id, "user_deleted")
        .await?;
    Ok(Json(serde_json::json!({
        "message_id": updated.message_id,
        "local_state": updated.local_state.as_str(),
        "provider_deleted": false
    })))
}

pub(crate) async fn post_v1_message_restore(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let updated = message_store(&state)?
        .restore_from_local_trash(&message_id)
        .await?;
    Ok(Json(serde_json::json!({
        "message_id": updated.message_id,
        "local_state": updated.local_state.as_str()
    })))
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub(crate) struct CertsQuery {
    pub(super) limit: Option<i64>,
}
#[derive(Serialize)]
pub(crate) struct CertsListResponse {
    pub(super) items: Vec<crate::domains::mail::signatures::CertificateRecord>,
}

pub(crate) async fn get_v1_certs(
    State(state): State<AppState>,
) -> Result<Json<CertsListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::signatures::CertificateStore::new(pool);
    Ok(Json(CertsListResponse {
        items: store.list().await?,
    }))
}

#[derive(Deserialize)]
pub(crate) struct NewCertRequest {
    pub(super) cert_id: String,
    pub(super) owner_name: String,
    pub(super) issuer: String,
    pub(super) serial_number: Option<String>,
    pub(super) fingerprint_sha256: Option<String>,
    pub(super) valid_from: Option<DateTime<Utc>>,
    pub(super) valid_until: Option<DateTime<Utc>>,
    pub(super) cert_type: Option<String>,
    pub(super) provider: Option<String>,
    pub(super) storage_kind: Option<String>,
    pub(super) storage_ref: Option<String>,
    pub(super) trust_status: Option<String>,
    pub(super) is_revoked: Option<bool>,
    pub(super) usage: Option<Vec<String>>,
    pub(super) linked_message_id: Option<String>,
    pub(super) metadata: Option<Value>,
}

pub(crate) async fn post_v1_cert(
    State(state): State<AppState>,
    Json(req): Json<NewCertRequest>,
) -> Result<Json<crate::domains::mail::signatures::CertificateRecord>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::signatures::CertificateStore::new(pool);
    Ok(Json(
        store
            .upsert(&crate::domains::mail::signatures::NewCertificate {
                cert_id: req.cert_id,
                owner_name: req.owner_name,
                issuer: req.issuer,
                serial_number: req.serial_number,
                fingerprint_sha256: req.fingerprint_sha256,
                valid_from: req.valid_from,
                valid_until: req.valid_until,
                cert_type: req
                    .cert_type
                    .as_deref()
                    .and_then(crate::domains::mail::signatures::CertificateType::parse)
                    .unwrap_or(crate::domains::mail::signatures::CertificateType::Unknown),
                provider: req
                    .provider
                    .as_deref()
                    .and_then(crate::domains::mail::signatures::CertificateProvider::parse)
                    .unwrap_or(crate::domains::mail::signatures::CertificateProvider::Other),
                storage_kind: req
                    .storage_kind
                    .as_deref()
                    .and_then(crate::domains::mail::signatures::CertificateStorageKind::parse)
                    .unwrap_or(
                        crate::domains::mail::signatures::CertificateStorageKind::EncryptedVault,
                    ),
                storage_ref: req.storage_ref,
                trust_status: req
                    .trust_status
                    .as_deref()
                    .and_then(crate::domains::mail::signatures::TrustStatus::parse)
                    .unwrap_or(crate::domains::mail::signatures::TrustStatus::Untrusted),
                is_revoked: req.is_revoked.unwrap_or(false),
                usage: req.usage.unwrap_or_default(),
                linked_message_id: req.linked_message_id,
                metadata: req.metadata.unwrap_or(serde_json::json!({})),
            })
            .await?,
    ))
}

#[derive(Deserialize)]
pub(crate) struct ExpiringQuery {
    pub(super) days: Option<i64>,
}
pub(crate) async fn get_v1_certs_expiring(
    State(state): State<AppState>,
    Query(query): Query<ExpiringQuery>,
) -> Result<Json<CertsListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::mail::signatures::CertificateStore::new(pool);
    Ok(Json(CertsListResponse {
        items: store.expiring_soon(query.days.unwrap_or(90)).await?,
    }))
}

pub(crate) async fn get_v1_signature_check(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<crate::domains::mail::signatures::SignatureDetection>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    Ok(Json(
        crate::domains::mail::signatures::SignatureDetector::detect_in_message(&msg.body_text, ""),
    ))
}

#[derive(Deserialize)]
pub(crate) struct ForwardRequest {
    pub(super) to: Vec<String>,
    pub(super) cc: Option<Vec<String>>,
    pub(super) note: Option<String>,
}

pub(crate) async fn post_v1_forward(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<ForwardRequest>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let cc = req.cc.unwrap_or_default();
    let note = req.note.as_deref().unwrap_or("");
    let fwd_body = format!(
        "{note}\n\n--- Forwarded message ---\nFrom: {}\nSubject: {}\nDate: {}\n\n{}",
        msg.sender,
        msg.subject,
        msg.occurred_at.map(|d| d.to_rfc2822()).unwrap_or_default(),
        msg.body_text
    );
    Ok(Json(
        serde_json::json!({"forwarded": true, "to": req.to, "cc": cc, "subject": format!("Fwd: {}", msg.subject), "body_preview": &fwd_body[..200.min(fwd_body.len())]}),
    ))
}

pub(crate) async fn get_v1_detect_language(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<crate::domains::mail::multilingual::LanguageDetection>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    Ok(Json(
        crate::domains::mail::multilingual::MultilingualService::detect_language(&msg.body_text),
    ))
}

#[derive(Deserialize)]
pub(crate) struct TranslateRequest {
    pub(super) target_language: String,
}

pub(crate) async fn post_v1_translate(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<TranslateRequest>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let service = email_multilingual_service(&state).await?;
    match service
        .translate(&msg.body_text, &req.target_language)
        .await?
    {
        Some(t) => Ok(Json(
            serde_json::json!({"translated": true, "text": t.translated_text, "target": t.target_language, "model": t.model}),
        )),
        None => Ok(Json(
            serde_json::json!({"translated": false, "reason": "no LLM configured"}),
        )),
    }
}

#[derive(Deserialize)]
pub(crate) struct AiReplyRequest {
    pub(super) tone: Option<String>,
    pub(super) language: Option<String>,
    pub(super) context: Option<String>,
}

pub(crate) async fn post_v1_ai_reply(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<AiReplyRequest>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let service = email_ai_reply_service(&state).await?;
    let opts = crate::domains::mail::ai_reply::AiReplyOptions {
        tone: req.tone,
        language: req.language,
        context: req.context,
    };
    match service.generate_reply(&msg, &opts).await? {
        Some(draft) => Ok(Json(
            serde_json::json!({"subject": draft.subject, "body": draft.body, "tone": draft.tone, "language": draft.language}),
        )),
        None => Ok(Json(
            serde_json::json!({"generated": false, "reason": "no LLM configured"}),
        )),
    }
}

#[derive(Deserialize)]
pub(crate) struct AiReplyVariantsRequest {
    pub(super) languages: Option<Vec<String>>,
    pub(super) tones: Option<Vec<String>>,
}

pub(crate) async fn post_v1_ai_reply_variants(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<AiReplyVariantsRequest>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let service = email_ai_reply_service(&state).await?;
    let languages = req
        .languages
        .unwrap_or_else(|| vec!["en".into(), "es".into(), "ru".into()]);
    let tones = req
        .tones
        .unwrap_or_else(|| vec!["professional".into(), "friendly".into()]);
    let variants = service
        .generate_reply_variants(&msg, &languages, &tones)
        .await?;
    Ok(Json(serde_json::json!({"variants": variants})))
}

#[derive(Deserialize)]
pub(crate) struct ReplyAllRequest {
    pub(super) body_text: String,
    pub(super) quote: Option<bool>,
}
pub(crate) async fn post_v1_reply_all(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<ReplyAllRequest>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let body = crate::domains::mail::actions::build_reply_body(
        &msg.sender,
        &msg.occurred_at.map(|d| d.to_rfc2822()).unwrap_or_default(),
        &msg.body_text,
        &req.body_text,
        req.quote.unwrap_or(true),
    );
    Ok(Json(
        serde_json::json!({"reply_all": true, "to": msg.recipients, "subject": format!("Re: {}", msg.subject), "body": body}),
    ))
}

#[derive(Deserialize)]
pub(crate) struct ForwardEmlRequest {
    pub(super) to: Vec<String>,
}
pub(crate) async fn post_v1_forward_eml(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<ForwardEmlRequest>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let eml = crate::domains::mail::actions::build_eml_forward(
        &msg.sender,
        &msg.occurred_at.map(|d| d.to_rfc2822()).unwrap_or_default(),
        &msg.subject,
        &msg.body_text,
        &req.to,
    );
    Ok(Json(
        serde_json::json!({"forward_eml": true, "eml_size": eml.len()}),
    ))
}

pub(crate) async fn get_v1_spf_dkim(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let auth = crate::domains::mail::spf_dkim::parse_auth_headers(&msg.body_text);
    let risk = crate::domains::mail::spf_dkim::assess_auth_risk(&auth);
    Ok(Json(serde_json::json!({"auth": auth, "risk": risk})))
}

pub(crate) async fn post_v1_extract_tasks(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let runtime_settings = ai_runtime_settings(&state).await?;
    let svc = crate::domains::mail::extract::EmailExtractService::new(
        ai_runtime_client(&state, &runtime_settings).ok(),
    );
    let tasks = svc.extract_tasks(&msg).await?;
    Ok(Json(serde_json::json!({"tasks": tasks})))
}

pub(crate) async fn post_v1_extract_notes(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let svc = crate::domains::mail::extract::EmailExtractService::new(None);
    let notes = svc.extract_notes(&msg).await?;
    Ok(Json(serde_json::json!({"notes": notes})))
}
