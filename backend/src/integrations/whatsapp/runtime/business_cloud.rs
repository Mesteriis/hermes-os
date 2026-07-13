use hermes_communications_api::accounts::ProviderAccountCommandPort;
use hermes_communications_api::accounts::{
    CommunicationProviderKind, ProviderAccount, ProviderSecretBindingCommandPort,
};
use std::sync::Arc;

use chrono::Utc;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::postgres::PgPool;

use super::{
    ShapedWhatsAppProviderRuntime, WhatsAppProviderCommandExecutionError,
    WhatsAppProviderCommandExecutionOutcome, WhatsAppProviderExecutableCommand,
    WhatsAppProviderRuntime, WhatsAppProviderRuntimeShape, WhatsAppRuntimeHealth, WhatsappWebError,
    WhatsappWebStore,
};
use crate::platform::communications::ProviderChannelMessageLookupPort;

pub(super) const BUSINESS_CLOUD_SMOKE_RUNTIME_KIND: &str = "business_cloud_smoke";
const BUSINESS_CLOUD_LIVE_SMOKE_OPT_IN_CONFIG_KEY: &str = "business_cloud_live_smoke_enabled";
const BUSINESS_CLOUD_GRAPH_API_VERSION_CONFIG_KEY: &str = "business_cloud_graph_api_version";
const BUSINESS_CLOUD_PHONE_NUMBER_ID_CONFIG_KEY: &str = "business_cloud_phone_number_id";
const BUSINESS_CLOUD_DEFAULT_GRAPH_API_VERSION: &str = "v24.0";
const BUSINESS_CLOUD_PUBLIC_AVAILABILITY_GATE: &str =
    "blocked_until_business_cloud_smoke_and_webhook_reconciliation";
const BUSINESS_CLOUD_VERIFIED_SUBMISSION_COMMANDS: &[&str] = &[
    "send_text",
    "send_template",
    "send_media",
    "send_voice_note",
];

pub(super) fn business_cloud_live_runtime_enabled() -> bool {
    false
}

pub(super) fn business_cloud_runtime_feature_blocker() -> &'static str {
    "whatsapp_business_cloud_runtime_feature_disabled"
}

