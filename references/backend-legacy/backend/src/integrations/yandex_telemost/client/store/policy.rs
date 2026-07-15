use super::*;

pub(super) fn merge_metadata(mut config: Value, metadata: &Value) -> Value {
    if let Some(object) = config.as_object_mut() {
        object.insert(
            "metadata".to_owned(),
            sanitize_yandex_telemost_payload(metadata.clone()),
        );
    }
    config
}

pub(super) fn runtime_status_from_account(
    account: YandexTelemostAccount,
    authorized: bool,
) -> YandexTelemostRuntimeStatus {
    let blockers = if authorized {
        vec![]
    } else {
        vec!["yandex_telemost_oauth_token_missing".to_owned()]
    };
    let mut capabilities = yandex_telemost_capabilities(authorized);
    if !authorized {
        capabilities.push(YandexTelemostCapabilityState {
            capability: "telemost.oauth_token.required".to_owned(),
            status: "blocked".to_owned(),
            source: "provider_secret_binding_store".to_owned(),
            confidence: 0.95,
            evidence: json!({
                "missing_secret_purpose": ProviderAccountSecretPurpose::YandexTelemostOauthToken.as_str()
            }),
        });
    }
    YandexTelemostRuntimeStatus {
        account_id: account.account_id,
        provider_kind: account.provider_kind,
        lifecycle_state: if authorized {
            "authorized".to_owned()
        } else {
            account.lifecycle_state
        },
        runtime_kind: if authorized {
            YANDEX_TELEMOST_LIVE_RUNTIME_KIND.to_owned()
        } else {
            YANDEX_TELEMOST_RUNTIME_KIND.to_owned()
        },
        checked_at: Utc::now(),
        api_base_url: account.api_base_url,
        authorized,
        blockers,
        capabilities,
    }
}

pub(super) fn client_for_account(
    account: &ProviderAccount,
) -> Result<YandexTelemostHttpClient, YandexTelemostError> {
    YandexTelemostHttpClient::new(account.config.get("api_base_url").and_then(Value::as_str))
}

pub(super) fn provider_payload<T: Serialize>(request: &T) -> Result<Value, YandexTelemostError> {
    let mut value = serde_json::to_value(request)?;
    if let Value::Object(ref mut object) = value {
        object.remove("metadata");
    }
    Ok(value)
}

pub(super) fn validate_account_setup_request(
    request: &YandexTelemostAccountSetupRequest,
) -> Result<(), YandexTelemostError> {
    validate_required("account_id", &request.account_id)?;
    validate_required("display_name", &request.display_name)?;
    validate_required("external_account_id", &request.external_account_id)?;
    validate_json_object("metadata", &request.metadata)?;
    validate_api_base_url(request.api_base_url.as_deref())?;
    if request
        .oauth_token
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_none()
        && request
            .oauth_token_ref
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .is_none()
    {
        return Err(YandexTelemostError::InvalidRequest(
            "oauth_token or oauth_token_ref must be provided for live Yandex Telemost API calls"
                .to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn validate_conference_request(
    request: &YandexTelemostConferenceRequest,
) -> Result<(), YandexTelemostError> {
    validate_json_object("metadata", &request.metadata)?;
    validate_cohosts(&request.cohosts)
}

pub(super) fn validate_conference_patch_request(
    request: &YandexTelemostConferencePatchRequest,
) -> Result<(), YandexTelemostError> {
    validate_json_object("metadata", &request.metadata)?;
    validate_cohosts(&request.cohosts)
}

pub(super) fn validate_cohosts(cohosts: &[TelemostCohost]) -> Result<(), YandexTelemostError> {
    if cohosts.len() > 30 {
        return Err(YandexTelemostError::InvalidRequest(
            "Yandex Telemost supports at most 30 cohosts".to_owned(),
        ));
    }
    for cohost in cohosts {
        validate_required("cohost.email", &cohost.email)?;
        if !cohost.email.contains('@') {
            return Err(YandexTelemostError::InvalidRequest(format!(
                "cohost `{}` must be an email address",
                cohost.email
            )));
        }
    }
    Ok(())
}
