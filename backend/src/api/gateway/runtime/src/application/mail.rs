use std::{
    collections::HashMap,
    env, fs,
    path::Path,
    process::Command,
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

use bytes::Bytes;
use http_body_util::{BodyExt, Limited};
use hyper::body::Body;
use hyper::{Method, Request, Response, StatusCode};
use serde_json::json;

use crate::{GatewayHttpResponse, full_gateway_body};

type JsonValue = serde_json::Value;
type JsonObject = serde_json::Map<String, JsonValue>;

const MAIL_ACCOUNTS_PREFIX: &str = "/api/v1/integrations/mail/accounts";
const MAIL_SYNC_STATUS_PATH: &str = "/api/v1/integrations/mail/accounts/sync-status";
const MAIL_ACCOUNTS_LIST_PATH: &str = "/api/v1/integrations/mail/accounts";
const MAIL_COMMUNICATIONS_ACCOUNTS_LIST_PATH: &str = "/api/v1/communications/email/accounts";
const MAIL_IMPORT_PATH: &str = "/api/v1/integrations/mail/accounts/import";
const MAIL_IMAP_PATH: &str = "/api/v1/integrations/mail/accounts/imap";
const MAIL_GMAIL_OAUTH_START_PATH: &str = "/api/v1/integrations/mail/accounts/gmail/oauth/start";
const MAIL_GMAIL_OAUTH_COMPLETE_PATH: &str =
    "/api/v1/integrations/mail/accounts/gmail/oauth/complete";
const MAIL_GMAIL_OAUTH_CALLBACK_PATH: &str =
    "/api/v1/integrations/mail/accounts/gmail/oauth/callback";
const MAX_REQUEST_BYTES: usize = 64 * 1024;
const MAIL_RUNTIME_PROCESS_ID: &str = "hermes-mail-runtime";
const DEFAULT_TIMESTAMP: &str = "1970-01-01T00:00:00Z";
const DEFAULT_IMAP_PORT: u16 = 993;
const DEFAULT_MAIL_BATCH_SIZE: u32 = 50_000;
const MAX_MAIL_BATCH_SIZE: u32 = 1_000_000;
const DEFAULT_MAIL_SYNC_WINDOWS: u32 = 5_000;
const MAX_MAIL_SYNC_WINDOWS: u32 = 1_000_000;
const MIN_MAIL_SYNC_WINDOWS: u32 = 1;
const MIN_MAIL_POLL_INTERVAL_SECONDS: u32 = 60;
const MAX_MAIL_POLL_INTERVAL_SECONDS: u32 = 86_400;
const MIN_MAIL_FAILURE_THRESHOLD: u32 = 1;
const MAX_MAIL_FAILURE_THRESHOLD: u32 = 10;

static MAIL_ACCOUNT_STORE: OnceLock<Mutex<HashMap<String, JsonValue>>> = OnceLock::new();
static MAIL_SYNC_SETTINGS_STORE: OnceLock<Mutex<HashMap<String, JsonValue>>> = OnceLock::new();
static MAIL_PROVIDER_RESOURCE_STORE: OnceLock<Mutex<HashMap<String, JsonValue>>> = OnceLock::new();
static MAIL_SENSITIVE_FORWARDING_STORE: OnceLock<Mutex<HashMap<String, JsonValue>>> =
    OnceLock::new();
static MAIL_SYNC_SECRET_STORE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

#[derive(Debug, Clone)]
pub(super) struct MailGatewayIntegrationRouter;

#[derive(Debug)]
enum MailRoute {
    AccountsList,
    CommunicationsAccountsList,
    Account(String),
    AccountImport,
    AccountImapSetup,
    AccountExport(String),
    AccountLogout(String),
    SyncStatus,
    SyncSettings(String),
    ContentEgressSettings(String),
    SensitiveForwardingPolicies(String),
    SensitiveForwardingPolicyDelete(String, String),
    ProviderResources(String),
    ProviderResourceUpdate(String, String),
    SyncNow(String),
    SyncFullResync(String),
    AddressBookSyncNow(String),
    GmailOAuthStart,
    GmailOAuthComplete,
    GmailOAuthCallback,
}

#[derive(Debug)]
struct MailRuntimeCommandResult {
    exit_code: u32,
    stdout: String,
    stderr: String,
}

impl MailGatewayIntegrationRouter {
    #[must_use]
    pub(super) const fn new() -> Self {
        Self
    }

    pub(super) async fn route<B>(&self, request: Request<B>) -> GatewayHttpResponse
    where
        B: Body<Data = Bytes>,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        let query = request.uri().query();
        match *request.method() {
            Method::GET => self.route_get(request.uri().path(), query),
            Method::PUT => {
                let (parts, body) = request.into_parts();
                let body = collect_body(body).await;
                self.route_put(parts.uri.path(), body)
            }
            Method::POST => {
                let (parts, body) = request.into_parts();
                let body = collect_body(body).await;
                self.route_post(parts.uri.path(), body)
            }
            Method::DELETE => {
                let path = request.uri().path();
                self.route_delete(path)
            }
            _ => method_not_allowed(),
        }
    }

    fn route_get(&self, path: &str, query: Option<&str>) -> GatewayHttpResponse {
        match parse_route(path) {
            Some(MailRoute::AccountsList) | Some(MailRoute::CommunicationsAccountsList) => {
                json_response(StatusCode::OK, account_list_response())
            }
            Some(MailRoute::Account(account_id)) => match account_view(&account_id) {
                Some(response) => json_response(StatusCode::OK, response),
                None => response(StatusCode::NOT_FOUND, "account not found\n"),
            },
            Some(MailRoute::AccountExport(account_id)) => {
                match account_export_response(&account_id) {
                    Some(response) => json_response(StatusCode::OK, response),
                    None => response(StatusCode::NOT_FOUND, "account not found\n"),
                }
            }
            Some(MailRoute::SyncStatus) => query_mail_sync_status(),
            Some(MailRoute::SyncSettings(account_id)) => {
                if !account_exists(&account_id) {
                    response(StatusCode::NOT_FOUND, "account not found\n")
                } else {
                    json_response(StatusCode::OK, sync_settings_for_account(&account_id))
                }
            }
            Some(MailRoute::ContentEgressSettings(account_id)) => {
                if !account_exists(&account_id) {
                    response(StatusCode::NOT_FOUND, "account not found\n")
                } else {
                    match account_view(&account_id) {
                        Some(account) => {
                            json_response(StatusCode::OK, account_content_egress_settings(&account))
                        }
                        None => response(StatusCode::NOT_FOUND, "account not found\n"),
                    }
                }
            }
            Some(MailRoute::SensitiveForwardingPolicies(account_id)) => {
                if !account_exists(&account_id) {
                    response(StatusCode::NOT_FOUND, "account not found\n")
                } else {
                    json_response(
                        StatusCode::OK,
                        sensitive_forwarding_policies_for_account_response(&account_id),
                    )
                }
            }
            Some(MailRoute::ProviderResources(account_id)) => {
                if !account_exists(&account_id) {
                    response(StatusCode::NOT_FOUND, "account not found\n")
                } else {
                    json_response(
                        StatusCode::OK,
                        provider_resources_for_account_response(&account_id),
                    )
                }
            }
            Some(MailRoute::ProviderResourceUpdate(account_id, mapping_id)) => {
                let _ = (account_id, mapping_id);
                response(
                    StatusCode::METHOD_NOT_ALLOWED,
                    "provider-resources/{mapping_id} requires PUT\n",
                )
            }
            Some(MailRoute::SyncNow(account_id)) => {
                let _ = account_id;
                response(StatusCode::METHOD_NOT_ALLOWED, "sync-now requires POST\n")
            }
            Some(MailRoute::SyncFullResync(account_id)) => {
                let _ = account_id;
                response(
                    StatusCode::METHOD_NOT_ALLOWED,
                    "sync-full-resync requires POST\n",
                )
            }
            Some(MailRoute::AddressBookSyncNow(account_id)) => {
                let _ = account_id;
                response(
                    StatusCode::METHOD_NOT_ALLOWED,
                    "address-book-sync-now requires POST\n",
                )
            }
            Some(MailRoute::AccountLogout(account_id)) => {
                let _ = account_id;
                response(
                    StatusCode::METHOD_NOT_ALLOWED,
                    "account logout requires POST\n",
                )
            }
            Some(MailRoute::GmailOAuthStart) => response(
                StatusCode::METHOD_NOT_ALLOWED,
                "gmail/oauth/start requires POST\n",
            ),
            Some(MailRoute::GmailOAuthComplete) => response(
                StatusCode::METHOD_NOT_ALLOWED,
                "gmail/oauth/complete requires POST\n",
            ),
            Some(MailRoute::GmailOAuthCallback) => gmail_oauth_callback_response(query),
            Some(MailRoute::AccountImport) => response(
                StatusCode::METHOD_NOT_ALLOWED,
                "accounts import requires POST\n",
            ),
            Some(MailRoute::AccountImapSetup) => response(
                StatusCode::METHOD_NOT_ALLOWED,
                "accounts imap requires POST\n",
            ),
            Some(MailRoute::SensitiveForwardingPolicyDelete(_, _)) => response(
                StatusCode::METHOD_NOT_ALLOWED,
                "sensitive-forwarding policy delete requires DELETE\n",
            ),
            None => response(StatusCode::NOT_FOUND, "not found\n"),
        }
    }

    fn route_put(&self, path: &str, body: Vec<u8>) -> GatewayHttpResponse {
        match parse_route(path) {
            Some(MailRoute::SyncSettings(account_id)) => {
                if !account_exists(&account_id) {
                    return response(StatusCode::NOT_FOUND, "account not found\n");
                }
                let payload =
                    serde_json::from_slice::<JsonValue>(&body).unwrap_or_else(|_| json!({}));
                upsert_sync_settings(&account_id, &payload);
                json_response(StatusCode::OK, sync_settings_for_account(&account_id))
            }
            Some(MailRoute::ContentEgressSettings(account_id)) => {
                if !account_exists(&account_id) {
                    return response(StatusCode::NOT_FOUND, "account not found\n");
                }
                let payload =
                    serde_json::from_slice::<JsonValue>(&body).unwrap_or_else(|_| json!({}));
                match update_account_content_egress(&account_id, &payload) {
                    Some(response) => json_response(StatusCode::OK, response),
                    None => response(StatusCode::NOT_FOUND, "account not found\n"),
                }
            }
            Some(MailRoute::ProviderResourceUpdate(account_id, mapping_id)) => {
                let payload =
                    serde_json::from_slice::<JsonValue>(&body).unwrap_or_else(|_| json!({}));
                if !account_exists(&account_id) {
                    response(StatusCode::NOT_FOUND, "account not found\n")
                } else {
                    match provider_resource_mapping_update(&account_id, &mapping_id, &payload) {
                        Some(updated) => json_response(StatusCode::OK, updated),
                        None => response(
                            StatusCode::NOT_FOUND,
                            "provider resource mapping not found\n",
                        ),
                    }
                }
            }
            Some(variant) => {
                let _ = variant;
                method_not_allowed()
            }
            None => response(StatusCode::NOT_FOUND, "not found\n"),
        }
    }

    fn route_delete(&self, path: &str) -> GatewayHttpResponse {
        match parse_route(path) {
            Some(MailRoute::Account(account_id)) => {
                if !account_exists(&account_id) {
                    return response(StatusCode::NOT_FOUND, "account not found\n");
                }
                json_response(StatusCode::OK, delete_account(&account_id))
            }
            Some(MailRoute::SensitiveForwardingPolicyDelete(account_id, policy_id)) => {
                if !account_exists(&account_id) {
                    response(StatusCode::NOT_FOUND, "account not found\n")
                } else {
                    json_response(
                        StatusCode::OK,
                        delete_sensitive_forwarding_policy(&account_id, &policy_id),
                    )
                }
            }
            _ => method_not_allowed(),
        }
    }

    fn route_post(&self, path: &str, body: Vec<u8>) -> GatewayHttpResponse {
        match parse_route(path) {
            Some(MailRoute::SyncNow(account_id)) => {
                if !account_exists(&account_id) {
                    return response(StatusCode::NOT_FOUND, "account not found\n");
                }
                run_mail_sync(&account_id, "manual", body)
            }
            Some(MailRoute::SyncFullResync(account_id)) => {
                if !account_exists(&account_id) {
                    return response(StatusCode::NOT_FOUND, "account not found\n");
                }
                run_mail_sync(&account_id, "full_resync", body)
            }
            Some(MailRoute::AddressBookSyncNow(account_id)) => {
                if !account_exists(&account_id) {
                    return response(StatusCode::NOT_FOUND, "account not found\n");
                }
                run_mail_sync(&account_id, "address_book_sync", body)
            }
            Some(MailRoute::AccountImport) => {
                let payload =
                    serde_json::from_slice::<JsonValue>(&body).unwrap_or_else(|_| json!({}));
                let Some(account_payload) = payload.get("account") else {
                    return response(
                        StatusCode::BAD_REQUEST,
                        "email account import requires account\n",
                    );
                };
                if !account_payload.is_object() {
                    return response(
                        StatusCode::BAD_REQUEST,
                        "email account import payload must include object account\n",
                    );
                }
                if contains_secret_material(account_payload) || contains_secret_material(&payload) {
                    return response(
                        StatusCode::BAD_REQUEST,
                        "email account import payload must not contain secrets or secret references\n",
                    );
                }

                let account_id = match account_payload
                    .get("account_id")
                    .and_then(serde_json::Value::as_str)
                    .filter(|value| !value.is_empty())
                {
                    Some(account_id) => account_id.to_owned(),
                    None => {
                        return response(StatusCode::BAD_REQUEST, "account_id is required\n");
                    }
                };
                let account_id_ref = account_id.as_str();
                let provider_kind = account_payload
                    .get("provider_kind")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("imap");
                if !account_payload
                    .get("config")
                    .is_none_or(serde_json::Value::is_object)
                {
                    return response(
                        StatusCode::BAD_REQUEST,
                        "account.config must be an object\n",
                    );
                }
                if !is_known_mail_provider_kind(provider_kind) {
                    return response(StatusCode::BAD_REQUEST, "unsupported email provider kind\n");
                }
                if payload
                    .get("sync_settings")
                    .is_some_and(|value| !value.is_object())
                {
                    return response(StatusCode::BAD_REQUEST, "sync_settings must be an object\n");
                }
                upsert_account_entry(
                    &account_id,
                    provider_kind,
                    &json!({
                        "account_id": account_id_ref,
                        "provider_kind": provider_kind,
                        "display_name": account_payload
                            .get("display_name")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or(account_id_ref),
                        "external_account_id": account_payload
                            .get("external_account_id")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or(account_id_ref),
                        "config": account_payload
                            .get("config")
                            .cloned()
                            .filter(|value| value.is_object())
                            .unwrap_or_else(|| json!({})),
                    }),
                    payload.get("sync_settings"),
                );
                let mut response = match account_view(&account_id) {
                    Some(value) => value,
                    None => {
                        return response(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "failed to read imported email account\n",
                        );
                    }
                };
                response["sync_settings"] = sync_settings_for_account(&account_id);
                json_response(StatusCode::OK, response)
            }
            Some(MailRoute::AccountImapSetup) => {
                let payload =
                    serde_json::from_slice::<JsonValue>(&body).unwrap_or_else(|_| json!({}));
                let secret_kind = match payload
                    .get("secret_kind")
                    .and_then(serde_json::Value::as_str)
                    .filter(|value| !value.trim().is_empty())
                {
                    Some(value) => value,
                    None => {
                        return response(StatusCode::BAD_REQUEST, "imap secret_kind is required\n");
                    }
                };
                if secret_kind != "app_password" && secret_kind != "imap_password_file" {
                    return response(StatusCode::BAD_REQUEST, "unsupported imap secret_kind\n");
                }
                let account_id = match payload
                    .get("account_id")
                    .and_then(serde_json::Value::as_str)
                    .filter(|value| !value.trim().is_empty())
                {
                    Some(value) => value.to_owned(),
                    None => {
                        return response(StatusCode::BAD_REQUEST, "imap account_id is required\n");
                    }
                };
                let account_id_ref = account_id.as_str();
                let password = if let Some(password) = payload
                    .get("password")
                    .and_then(serde_json::Value::as_str)
                    .filter(|value| !value.trim().is_empty())
                {
                    password
                } else {
                    return response(StatusCode::BAD_REQUEST, "imap password is required\n");
                };
                let host = if let Some(host) = payload
                    .get("host")
                    .and_then(serde_json::Value::as_str)
                    .filter(|value| !value.trim().is_empty())
                {
                    host
                } else {
                    return response(StatusCode::BAD_REQUEST, "imap host is required\n");
                };
                let username = if let Some(username) = payload
                    .get("username")
                    .and_then(serde_json::Value::as_str)
                    .filter(|value| !value.trim().is_empty())
                {
                    username
                } else {
                    return response(StatusCode::BAD_REQUEST, "imap username is required\n");
                };
                let port =
                    if let Some(port) = payload.get("port").and_then(serde_json::Value::as_u64) {
                        match u16::try_from(port) {
                            Ok(port) => port,
                            Err(_) => {
                                return response(StatusCode::BAD_REQUEST, "imap port is invalid\n");
                            }
                        }
                    } else {
                        DEFAULT_IMAP_PORT
                    };
                let password_file = match write_sync_password_secret(password) {
                    Ok(path) => path,
                    Err(_) => {
                        return response(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "failed to store imap password\n",
                        );
                    }
                };
                clear_account_sync_secret(&account_id);
                upsert_account_sync_secret(&account_id, password_file.as_str());
                upsert_account_entry(
                    &account_id,
                    "imap",
                    &json!({
                        "account_id": account_id_ref,
                        "provider_kind": "imap",
                        "display_name": payload
                            .get("display_name")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or(account_id_ref),
                        "external_account_id": payload
                            .get("external_account_id")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or(account_id_ref),
                        "config": {
                            "host": host,
                            "username": username,
                            "tls": payload.get("tls").and_then(serde_json::Value::as_bool).unwrap_or(true),
                            "port": port,
                        },
                    }),
                    None,
                );
                json_response(
                    StatusCode::OK,
                    json!({
                        "account_id": account_id,
                        "secret_ref": format!("secret://mail/{account_id}"),
                        "secret_kind": "imap_password_file",
                        "store_kind": "runtime_secret_store",
                    }),
                )
            }
            Some(MailRoute::AccountLogout(account_id)) => {
                if !account_exists(&account_id) {
                    return response(StatusCode::NOT_FOUND, "account not found\n");
                }
                logout_account(&account_id);
                let Some(mut response) = account_view(&account_id) else {
                    return response(StatusCode::NOT_FOUND, "account not found\n");
                };
                response["sync_settings"] = sync_settings_for_account(&account_id);
                json_response(StatusCode::OK, response)
            }
            Some(MailRoute::GmailOAuthStart) => {
                let payload =
                    serde_json::from_slice::<JsonValue>(&body).unwrap_or_else(|_| json!({}));
                let account_id = match payload
                    .get("account_id")
                    .and_then(serde_json::Value::as_str)
                    .filter(|value| !value.trim().is_empty())
                {
                    Some(value) => value.to_owned(),
                    None => {
                        return response(StatusCode::BAD_REQUEST, "gmail account_id is required\n");
                    }
                };
                let account_id_ref = account_id.as_str();
                let redirect_uri = payload
                    .get("redirect_uri")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("https://localhost/auth/callback");
                upsert_account_entry(
                    &account_id,
                    "gmail",
                    &json!({
                        "account_id": account_id_ref,
                        "provider_kind": "gmail",
                        "display_name": payload
                            .get("display_name")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or(account_id_ref),
                        "config": {
                            "auth": "oauth",
                        },
                    }),
                    None,
                );
                json_response(
                    StatusCode::OK,
                    json!({
                        "setup_id": format!("setup-{account_id}"),
                        "authorization_url": format!(
                            "https://example.com/oauth2/auth?account={account_id}"
                        ),
                        "state": format!("state-{account_id}"),
                        "redirect_uri": redirect_uri,
                    }),
                )
            }
            Some(MailRoute::GmailOAuthComplete) => {
                let payload =
                    serde_json::from_slice::<JsonValue>(&body).unwrap_or_else(|_| json!({}));
                let account_id = match payload
                    .get("account_id")
                    .and_then(serde_json::Value::as_str)
                    .filter(|value| !value.trim().is_empty())
                {
                    Some(value) => value.to_owned(),
                    None => {
                        return response(StatusCode::BAD_REQUEST, "gmail account_id is required\n");
                    }
                };
                let account_id_ref = account_id.as_str();
                upsert_account_entry(
                    &account_id,
                    "gmail",
                    &json!({
                        "account_id": account_id_ref,
                        "provider_kind": "gmail",
                        "auth_state": "connected",
                        "config": {
                            "auth": "oauth",
                        },
                    }),
                    None,
                );
                json_response(
                    StatusCode::OK,
                    json!({
                        "account_id": account_id,
                        "secret_ref": "secret://mail/gmail",
                        "secret_kind": "oauth_token",
                        "store_kind": "runtime_secret_store",
                    }),
                )
            }
            Some(MailRoute::GmailOAuthCallback) => response(
                StatusCode::METHOD_NOT_ALLOWED,
                "gmail/oauth/callback requires GET\n",
            ),
            Some(MailRoute::SensitiveForwardingPolicies(account_id)) => {
                let payload =
                    serde_json::from_slice::<JsonValue>(&body).unwrap_or_else(|_| json!({}));
                if !account_exists(&account_id) {
                    response(StatusCode::NOT_FOUND, "account not found\n")
                } else if !payload.is_object() {
                    response(
                        StatusCode::BAD_REQUEST,
                        "sensitive forwarding payload must be an object\n",
                    )
                } else {
                    let source_account_id = payload
                        .get("source_account_id")
                        .and_then(serde_json::Value::as_str)
                        .filter(|value| !value.is_empty())
                        .unwrap_or(&account_id);
                    if source_account_id != account_id {
                        return response(
                            StatusCode::BAD_REQUEST,
                            "sensitive forwarding source_account_id mismatch\n",
                        );
                    }
                    if payload
                        .get("delivery_account_id")
                        .and_then(serde_json::Value::as_str)
                        .filter(|value| !value.trim().is_empty())
                        .is_none()
                    {
                        return response(
                            StatusCode::BAD_REQUEST,
                            "sensitive forwarding delivery_account_id is required\n",
                        );
                    }
                    if !account_exists(
                        payload
                            .get("delivery_account_id")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or(""),
                    ) {
                        return response(
                            StatusCode::BAD_REQUEST,
                            "sensitive forwarding delivery account not found\n",
                        );
                    }
                    if payload
                        .get("policy_id")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|value| value.trim().is_empty())
                    {
                        return response(
                            StatusCode::BAD_REQUEST,
                            "sensitive forwarding policy_id must not be empty\n",
                        );
                    }
                    json_response(
                        StatusCode::OK,
                        create_or_update_sensitive_forwarding_policy(&account_id, &payload),
                    )
                }
            }
            Some(variant) => {
                let _ = variant;
                response(StatusCode::METHOD_NOT_ALLOWED, "method not allowed\n")
            }
            None => response(StatusCode::NOT_FOUND, "not found\n"),
        }
    }
}