pub(super) fn business_cloud_live_smoke_opted_in(config: &Value) -> bool {
    config
        .get(BUSINESS_CLOUD_LIVE_SMOKE_OPT_IN_CONFIG_KEY)
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

#[derive(Clone)]
pub(super) struct BusinessCloudRuntimeManager {
    provider_account_store: Arc<dyn ProviderAccountCommandPort>,
    http_client: reqwest::Client,
}

impl BusinessCloudRuntimeManager {
    pub(super) fn new(provider_account_store: Arc<dyn ProviderAccountCommandPort>) -> Self {
        Self {
            provider_account_store,
            http_client: reqwest::Client::new(),
        }
    }

    pub(super) async fn execute_live_provider_command(
        &self,
        command: &WhatsAppProviderExecutableCommand,
    ) -> Result<WhatsAppProviderCommandExecutionOutcome, WhatsAppProviderCommandExecutionError>
    {
        let account = self
            .lookup_account(&command.account_id)
            .await
            .map_err(|error| {
                WhatsAppProviderCommandExecutionError::new(
                    "business_cloud_account_lookup_failed",
                    error.to_string(),
                    Some(30),
                )
            })?;
        validate_business_cloud_account(&account)?;
        if account_runtime_kind(&account) != BUSINESS_CLOUD_SMOKE_RUNTIME_KIND
            || !business_cloud_live_smoke_opted_in(&account.config)
        {
            return Err(WhatsAppProviderCommandExecutionError::new(
                "whatsapp_business_cloud_live_smoke_opt_in_required",
                "Business Cloud live execution requires runtime=business_cloud_smoke and explicit account smoke opt-in",
                None,
            ));
        }
        if !business_cloud_submission_command_supported(&command.command_kind) {
            return Err(WhatsAppProviderCommandExecutionError::unsupported(
                &command.command_kind,
            ));
        }

        match command.command_kind.as_str() {
            "send_text" => self.execute_send_text(&account, command).await,
            "send_template" => self.execute_send_template(&account, command).await,
            "send_media" | "send_voice_note" => self.execute_send_media(&account, command).await,
            _ => Err(WhatsAppProviderCommandExecutionError::unsupported(
                &command.command_kind,
            )),
        }
    }

    pub(super) async fn decorate_runtime_health(
        &self,
        health: &mut WhatsAppRuntimeHealth,
        account_id: &str,
    ) {
        let manager_health = self.manager_health(account_id).await;
        health.checks["business_cloud_manager"] = manager_health.clone();
        health.checks["runtime"]["business_cloud_manager"] = manager_health;
    }

    async fn execute_send_text(
        &self,
        account: &ProviderAccount,
        command: &WhatsAppProviderExecutableCommand,
    ) -> Result<WhatsAppProviderCommandExecutionOutcome, WhatsAppProviderCommandExecutionError>
    {
        let access_token = command.api_access_token.as_ref().ok_or_else(|| {
            WhatsAppProviderCommandExecutionError::new(
                "business_cloud_access_token_unavailable",
                "Business Cloud command execution requires a host-vault access token binding",
                Some(30),
            )
        })?;
        let graph_api_version = business_cloud_graph_api_version(&account.config);
        let phone_number_id = business_cloud_phone_number_id(account)?;
        let recipient = required_command_value(&command.provider_chat_id, "provider_chat_id")?;
        let text = required_json_string(&command.payload, "text")?;
        let request_payload = json!({
            "messaging_product": "whatsapp",
            "to": recipient,
            "type": "text",
            "text": {
                "body": text,
                "preview_url": false,
            },
        });
        let response_payload = self
            .post_business_cloud_message(
                access_token.expose_for_runtime(),
                &graph_api_version,
                &phone_number_id,
                "send_text",
                &request_payload,
            )
            .await?;

        Ok(business_cloud_message_submitted_outcome(
            command,
            access_token.secret_ref(),
            &graph_api_version,
            &phone_number_id,
            &recipient,
            business_cloud_text_operation_metadata(&text),
            response_payload,
        ))
    }

    async fn execute_send_template(
        &self,
        account: &ProviderAccount,
        command: &WhatsAppProviderExecutableCommand,
    ) -> Result<WhatsAppProviderCommandExecutionOutcome, WhatsAppProviderCommandExecutionError>
    {
        let access_token = command.api_access_token.as_ref().ok_or_else(|| {
            WhatsAppProviderCommandExecutionError::new(
                "business_cloud_access_token_unavailable",
                "Business Cloud send_template requires a host-vault access token binding",
                Some(30),
            )
        })?;
        let graph_api_version = business_cloud_graph_api_version(&account.config);
        let phone_number_id = business_cloud_phone_number_id(account)?;
        let recipient = required_command_value(&command.provider_chat_id, "provider_chat_id")?;
        let template = business_cloud_template_payload(&command.payload)?;
        let request_payload = json!({
            "messaging_product": "whatsapp",
            "to": recipient,
            "type": "template",
            "template": template,
        });
        let response_payload = self
            .post_business_cloud_message(
                access_token.expose_for_runtime(),
                &graph_api_version,
                &phone_number_id,
                "send_template",
                &request_payload,
            )
            .await?;

        Ok(business_cloud_message_submitted_outcome(
            command,
            access_token.secret_ref(),
            &graph_api_version,
            &phone_number_id,
            &recipient,
            business_cloud_template_operation_metadata(&request_payload["template"]),
            response_payload,
        ))
    }

    async fn execute_send_media(
        &self,
        account: &ProviderAccount,
        command: &WhatsAppProviderExecutableCommand,
    ) -> Result<WhatsAppProviderCommandExecutionOutcome, WhatsAppProviderCommandExecutionError>
    {
        let access_token = command.api_access_token.as_ref().ok_or_else(|| {
            WhatsAppProviderCommandExecutionError::new(
                "business_cloud_access_token_unavailable",
                "Business Cloud media submission requires a host-vault access token binding",
                Some(30),
            )
        })?;
        let media_bytes = command.media_bytes.as_ref().ok_or_else(|| {
            WhatsAppProviderCommandExecutionError::new(
                "business_cloud_media_bytes_unavailable",
                "Business Cloud media submission requires redacted in-memory local blob bytes",
                Some(30),
            )
        })?;
        if media_bytes.is_empty() {
            return Err(WhatsAppProviderCommandExecutionError::new(
                "business_cloud_media_bytes_empty",
                "Business Cloud media upload received an empty local blob",
                None,
            ));
        }

        let graph_api_version = business_cloud_graph_api_version(&account.config);
        let phone_number_id = business_cloud_phone_number_id(account)?;
        let recipient = required_command_value(&command.provider_chat_id, "provider_chat_id")?;
        let content_type = required_json_string_for_command(
            command,
            "content_type",
            "Business Cloud media submission",
        )?;
        let media_type = business_cloud_media_type(command, &content_type)?;
        let media_filename = business_cloud_media_filename(command, &content_type);
        let uploaded_media_id = self
            .upload_business_cloud_media(
                access_token.expose_for_runtime(),
                &graph_api_version,
                &phone_number_id,
                &media_filename,
                &content_type,
                media_bytes.clone_bytes(),
            )
            .await?;
        let media_message_object =
            business_cloud_media_message_object(command, &media_type, &uploaded_media_id);
        let request_payload = json!({
            "messaging_product": "whatsapp",
            "to": recipient,
            "type": media_type.clone(),
            media_type.clone(): media_message_object,
        });
        let response_payload = self
            .post_business_cloud_message(
                access_token.expose_for_runtime(),
                &graph_api_version,
                &phone_number_id,
                &command.command_kind,
                &request_payload,
            )
            .await?;

        Ok(business_cloud_message_submitted_outcome(
            command,
            access_token.secret_ref(),
            &graph_api_version,
            &phone_number_id,
            &recipient,
            business_cloud_media_operation_metadata(
                command,
                &media_type,
                &content_type,
                media_bytes.len(),
                &media_filename,
                &uploaded_media_id,
            ),
            response_payload,
        ))
    }

    async fn post_business_cloud_message(
        &self,
        access_token: &str,
        graph_api_version: &str,
        phone_number_id: &str,
        operation: &str,
        request_payload: &Value,
    ) -> Result<Value, WhatsAppProviderCommandExecutionError> {
        let endpoint = business_cloud_messages_endpoint(graph_api_version, phone_number_id);
        let response = self
            .http_client
            .post(&endpoint)
            .bearer_auth(access_token)
            .json(request_payload)
            .send()
            .await
            .map_err(|error| {
                WhatsAppProviderCommandExecutionError::new(
                    "business_cloud_http_request_failed",
                    format!("Business Cloud {operation} request failed: {error}"),
                    Some(30),
                )
            })?;
        business_cloud_response_json(operation, response).await
    }

    async fn upload_business_cloud_media(
        &self,
        access_token: &str,
        graph_api_version: &str,
        phone_number_id: &str,
        filename: &str,
        content_type: &str,
        bytes: Vec<u8>,
    ) -> Result<String, WhatsAppProviderCommandExecutionError> {
        let endpoint = business_cloud_media_endpoint(graph_api_version, phone_number_id);
        let part = reqwest::multipart::Part::bytes(bytes)
            .file_name(filename.to_owned())
            .mime_str(content_type)
            .map_err(|error| {
                WhatsAppProviderCommandExecutionError::new(
                    "business_cloud_media_content_type_invalid",
                    format!("Business Cloud media content_type is invalid: {error}"),
                    None,
                )
            })?;
        let form = reqwest::multipart::Form::new()
            .text("messaging_product", "whatsapp")
            .part("file", part);
        let response = self
            .http_client
            .post(&endpoint)
            .bearer_auth(access_token)
            .multipart(form)
            .send()
            .await
            .map_err(|error| {
                WhatsAppProviderCommandExecutionError::new(
                    "business_cloud_media_upload_request_failed",
                    format!("Business Cloud media upload request failed: {error}"),
                    Some(30),
                )
            })?;
        let response_payload = business_cloud_response_json("media_upload", response).await?;
        response_payload
            .get("id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .ok_or_else(|| {
                WhatsAppProviderCommandExecutionError::new(
                    "business_cloud_media_upload_id_missing",
                    "Business Cloud media upload response did not include media id",
                    Some(30),
                )
            })
    }

    async fn lookup_account(&self, account_id: &str) -> Result<ProviderAccount, WhatsappWebError> {
        self.provider_account_store
            .get(account_id)
            .await
            .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?
            .ok_or_else(|| {
                WhatsappWebError::InvalidRequest(format!(
                    "WhatsApp account `{account_id}` is not configured"
                ))
            })
    }

    async fn maybe_account(&self, account_id: &str) -> Option<ProviderAccount> {
        self.provider_account_store
            .get(account_id)
            .await
            .ok()
            .flatten()
    }

    async fn manager_health(&self, account_id: &str) -> Value {
        let account = self.maybe_account(account_id).await;
        let smoke_opt_in = account
            .as_ref()
            .map(|item| business_cloud_live_smoke_opted_in(&item.config))
            .unwrap_or(false);
        let runtime_kind = account
            .as_ref()
            .map(account_runtime_kind)
            .unwrap_or_else(|| "unknown".to_owned());
        json!({
            "wired": true,
            "account_scoped": true,
            "account_id": account_id,
            "runtime_kind": runtime_kind,
            "start_policy": "explicit_account_config_smoke_opt_in",
            "smoke_opt_in": smoke_opt_in,
            "smoke_opt_in_config_key": BUSINESS_CLOUD_LIVE_SMOKE_OPT_IN_CONFIG_KEY,
            "smoke_runtime_kind": BUSINESS_CLOUD_SMOKE_RUNTIME_KIND,
            "graph_api_version_config_key": BUSINESS_CLOUD_GRAPH_API_VERSION_CONFIG_KEY,
            "phone_number_id_config_key": BUSINESS_CLOUD_PHONE_NUMBER_ID_CONFIG_KEY,
            "default_graph_api_version": BUSINESS_CLOUD_DEFAULT_GRAPH_API_VERSION,
            "requires_access_token_secret_purpose": "whatsapp_business_cloud_access_token",
            "provider_command_surface": {
                "verified_submission_subset": BUSINESS_CLOUD_VERIFIED_SUBMISSION_COMMANDS,
                "completion_rule": "provider_observed_event_reconciliation_required",
                "webhook_reconciliation": "required_before_completion",
                "payload_policy": "sanitized_metadata_only",
                "template_submission": "smoke_gated_graph_messages_api",
                "media_upload_submission": "smoke_gated_graph_media_then_messages_api",
                "voice_note_submission": "submitted_as_business_cloud_audio_message",
            },
            "public_availability_gate": BUSINESS_CLOUD_PUBLIC_AVAILABILITY_GATE,
            "direct_domain_calls": "forbidden",
        })
    }
}

