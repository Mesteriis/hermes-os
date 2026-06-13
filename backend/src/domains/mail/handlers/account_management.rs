use super::*;

pub(crate) async fn get_v1_email_accounts(
    State(state): State<AppState>,
) -> Result<Json<EmailAccountListResponse>, ApiError> {
    let accounts = communication_ingestion_store(&state)?
        .list_provider_accounts()
        .await?
        .into_iter()
        .filter(|account| account.provider_kind.is_email())
        .map(email_account_view)
        .collect();

    Ok(Json(EmailAccountListResponse { items: accounts }))
}

pub(crate) async fn get_v1_email_account(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<EmailAccountView>, ApiError> {
    let account = email_account_or_not_found(&state, &account_id).await?;
    Ok(Json(email_account_view(account)))
}

pub(crate) async fn get_v1_email_account_export(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<EmailAccountExportResponse>, ApiError> {
    let account = email_account_or_not_found(&state, &account_id).await?;
    let settings = mail_sync_store(&state)
        .map_err(mail_sync_api_error)?
        .settings_for_account(&account.account_id)
        .await
        .map_err(mail_sync_api_error)?;
    let sanitized_account = ProviderAccount {
        config: sanitize_account_config(&account.config),
        ..account
    };

    Ok(Json(EmailAccountExportResponse {
        exported_at: Utc::now(),
        capabilities: email_account_capabilities(&sanitized_account),
        account: sanitized_account,
        sync_settings: settings,
    }))
}

pub(crate) async fn post_v1_email_account_import(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<EmailAccountLogoutResponse>, ApiError> {
    if contains_secret_material(&payload) {
        return Err(ApiError::InvalidCommunicationQuery(
            "account import payload must not contain secrets or secret references",
        ));
    }

    let request: EmailAccountImportRequest = serde_json::from_value(payload)
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid account import payload"))?;
    let provider_kind = EmailProviderKind::try_from(request.account.provider_kind.as_str())
        .map_err(|_| ApiError::InvalidCommunicationQuery("unsupported email provider kind"))?;
    if !provider_kind.is_email() {
        return Err(ApiError::InvalidCommunicationQuery(
            "provider kind is not an email provider",
        ));
    }
    let config = if request.account.config.is_null() {
        json!({})
    } else {
        request.account.config
    };
    if !config.is_object() {
        return Err(ApiError::InvalidCommunicationQuery(
            "account config must be an object",
        ));
    }

    let account = communication_ingestion_store(&state)?
        .upsert_provider_account(
            &crate::domains::mail::core::NewProviderAccount::new(
                request.account.account_id,
                provider_kind,
                request.account.display_name,
                request.account.external_account_id,
            )
            .config(config),
        )
        .await?;

    let current_settings = mail_sync_store(&state)
        .map_err(mail_sync_api_error)?
        .settings_for_account(&account.account_id)
        .await
        .map_err(mail_sync_api_error)?;
    let settings_update = request.sync_settings.map_or(
        MailSyncSettingsUpdate {
            sync_enabled: current_settings.sync_enabled,
            batch_size: current_settings.batch_size,
            poll_interval_seconds: current_settings.poll_interval_seconds,
        },
        |settings| MailSyncSettingsUpdate {
            sync_enabled: settings
                .sync_enabled
                .unwrap_or(current_settings.sync_enabled),
            batch_size: settings.batch_size.unwrap_or(current_settings.batch_size),
            poll_interval_seconds: settings
                .poll_interval_seconds
                .unwrap_or(current_settings.poll_interval_seconds),
        },
    );
    let sync_settings = mail_sync_store(&state)
        .map_err(mail_sync_api_error)?
        .update_settings(&account.account_id, settings_update)
        .await
        .map_err(mail_sync_api_error)?;

    Ok(Json(EmailAccountLogoutResponse {
        capabilities: email_account_capabilities(&account),
        account,
        sync_settings,
    }))
}

pub(crate) async fn post_v1_email_account_logout(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<EmailAccountLogoutResponse>, ApiError> {
    let account = email_account_or_not_found(&state, &account_id).await?;
    let mut config = account.config.clone();
    let config_object = config
        .as_object_mut()
        .ok_or(ApiError::InvalidCommunicationQuery(
            "account config must be an object",
        ))?;
    config_object.insert("auth_state".to_owned(), json!("logged_out"));
    config_object.insert("logged_out_at".to_owned(), json!(Utc::now()));

    let updated_account = communication_ingestion_store(&state)?
        .update_provider_account_config(&account.account_id, &config)
        .await?
        .ok_or(ApiError::NotFound)?;
    let current_settings = mail_sync_store(&state)
        .map_err(mail_sync_api_error)?
        .settings_for_account(&account.account_id)
        .await
        .map_err(mail_sync_api_error)?;
    let sync_settings = mail_sync_store(&state)
        .map_err(mail_sync_api_error)?
        .update_settings(
            &account.account_id,
            MailSyncSettingsUpdate {
                sync_enabled: false,
                batch_size: current_settings.batch_size,
                poll_interval_seconds: current_settings.poll_interval_seconds,
            },
        )
        .await
        .map_err(mail_sync_api_error)?;

    Ok(Json(EmailAccountLogoutResponse {
        capabilities: email_account_capabilities(&updated_account),
        account: updated_account,
        sync_settings,
    }))
}

pub(crate) async fn delete_v1_email_account(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<EmailAccountDeleteResponse>, ApiError> {
    let account = email_account_or_not_found(&state, &account_id).await?;
    let store = communication_ingestion_store(&state)?;
    let usage = store.provider_account_usage(&account.account_id).await?;
    if usage.has_retained_evidence() {
        return Err(ApiError::EmailAccountDeleteConflict);
    }

    let deleted = store
        .delete_provider_account_metadata(&account.account_id)
        .await?;

    Ok(Json(EmailAccountDeleteResponse {
        account_id: account.account_id,
        deleted: deleted.account.is_some(),
        unbound_secret_refs: deleted.unbound_secret_refs,
    }))
}

pub(crate) async fn get_v1_email_account_sync_status(
    State(state): State<AppState>,
) -> Result<Json<MailSyncStatusListResponse>, ApiError> {
    let statuses = mail_sync_store(&state)
        .map_err(mail_sync_api_error)?
        .sync_statuses()
        .await
        .map_err(mail_sync_api_error)?;

    Ok(Json(MailSyncStatusListResponse { items: statuses }))
}

pub(crate) async fn get_v1_email_account_sync_settings(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<MailSyncSettings>, ApiError> {
    Ok(Json(
        mail_sync_store(&state)
            .map_err(mail_sync_api_error)?
            .settings_for_account(&account_id)
            .await
            .map_err(mail_sync_api_error)?,
    ))
}

pub(crate) async fn put_v1_email_account_sync_settings(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
    Json(request): Json<MailSyncSettingsUpdate>,
) -> Result<Json<MailSyncSettings>, ApiError> {
    Ok(Json(
        mail_sync_store(&state)
            .map_err(mail_sync_api_error)?
            .update_settings(&account_id, request)
            .await
            .map_err(mail_sync_api_error)?,
    ))
}

pub(crate) async fn post_v1_email_account_sync_now(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<MailSyncRunResponse>, ApiError> {
    Ok(Json(
        mail_sync_service(&state)
            .map_err(mail_sync_api_error)?
            .run_account(&account_id, MailSyncTrigger::Manual)
            .await
            .map_err(mail_sync_api_error)?,
    ))
}

pub(crate) async fn post_v1_email_account_sync_full_resync(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<MailSyncRunResponse>, ApiError> {
    Ok(Json(
        mail_sync_service(&state)
            .map_err(mail_sync_api_error)?
            .run_account_full_resync(&account_id)
            .await
            .map_err(mail_sync_api_error)?,
    ))
}
