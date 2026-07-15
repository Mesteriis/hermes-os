use std::fmt;

use serde::de::DeserializeOwned;
use thiserror::Error;
use url::Url;

use super::models::{
    ZulipBasicResponse, ZulipEventsResponse, ZulipRegisterQueueResponse, ZulipSendMessageResponse,
    ZulipUploadFileResponse,
};

#[derive(Clone, Eq, PartialEq)]
pub struct ZulipClientConfig {
    pub base_url: Url,
    pub email: String,
    pub api_key: String,
}

impl ZulipClientConfig {
    pub fn new(
        base_url: impl AsRef<str>,
        email: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Result<Self, ZulipClientError> {
        Ok(Self {
            base_url: Url::parse(base_url.as_ref())?,
            email: email.into(),
            api_key: api_key.into(),
        })
    }
}

impl fmt::Debug for ZulipClientConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ZulipClientConfig")
            .field("base_url", &self.base_url)
            .field("email", &self.email)
            .field("api_key", &"<redacted>")
            .finish()
    }
}

#[derive(Clone)]
pub struct ZulipApiClient {
    http: reqwest::Client,
    config: ZulipClientConfig,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipDownloadedFile {
    pub bytes: Vec<u8>,
    pub content_type: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ZulipUpdateMessageRequest {
    content: Option<String>,
    topic: Option<String>,
    stream_id: Option<i64>,
    propagate_mode: Option<String>,
}

impl ZulipUpdateMessageRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    pub fn topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
        self
    }

    pub fn stream_id(mut self, stream_id: i64) -> Self {
        self.stream_id = Some(stream_id);
        self
    }

    pub fn propagate_mode(mut self, propagate_mode: impl Into<String>) -> Self {
        self.propagate_mode = Some(propagate_mode.into());
        self
    }