fn validate_business_cloud_account(
    account: &ProviderAccount,
) -> Result<(), WhatsAppProviderCommandExecutionError> {
    if account.provider_kind != CommunicationProviderKind::WhatsappBusinessCloud {
        return Err(WhatsAppProviderCommandExecutionError::new(
            "business_cloud_account_kind_mismatch",
            "Business Cloud runtime can execute only whatsapp_business_cloud accounts",
            None,
        ));
    }
    let provider_shape = account
        .config
        .get("provider_shape")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or_default();
    if provider_shape != "whatsapp_business_cloud" {
        return Err(WhatsAppProviderCommandExecutionError::new(
            "business_cloud_provider_shape_mismatch",
            "Business Cloud runtime can execute only provider_shape=whatsapp_business_cloud",
            None,
        ));
    }
    Ok(())
}

fn account_runtime_kind(account: &ProviderAccount) -> String {
    account
        .config
        .get("runtime")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| "live_blocked".to_owned())
}

fn business_cloud_submission_command_supported(command_kind: &str) -> bool {
    BUSINESS_CLOUD_VERIFIED_SUBMISSION_COMMANDS.contains(&command_kind)
}

fn business_cloud_messages_endpoint(graph_api_version: &str, phone_number_id: &str) -> String {
    format!(
        "https://graph.facebook.com/{}/{}/messages",
        graph_api_version, phone_number_id
    )
}