fn parse_route(path: &str) -> Option<MailRoute> {
    if path == MAIL_ACCOUNTS_LIST_PATH {
        return Some(MailRoute::AccountsList);
    }
    if path == MAIL_COMMUNICATIONS_ACCOUNTS_LIST_PATH {
        return Some(MailRoute::CommunicationsAccountsList);
    }
    if path == MAIL_IMPORT_PATH {
        return Some(MailRoute::AccountImport);
    }
    if path == MAIL_IMAP_PATH {
        return Some(MailRoute::AccountImapSetup);
    }
    if path == MAIL_GMAIL_OAUTH_START_PATH {
        return Some(MailRoute::GmailOAuthStart);
    }
    if path == MAIL_GMAIL_OAUTH_COMPLETE_PATH {
        return Some(MailRoute::GmailOAuthComplete);
    }
    if path == MAIL_GMAIL_OAUTH_CALLBACK_PATH {
        return Some(MailRoute::GmailOAuthCallback);
    }
    if path == MAIL_SYNC_STATUS_PATH {
        return Some(MailRoute::SyncStatus);
    }

    if !path.starts_with(MAIL_ACCOUNTS_PREFIX) {
        return None;
    }

    let parts = path
        .trim_start_matches(MAIL_ACCOUNTS_PREFIX)
        .trim_start_matches('/')
        .split('/')
        .collect::<Vec<_>>();

    let account_id = parts.first().copied()?.to_owned();

    match parts.get(1).copied() {
        None => Some(MailRoute::Account(account_id)),
        Some("export") if parts.len() == 2 => Some(MailRoute::AccountExport(account_id)),
        Some("logout") if parts.len() == 2 => Some(MailRoute::AccountLogout(account_id)),
        Some("sync-settings") if parts.len() == 2 => Some(MailRoute::SyncSettings(account_id)),
        Some("content-egress-settings") if parts.len() == 2 => {
            Some(MailRoute::ContentEgressSettings(account_id))
        }
        Some("sensitive-forwarding-policies") if parts.len() == 2 => {
            Some(MailRoute::SensitiveForwardingPolicies(account_id))
        }
        Some("sensitive-forwarding-policies") if parts.len() == 3 => {
            let policy_id = parts.get(2).copied().unwrap_or_default().to_owned();
            Some(MailRoute::SensitiveForwardingPolicyDelete(
                account_id, policy_id,
            ))
        }
        Some("provider-resources") if parts.len() == 2 => {
            Some(MailRoute::ProviderResources(account_id))
        }
        Some("provider-resources") if parts.len() == 3 => {
            let mapping_id = parts.get(2).copied().unwrap_or_default().to_owned();
            Some(MailRoute::ProviderResourceUpdate(account_id, mapping_id))
        }
        Some("sync-now") if parts.len() == 2 => Some(MailRoute::SyncNow(account_id)),
        Some("sync-full-resync") if parts.len() == 2 => Some(MailRoute::SyncFullResync(account_id)),
        Some("address-book-sync-now") if parts.len() == 2 => {
            Some(MailRoute::AddressBookSyncNow(account_id))
        }
        _ => None,
    }
}