    fn form_fields(&self) -> Result<Vec<(&'static str, String)>, ZulipClientError> {
        let mut fields = Vec::new();
        if let Some(content) = trim_optional("content", self.content.as_deref())? {
            fields.push(("content", content.to_owned()));
        }
        if let Some(topic) = trim_optional("topic", self.topic.as_deref())? {
            fields.push(("topic", topic.to_owned()));
        }
        if let Some(stream_id) = self.stream_id {
            fields.push(("stream_id", stream_id.to_string()));
        }
        if let Some(propagate_mode) =
            trim_optional("propagate_mode", self.propagate_mode.as_deref())?
        {
            fields.push(("propagate_mode", propagate_mode.to_owned()));
        }
        if fields.is_empty() {
            return Err(ZulipClientError::InvalidRequest(
                "message update must include at least one field".to_owned(),
            ));
        }
        Ok(fields)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipReactionRequest {
    emoji_name: String,
    emoji_code: Option<String>,
    reaction_type: Option<String>,
}

impl ZulipReactionRequest {
    pub fn new(emoji_name: impl Into<String>) -> Self {
        Self {
            emoji_name: emoji_name.into(),
            emoji_code: None,
            reaction_type: None,
        }
    }

    pub fn emoji_code(mut self, emoji_code: impl Into<String>) -> Self {
        self.emoji_code = Some(emoji_code.into());
        self
    }

    pub fn reaction_type(mut self, reaction_type: impl Into<String>) -> Self {
        self.reaction_type = Some(reaction_type.into());
        self
    }

    fn form_fields(&self) -> Result<Vec<(&'static str, String)>, ZulipClientError> {
        let emoji_name = trim_required("emoji_name", &self.emoji_name)?;
        let mut fields = vec![("emoji_name", emoji_name.to_owned())];
        if let Some(emoji_code) = trim_optional("emoji_code", self.emoji_code.as_deref())? {
            fields.push(("emoji_code", emoji_code.to_owned()));
        }
        if let Some(reaction_type) = trim_optional("reaction_type", self.reaction_type.as_deref())?
        {
            fields.push(("reaction_type", reaction_type.to_owned()));
        }
        Ok(fields)
    }
}

impl ZulipApiClient {
    pub fn new(config: ZulipClientConfig) -> Self {
        Self {
            http: reqwest::Client::new(),
            config,
        }
    }

    pub async fn register_event_queue(
        &self,
        event_types: &[&str],
    ) -> Result<ZulipRegisterQueueResponse, ZulipClientError> {
        let event_types = serde_json::to_string(event_types).unwrap_or_else(|_| "[]".to_owned());
        let response = self
            .authenticated(
                self.http
                    .post(self.endpoint("api/v1/register")?)
                    .form(&[("event_types", event_types)]),
            )
            .send()
            .await?;

        decode_response(response).await
    }

    pub async fn get_events(
        &self,
        queue_id: &str,
        last_event_id: i64,
        dont_block: bool,
    ) -> Result<ZulipEventsResponse, ZulipClientError> {
        let response = self
            .authenticated(self.http.get(self.endpoint("api/v1/events")?).query(&[
                ("queue_id", queue_id.to_owned()),
                ("last_event_id", last_event_id.to_string()),
                ("dont_block", dont_block.to_string()),
            ]))
            .send()
            .await?;

        decode_response(response).await
    }

    pub async fn send_stream_message(
        &self,
        stream: &str,
        topic: &str,
        content: &str,
    ) -> Result<ZulipSendMessageResponse, ZulipClientError> {
        let response = self
            .authenticated(self.http.post(self.endpoint("api/v1/messages")?).form(&[
                ("type", "stream".to_owned()),
                ("to", stream.to_owned()),
                ("topic", topic.to_owned()),
                ("content", content.to_owned()),
            ]))
            .send()
            .await?;

        decode_response(response).await
    }

    pub async fn send_direct_message(
        &self,
        recipients: &[&str],
        content: &str,
    ) -> Result<ZulipSendMessageResponse, ZulipClientError> {
        if recipients.is_empty() {
            return Err(ZulipClientError::InvalidRequest(
                "direct message must include at least one recipient".to_owned(),
            ));
        }
        let recipients = recipients
            .iter()
            .map(|recipient| trim_required("recipient", recipient).map(ToOwned::to_owned))
            .collect::<Result<Vec<_>, _>>()?;
        self.send_direct_message_to_json(&serde_json::to_string(&recipients)?, content)
            .await
    }

    pub async fn send_direct_message_to_user_ids(
        &self,
        recipient_user_ids: &[i64],
        content: &str,
    ) -> Result<ZulipSendMessageResponse, ZulipClientError> {
        if recipient_user_ids.is_empty() {
            return Err(ZulipClientError::InvalidRequest(
                "direct message must include at least one recipient".to_owned(),
            ));
        }
        self.send_direct_message_to_json(&serde_json::to_string(recipient_user_ids)?, content)
            .await
    }

    async fn send_direct_message_to_json(
        &self,
        to: &str,
        content: &str,
    ) -> Result<ZulipSendMessageResponse, ZulipClientError> {
        let response = self
            .authenticated(self.http.post(self.endpoint("api/v1/messages")?).form(&[
                ("type", "direct".to_owned()),
                ("to", to.to_owned()),
                ("content", trim_required("content", content)?.to_owned()),
            ]))
            .send()
            .await?;

        decode_response(response).await
    }

    pub async fn update_message(
        &self,
        message_id: i64,
        request: &ZulipUpdateMessageRequest,
    ) -> Result<ZulipBasicResponse, ZulipClientError> {
        let response = self
            .authenticated(
                self.http
                    .patch(self.endpoint(&format!("api/v1/messages/{message_id}"))?)
                    .form(&request.form_fields()?),
            )
            .send()
            .await?;

        decode_response(response).await
    }

    pub async fn delete_message(
        &self,
        message_id: i64,
    ) -> Result<ZulipBasicResponse, ZulipClientError> {
        let response = self
            .authenticated(
                self.http
                    .delete(self.endpoint(&format!("api/v1/messages/{message_id}"))?),
            )
            .send()
            .await?;

        decode_response(response).await
    }

    pub async fn add_reaction(
        &self,
        message_id: i64,
        reaction: &ZulipReactionRequest,
    ) -> Result<ZulipBasicResponse, ZulipClientError> {
        self.submit_reaction("POST", message_id, reaction).await
    }

    pub async fn remove_reaction(
        &self,
        message_id: i64,
        reaction: &ZulipReactionRequest,
    ) -> Result<ZulipBasicResponse, ZulipClientError> {
        self.submit_reaction("DELETE", message_id, reaction).await
    }

    pub async fn upload_file_bytes(
        &self,
        filename: &str,
        bytes: Vec<u8>,
    ) -> Result<ZulipUploadFileResponse, ZulipClientError> {
        let filename = trim_required("filename", filename)?;
        let part = reqwest::multipart::Part::bytes(bytes).file_name(filename.to_owned());
        let form = reqwest::multipart::Form::new().part("file", part);
        let response = self
            .authenticated(
                self.http
                    .post(self.endpoint("api/v1/user_uploads")?)
                    .multipart(form),
            )
            .send()
            .await?;

        decode_response(response).await
    }

    pub async fn download_user_upload(
        &self,
        upload_url: &str,
    ) -> Result<ZulipDownloadedFile, ZulipClientError> {
        let upload_url = self.user_upload_url(upload_url)?;
        let response = self.authenticated(self.http.get(upload_url)).send().await?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ZulipClientError::Api {
                status: status.as_u16(),
                body,
            });
        }
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        let bytes = response.bytes().await?.to_vec();

        Ok(ZulipDownloadedFile {
            bytes,
            content_type,
        })
    }