fn business_cloud_media_endpoint(graph_api_version: &str, phone_number_id: &str) -> String {
    format!(
        "https://graph.facebook.com/{}/{}/media",
        graph_api_version, phone_number_id
    )
}

async fn business_cloud_response_json(
    operation: &str,
    response: reqwest::Response,
) -> Result<Value, WhatsAppProviderCommandExecutionError> {
    let status = response.status();
    let retry_after_seconds = business_cloud_retry_after(status, response.headers());
    let response_payload = response.json::<Value>().await.map_err(|error| {
        WhatsAppProviderCommandExecutionError::new(
            "business_cloud_response_json_invalid",
            format!("Business Cloud {operation} response JSON is invalid: {error}"),
            Some(30),
        )
    })?;
    if !status.is_success() {
        return Err(business_cloud_provider_rejection_error(
            operation,
            status,
            &response_payload,
            retry_after_seconds,
        ));
    }
    Ok(response_payload)
}

fn business_cloud_graph_api_version(config: &Value) -> String {
    config
        .get(BUSINESS_CLOUD_GRAPH_API_VERSION_CONFIG_KEY)
        .or_else(|| config.get("graph_api_version"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| BUSINESS_CLOUD_DEFAULT_GRAPH_API_VERSION.to_owned())
}

fn business_cloud_phone_number_id(
    account: &ProviderAccount,
) -> Result<String, WhatsAppProviderCommandExecutionError> {
    account
        .config
        .get(BUSINESS_CLOUD_PHONE_NUMBER_ID_CONFIG_KEY)
        .or_else(|| account.config.get("phone_number_id"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .or_else(|| {
            let value = account.external_account_id.trim();
            (!value.is_empty()).then(|| value.to_owned())
        })
        .ok_or_else(|| {
            WhatsAppProviderCommandExecutionError::new(
                "business_cloud_phone_number_id_missing",
                "Business Cloud send_text requires a phone-number id in account config or external_account_id",
                None,
            )
        })
}

fn required_command_value(
    value: &str,
    field: &'static str,
) -> Result<String, WhatsAppProviderCommandExecutionError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(WhatsAppProviderCommandExecutionError::new(
            "business_cloud_command_field_missing",
            format!("Business Cloud send_text requires `{field}`"),
            None,
        ));
    }
    Ok(trimmed.to_owned())
}

fn required_json_string(
    payload: &Value,
    field: &'static str,
) -> Result<String, WhatsAppProviderCommandExecutionError> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            WhatsAppProviderCommandExecutionError::new(
                "business_cloud_command_field_missing",
                format!("Business Cloud send_text requires payload `{field}`"),
                None,
            )
        })
}