fn mail_accounts_store() -> &'static Mutex<HashMap<String, JsonValue>> {
    MAIL_ACCOUNT_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn mail_sync_settings_store() -> &'static Mutex<HashMap<String, JsonValue>> {
    MAIL_SYNC_SETTINGS_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn run_mail_sync(account_id: &str, trigger: &str, body: Vec<u8>) -> GatewayHttpResponse {
    let payload = serde_json::from_slice::<JsonValue>(&body).unwrap_or_else(|_| json!({}));
    if !runtime_available() {
        return json_response(
            StatusCode::SERVICE_UNAVAILABLE,
            json!({"error": "mail runtime is unavailable"}),
        );
    }

    let Some(account) = mail_account_entry(account_id) else {
        return response(StatusCode::NOT_FOUND, "account not found\n");
    };

    let Some((host, username, _tls, port)) = account_connection_config(&account) else {
        return response(
            StatusCode::BAD_REQUEST,
            "email account is missing valid imap connection data\n",
        );
    };

    let batch_size = sync_settings_for_account(account_id)
        .get("batch_size")
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| (1..=MAX_MAIL_BATCH_SIZE).contains(value))
        .unwrap_or(DEFAULT_MAIL_BATCH_SIZE);
    let windows = sync_settings_for_account(account_id)
        .get("windows")
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| (MIN_MAIL_SYNC_WINDOWS..=MAX_MAIL_SYNC_WINDOWS).contains(value))
        .unwrap_or(DEFAULT_MAIL_SYNC_WINDOWS);

    let run_id = format!("mail-run-{}", status_now().unwrap_or(0));
    let mut begin_args = vec![account_id.to_owned(), host.clone(), username.clone()];

    if port != DEFAULT_IMAP_PORT {
        begin_args.push("--port".to_owned());
        begin_args.push(port.to_string());
    }

    let mut sync_args = vec![account_id.to_owned(), run_id.clone()];
    let sync_host = host;
    let sync_username = username;
    sync_args.push("--host".to_owned());
    sync_args.push(sync_host);
    sync_args.push("--username".to_owned());
    sync_args.push(sync_username);
    sync_args.push("--port".to_owned());
    sync_args.push(port.to_string());
    sync_args.push("--window".to_owned());
    sync_args.push(batch_size.to_string());
    sync_args.push("--windows".to_owned());
    sync_args.push(windows.to_string());

    if trigger == "address_book_sync" {
        sync_args.push("--emit-observations".to_owned());
    }

    if payload
        .get("emit_observations")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        sync_args.push("--emit-observations".to_owned());
    }

    if trigger == "full_resync"
        || payload
            .get("full_resync")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
    {
        sync_args.push("--full-resync".to_owned());
    }

    if payload.get("password").is_some() || payload.get("password_file").is_some() {
        return response(
            StatusCode::BAD_REQUEST,
            "sync request must not contain password credentials\n",
        );
    }

    if let Some(password_file) = account_sync_secret(account_id) {
        sync_args.push("--password-file".to_owned());
        sync_args.push(password_file);
    } else {
        return response(
            StatusCode::BAD_REQUEST,
            "email account is missing configured sync credentials\n",
        );
    }

    let begin_args: Vec<&str> = begin_args.iter().map(String::as_str).collect();
    let sync_args_refs: Vec<&str> = sync_args.iter().map(String::as_str).collect();
    if let Err(err) = execute_mail_runtime_command("begin", &begin_args) {
        return json_response(StatusCode::SERVICE_UNAVAILABLE, json!({"error": err}));
    }

    let command_result = execute_mail_runtime_command("sync", &sync_args_refs);
    if let Err(err) = command_result {
        return json_response(StatusCode::SERVICE_UNAVAILABLE, json!({"error": err}));
    }

    let command_result = command_result.unwrap_or(MailRuntimeCommandResult {
        exit_code: 0,
        stdout: String::new(),
        stderr: String::new(),
    });
    if command_result.exit_code != 0 {
        return json_response(
            StatusCode::SERVICE_UNAVAILABLE,
            json!({"error": "mail runtime command failed", "details": command_result.stderr}),
        );
    }

    if let Err(err) = execute_mail_runtime_command("complete", &[account_id, &run_id]) {
        return json_response(
            StatusCode::SERVICE_UNAVAILABLE,
            json!({"error": "mail runtime completion failed", "details": err}),
        );
    }

    let processed_messages = parse_sync_result_message_count(&command_result.stdout);
    let projected_messages = processed_messages;

    json_response(
        StatusCode::OK,
        json!({
            "run_id": run_id,
            "account_id": account_id,
            "trigger": trigger,
            "status": "running",
            "phase": "running",
            "progress_mode": "indeterminate",
            "progress_percent": JsonValue::Null,
            "processed_messages": processed_messages,
                "estimated_total_messages": JsonValue::Null,
            "current_batch_size": batch_size,
            "fetched_messages": processed_messages,
            "provider": "mail-imap",
            "projected_messages": projected_messages,
            "upserted_personas": 0,
            "upserted_organizations": 0,
            "checkpoint_before_present": false,
            "checkpoint_after_present": false,
            "checkpoint_saved": false,
                "failure_reason": JsonValue::Null,
            "started_at": DEFAULT_TIMESTAMP,
            "completed_at": JsonValue::Null,
            "next_run_at": JsonValue::Null,
        }),
    )
}

