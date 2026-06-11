use std::fmt::Debug;
use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chrono::{DateTime, TimeZone, Utc};
use futures::TryStreamExt;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::domains::mail::core::EmailProviderKind;
use crate::domains::mail::sync::{EmailSyncBatch, FetchedEmailMessage, imap_mailbox_stream_id};
use crate::platform::secrets::ResolvedSecret;

#[derive(Clone)]
pub struct GmailApiClient {
    http: reqwest::Client,
    base_url: String,
    user_id: String,
}

impl GmailApiClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("reqwest client configuration must be valid");

        Self {
            http,
            base_url: trim_base_url(base_url.into()),
            user_id: "me".to_owned(),
        }
    }

    pub fn user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = user_id.into();
        self
    }

    pub async fn fetch_raw_messages(
        &self,
        access_token: &ResolvedSecret,
        options: &GmailFetchOptions,
    ) -> Result<EmailSyncBatch, EmailProviderNetworkError> {
        validate_non_empty("base_url", &self.base_url)?;
        validate_non_empty("user_id", &self.user_id)?;
        options.validate()?;

        let list_url = format!("{}/gmail/v1/users/{}/messages", self.base_url, self.user_id);
        let mut query = vec![("maxResults", options.max_results.to_string())];
        if let Some(page_token) = &options.page_token {
            query.push(("pageToken", page_token.clone()));
        }
        if let Some(search_query) = &options.query {
            query.push(("q", search_query.clone()));
        }
        for label_id in &options.label_ids {
            query.push(("labelIds", label_id.clone()));
        }

        let list_response = self
            .http
            .get(list_url)
            .bearer_auth(access_token.expose_for_runtime())
            .query(&query)
            .send()
            .await?
            .error_for_status()?
            .json::<GmailListResponse>()
            .await?;

        let mut messages = Vec::new();
        let mut latest_history_id = None;
        for listed_message in list_response.messages.unwrap_or_default() {
            validate_non_empty("gmail_message_id", &listed_message.id)?;
            let message_url = format!(
                "{}/gmail/v1/users/{}/messages/{}",
                self.base_url, self.user_id, listed_message.id
            );
            let raw_message = self
                .http
                .get(message_url)
                .bearer_auth(access_token.expose_for_runtime())
                .query(&[("format", "raw")])
                .send()
                .await?
                .error_for_status()?
                .json::<GmailRawMessage>()
                .await?;

            let provider_record_id = raw_message.id.unwrap_or(listed_message.id);
            let raw = raw_message
                .raw
                .ok_or(EmailProviderNetworkError::MissingProviderField { field: "raw" })?;
            let occurred_at = parse_gmail_internal_date(raw_message.internal_date.as_deref())?;
            latest_history_id =
                select_latest_history_id(latest_history_id, raw_message.history_id.as_deref());

            messages.push(FetchedEmailMessage {
                source_fingerprint: sha256_fingerprint([
                    "gmail".as_bytes(),
                    provider_record_id.as_bytes(),
                    raw.as_bytes(),
                ]),
                provider_record_id: provider_record_id.clone(),
                occurred_at,
                payload: json!({
                    "provider": "gmail",
                    "id": provider_record_id,
                    "thread_id": raw_message.thread_id.or(listed_message.thread_id),
                    "label_ids": raw_message.label_ids,
                    "history_id": raw_message.history_id,
                    "internal_date": raw_message.internal_date,
                    "raw_base64url": raw
                }),
            });
        }

        let checkpoint = gmail_checkpoint(latest_history_id, list_response.next_page_token);

        Ok(EmailSyncBatch {
            provider_kind: EmailProviderKind::Gmail,
            stream_id: "gmail:history".to_owned(),
            checkpoint,
            messages,
        })
    }

    pub async fn fetch_history_raw_messages(
        &self,
        access_token: &ResolvedSecret,
        options: &GmailHistoryFetchOptions,
    ) -> Result<EmailSyncBatch, EmailProviderNetworkError> {
        validate_non_empty("base_url", &self.base_url)?;
        validate_non_empty("user_id", &self.user_id)?;
        options.validate()?;

        let history_url = format!("{}/gmail/v1/users/{}/history", self.base_url, self.user_id);
        let mut query = vec![
            ("startHistoryId", options.start_history_id.clone()),
            ("maxResults", options.max_results.to_string()),
            ("historyTypes", "messageAdded".to_owned()),
        ];
        if let Some(page_token) = &options.page_token {
            query.push(("pageToken", page_token.clone()));
        }

        let history_response = self
            .http
            .get(history_url)
            .bearer_auth(access_token.expose_for_runtime())
            .query(&query)
            .send()
            .await?
            .error_for_status()?
            .json::<GmailHistoryResponse>()
            .await?;

        let mut message_ids = Vec::new();
        for history in history_response.history.unwrap_or_default() {
            for added in history.messages_added.unwrap_or_default() {
                if !message_ids.contains(&added.message.id) {
                    message_ids.push(added.message.id);
                }
            }
        }

        let mut messages = Vec::new();
        let mut latest_history_id = history_response.history_id.clone();
        for message_id in message_ids.into_iter().take(options.max_results as usize) {
            let raw_message = self.fetch_raw_message(access_token, &message_id).await?;
            let provider_record_id = raw_message.id.unwrap_or(message_id);
            let raw = raw_message
                .raw
                .ok_or(EmailProviderNetworkError::MissingProviderField { field: "raw" })?;
            let occurred_at = parse_gmail_internal_date(raw_message.internal_date.as_deref())?;
            latest_history_id =
                select_latest_history_id(latest_history_id, raw_message.history_id.as_deref());

            messages.push(FetchedEmailMessage {
                source_fingerprint: sha256_fingerprint([
                    "gmail".as_bytes(),
                    provider_record_id.as_bytes(),
                    raw.as_bytes(),
                ]),
                provider_record_id: provider_record_id.clone(),
                occurred_at,
                payload: json!({
                    "provider": "gmail",
                    "id": provider_record_id,
                    "thread_id": raw_message.thread_id,
                    "label_ids": raw_message.label_ids,
                    "history_id": raw_message.history_id,
                    "internal_date": raw_message.internal_date,
                    "raw_base64url": raw
                }),
            });
        }

        let checkpoint = gmail_checkpoint(latest_history_id, history_response.next_page_token);

        Ok(EmailSyncBatch {
            provider_kind: EmailProviderKind::Gmail,
            stream_id: "gmail:history".to_owned(),
            checkpoint,
            messages,
        })
    }

    async fn fetch_raw_message(
        &self,
        access_token: &ResolvedSecret,
        message_id: &str,
    ) -> Result<GmailRawMessage, EmailProviderNetworkError> {
        validate_non_empty("gmail_message_id", message_id)?;
        let message_url = format!(
            "{}/gmail/v1/users/{}/messages/{}",
            self.base_url, self.user_id, message_id
        );

        Ok(self
            .http
            .get(message_url)
            .bearer_auth(access_token.expose_for_runtime())
            .query(&[("format", "raw")])
            .send()
            .await?
            .error_for_status()?
            .json::<GmailRawMessage>()
            .await?)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailFetchOptions {
    max_results: u16,
    query: Option<String>,
    page_token: Option<String>,
    label_ids: Vec<String>,
}

impl GmailFetchOptions {
    pub fn new(max_results: u16) -> Self {
        Self {
            max_results,
            query: None,
            page_token: None,
            label_ids: Vec::new(),
        }
    }

    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    pub fn page_token(mut self, page_token: impl Into<String>) -> Self {
        self.page_token = Some(page_token.into());
        self
    }

    pub fn label_id(mut self, label_id: impl Into<String>) -> Self {
        self.label_ids.push(label_id.into());
        self
    }

    fn validate(&self) -> Result<(), EmailProviderNetworkError> {
        if self.max_results == 0 || self.max_results > 500 {
            return Err(EmailProviderNetworkError::InvalidProviderRequest {
                field: "max_results",
                message: "must be between 1 and 500",
            });
        }
        if let Some(query) = &self.query {
            validate_non_empty("query", query)?;
        }
        if let Some(page_token) = &self.page_token {
            validate_non_empty("page_token", page_token)?;
        }
        for label_id in &self.label_ids {
            validate_non_empty("label_id", label_id)?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailHistoryFetchOptions {
    start_history_id: String,
    max_results: u16,
    page_token: Option<String>,
}

impl GmailHistoryFetchOptions {
    pub fn new(start_history_id: impl Into<String>, max_results: u16) -> Self {
        Self {
            start_history_id: start_history_id.into(),
            max_results,
            page_token: None,
        }
    }

    pub fn page_token(mut self, page_token: impl Into<String>) -> Self {
        self.page_token = Some(page_token.into());
        self
    }

    fn validate(&self) -> Result<(), EmailProviderNetworkError> {
        validate_non_empty("start_history_id", &self.start_history_id)?;
        if self.max_results == 0 || self.max_results > 500 {
            return Err(EmailProviderNetworkError::InvalidProviderRequest {
                field: "max_results",
                message: "must be between 1 and 500",
            });
        }
        if let Some(page_token) = &self.page_token {
            validate_non_empty("page_token", page_token)?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImapFetchOptions {
    pub provider_kind: EmailProviderKind,
    pub host: String,
    pub port: u16,
    pub tls: bool,
    pub mailbox: String,
    pub username: String,
    pub last_seen_uid: Option<u32>,
    pub max_messages: usize,
    pub latest_messages: bool,
}

impl ImapFetchOptions {
    pub fn new(
        host: impl Into<String>,
        port: u16,
        tls: bool,
        mailbox: impl Into<String>,
        username: impl Into<String>,
    ) -> Self {
        Self {
            provider_kind: EmailProviderKind::Imap,
            host: host.into(),
            port,
            tls,
            mailbox: mailbox.into(),
            username: username.into(),
            last_seen_uid: None,
            max_messages: 100,
            latest_messages: false,
        }
    }

    pub fn provider_kind(mut self, provider_kind: EmailProviderKind) -> Self {
        self.provider_kind = provider_kind;
        self
    }

    pub fn last_seen_uid(mut self, last_seen_uid: u32) -> Self {
        self.last_seen_uid = Some(last_seen_uid);
        self
    }

    pub fn max_messages(mut self, max_messages: usize) -> Self {
        self.max_messages = max_messages;
        self
    }

    pub fn latest_messages(mut self) -> Self {
        self.latest_messages = true;
        self
    }

    fn validate(&self) -> Result<(), EmailProviderNetworkError> {
        validate_non_empty("host", &self.host)?;
        validate_non_empty("mailbox", &self.mailbox)?;
        validate_non_empty("username", &self.username)?;
        if self.port == 0 {
            return Err(EmailProviderNetworkError::InvalidProviderRequest {
                field: "port",
                message: "must be greater than zero",
            });
        }
        if self.max_messages == 0 {
            return Err(EmailProviderNetworkError::InvalidProviderRequest {
                field: "max_messages",
                message: "must be greater than zero",
            });
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct ImapNetworkClient;

impl ImapNetworkClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn fetch_raw_messages(
        &self,
        password: &ResolvedSecret,
        options: &ImapFetchOptions,
    ) -> Result<EmailSyncBatch, EmailProviderNetworkError> {
        options.validate()?;

        let address = (options.host.as_str(), options.port);
        let tcp_stream = tokio::net::TcpStream::connect(address).await?;
        if options.tls {
            let tls_stream = async_native_tls::connect(options.host.as_str(), tcp_stream).await?;
            fetch_imap_with_client(async_imap::Client::new(tls_stream), password, options).await
        } else {
            fetch_imap_with_client(async_imap::Client::new(tcp_stream), password, options).await
        }
    }
}

async fn fetch_imap_with_client<T>(
    mut client: async_imap::Client<T>,
    password: &ResolvedSecret,
    options: &ImapFetchOptions,
) -> Result<EmailSyncBatch, EmailProviderNetworkError>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send,
{
    client
        .read_response()
        .await?
        .ok_or(EmailProviderNetworkError::UnexpectedProviderResponse {
            message: "missing IMAP greeting",
        })?;

    let mut session = client
        .login(&options.username, password.expose_for_runtime())
        .await
        .map_err(|(error, _client)| EmailProviderNetworkError::Imap(error))?;
    let mailbox = session.examine(&options.mailbox).await?;
    let first_uid = options
        .last_seen_uid
        .and_then(|uid| uid.checked_add(1))
        .unwrap_or(1);
    let search_query = format!("{first_uid}:*");
    let uids: Vec<u32> = session
        .uid_search(search_query)
        .await?
        .into_iter()
        .collect();
    let uids = select_uids_for_fetch(uids, options.max_messages, options.latest_messages);

    let mut messages = Vec::new();
    if !uids.is_empty() {
        let uid_set = uid_set(&uids);
        let fetched_messages = session
            .uid_fetch(uid_set, "(UID BODY.PEEK[] RFC822.SIZE INTERNALDATE)")
            .await?
            .try_collect::<Vec<_>>()
            .await?;

        for fetched_message in fetched_messages {
            let uid = fetched_message
                .uid
                .ok_or(EmailProviderNetworkError::MissingProviderField { field: "uid" })?;
            let body = fetched_message
                .body()
                .ok_or(EmailProviderNetworkError::MissingProviderField { field: "rfc822" })?;
            let uid_string = uid.to_string();
            let occurred_at = fetched_message
                .internal_date()
                .map(|internal_date| internal_date.with_timezone(&Utc));

            messages.push(FetchedEmailMessage {
                provider_record_id: uid_string.clone(),
                source_fingerprint: sha256_fingerprint([
                    "imap".as_bytes(),
                    uid_string.as_bytes(),
                    body,
                ]),
                occurred_at,
                payload: json!({
                    "provider": options.provider_kind.as_str(),
                    "transport": "imap",
                    "mailbox": options.mailbox,
                    "uid": uid,
                    "uid_validity": mailbox.uid_validity,
                    "raw_rfc822_base64": BASE64_STANDARD.encode(body),
                    "rfc822_size": fetched_message.size
                }),
            });
        }
    }

    let latest_uid = messages
        .iter()
        .filter_map(|message| message.provider_record_id.parse::<u32>().ok())
        .max()
        .or(options.last_seen_uid);
    session.logout().await?;

    Ok(EmailSyncBatch {
        provider_kind: options.provider_kind,
        stream_id: imap_mailbox_stream_id(&options.mailbox),
        checkpoint: Some(imap_checkpoint(
            &options.mailbox,
            mailbox.uid_validity,
            latest_uid,
        )),
        messages,
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GmailListResponse {
    messages: Option<Vec<GmailListedMessage>>,
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GmailListedMessage {
    id: String,
    thread_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GmailRawMessage {
    id: Option<String>,
    thread_id: Option<String>,
    label_ids: Option<Vec<String>>,
    history_id: Option<String>,
    internal_date: Option<String>,
    raw: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GmailHistoryResponse {
    history: Option<Vec<GmailHistoryItem>>,
    history_id: Option<String>,
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GmailHistoryItem {
    messages_added: Option<Vec<GmailHistoryMessageAdded>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GmailHistoryMessageAdded {
    message: GmailHistoryMessage,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GmailHistoryMessage {
    id: String,
}

fn trim_base_url(base_url: String) -> String {
    base_url.trim().trim_end_matches('/').to_owned()
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), EmailProviderNetworkError> {
    if value.trim().is_empty() {
        return Err(EmailProviderNetworkError::InvalidProviderRequest {
            field,
            message: "must not be empty",
        });
    }

    Ok(())
}

fn parse_gmail_internal_date(
    internal_date: Option<&str>,
) -> Result<Option<DateTime<Utc>>, EmailProviderNetworkError> {
    let Some(internal_date) = internal_date else {
        return Ok(None);
    };
    let millis = internal_date.parse::<i64>().map_err(|_| {
        EmailProviderNetworkError::InvalidProviderResponse {
            field: "internal_date",
            message: "expected epoch milliseconds",
        }
    })?;

    Utc.timestamp_millis_opt(millis)
        .single()
        .ok_or(EmailProviderNetworkError::InvalidProviderResponse {
            field: "internal_date",
            message: "timestamp is out of range",
        })
        .map(Some)
}

fn select_latest_history_id(current: Option<String>, candidate: Option<&str>) -> Option<String> {
    let Some(candidate) = candidate else {
        return current;
    };
    let Some(current) = current else {
        return Some(candidate.to_owned());
    };

    let current_number = current.parse::<u64>();
    let candidate_number = candidate.parse::<u64>();
    match (current_number, candidate_number) {
        (Ok(current_number), Ok(candidate_number)) if current_number >= candidate_number => {
            Some(current)
        }
        _ => Some(candidate.to_owned()),
    }
}

fn gmail_checkpoint(history_id: Option<String>, next_page_token: Option<String>) -> Option<Value> {
    let history_id = history_id?;
    let mut checkpoint = json!({
        "provider": "gmail",
        "history_id": history_id
    });

    if let Some(next_page_token) = next_page_token {
        checkpoint["next_page_token"] = json!(next_page_token);
    }

    Some(checkpoint)
}

fn imap_checkpoint(mailbox: &str, uid_validity: Option<u32>, latest_uid: Option<u32>) -> Value {
    let mut checkpoint = json!({
        "provider": "imap",
        "mailbox": mailbox
    });

    if let Some(uid_validity) = uid_validity {
        checkpoint["uid_validity"] = json!(uid_validity);
    }
    if let Some(latest_uid) = latest_uid {
        checkpoint["last_seen_uid"] = json!(latest_uid);
    }

    checkpoint
}

fn uid_set(uids: &[u32]) -> String {
    uids.iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn select_uids_for_fetch(
    mut uids: Vec<u32>,
    max_messages: usize,
    latest_messages: bool,
) -> Vec<u32> {
    uids.sort_unstable();
    if latest_messages && uids.len() > max_messages {
        uids[uids.len() - max_messages..].to_vec()
    } else {
        uids.truncate(max_messages);
        uids
    }
}

fn sha256_fingerprint<'a>(parts: impl IntoIterator<Item = &'a [u8]>) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part);
    }

    format!("sha256:{}", hex_lower(&hasher.finalize()))
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }

    output
}

#[derive(Debug, Error)]
pub enum EmailProviderNetworkError {
    #[error("invalid provider request field {field}: {message}")]
    InvalidProviderRequest {
        field: &'static str,
        message: &'static str,
    },

    #[error("invalid provider response field {field}: {message}")]
    InvalidProviderResponse {
        field: &'static str,
        message: &'static str,
    },

    #[error("provider response is missing required field: {field}")]
    MissingProviderField { field: &'static str },

    #[error("unexpected provider response: {message}")]
    UnexpectedProviderResponse { message: &'static str },

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Tls(#[from] async_native_tls::Error),

    #[error(transparent)]
    Imap(#[from] async_imap::error::Error),
}

#[cfg(test)]
mod tests {
    use super::select_uids_for_fetch;

    #[test]
    fn select_uids_for_fetch_keeps_latest_window_when_requested() {
        assert_eq!(
            select_uids_for_fetch(vec![43, 41, 42], 2, true),
            vec![42, 43]
        );
    }

    #[test]
    fn select_uids_for_fetch_keeps_oldest_window_for_sync_default() {
        assert_eq!(
            select_uids_for_fetch(vec![43, 41, 42], 2, false),
            vec![41, 42]
        );
    }
}