fn required_json_string_for_command(
    command: &WhatsAppProviderExecutableCommand,
    field: &'static str,
    context: &'static str,
) -> Result<String, WhatsAppProviderCommandExecutionError> {
    command
        .payload
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            WhatsAppProviderCommandExecutionError::new(
                "business_cloud_command_field_missing",
                format!("{context} requires payload `{field}`"),
                None,
            )
        })
}

fn optional_payload_string(payload: &Value, field: &'static str) -> Option<String> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn business_cloud_template_payload(
    payload: &Value,
) -> Result<Value, WhatsAppProviderCommandExecutionError> {
    if let Some(template) = payload.get("template").filter(|value| value.is_object()) {
        validate_business_cloud_template_payload(template)?;
        return Ok(template.clone());
    }

    let name = optional_payload_string(payload, "template_name")
        .or_else(|| optional_payload_string(payload, "name"))
        .ok_or_else(|| {
            WhatsAppProviderCommandExecutionError::new(
                "business_cloud_template_name_missing",
                "Business Cloud send_template requires payload `template.name` or `template_name`",
                None,
            )
        })?;
    let language_code = optional_payload_string(payload, "language_code")
        .or_else(|| optional_payload_string(payload, "template_language"))
        .ok_or_else(|| {
            WhatsAppProviderCommandExecutionError::new(
                "business_cloud_template_language_missing",
                "Business Cloud send_template requires payload `language_code`",
                None,
            )
        })?;
    let mut template = json!({
        "name": name,
        "language": {
            "code": language_code,
        },
    });
    if let Some(components) = payload.get("components").filter(|value| value.is_array()) {
        template["components"] = components.clone();
    }
    validate_business_cloud_template_payload(&template)?;
    Ok(template)
}