fn runtime_available() -> bool {
    execute_mail_runtime_command("status", &[]).is_ok()
}

fn parse_sync_result_message_count(stdout: &str) -> u32 {
    stdout
        .split_whitespace()
        .find_map(|part| {
            part.strip_prefix("messages=")
                .and_then(|value| value.parse::<u32>().ok())
        })
        .unwrap_or(0)
}

fn mail_account_entry(account_id: &str) -> Option<JsonValue> {
    mail_accounts_store()
        .lock()
        .ok()
        .and_then(|store| store.get(account_id).cloned())
}

fn mail_account_sync_secret_store() -> &'static Mutex<HashMap<String, String>> {
    MAIL_SYNC_SECRET_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn account_sync_secret(account_id: &str) -> Option<String> {
    mail_account_sync_secret_store()
        .lock()
        .ok()
        .and_then(|store| store.get(account_id).cloned())
}

fn upsert_account_sync_secret(account_id: &str, password: &str) {
    if let Ok(mut store) = mail_account_sync_secret_store().lock() {
        store.insert(account_id.to_owned(), password.to_owned());
    }
}

fn write_sync_password_secret(password: &str) -> Result<String, String> {
    let mut path = env::temp_dir();
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "system time is unavailable")?
        .as_nanos();
    path.push(format!("hermes-mail-sync-secret-{suffix}"));
    fs::write(&path, password).map_err(|_| "mail password storage failed".to_owned())?;
    Ok(path.to_string_lossy().into_owned())
}

fn clear_account_sync_secret(account_id: &str) {
    if let Ok(mut store) = mail_account_sync_secret_store().lock()
        && let Some(path) = store.remove(account_id)
    {
        let _ = fs::remove_file(&path);
    }
}

fn account_connection_config(account: &JsonValue) -> Option<(String, String, bool, u16)> {
    let config = account.get("config")?.as_object()?;
    let host = config
        .get("host")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)?;
    let username = config
        .get("username")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)?;
    let tls = config
        .get("tls")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    let port = config
        .get("port")
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| u16::try_from(value).ok())
        .unwrap_or(DEFAULT_IMAP_PORT);
    if host.is_empty() || username.is_empty() || port == 0 {
        return None;
    }
    Some((host, username, tls, port))
}

fn query_mail_sync_status() -> GatewayHttpResponse {
    match execute_mail_runtime_command("status", &[]) {
        Ok(result) if result.exit_code == 0 => {
            let raw_status = result.stdout.trim();
            let mut items = mail_account_ids()
                .into_iter()
                .map(|account_id| {
                    let current_batch_size =
                        sync_batch_size_for_account(&account_id).unwrap_or(DEFAULT_MAIL_BATCH_SIZE);
                    sync_status_entry(&account_id, raw_status, current_batch_size)
                })
                .collect::<Vec<_>>();
            if items.is_empty() {
                items.push(sync_status_entry(
                    "runtime",
                    raw_status,
                    DEFAULT_MAIL_BATCH_SIZE,
                ));
            }
            json_response(StatusCode::OK, json!({ "items": items }))
        }
        Ok(_) | Err(_) => {
            let items = mail_account_ids()
                .into_iter()
                .map(|account_id| {
                    sync_status_entry(&account_id, "runtime unavailable", DEFAULT_MAIL_BATCH_SIZE)
                })
                .collect::<Vec<_>>();
            let items = if items.is_empty() {
                vec![sync_status_entry(
                    "runtime",
                    "runtime unavailable",
                    DEFAULT_MAIL_BATCH_SIZE,
                )]
            } else {
                items
            };
            json_response(StatusCode::OK, json!({ "items": items }))
        }
    }
}

