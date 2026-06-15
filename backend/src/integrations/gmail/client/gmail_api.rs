use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde_json::json;

use crate::domains::mail::core::EmailProviderKind;
use crate::domains::mail::send::{OutgoingEmail, SendResult, build_rfc2822_message};
use crate::domains::mail::sync::{EmailSyncBatch, FetchedEmailMessage};
use crate::platform::secrets::ResolvedSecret;

use super::errors::EmailProviderNetworkError;
use super::helpers::{
    gmail_history_checkpoint, gmail_message_list_checkpoint, parse_gmail_internal_date,
    select_latest_history_id, sha256_fingerprint, trim_base_url, validate_non_empty,
};
use super::models::{GmailHistoryResponse, GmailListResponse, GmailRawMessage, GmailSendResponse};
use super::options::{GmailFetchOptions, GmailHistoryFetchOptions};

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
        let mut query = vec![
            ("maxResults", options.max_results.to_string()),
            ("includeSpamTrash", options.include_spam_trash.to_string()),
        ];
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

        let checkpoint =
            gmail_message_list_checkpoint(latest_history_id, list_response.next_page_token);

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

        let checkpoint = gmail_history_checkpoint(
            &options.start_history_id,
            latest_history_id,
            history_response.next_page_token,
        );

        Ok(EmailSyncBatch {
            provider_kind: EmailProviderKind::Gmail,
            stream_id: "gmail:history".to_owned(),
            checkpoint,
            messages,
        })
    }

    pub async fn send_message(
        &self,
        access_token: &ResolvedSecret,
        email: &OutgoingEmail,
    ) -> Result<SendResult, EmailProviderNetworkError> {
        validate_non_empty("base_url", &self.base_url)?;
        validate_non_empty("user_id", &self.user_id)?;
        if email
            .to
            .iter()
            .chain(email.cc.iter())
            .chain(email.bcc.iter())
            .all(|recipient| recipient.trim().is_empty())
        {
            return Err(EmailProviderNetworkError::InvalidProviderRequest {
                field: "recipients",
                message: "at least one recipient is required",
            });
        }

        let raw = URL_SAFE_NO_PAD.encode(build_rfc2822_message(email).as_bytes());
        let send_url = format!(
            "{}/gmail/v1/users/{}/messages/send",
            self.base_url, self.user_id
        );
        let response = self
            .http
            .post(send_url)
            .bearer_auth(access_token.expose_for_runtime())
            .json(&json!({ "raw": raw }))
            .send()
            .await?
            .error_for_status()?
            .json::<GmailSendResponse>()
            .await?;
        let message_id = response
            .id
            .ok_or(EmailProviderNetworkError::MissingProviderField { field: "id" })?;

        Ok(SendResult {
            message_id,
            accepted_recipients: email
                .to
                .iter()
                .chain(email.cc.iter())
                .chain(email.bcc.iter())
                .cloned()
                .collect(),
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