    async fn submit_reaction(
        &self,
        method: &str,
        message_id: i64,
        reaction: &ZulipReactionRequest,
    ) -> Result<ZulipBasicResponse, ZulipClientError> {
        let request = match method {
            "POST" => self
                .http
                .post(self.endpoint(&format!("api/v1/messages/{message_id}/reactions"))?),
            "DELETE" => self
                .http
                .delete(self.endpoint(&format!("api/v1/messages/{message_id}/reactions"))?),
            other => {
                return Err(ZulipClientError::InvalidRequest(format!(
                    "unsupported reaction method {other}"
                )));
            }
        };
        let response = self
            .authenticated(request.form(&reaction.form_fields()?))
            .send()
            .await?;

        decode_response(response).await
    }

    fn endpoint(&self, relative_path: &str) -> Result<Url, ZulipClientError> {
        Ok(self.config.base_url.join(relative_path)?)
    }

    fn user_upload_url(&self, upload_url: &str) -> Result<Url, ZulipClientError> {
        let upload_url = trim_required("upload_url", upload_url)?;
        let parsed = match Url::parse(upload_url) {
            Ok(parsed) => parsed,
            Err(url::ParseError::RelativeUrlWithoutBase) => {
                self.config.base_url.join(upload_url)?
            }
            Err(error) => return Err(error.into()),
        };
        if !same_origin(&self.config.base_url, &parsed) || !is_user_upload_path(parsed.path()) {
            return Err(ZulipClientError::InvalidRequest(
                "Zulip upload URL must be same-realm /user_uploads path".to_owned(),
            ));
        }
        Ok(parsed)
    }

    fn authenticated(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        request.basic_auth(&self.config.email, Some(&self.config.api_key))
    }
}

async fn decode_response<T>(response: reqwest::Response) -> Result<T, ZulipClientError>
where
    T: DeserializeOwned,
{
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(ZulipClientError::Api {
            status: status.as_u16(),
            body,
        });
    }

    Ok(response.json::<T>().await?)
}

fn trim_required<'a>(field: &'static str, value: &'a str) -> Result<&'a str, ZulipClientError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ZulipClientError::InvalidRequest(format!(
            "{field} must not be empty"
        )));
    }
    Ok(value)
}

fn trim_optional<'a>(
    field: &'static str,
    value: Option<&'a str>,
) -> Result<Option<&'a str>, ZulipClientError> {
    value.map(|value| trim_required(field, value)).transpose()
}

fn same_origin(left: &Url, right: &Url) -> bool {
    left.scheme() == right.scheme()
        && left.host_str() == right.host_str()
        && left.port_or_known_default() == right.port_or_known_default()
}

fn is_user_upload_path(path: &str) -> bool {
    path == "/user_uploads" || path.starts_with("/user_uploads/")
}

#[derive(Debug, Error)]
pub enum ZulipClientError {
    #[error("invalid Zulip base URL: {0}")]
    Url(#[from] url::ParseError),
    #[error("invalid Zulip request: {0}")]
    InvalidRequest(String),
    #[error("invalid Zulip JSON payload: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Zulip HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Zulip API returned HTTP {status}: {body}")]
    Api { status: u16, body: String },
}