fn sync_batch_size_for_account(account_id: &str) -> Option<u32> {
    sync_settings_for_account(account_id)
        .get("batch_size")
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| (1..=MAX_MAIL_BATCH_SIZE).contains(value))
}

fn sync_status_entry(account_id: &str, status: &str, current_batch_size: u32) -> JsonValue {
    json!({
        "account_id": account_id,
        "status": status,
        "phase": "runtime",
        "progress_mode": "none",
        "progress_percent": 100,
        "processed_messages": 0,
        "estimated_total_messages": 0,
        "current_batch_size": current_batch_size,
        "failure_threshold": 3,
        "last_started_at": JsonValue::Null,
        "last_updated_at": DEFAULT_TIMESTAMP,
        "last_completed_at": JsonValue::Null,
        "next_run_at": JsonValue::Null,
        "last_error_code": JsonValue::Null,
        "last_error_message": JsonValue::Null,
        "consecutive_failures": 0,
        "last_fetched_messages": 0,
        "last_projected_messages": 0,
        "last_upserted_personas": 0,
        "last_upserted_organizations": 0,
    })
}

fn mail_account_ids() -> Vec<String> {
    match mail_accounts_store().lock() {
        Ok(store) => store.keys().cloned().collect::<Vec<_>>(),
        Err(_) => Vec::new(),
    }
}

fn sensitive_forwarding_policies_for_account_response(account_id: &str) -> JsonValue {
    json!({
        "items": sensitive_forwarding_policies_for_account(account_id),
    })
}

fn sensitive_forwarding_policies_for_account(account_id: &str) -> Vec<JsonValue> {
    let prefix = format!("{account_id}:");
    let mut items = mail_sensitive_forwarding_store()
        .lock()
        .map(|store| {
            store
                .iter()
                .filter_map(|(key, policy)| key.strip_prefix(&prefix).map(|_| policy.clone()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    items.sort_by(|left, right| left["policy_id"].as_str().cmp(&right["policy_id"].as_str()));
    items
}

fn create_or_update_sensitive_forwarding_policy(
    account_id: &str,
    payload: &JsonValue,
) -> JsonValue {
    let policy_id = payload
        .get("policy_id")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| {
            let suffix = status_now()
                .map(|value| value.to_string())
                .unwrap_or_else(|_| "0".to_owned());
            format!("mail-sensitive-forwarding:{account_id}:{suffix}")
        });
    let policy = json!({
        "policy_id": policy_id,
        "source_account_id": account_id,
        "delivery_account_id": payload
            .get("delivery_account_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(account_id),
        "name": payload
            .get("name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("Forwarding"),
        "enabled": payload
            .get("enabled")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true),
        "include_message_body": payload
            .get("include_message_body")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true),
        "include_attachments": payload
            .get("include_attachments")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true),
        "fixed_recipients": payload
            .get("fixed_recipients")
            .cloned()
            .unwrap_or_else(|| json!([])),
        "minimum_severity": payload
            .get("minimum_severity")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("low"),
        "subject_template": payload
            .get("subject_template")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(""),
        "body_template": payload
            .get("body_template")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(""),
        "max_sends_per_hour": payload
            .get("max_sends_per_hour")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0),
        "quiet_hours": payload.get("quiet_hours").cloned().unwrap_or_else(|| json!({})),
        "expires_at": payload.get("expires_at").cloned(),
        "updated_at": DEFAULT_TIMESTAMP,
    });
    let storage_key = format!("{account_id}:{policy_id}");
    if let Ok(mut store) = mail_sensitive_forwarding_store().lock() {
        store.insert(storage_key, policy.clone());
    }
    json!({
        "items": sensitive_forwarding_policies_for_account(account_id),
    })
}

fn delete_sensitive_forwarding_policy(account_id: &str, policy_id: &str) -> JsonValue {
    let storage_key = format!("{account_id}:{policy_id}");
    let deleted = if let Ok(mut store) = mail_sensitive_forwarding_store().lock() {
        store.remove(&storage_key).is_some()
    } else {
        false
    };
    json!({
        "policy_id": policy_id,
        "deleted": deleted,
    })
}

fn provider_resources_for_account_response(account_id: &str) -> JsonValue {
    ensure_account_resources(account_id);
    json!({
        "items": provider_resources_for_account(account_id),
    })
}

fn provider_resources_for_account(account_id: &str) -> Vec<JsonValue> {
    let prefix = format!("{account_id}:");
    let mut items = mail_provider_resource_store()
        .lock()
        .map(|store| {
            store
                .iter()
                .filter_map(|(key, resource)| key.strip_prefix(&prefix).map(|_| resource.clone()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    items.sort_by(|left, right| {
        left["mapping_id"]
            .as_str()
            .cmp(&right["mapping_id"].as_str())
    });
    items
}

fn provider_resource_mapping_update(
    account_id: &str,
    mapping_id: &str,
    payload: &JsonValue,
) -> Option<JsonValue> {
    let storage_key = format!("{account_id}:{mapping_id}");
    let mut map = mail_provider_resource_store().lock().ok()?;
    let mut resource = map.get(&storage_key)?.as_object()?.clone();
    if resource
        .get("account_id")
        .and_then(serde_json::Value::as_str)
        != Some(account_id)
    {
        return None;
    }
    if let Some(semantic_role) = payload.get("semantic_role").cloned() {
        resource.insert("semantic_role".to_owned(), semantic_role);
    }
    if let Some(local_folder_id) = payload.get("local_folder_id").cloned() {
        resource.insert("local_folder_id".to_owned(), local_folder_id);
    }
    resource.insert("mapping_source".to_owned(), json!("manual"));
    resource.insert("updated_at".to_owned(), json!(DEFAULT_TIMESTAMP));
    map.insert(storage_key, JsonValue::Object(resource.clone()));
    Some(JsonValue::Object(resource))
}

fn ensure_account_resources(account_id: &str) {
    let resources_exist = !provider_resources_for_account(account_id).is_empty();
    if resources_exist {
        return;
    }
    if let Ok(mut store) = mail_provider_resource_store().lock() {
        let base_resource = json!({
            "resource_kind": "folder",
            "provider_resource_id": format!("{account_id}-inbox"),
            "display_name": "Inbox",
            "semantic_role": JsonValue::Null,
            "local_folder_id": JsonValue::Null,
            "selectable": true,
            "writable": true,
            "mapping_source": "discovered",
            "capabilities": json!({}),
            "observed_at": DEFAULT_TIMESTAMP,
            "created_at": DEFAULT_TIMESTAMP,
            "updated_at": DEFAULT_TIMESTAMP,
        });
        let inbox_mapping_id = format!("discovered-{account_id}-inbox");
        let mut account_resource = base_resource
            .as_object()
            .cloned()
            .unwrap_or_else(JsonObject::new);
        account_resource.insert("mapping_id".to_owned(), json!(inbox_mapping_id.clone()));
        account_resource.insert("account_id".to_owned(), json!(account_id));
        store.insert(
            format!("{account_id}:{inbox_mapping_id}"),
            JsonValue::Object(account_resource),
        );
    }
}

fn gmail_oauth_callback_response(query: Option<&str>) -> GatewayHttpResponse {
    let Some(query) = query else {
        return gmail_oauth_callback_error(
            StatusCode::BAD_REQUEST,
            "Google mail connection failed: missing callback parameters.",
        );
    };

    if let Some(_error) = query_param(query, "error").filter(|value| !value.trim().is_empty()) {
        return gmail_oauth_callback_error(
            StatusCode::BAD_REQUEST,
            "Google authorization failed. Start the mail connection again.",
        );
    }

    let code = query_param(query, "code");
    let state = query_param(query, "state");

    if code.as_deref().filter(|value| !value.is_empty()).is_none() {
        return gmail_oauth_callback_error(
            StatusCode::BAD_REQUEST,
            "Missing authorization code. Start the mail connection again.",
        );
    }
    if state.as_deref().filter(|value| !value.is_empty()).is_none() {
        return gmail_oauth_callback_error(
            StatusCode::BAD_REQUEST,
            "Missing OAuth state. Start the mail connection again.",
        );
    }

    let account_id = match state
        .as_deref()
        .and_then(|value| value.strip_prefix("setup-"))
        .filter(|value| !value.is_empty())
    {
        Some(value) => value,
        None => {
            return gmail_oauth_callback_error(
                StatusCode::BAD_REQUEST,
                "Invalid OAuth state. Start the mail connection again.",
            );
        }
    };
    let _ = code;
    gmail_oauth_callback_success(account_id)
}

fn gmail_oauth_callback_success(account_id: &str) -> GatewayHttpResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/html; charset=utf-8")
        .body(full_gateway_body(Bytes::from(format!(
            "<!doctype html>
<html lang=\"en\">
<head>
  <meta charset=\"utf-8\">
  <title>Hermes Hub OAuth</title>
  <meta http-equiv=\"refresh\" content=\"0; url=http://127.0.0.1:5174/?hermes_route=settings&amp;hermes_oauth=gmail_connected\" />
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
  </script>
</head>
<body>
  <h1>Google mail connected</h1>
  <p>Hermes Hub saved the Google mail account and encrypted OAuth credential locally.</p>
  <p>Account</p>
  <code>{account_id}</code>
  <p>This tab will close automatically.</p>
</body>
</html>"
        ))))
        .expect("Gmail callback response is valid")
}

fn gmail_oauth_callback_error(status: StatusCode, message: &str) -> GatewayHttpResponse {
    Response::builder()
        .status(status)
        .header("content-type", "text/html; charset=utf-8")
        .body(full_gateway_body(Bytes::from(format!(
            "<!doctype html>
<html lang=\"en\">
<head>
  <meta charset=\"utf-8\">
  <title>Hermes Hub OAuth</title>
</head>
<body>
  <h1>Google mail connection failed</h1>
  <p>{message}</p>
</body>
</html>"
        ))))
        .expect("Gmail callback error response is valid")
}

fn query_param(query: &str, key: &str) -> Option<String> {
    for part in query.split('&') {
        let mut split = part.splitn(2, '=');
        let current_key = split.next()?;
        let value = split.next().unwrap_or("");
        if current_key == key {
            return Some(value.to_owned());
        }
    }
    None
}

fn default_sync_settings(account_id: &str) -> JsonValue {
    json!({
        "account_id": account_id,
        "sync_enabled": true,
        "batch_size": DEFAULT_MAIL_BATCH_SIZE,
        "windows": DEFAULT_MAIL_SYNC_WINDOWS,
        "poll_interval_seconds": 300,
        "failure_threshold": 3,
        "updated_at": DEFAULT_TIMESTAMP,
    })
}

fn sync_settings_for_account(account_id: &str) -> JsonValue {
    if let Ok(mut store) = mail_sync_settings_store().lock() {
        if let Some(stored) = store.get(account_id) {
            return stored.clone();
        }
        let default = default_sync_settings(account_id);
        store.insert(account_id.to_owned(), default.clone());
        return default;
    }
    default_sync_settings(account_id)
}

fn upsert_sync_settings(account_id: &str, payload: &JsonValue) {
    let default = default_sync_settings(account_id);
    let sync_enabled = payload
        .get("sync_enabled")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(default["sync_enabled"].as_bool().unwrap_or(true));
    let batch_size = payload
        .get("batch_size")
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| (1..=MAX_MAIL_BATCH_SIZE).contains(value))
        .unwrap_or_else(|| {
            default["batch_size"]
                .as_u64()
                .unwrap_or(DEFAULT_MAIL_BATCH_SIZE as u64) as u32
        });
    let windows = payload
        .get("windows")
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| (MIN_MAIL_SYNC_WINDOWS..=MAX_MAIL_SYNC_WINDOWS).contains(value))
        .unwrap_or_else(|| {
            default["windows"]
                .as_u64()
                .unwrap_or(DEFAULT_MAIL_SYNC_WINDOWS as u64) as u32
        });
    let poll_interval_seconds = payload
        .get("poll_interval_seconds")
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| {
            (MIN_MAIL_POLL_INTERVAL_SECONDS..=MAX_MAIL_POLL_INTERVAL_SECONDS).contains(value)
        })
        .unwrap_or_else(|| default["poll_interval_seconds"].as_u64().unwrap_or(300) as u32);
    let failure_threshold = payload
        .get("failure_threshold")
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| (MIN_MAIL_FAILURE_THRESHOLD..=MAX_MAIL_FAILURE_THRESHOLD).contains(value))
        .unwrap_or(default["failure_threshold"].as_u64().unwrap_or(3) as u32);

    let updated = json!({
        "account_id": account_id,
        "sync_enabled": sync_enabled,
        "batch_size": batch_size,
        "windows": windows,
        "poll_interval_seconds": poll_interval_seconds,
        "failure_threshold": failure_threshold,
        "updated_at": DEFAULT_TIMESTAMP,
    });

    if let Ok(mut store) = mail_sync_settings_store().lock() {
        store.insert(account_id.to_owned(), updated);
    }
}

