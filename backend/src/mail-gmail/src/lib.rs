//! Gmail REST adapter owned by the Mail integration.
//!
//! It exposes provider operations only. Communications evidence mapping, durable
//! state and credential leasing stay in their respective owner packages.

use std::{collections::BTreeSet, fmt};

use async_native_tls::TlsConnector;
use async_std::net::TcpStream;
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use futures_util::io::{AsyncReadExt, AsyncWriteExt};
use serde::Deserialize;

pub const GMAIL_API_HOST: &str = "gmail.googleapis.com";
const MAX_RESPONSE_BYTES: usize = 16 * 1024 * 1024;
const MAX_MESSAGE_IDS: usize = 500;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailApiClientV1 {
    host: String,
    user_id: String,
}

pub fn decode_raw_rfc822(raw: &str) -> Result<Vec<u8>, GmailAdapterErrorV1> {
    if raw.is_empty() || raw.len() > MAX_RESPONSE_BYTES * 2 {
        return Err(GmailAdapterErrorV1::InvalidResponse);
    }
    URL_SAFE_NO_PAD
        .decode(raw)
        .map_err(|_| GmailAdapterErrorV1::InvalidResponse)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailAuthorizationCodeExchangeV1 {
    pub token_endpoint: String,
    pub authorization_code: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub redirect_uri: String,
    pub code_verifier: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailRefreshTokenRequestV1 {
    pub token_endpoint: String,
    pub refresh_token: String,
    pub client_id: String,
    pub client_secret: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct GmailOAuthTokenResponseV1 {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
    pub token_type: Option<String>,
    pub scope: Option<String>,
}

pub async fn exchange_authorization_code(
    request: &GmailAuthorizationCodeExchangeV1,
) -> Result<GmailOAuthTokenResponseV1, GmailAdapterErrorV1> {
    let mut form = vec![
        ("grant_type", "authorization_code".to_owned()),
        ("code", request.authorization_code.clone()),
        ("client_id", request.client_id.clone()),
        ("redirect_uri", request.redirect_uri.clone()),
        ("code_verifier", request.code_verifier.clone()),
    ];
    if let Some(client_secret) = &request.client_secret {
        form.push(("client_secret", client_secret.clone()));
    }
    request_oauth_token(&request.token_endpoint, &form).await
}

pub async fn refresh_access_token(
    request: &GmailRefreshTokenRequestV1,
) -> Result<GmailOAuthTokenResponseV1, GmailAdapterErrorV1> {
    let mut form = vec![
        ("grant_type", "refresh_token".to_owned()),
        ("refresh_token", request.refresh_token.clone()),
        ("client_id", request.client_id.clone()),
    ];
    if let Some(client_secret) = &request.client_secret {
        form.push(("client_secret", client_secret.clone()));
    }
    request_oauth_token(&request.token_endpoint, &form).await
}

impl GmailApiClientV1 {
    pub fn new(user_id: impl Into<String>) -> Result<Self, GmailAdapterErrorV1> {
        Self::for_host(GMAIL_API_HOST, user_id)
    }

    pub fn for_host(
        host: impl Into<String>,
        user_id: impl Into<String>,
    ) -> Result<Self, GmailAdapterErrorV1> {
        let host = host.into();
        let user_id = user_id.into();
        if !valid_host(&host) || !valid_provider_id(&user_id) {
            return Err(GmailAdapterErrorV1::InvalidRequest);
        }
        Ok(Self { host, user_id })
    }

    pub async fn list_labels(
        &self,
        access_token: &str,
    ) -> Result<Vec<GmailLabelV1>, GmailAdapterErrorV1> {
        let response: GmailLabelsResponse = self
            .get(access_token, &format!("/gmail/v1/users/{}/labels", self.user_id))
            .await?;
        Ok(response.labels.unwrap_or_default())
    }

    pub async fn list_messages(
        &self,
        access_token: &str,
        request: &GmailListMessagesRequestV1,
    ) -> Result<GmailMessagePageV1, GmailAdapterErrorV1> {
        if request.max_results == 0 || request.max_results > 500 {
            return Err(GmailAdapterErrorV1::InvalidRequest);
        }
        let mut query = vec![format!("maxResults={}", request.max_results)];
        if let Some(page_token) = request.page_token.as_deref() {
            query.push(format!("pageToken={}", percent_encode(page_token)?));
        }
        if let Some(filter) = request.query.as_deref() {
            query.push(format!("q={}", percent_encode(filter)?));
        }
        for label_id in &request.label_ids {
            query.push(format!("labelIds={}", percent_encode(label_id)?));
        }
        let path = format!(
            "/gmail/v1/users/{}/messages?{}",
            self.user_id,
            query.join("&")
        );
        let response: GmailListResponse = self.get(access_token, &path).await?;
        Ok(GmailMessagePageV1 {
            messages: response.messages.unwrap_or_default(),
            next_page_token: response.next_page_token,
        })
    }

    pub async fn fetch_raw_message(
        &self,
        access_token: &str,
        message_id: &str,
    ) -> Result<GmailRawMessageV1, GmailAdapterErrorV1> {
        let message_id = provider_id(message_id)?;
        self.get(
            access_token,
            &format!(
                "/gmail/v1/users/{}/messages/{message_id}?format=raw",
                self.user_id
            ),
        )
        .await
    }

    pub async fn list_history(
        &self,
        access_token: &str,
        start_history_id: &str,
        page_token: Option<&str>,
    ) -> Result<GmailHistoryPageV1, GmailAdapterErrorV1> {
        let mut query = vec![format!("startHistoryId={}", provider_id(start_history_id)?)];
        query.push("historyTypes=messageAdded".to_owned());
        query.push("historyTypes=labelAdded".to_owned());
        query.push("historyTypes=labelRemoved".to_owned());
        if let Some(page_token) = page_token {
            query.push(format!("pageToken={}", percent_encode(page_token)?));
        }
        self.get(
            access_token,
            &format!("/gmail/v1/users/{}/history?{}", self.user_id, query.join("&")),
        )
        .await
    }

    pub async fn send_raw_message(
        &self,
        access_token: &str,
        rfc822: &[u8],
        thread_id: Option<&str>,
    ) -> Result<GmailSentMessageV1, GmailAdapterErrorV1> {
        if rfc822.is_empty() || rfc822.len() > MAX_RESPONSE_BYTES {
            return Err(GmailAdapterErrorV1::InvalidRequest);
        }
        let mut body = serde_json::json!({ "raw": URL_SAFE_NO_PAD.encode(rfc822) });
        if let Some(thread_id) = thread_id {
            body["threadId"] = serde_json::Value::String(provider_id(thread_id)?);
        }
        self.request_json(
            access_token,
            "POST",
            &format!("/gmail/v1/users/{}/messages/send", self.user_id),
            Some(body.to_string().as_bytes()),
        )
        .await
    }

    pub async fn batch_modify(
        &self,
        access_token: &str,
        request: &GmailBatchModifyRequestV1,
    ) -> Result<(), GmailAdapterErrorV1> {
        if request.message_ids.is_empty() || request.message_ids.len() > MAX_MESSAGE_IDS {
            return Err(GmailAdapterErrorV1::InvalidRequest);
        }
        let message_ids = request
            .message_ids
            .iter()
            .map(|id| provider_id(id))
            .collect::<Result<Vec<_>, _>>()?;
        let add_label_ids = request
            .add_label_ids
            .iter()
            .map(|id| provider_id(id))
            .collect::<Result<Vec<_>, _>>()?;
        let remove_label_ids = request
            .remove_label_ids
            .iter()
            .map(|id| provider_id(id))
            .collect::<Result<Vec<_>, _>>()?;
        if add_label_ids.is_empty() && remove_label_ids.is_empty() {
            return Err(GmailAdapterErrorV1::InvalidRequest);
        }
        let body = serde_json::json!({
            "ids": message_ids,
            "addLabelIds": add_label_ids,
            "removeLabelIds": remove_label_ids,
        });
        let _: serde_json::Value = self
            .request_json(
                access_token,
                "POST",
                &format!("/gmail/v1/users/{}/messages/batchModify", self.user_id),
                Some(body.to_string().as_bytes()),
            )
            .await?;
        Ok(())
    }

    async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        access_token: &str,
        path: &str,
    ) -> Result<T, GmailAdapterErrorV1> {
        self.request_json(access_token, "GET", path, None).await
    }

    async fn request_json<T: for<'de> Deserialize<'de>>(
        &self,
        access_token: &str,
        method: &str,
        path: &str,
        body: Option<&[u8]>,
    ) -> Result<T, GmailAdapterErrorV1> {
        if access_token.trim().is_empty() || !path.starts_with('/') || path.contains('\r') || path.contains('\n') {
            return Err(GmailAdapterErrorV1::InvalidRequest);
        }
        let stream = TcpStream::connect((self.host.as_str(), 443))
            .await
            .map_err(|_| GmailAdapterErrorV1::Transport)?;
        let mut stream = TlsConnector::new()
            .connect(self.host.as_str(), stream)
            .await
            .map_err(|_| GmailAdapterErrorV1::Transport)?;
        let body = body.unwrap_or_default();
        let request = format!(
            "{method} {path} HTTP/1.1\r\nHost: {}\r\nAuthorization: Bearer {access_token}\r\nAccept: application/json\r\nAccept-Encoding: identity\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            self.host,
            body.len(),
        );
        stream.write_all(request.as_bytes()).await.map_err(|_| GmailAdapterErrorV1::Transport)?;
        if !body.is_empty() {
            stream.write_all(body).await.map_err(|_| GmailAdapterErrorV1::Transport)?;
        }
        stream.flush().await.map_err(|_| GmailAdapterErrorV1::Transport)?;
        let mut response = Vec::new();
        stream.read_to_end(&mut response).await.map_err(|_| GmailAdapterErrorV1::Transport)?;
        parse_json_response(&response)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailListMessagesRequestV1 {
    pub max_results: u16,
    pub page_token: Option<String>,
    pub query: Option<String>,
    pub label_ids: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GmailListedMessageV1 { pub id: String, pub thread_id: Option<String> }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailMessagePageV1 { pub messages: Vec<GmailListedMessageV1>, pub next_page_token: Option<String> }

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GmailRawMessageV1 {
    pub id: Option<String>, pub thread_id: Option<String>, pub label_ids: Option<Vec<String>>,
    pub history_id: Option<String>, pub internal_date: Option<String>, pub raw: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct GmailLabelV1 { pub id: Option<String>, pub name: Option<String>, #[serde(rename = "type")] pub label_type: Option<String> }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailBatchModifyRequestV1 { pub message_ids: Vec<String>, pub add_label_ids: Vec<String>, pub remove_label_ids: Vec<String> }

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GmailSentMessageV1 { pub id: Option<String>, pub thread_id: Option<String> }

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GmailHistoryPageV1 { pub history: Option<Vec<GmailHistoryItemV1>>, pub history_id: Option<String>, pub next_page_token: Option<String> }

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GmailHistoryItemV1 { pub messages_added: Option<Vec<GmailHistoryMessageAddedV1>>, pub labels_added: Option<Vec<GmailHistoryMessageAddedV1>>, pub labels_removed: Option<Vec<GmailHistoryMessageAddedV1>> }

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct GmailHistoryMessageAddedV1 { pub message: GmailHistoryMessageV1 }

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct GmailHistoryMessageV1 { pub id: String }

pub fn history_message_ids(page: &GmailHistoryPageV1) -> Vec<String> {
    let mut message_ids = BTreeSet::new();
    for item in page.history.as_deref().unwrap_or_default() {
        for changes in [&item.messages_added, &item.labels_added, &item.labels_removed] {
            for change in changes.as_deref().unwrap_or_default() {
                if valid_provider_id(&change.message.id) {
                    message_ids.insert(change.message.id.clone());
                }
            }
        }
    }
    message_ids.into_iter().collect()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GmailLabelsResponse { labels: Option<Vec<GmailLabelV1>> }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GmailListResponse { messages: Option<Vec<GmailListedMessageV1>>, next_page_token: Option<String> }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GmailAdapterErrorV1 { InvalidRequest, Transport, ProviderStatus(u16), InvalidResponse }

impl fmt::Display for GmailAdapterErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result { write!(formatter, "{self:?}") }
}

impl std::error::Error for GmailAdapterErrorV1 {}

fn parse_json_response<T: for<'de> Deserialize<'de>>(response: &[u8]) -> Result<T, GmailAdapterErrorV1> {
    if response.len() > MAX_RESPONSE_BYTES { return Err(GmailAdapterErrorV1::InvalidResponse); }
    let split = response.windows(4).position(|value| value == b"\r\n\r\n").ok_or(GmailAdapterErrorV1::InvalidResponse)?;
    let headers = std::str::from_utf8(&response[..split]).map_err(|_| GmailAdapterErrorV1::InvalidResponse)?;
    let status = headers.split_whitespace().nth(1).and_then(|value| value.parse::<u16>().ok()).ok_or(GmailAdapterErrorV1::InvalidResponse)?;
    if !(200..300).contains(&status) { return Err(GmailAdapterErrorV1::ProviderStatus(status)); }
    serde_json::from_slice(&response[split + 4..]).map_err(|_| GmailAdapterErrorV1::InvalidResponse)
}

async fn request_oauth_token(
    token_endpoint: &str,
    form: &[(&str, String)],
) -> Result<GmailOAuthTokenResponseV1, GmailAdapterErrorV1> {
    let (host, path) = https_endpoint(token_endpoint)?;
    if form.iter().any(|(name, value)| name.is_empty() || value.trim().is_empty() || value.len() > 8192) {
        return Err(GmailAdapterErrorV1::InvalidRequest);
    }
    let body = form.iter().map(|(name, value)| {
        let name = percent_encode(name)?;
        let value = percent_encode(value)?;
        Ok(format!("{name}={value}"))
    }).collect::<Result<Vec<_>, GmailAdapterErrorV1>>()?.join("&");
    let stream = TcpStream::connect((host.as_str(), 443)).await.map_err(|_| GmailAdapterErrorV1::Transport)?;
    let mut stream = TlsConnector::new().connect(host.as_str(), stream).await.map_err(|_| GmailAdapterErrorV1::Transport)?;
    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: {host}\r\nAccept: application/json\r\nAccept-Encoding: identity\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len(),
    );
    stream.write_all(request.as_bytes()).await.map_err(|_| GmailAdapterErrorV1::Transport)?;
    stream.write_all(body.as_bytes()).await.map_err(|_| GmailAdapterErrorV1::Transport)?;
    stream.flush().await.map_err(|_| GmailAdapterErrorV1::Transport)?;
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await.map_err(|_| GmailAdapterErrorV1::Transport)?;
    let token: GmailOAuthTokenResponseV1 = parse_json_response(&response)?;
    if token.access_token.trim().is_empty() || token.expires_in == 0 {
        return Err(GmailAdapterErrorV1::InvalidResponse);
    }
    Ok(token)
}

fn https_endpoint(value: &str) -> Result<(String, String), GmailAdapterErrorV1> {
    let Some(endpoint) = value.strip_prefix("https://") else { return Err(GmailAdapterErrorV1::InvalidRequest); };
    let (host, path) = endpoint.split_once('/').unwrap_or((endpoint, ""));
    if !valid_host(host) || host.len() > 253 || path.contains('\r') || path.contains('\n') || path.len() > 4096 {
        return Err(GmailAdapterErrorV1::InvalidRequest);
    }
    Ok((host.to_owned(), if path.is_empty() { "/".to_owned() } else { format!("/{path}") }))
}

fn valid_host(host: &str) -> bool { !host.is_empty() && host.len() <= 253 && host.bytes().all(|value| value.is_ascii_alphanumeric() || matches!(value, b'.' | b'-')) }
fn valid_provider_id(value: &str) -> bool { !value.is_empty() && value.len() <= 512 && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'@')) }
fn provider_id(value: &str) -> Result<String, GmailAdapterErrorV1> { valid_provider_id(value).then(|| value.to_owned()).ok_or(GmailAdapterErrorV1::InvalidRequest) }
fn percent_encode(value: &str) -> Result<String, GmailAdapterErrorV1> { if value.len() > 4096 || value.contains('\r') || value.contains('\n') { return Err(GmailAdapterErrorV1::InvalidRequest); } Ok(value.bytes().flat_map(|byte| if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') { vec![char::from(byte)] } else { format!("%{byte:02X}").chars().collect() }).collect()) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn parses_a_bounded_success_response() { let value: GmailLabelsResponse = parse_json_response(b"HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\n{\"labels\":[]}").expect("response"); assert!(value.labels.unwrap_or_default().is_empty()); }
    #[test] fn rejects_non_success_status() { let value: Result<serde_json::Value, _> = parse_json_response(b"HTTP/1.1 401 Unauthorized\r\n\r\n{}"); assert_eq!(value, Err(GmailAdapterErrorV1::ProviderStatus(401))); }
    #[test] fn percent_encodes_query_values() { assert_eq!(percent_encode("label:inbox hello").expect("encoded"), "label%3Ainbox%20hello"); }
    #[test] fn accepts_only_safe_https_oauth_endpoints() {
        assert_eq!(https_endpoint("https://oauth2.googleapis.com/token"), Ok(("oauth2.googleapis.com".to_owned(), "/token".to_owned())));
        assert_eq!(https_endpoint("http://oauth2.googleapis.com/token"), Err(GmailAdapterErrorV1::InvalidRequest));
        assert_eq!(https_endpoint("https://oauth2.googleapis.com\r/token"), Err(GmailAdapterErrorV1::InvalidRequest));
    }
    #[test] fn history_collects_unique_valid_message_ids_from_supported_change_families() {
        let page = GmailHistoryPageV1 {
            history: Some(vec![GmailHistoryItemV1 {
                messages_added: Some(vec![GmailHistoryMessageAddedV1 { message: GmailHistoryMessageV1 { id: "message-2".into() } }]),
                labels_added: Some(vec![GmailHistoryMessageAddedV1 { message: GmailHistoryMessageV1 { id: "message-1".into() } }]),
                labels_removed: Some(vec![
                    GmailHistoryMessageAddedV1 { message: GmailHistoryMessageV1 { id: "message-2".into() } },
                    GmailHistoryMessageAddedV1 { message: GmailHistoryMessageV1 { id: "invalid id".into() } },
                ]),
            }]),
            history_id: Some("42".into()),
            next_page_token: None,
        };
        assert_eq!(history_message_ids(&page), vec!["message-1", "message-2"]);
    }
}