fn validate_business_cloud_template_payload(
    template: &Value,
) -> Result<(), WhatsAppProviderCommandExecutionError> {
    let name = template
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let language_code = template
        .get("language")
        .and_then(|language| language.get("code"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if name.is_none() || language_code.is_none() {
        return Err(WhatsAppProviderCommandExecutionError::new(
            "business_cloud_template_payload_invalid",
            "Business Cloud template payload requires non-empty name and language.code",
            None,
        ));
    }
    if let Some(components) = template.get("components")
        && !components.is_array()
    {
        return Err(WhatsAppProviderCommandExecutionError::new(
            "business_cloud_template_components_invalid",
            "Business Cloud template components must be an array",
            None,
        ));
    }
    Ok(())
}

fn business_cloud_media_type(
    command: &WhatsAppProviderExecutableCommand,
    content_type: &str,
) -> Result<String, WhatsAppProviderCommandExecutionError> {
    if command.command_kind == "send_voice_note" {
        return Ok("audio".to_owned());
    }
    if let Some(media_type) = optional_payload_string(&command.payload, "media_type") {
        let normalized = match media_type.as_str() {
            "image" | "video" | "audio" | "document" => Some(media_type),
            "voice_note" => Some("audio".to_owned()),
            _ => None,
        };
        if let Some(normalized) = normalized {
            return Ok(normalized);
        }
    }
    let inferred = if content_type.starts_with("image/") {
        "image"
    } else if content_type.starts_with("video/") {
        "video"
    } else if content_type.starts_with("audio/") {
        "audio"
    } else {
        "document"
    };
    Ok(inferred.to_owned())
}

fn business_cloud_media_filename(
    command: &WhatsAppProviderExecutableCommand,
    content_type: &str,
) -> String {
    optional_payload_string(&command.payload, "filename").unwrap_or_else(|| {
        let extension = match content_type {
            "image/jpeg" => "jpg",
            "image/png" => "png",
            "image/webp" => "webp",
            "video/mp4" => "mp4",
            "audio/aac" => "aac",
            "audio/mp4" => "m4a",
            "audio/mpeg" => "mp3",
            "audio/ogg" => "ogg",
            "audio/opus" => "opus",
            "application/pdf" => "pdf",
            _ => "bin",
        };
        format!("whatsapp-business-cloud-media.{extension}")
    })
}

fn business_cloud_media_message_object(
    command: &WhatsAppProviderExecutableCommand,
    media_type: &str,
    uploaded_media_id: &str,
) -> Value {
    let mut object = json!({ "id": uploaded_media_id });
    if matches!(media_type, "image" | "video" | "document")
        && let Some(caption) = optional_payload_string(&command.payload, "caption")
    {
        object["caption"] = Value::String(caption);
    }
    if media_type == "document"
        && let Some(filename) = optional_payload_string(&command.payload, "filename")
    {
        object["filename"] = Value::String(filename);
    }
    object
}

fn business_cloud_provider_rejection_error(
    operation: &str,
    status: reqwest::StatusCode,
    response_payload: &Value,
    retry_after_seconds: Option<i64>,
) -> WhatsAppProviderCommandExecutionError {
    let provider_code = response_payload
        .get("error")
        .and_then(|error| error.get("code"))
        .and_then(|value| {
            value
                .as_i64()
                .map(|number| number.to_string())
                .or_else(|| value.as_str().map(str::to_owned))
        })
        .map(|value| sanitize_business_cloud_error_code(&value));
    let provider_type = response_payload
        .get("error")
        .and_then(|error| error.get("type"))
        .and_then(Value::as_str)
        .map(sanitize_business_cloud_error_code);
    let error_code = if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        "business_cloud_rate_limited".to_owned()
    } else if let Some(provider_code) = provider_code.as_deref() {
        format!("business_cloud_provider_error_{provider_code}")
    } else {
        format!("business_cloud_provider_rejected_{operation}")
    };
    let provider_code = provider_code.unwrap_or_else(|| "unknown".to_owned());
    let provider_type = provider_type.unwrap_or_else(|| "unknown".to_owned());
    WhatsAppProviderCommandExecutionError::new(
        error_code,
        format!(
            "Business Cloud {operation} provider rejected request: http_status={}, provider_error_code={}, provider_error_type={}",
            status.as_u16(),
            provider_code,
            provider_type
        ),
        retry_after_seconds,
    )
}

fn business_cloud_retry_after(
    status: reqwest::StatusCode,
    headers: &reqwest::header::HeaderMap,
) -> Option<i64> {
    if let Some(value) = headers
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(parse_business_cloud_retry_after_seconds)
    {
        return Some(value);
    }
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Some(60);
    }
    if status.is_server_error() {
        return Some(30);
    }
    None
}