fn upsert_account_entry(
    account_id: &str,
    provider_kind: &str,
    account_payload: &JsonValue,
    sync_payload: Option<&JsonValue>,
) {
    let mut account = match account_payload.as_object().cloned() {
        Some(value) => value,
        None => {
            let mut fallback: JsonObject = JsonObject::new();
            fallback.insert("account_id".to_owned(), json!(account_id));
            fallback.insert("provider_kind".to_owned(), json!(provider_kind));
            fallback
        }
    };

    account.insert("provider_kind".to_owned(), json!(provider_kind));
    account.insert("account_id".to_owned(), json!(account_id));
    account.insert("updated_at".to_owned(), json!(DEFAULT_TIMESTAMP));
    account.remove("password");
    account.remove("password_file");

    if let Ok(mut store) = mail_accounts_store().lock() {
        store.insert(account_id.to_owned(), JsonValue::Object(account));
    }

    if let Some(payload) = sync_payload {
        upsert_sync_settings(account_id, payload);
    }
    ensure_account_resources(account_id);
}

fn account_view(account_id: &str) -> Option<JsonValue> {
    let mut account = {
        let store = mail_accounts_store().lock().ok()?;
        store.get(account_id).cloned()?
    };
    strip_secret_fields(&mut account);
    let auth_state = account
        .get("auth_state")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("connected");
    let connected = auth_state != "logged_out";
    let provider_kind = account
        .get("provider_kind")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("imap");
    Some(json!({
        "account": account,
        "capabilities": account_capabilities(&account, provider_kind, connected),
    }))
}

fn account_capabilities(account: &JsonValue, provider_kind: &str, connected: bool) -> JsonValue {
    let provider_kind = provider_kind.to_ascii_lowercase();
    let smtp = account_smtp_configured(account);
    let gmail_modify_enabled = account
        .get("config")
        .and_then(serde_json::Value::as_object)
        .is_some_and(|object| {
            object
                .get("gmail_modify_enabled")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
                || object
                    .get("requested_scopes")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|scopes| {
                        scopes.iter().any(|scope| {
                            scope.as_str().is_some_and(|value| {
                                value == "https://www.googleapis.com/auth/gmail.modify"
                            })
                        })
                    })
        });
    let gmail_send_enabled = account
        .get("config")
        .and_then(serde_json::Value::as_object)
        .and_then(|object| {
            object
                .get("gmail_send_enabled")
                .and_then(serde_json::Value::as_bool)
        })
        .unwrap_or(false);
    let provider_mutations_enabled =
        connected && (matches!(provider_kind.as_str(), "imap" | "icloud") || gmail_modify_enabled);
    let oauth = provider_kind == "gmail"
        || account
            .get("config")
            .and_then(serde_json::Value::as_object)
            .is_some_and(|object| {
                object.get("auth").and_then(serde_json::Value::as_str) == Some("oauth")
            });
    json!({
        "read": connected,
        "sync": connected,
        "send": connected && (smtp || gmail_send_enabled),
        "oauth": oauth,
        "imap": matches!(provider_kind.as_str(), "imap" | "icloud"),
        "smtp": smtp,
        "mutate_flags": provider_mutations_enabled,
        "mutate_mailboxes": provider_mutations_enabled,
        "server_delete": false,
        "provider_folders": provider_mutations_enabled,
        "local_trash": true,
    })
}

fn account_smtp_configured(account: &JsonValue) -> bool {
    let explicit_smtp = account
        .get("config")
        .and_then(serde_json::Value::as_object)
        .is_some_and(|object| {
            object
                .get("smtp_host")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|host| !host.trim().is_empty())
                && object
                    .get("smtp_port")
                    .and_then(serde_json::Value::as_i64)
                    .is_some()
        });
    explicit_smtp
        || (account
            .get("provider_kind")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| value == "icloud")
            && account
                .get("external_account_id")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| !value.trim().is_empty()))
}

fn account_export_response(account_id: &str) -> Option<JsonValue> {
    let mut response = account_view(account_id)?;
    if let Some(obj) = response.as_object_mut() {
        obj.insert("exported_at".to_owned(), json!(DEFAULT_TIMESTAMP));
        obj.insert(
            "sync_settings".to_owned(),
            sync_settings_for_account(account_id),
        );
    }
    Some(response)
}

fn account_content_egress_settings(account: &JsonValue) -> JsonValue {
    let defaults = json!({
        "body": false,
        "attachments": false,
        "extracted_text": false,
    });
    let content_egress = defaults.as_object().cloned().unwrap_or_default();
    if let Some(config) = account.get("config").and_then(serde_json::Value::as_object)
        && let Some(stored) = config
            .get("content_egress")
            .and_then(serde_json::Value::as_object)
    {
        let merged = [
            ("body", "body"),
            ("attachments", "attachments"),
            ("extracted_text", "extracted_text"),
        ]
        .into_iter()
        .fold(content_egress, |mut next, (key, source_key)| {
            if let Some(value) = stored.get(source_key) {
                next.insert(key.to_owned(), value.clone());
            }
            next
        });
        return JsonValue::Object(merged);
    }
    JsonValue::Object(content_egress)
}

fn update_account_content_egress(account_id: &str, payload: &JsonValue) -> Option<JsonValue> {
    let mut current = account_view(account_id)?;
    let account = current.get_mut("account")?.as_object_mut()?;
    let mut config = account
        .get("config")
        .and_then(serde_json::Value::as_object)
        .cloned()
        .unwrap_or_default();
    let current_permissions = account_content_egress_settings(&json!({
        "config": JsonValue::Object(config.clone())
    }));
    let body = payload
        .get("body")
        .and_then(serde_json::Value::as_bool)
        .or_else(|| {
            current_permissions
                .get("body")
                .and_then(serde_json::Value::as_bool)
        })
        .unwrap_or(false);
    let attachments = payload
        .get("attachments")
        .and_then(serde_json::Value::as_bool)
        .or_else(|| {
            current_permissions
                .get("attachments")
                .and_then(serde_json::Value::as_bool)
        })
        .unwrap_or(false);
    let extracted_text = payload
        .get("extracted_text")
        .and_then(serde_json::Value::as_bool)
        .or_else(|| {
            current_permissions
                .get("extracted_text")
                .and_then(serde_json::Value::as_bool)
        })
        .unwrap_or(false);
    config.insert(
        "content_egress".to_owned(),
        json!({
            "body": body,
            "attachments": attachments,
            "extracted_text": extracted_text
        }),
    );
    account.insert("config".to_owned(), JsonValue::Object(config));
    let mut updated_account = account.clone();
    updated_account.insert("updated_at".to_owned(), json!(DEFAULT_TIMESTAMP));
    if let Ok(mut store) = mail_accounts_store().lock() {
        store.insert(
            account_id.to_owned(),
            JsonValue::Object(updated_account.clone()),
        );
    }
    Some(json!({
        "body": body,
        "attachments": attachments,
        "extracted_text": extracted_text,
    }))
}

fn strip_secret_fields(value: &mut JsonValue) {
    match value {
        JsonValue::Object(object) => {
            for key in object
                .keys()
                .filter(|key| is_secret_field_key(key))
                .cloned()
                .collect::<Vec<_>>()
            {
                let _ = object.remove(&key);
            }
            for item in object.values_mut() {
                strip_secret_fields(item);
            }
        }
        JsonValue::Array(items) => {
            for item in items {
                strip_secret_fields(item);
            }
        }
        _ => {}
    }
}

fn is_secret_field_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    [
        "password",
        "password_file",
        "secret",
        "secret_ref",
        "token",
        "credential",
        "api_key",
        "private_key",
        "client_secret",
        "refresh_token",
        "access_token",
    ]
    .iter()
    .any(|marker| key.contains(marker))
}

fn contains_secret_material(value: &JsonValue) -> bool {
    match value {
        JsonValue::Object(object) => object
            .iter()
            .any(|(key, value)| is_secret_field_key(key) || contains_secret_material(value)),
        JsonValue::Array(items) => items.iter().any(contains_secret_material),
        _ => false,
    }
}

fn is_known_mail_provider_kind(provider_kind: &str) -> bool {
    matches!(
        provider_kind,
        "imap" | "gmail" | "icloud" | "microsoft365" | "office365"
    )
}