fn parse_business_cloud_retry_after_seconds(value: &str) -> Option<i64> {
    value
        .trim()
        .parse::<i64>()
        .ok()
        .filter(|seconds| (0..=86_400).contains(seconds))
}

fn sanitize_business_cloud_error_code(value: &str) -> String {
    let mut sanitized = value
        .trim()
        .chars()
        .filter_map(|character| {
            if character.is_ascii_alphanumeric() {
                Some(character.to_ascii_lowercase())
            } else if matches!(character, '_' | '-' | '.' | ' ') {
                Some('_')
            } else {
                None
            }
        })
        .collect::<String>();
    while sanitized.contains("__") {
        sanitized = sanitized.replace("__", "_");
    }
    let sanitized = sanitized.trim_matches('_').to_owned();
    if sanitized.is_empty() {
        "unknown".to_owned()
    } else {
        sanitized
    }
}

fn business_cloud_message_submitted_outcome(
    command: &WhatsAppProviderExecutableCommand,
    secret_ref: &str,
    graph_api_version: &str,
    phone_number_id: &str,
    recipient: &str,
    operation_metadata: Value,
    response_payload: Value,
) -> WhatsAppProviderCommandExecutionOutcome {
    let submitted_at = Utc::now();
    let provider_message_id = response_payload
        .get("messages")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .and_then(|item| item.get("id"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let provider_observed_completion_target =
        business_cloud_provider_observed_completion_target(provider_message_id.as_deref());
    WhatsAppProviderCommandExecutionOutcome {
        command_id: command.command_id.clone(),
        provider_request_id: provider_message_id.clone(),
        result_payload: json!({
            "provider_message_id": provider_message_id.clone(),
            "provider_submission": {
                "submitted": true,
                "submitted_at": submitted_at,
                "submitted_via": "whatsapp_business_cloud_graph_api",
                "provider_shape": "whatsapp_business_cloud",
                "runtime_driver": "business_cloud_graph_api",
                "command_kind": command.command_kind,
                "provider_request_id": provider_message_id.clone(),
                "completion_rule": "provider_observed_event_reconciliation_required",
                "provider_observed_completion_target": provider_observed_completion_target.clone(),
                "payload_policy": "sanitized_metadata_only",
                "message_body": "excluded",
                "template_components": "excluded",
                "media_bytes": "excluded",
                "media_filename": "excluded",
                "media_caption": "excluded",
                "api_access_token": "excluded",
                "session_material": "excluded",
                "raw_provider_payload": "excluded",
                "direct_domain_calls": "forbidden",
                "operation": merge_business_cloud_operation_metadata(
                    operation_metadata,
                    json!({
                        "operation": command.command_kind,
                    "graph_api_version": graph_api_version,
                    "graph_endpoint_template": "/{graph_api_version}/{phone_number_id}/messages",
                    "phone_number_id_hash": short_hash(phone_number_id),
                    "recipient_hash": short_hash(recipient),
                    "access_token_secret_ref": secret_ref,
                    "access_token_secret_purpose": "whatsapp_business_cloud_access_token"
                    })
                ),
            }
        }),
        provider_state: json!({
            "provider_message_id": provider_message_id.clone(),
            "business_cloud": {
                "submitted": true,
                "submitted_at": submitted_at,
                "runtime_driver": "business_cloud_graph_api",
                "provider_message_id": provider_message_id.clone(),
                "provider_request_id": provider_message_id.clone(),
                "provider_observed_completion_target": provider_observed_completion_target,
                "reconciliation_status": "awaiting_provider_observed_event",
                "completion_rule": "provider_observed_event_reconciliation_required",
            }
        }),
        downloaded_media_bytes: None,
    }
}

fn business_cloud_provider_observed_completion_target(provider_message_id: Option<&str>) -> Value {
    json!({
        "accepted_event_kind": "signal.accepted.whatsapp.receipt",
        "raw_record_kind": "whatsapp_web_receipt",
        "provider_message_id": provider_message_id,
        "match_policy": "provider_request_id_equals_observed_receipt_provider_message_id",
        "webhook_status_source": "business_cloud_statuses",
        "payload_policy": "sanitized_metadata_only",
        "message_body": "excluded",
        "template_components": "excluded",
        "media_bytes": "excluded",
        "api_access_token": "excluded",
        "raw_provider_payload": "excluded",
    })
}

fn business_cloud_text_operation_metadata(text: &str) -> Value {
    json!({
        "text": {
            "length": text.chars().count(),
            "sha256": full_hash(text),
        },
    })
}

fn business_cloud_template_operation_metadata(template: &Value) -> Value {
    let name = template
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or_default();
    let language_code = template
        .get("language")
        .and_then(|language| language.get("code"))
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or_default();
    let component_count = template
        .get("components")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    json!({
        "template": {
            "name": name,
            "language_code": language_code,
            "component_count": component_count,
            "components_payload": "excluded",
        },
    })
}

fn business_cloud_media_operation_metadata(
    command: &WhatsAppProviderExecutableCommand,
    media_type: &str,
    content_type: &str,
    size_bytes: usize,
    filename: &str,
    uploaded_media_id: &str,
) -> Value {
    let caption = optional_payload_string(&command.payload, "caption");
    let payload_sha256 = optional_payload_string(&command.payload, "sha256");
    json!({
        "media": {
            "business_cloud_message_type": media_type,
            "content_type": content_type,
            "size_bytes": size_bytes,
            "payload_sha256": payload_sha256,
            "filename": {
                "length": filename.chars().count(),
                "sha256": full_hash(filename),
            },
            "caption": caption.as_ref().map(|value| json!({
                "length": value.chars().count(),
                "sha256": full_hash(value),
            })),
            "provider_media_id": uploaded_media_id,
            "media_upload_endpoint_template": "/{graph_api_version}/{phone_number_id}/media",
            "media_message_endpoint_template": "/{graph_api_version}/{phone_number_id}/messages",
            "voice_note_semantics": if command.command_kind == "send_voice_note" {
                "submitted_as_business_cloud_audio_message"
            } else {
                "not_applicable"
            },
        },
    })
}

fn merge_business_cloud_operation_metadata(mut left: Value, right: Value) -> Value {
    if let (Some(left), Some(right)) = (left.as_object_mut(), right.as_object()) {
        for (key, value) in right {
            left.insert(key.clone(), value.clone());
        }
    }
    left
}

fn full_hash(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("sha256:{:x}", hasher.finalize())
}

fn short_hash(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())[..12].to_owned()
}

pub(crate) fn build_runtime(
    pool: PgPool,
    provider_account_store: Arc<dyn ProviderAccountCommandPort>,
    provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
    provider_channel_message_store: Arc<dyn ProviderChannelMessageLookupPort>,
) -> Arc<dyn WhatsAppProviderRuntime> {
    let manager = BusinessCloudRuntimeManager::new(provider_account_store.clone());
    Arc::new(
        ShapedWhatsAppProviderRuntime::new(
            WhatsAppProviderRuntimeShape::BusinessCloud,
            Arc::new(WhatsappWebStore::new(
                pool,
                provider_account_store,
                provider_secret_binding_store,
                provider_channel_message_store,
            )),
        )
        .with_business_cloud_manager(manager),
    )
}