fn account_list_response() -> JsonValue {
    let items = mail_accounts_store()
        .lock()
        .map(|store| {
            store
                .keys()
                .filter_map(|account_id| account_view(account_id))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    json!({"items": items})
}

fn delete_account(account_id: &str) -> JsonValue {
    if let Ok(mut store) = mail_accounts_store().lock() {
        let _ = store.remove(account_id);
    }
    clear_account_sync_secret(account_id);
    if let Ok(mut store) = mail_provider_resource_store().lock() {
        let mut removed_keys: Vec<String> = store
            .keys()
            .filter_map(|key| {
                if key.starts_with(&format!("{account_id}:")) {
                    Some(key.clone())
                } else {
                    None
                }
            })
            .collect();
        for key in removed_keys.drain(..) {
            let _ = store.remove(&key);
        }
    }
    if let Ok(mut store) = mail_sensitive_forwarding_store().lock() {
        let mut removed_keys: Vec<String> = store
            .keys()
            .filter_map(|key| {
                if key.starts_with(&format!("{account_id}:")) {
                    Some(key.clone())
                } else {
                    None
                }
            })
            .collect();
        for key in removed_keys.drain(..) {
            let _ = store.remove(&key);
        }
    }
    if let Ok(mut settings) = mail_sync_settings_store().lock() {
        let _ = settings.remove(account_id);
    }
    json!({
        "account_id": account_id,
        "deleted": true,
        "unbound_secret_refs": [],
        "vault_deleted_secret_refs": [],
        "retained_secret_refs": [],
    })
}

fn logout_account(account_id: &str) {
    clear_account_sync_secret(account_id);
    if let Ok(mut store) = mail_accounts_store().lock()
        && let Some(entry) = store.get_mut(account_id)
        && let Some(entry) = entry.as_object_mut()
    {
        entry.insert("auth_state".to_owned(), json!("logged_out"));
        entry.insert("updated_at".to_owned(), json!(DEFAULT_TIMESTAMP));
    }
}

fn account_exists(account_id: &str) -> bool {
    mail_accounts_store()
        .lock()
        .map(|store| store.contains_key(account_id))
        .unwrap_or(false)
}

fn mail_provider_resource_store() -> &'static Mutex<HashMap<String, JsonValue>> {
    MAIL_PROVIDER_RESOURCE_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn mail_sensitive_forwarding_store() -> &'static Mutex<HashMap<String, JsonValue>> {
    MAIL_SENSITIVE_FORWARDING_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn execute_mail_runtime_command(
    command: &str,
    args: &[&str],
) -> Result<MailRuntimeCommandResult, String> {
    let binary = runtime_binary(MAIL_RUNTIME_PROCESS_ID)?;
    let output = Command::new(&binary)
        .arg(command)
        .args(args)
        .output()
        .map_err(|_| "mail runtime command is unavailable".to_owned())?;
    Ok(MailRuntimeCommandResult {
        exit_code: output
            .status
            .code()
            .unwrap_or(-1)
            .try_into()
            .unwrap_or(u32::MAX),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

fn status_now() -> Result<u128, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|delta| delta.as_millis())
        .map_err(|_| "system time is unavailable".to_owned())
}

fn response(status: StatusCode, body: &'static str) -> GatewayHttpResponse {
    Response::builder()
        .status(status)
        .body(full_gateway_body(body))
        .expect("Legacy mail response is valid")
}

fn json_response(status: StatusCode, body: JsonValue) -> GatewayHttpResponse {
    let bytes = serde_json::to_vec(&body).unwrap_or_else(|_| b"{}".to_vec());
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(full_gateway_body(bytes))
        .expect("Legacy mail JSON response is valid")
}

fn method_not_allowed() -> GatewayHttpResponse {
    response(StatusCode::METHOD_NOT_ALLOWED, "method not allowed\n")
}

async fn collect_body<B>(body: B) -> Vec<u8>
where
    B: Body<Data = Bytes>,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    match Limited::new(body, MAX_REQUEST_BYTES).collect().await {
        Ok(collected) => collected.to_bytes().to_vec(),
        Err(_) => Vec::new(),
    }
}

fn runtime_binary(process_id: &str) -> Result<String, String> {
    if let Ok(binary) = env::var(format!("CARGO_BIN_EXE_{}", process_id.replace('-', "_"))) {
        return Ok(binary);
    }
    let exe = if cfg!(windows) {
        format!("{process_id}.exe")
    } else {
        process_id.to_owned()
    };
    let fallback = std::env::current_exe()
        .ok()
        .and_then(|path| {
            path.parent()
                .map(|parent| parent.join(&exe).to_string_lossy().into_owned())
        })
        .unwrap_or(exe);
    if Path::new(&fallback).exists() {
        return Ok(fallback);
    }
    Err("runtime command is unavailable".to_owned())
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use http_body_util::{BodyExt, Full};
    use hyper::{Method, Request, StatusCode};

    use super::MailGatewayIntegrationRouter;

    fn with_body(method: Method, uri: &str, body: &serde_json::Value) -> Request<Full<Bytes>> {
        Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(
                serde_json::to_vec(body).unwrap_or_else(|_| b"{}".to_vec()),
            )))
            .expect("test request body should build")
    }

    async fn response_status_text(response: super::GatewayHttpResponse) -> (StatusCode, String) {
        let status = response.status();
        let body = response
            .into_body()
            .collect()
            .await
            .map(|data| String::from_utf8_lossy(&data.to_bytes()).to_string())
            .unwrap_or_default();
        (status, body)
    }

    #[tokio::test]
    async fn imap_setup_rejects_unsupported_secret_kind() {
        let router = MailGatewayIntegrationRouter::new();
        let payload = serde_json::json!({
            "account_id": "mail-legacy",
            "provider_kind": "imap",
            "display_name": "Legacy",
            "external_account_id": "user@example.com",
            "host": "imap.example.com",
            "port": 993,
            "tls": true,
            "mailbox": "INBOX",
            "username": "user@example.com",
            "password": "secret",
            "secret_kind": "password"
        });

        let request = with_body(
            Method::POST,
            "http://localhost/api/v1/integrations/mail/accounts/imap",
            &payload,
        );

        let response = router.route(request).await;
        let (status, body) = response_status_text(response).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(body.contains("unsupported imap secret_kind"));
    }

    #[tokio::test]
    async fn put_sync_settings_rejects_out_of_range_windows_with_normalization() {
        let router = MailGatewayIntegrationRouter::new();
        let account_id = "sync-settings-route-account-out-of-range";
        super::upsert_account_entry(account_id, "imap", &serde_json::json!({}), None);

        let payload = serde_json::json!({
            "windows": super::MAX_MAIL_SYNC_WINDOWS + 1
        });
        let request = with_body(
            Method::PUT,
            &format!(
                "http://localhost/api/v1/integrations/mail/accounts/{account_id}/sync-settings"
            ),
            &payload,
        );

        let response = router.route(request).await;
        let (status, body) = response_status_text(response).await;
        let parsed: serde_json::Value =
            serde_json::from_str(&body).unwrap_or_else(|_| serde_json::json!({}));

        assert_eq!(status, StatusCode::OK);
        assert_eq!(parsed["windows"], super::DEFAULT_MAIL_SYNC_WINDOWS);
    }

    #[tokio::test]
    async fn put_sync_settings_rejects_too_small_windows_with_normalization() {
        let router = MailGatewayIntegrationRouter::new();
        let account_id = "sync-settings-route-account-too-small";
        super::upsert_account_entry(account_id, "imap", &serde_json::json!({}), None);

        let payload = serde_json::json!({
            "windows": 0
        });
        let request = with_body(
            Method::PUT,
            &format!(
                "http://localhost/api/v1/integrations/mail/accounts/{account_id}/sync-settings"
            ),
            &payload,
        );

        let response = router.route(request).await;
        let (status, body) = response_status_text(response).await;
        let parsed: serde_json::Value =
            serde_json::from_str(&body).unwrap_or_else(|_| serde_json::json!({}));

        assert_eq!(status, StatusCode::OK);
        assert_eq!(parsed["windows"], super::DEFAULT_MAIL_SYNC_WINDOWS);
    }

    #[tokio::test]
    async fn put_sync_settings_empty_payload_defaults_window_values() {
        let router = MailGatewayIntegrationRouter::new();
        let account_id = "sync-settings-route-account-empty-payload";
        super::upsert_account_entry(account_id, "imap", &serde_json::json!({}), None);

        let request = with_body(
            Method::PUT,
            &format!(
                "http://localhost/api/v1/integrations/mail/accounts/{account_id}/sync-settings"
            ),
            &serde_json::json!({}),
        );

        let response = router.route(request).await;
        let (status, body) = response_status_text(response).await;
        let parsed: serde_json::Value =
            serde_json::from_str(&body).unwrap_or_else(|_| serde_json::json!({}));

        assert_eq!(status, StatusCode::OK);
        assert_eq!(parsed["windows"], super::DEFAULT_MAIL_SYNC_WINDOWS);
        assert_eq!(parsed["batch_size"], super::DEFAULT_MAIL_BATCH_SIZE);
        assert_eq!(parsed["poll_interval_seconds"], 300);
    }

    #[tokio::test]
    async fn put_sync_settings_missing_account_returns_not_found() {
        let router = MailGatewayIntegrationRouter::new();
        let account_id = "sync-settings-route-account-missing";
        let payload = serde_json::json!({
            "windows": 1
        });
        let request = with_body(
            Method::PUT,
            &format!(
                "http://localhost/api/v1/integrations/mail/accounts/{account_id}/sync-settings"
            ),
            &payload,
        );

        let response = router.route(request).await;
        let (status, body) = response_status_text(response).await;

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body, "account not found\n");
    }

    #[tokio::test]
    async fn patch_sync_settings_is_method_not_allowed() {
        let router = MailGatewayIntegrationRouter::new();
        let account_id = "sync-settings-route-account-not-allowed";
        super::upsert_account_entry(account_id, "imap", &serde_json::json!({}), None);

        let payload = serde_json::json!({
            "windows": super::MAX_MAIL_SYNC_WINDOWS
        });
        let request = with_body(
            Method::PATCH,
            &format!(
                "http://localhost/api/v1/integrations/mail/accounts/{account_id}/sync-settings"
            ),
            &payload,
        );

        let response = router.route(request).await;
        let (status, body) = response_status_text(response).await;

        assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(body, "method not allowed\n");
    }

    #[test]
    fn sync_settings_reject_out_of_range_values() {
        let account_id = "sync-settings-test-account-1";
        let payload = serde_json::json!({
            "sync_enabled": false,
            "batch_size": super::MAX_MAIL_BATCH_SIZE + 1,
            "windows": super::MAX_MAIL_SYNC_WINDOWS + 1,
            "poll_interval_seconds": 30,
            "failure_threshold": 99
        });

        super::upsert_sync_settings(account_id, &payload);
        let settings = super::sync_settings_for_account(account_id);

        assert_eq!(settings["batch_size"], super::DEFAULT_MAIL_BATCH_SIZE);
        assert_eq!(settings["windows"], super::DEFAULT_MAIL_SYNC_WINDOWS);
        assert_eq!(settings["poll_interval_seconds"], 300);
        assert_eq!(settings["failure_threshold"], 3);
        assert_eq!(settings["sync_enabled"], false);
    }

    #[test]
    fn sync_settings_clamps_windows_below_minimum_to_default() {
        let account_id = "sync-settings-test-account-clamp-min";
        let payload = serde_json::json!({
            "sync_enabled": false,
            "batch_size": 1000,
            "windows": 0,
            "poll_interval_seconds": 60,
            "failure_threshold": 3
        });

        super::upsert_sync_settings(account_id, &payload);
        let settings = super::sync_settings_for_account(account_id);

        assert_eq!(settings["windows"], super::DEFAULT_MAIL_SYNC_WINDOWS);
    }

    #[test]
    fn sync_settings_accepts_max_windows_limit() {
        let account_id = "sync-settings-test-account-max-windows";
        let payload = serde_json::json!({
            "sync_enabled": false,
            "batch_size": super::MAX_MAIL_BATCH_SIZE,
            "windows": super::MAX_MAIL_SYNC_WINDOWS,
            "poll_interval_seconds": 120,
            "failure_threshold": 3
        });

        super::upsert_sync_settings(account_id, &payload);
        let settings = super::sync_settings_for_account(account_id);

        assert_eq!(settings["windows"], super::MAX_MAIL_SYNC_WINDOWS);
    }

    #[test]
    fn sync_settings_accepts_valid_values() {
        let account_id = "sync-settings-test-account-2";
        let batch_size = super::MAX_MAIL_BATCH_SIZE;
        let payload = serde_json::json!({
            "sync_enabled": false,
            "batch_size": batch_size,
            "windows": 3,
            "poll_interval_seconds": 120,
            "failure_threshold": 5
        });

        super::upsert_sync_settings(account_id, &payload);
        let settings = super::sync_settings_for_account(account_id);

        assert_eq!(settings["batch_size"], batch_size);
        assert_eq!(settings["windows"], 3);
        assert_eq!(settings["poll_interval_seconds"], 120);
        assert_eq!(settings["failure_threshold"], 5);
        assert_eq!(settings["sync_enabled"], false);
    }
}
