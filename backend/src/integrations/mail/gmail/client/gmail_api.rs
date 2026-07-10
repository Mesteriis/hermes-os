use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde_json::{Map, Value, json};

use crate::integrations::mail::send::{OutgoingEmail, SendResult, build_rfc2822_message};
use crate::integrations::mail::sync::{EmailSyncBatch, FetchedCommunicationSourceMessage};
use crate::platform::communications::{
    AddressBookProviderBatch, AddressBookProviderEntry, AddressBookProviderUpsertRequest,
    EmailProviderKind,
};
use crate::platform::secrets::ResolvedSecret;

use super::errors::EmailProviderNetworkError;
use super::helpers::{
    gmail_history_checkpoint, gmail_message_list_checkpoint, parse_gmail_internal_date,
    select_latest_history_id, sha256_fingerprint, trim_base_url, validate_non_empty,
};
use super::models::{
    GmailHistoryResponse, GmailListResponse, GmailRawMessage, GmailSendResponse,
    GooglePeopleConnectionsResponse, GooglePeoplePerson,
};
use super::options::{GmailContactFetchOptions, GmailFetchOptions, GmailHistoryFetchOptions};

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

            messages.push(FetchedCommunicationSourceMessage {
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

            messages.push(FetchedCommunicationSourceMessage {
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

    pub async fn fetch_entries(
        &self,
        access_token: &ResolvedSecret,
        options: &GmailContactFetchOptions,
    ) -> Result<AddressBookProviderBatch, EmailProviderNetworkError> {
        validate_non_empty("base_url", &self.base_url)?;
        options.validate()?;

        let contacts_url = format!("{}/v1/people/me/connections", self.people_api_base_url());
        let mut query = vec![
            ("pageSize", options.page_size.to_string()),
            (
                "personFields",
                "names,emailAddresses,phoneNumbers,metadata".to_owned(),
            ),
        ];
        if let Some(page_token) = &options.page_token {
            query.push(("pageToken", page_token.clone()));
        }

        let response = self
            .http
            .get(contacts_url)
            .bearer_auth(access_token.expose_for_runtime())
            .query(&query)
            .send()
            .await?
            .error_for_status()?
            .json::<GooglePeopleConnectionsResponse>()
            .await?;

        Ok(AddressBookProviderBatch {
            entries: response
                .connections
                .unwrap_or_default()
                .into_iter()
                .filter_map(google_person_to_address_book_entry)
                .collect(),
            next_page_token: response.next_page_token,
        })
    }

    pub async fn upsert_entry(
        &self,
        access_token: &ResolvedSecret,
        request: &AddressBookProviderUpsertRequest,
    ) -> Result<AddressBookProviderEntry, EmailProviderNetworkError> {
        if request.provider_address_book_entry_id.is_some() {
            self.update_contact(access_token, request).await
        } else {
            self.create_contact(access_token, request).await
        }
    }

    async fn create_contact(
        &self,
        access_token: &ResolvedSecret,
        request: &AddressBookProviderUpsertRequest,
    ) -> Result<AddressBookProviderEntry, EmailProviderNetworkError> {
        validate_non_empty("base_url", &self.base_url)?;
        validate_non_empty("display_name", &request.display_name)?;
        validate_contact_channels(request)?;

        let contacts_url = format!("{}/v1/people:createContact", self.people_api_base_url());
        let person = self
            .http
            .post(contacts_url)
            .bearer_auth(access_token.expose_for_runtime())
            .json(&google_people_contact_payload(request, None))
            .send()
            .await?
            .error_for_status()?
            .json::<GooglePeoplePerson>()
            .await?;

        google_person_to_address_book_entry(person).ok_or(
            EmailProviderNetworkError::MissingProviderField {
                field: "resourceName",
            },
        )
    }

    async fn update_contact(
        &self,
        access_token: &ResolvedSecret,
        request: &AddressBookProviderUpsertRequest,
    ) -> Result<AddressBookProviderEntry, EmailProviderNetworkError> {
        validate_non_empty("base_url", &self.base_url)?;
        validate_non_empty("display_name", &request.display_name)?;
        validate_contact_channels(request)?;
        let provider_address_book_entry_id = request
            .provider_address_book_entry_id
            .as_deref()
            .ok_or(EmailProviderNetworkError::InvalidProviderRequest {
                field: "provider_address_book_entry_id",
                message: "must be present for contact update",
            })?;
        let provider_etag = request.provider_etag.as_deref().ok_or(
            EmailProviderNetworkError::InvalidProviderRequest {
                field: "provider_etag",
                message: "must be present for contact update",
            },
        )?;
        validate_non_empty(
            "provider_address_book_entry_id",
            provider_address_book_entry_id,
        )?;
        validate_non_empty("provider_etag", provider_etag)?;

        let resource_name = provider_address_book_entry_id
            .trim()
            .trim_start_matches('/');
        if !resource_name.starts_with("people/") {
            return Err(EmailProviderNetworkError::InvalidProviderRequest {
                field: "provider_address_book_entry_id",
                message: "must be a People API resource name",
            });
        }

        let contacts_url = format!(
            "{}/v1/{}:updateContact",
            self.people_api_base_url(),
            resource_name
        );
        let person = self
            .http
            .patch(contacts_url)
            .bearer_auth(access_token.expose_for_runtime())
            .query(&[
                ("updatePersonFields", google_people_update_fields(request)),
                ("personFields", "names,emailAddresses,phoneNumbers,metadata"),
            ])
            .json(&google_people_contact_payload(
                request,
                Some((resource_name, provider_etag)),
            ))
            .send()
            .await?
            .error_for_status()?
            .json::<GooglePeoplePerson>()
            .await?;

        google_person_to_address_book_entry(person).ok_or(
            EmailProviderNetworkError::MissingProviderField {
                field: "resourceName",
            },
        )
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

    fn people_api_base_url(&self) -> String {
        if self.base_url == "https://www.googleapis.com" {
            "https://people.googleapis.com".to_owned()
        } else {
            self.base_url.clone()
        }
    }
}

fn validate_contact_channels(
    request: &AddressBookProviderUpsertRequest,
) -> Result<(), EmailProviderNetworkError> {
    let has_email = request
        .email_address
        .as_deref()
        .is_some_and(|email| !email.trim().is_empty());
    let has_phone = request
        .phone_numbers
        .iter()
        .any(|phone| !phone.trim().is_empty());
    if has_email || has_phone {
        return Ok(());
    }

    Err(EmailProviderNetworkError::InvalidProviderRequest {
        field: "contact_channels",
        message: "must include at least one email address or phone number",
    })
}

fn google_people_update_fields(request: &AddressBookProviderUpsertRequest) -> &'static str {
    if request
        .email_address
        .as_deref()
        .is_some_and(|email| !email.trim().is_empty())
        && request
            .phone_numbers
            .iter()
            .any(|phone| !phone.trim().is_empty())
    {
        "names,emailAddresses,phoneNumbers"
    } else if request
        .phone_numbers
        .iter()
        .any(|phone| !phone.trim().is_empty())
    {
        "names,phoneNumbers"
    } else {
        "names,emailAddresses"
    }
}

fn google_people_contact_payload(
    request: &AddressBookProviderUpsertRequest,
    update_metadata: Option<(&str, &str)>,
) -> Value {
    let mut payload = Map::from_iter([(
        "names".to_owned(),
        json!([{ "unstructuredName": request.display_name }]),
    )]);

    if let Some(email_address) = request
        .email_address
        .as_deref()
        .map(str::trim)
        .filter(|email| !email.is_empty())
    {
        payload.insert(
            "emailAddresses".to_owned(),
            json!([{ "value": email_address }]),
        );
    }

    let phone_numbers = request
        .phone_numbers
        .iter()
        .map(String::as_str)
        .map(str::trim)
        .filter(|phone| !phone.is_empty())
        .map(|phone| json!({ "value": phone }))
        .collect::<Vec<_>>();
    if !phone_numbers.is_empty() {
        payload.insert("phoneNumbers".to_owned(), Value::Array(phone_numbers));
    }

    if let Some((resource_name, provider_etag)) = update_metadata {
        payload.insert("resourceName".to_owned(), json!(resource_name));
        payload.insert("etag".to_owned(), json!(provider_etag));
        payload.insert(
            "metadata".to_owned(),
            json!({
                "sources": [
                    {
                        "type": "CONTACT",
                        "etag": provider_etag,
                    }
                ]
            }),
        );
    }

    Value::Object(payload)
}

fn google_person_to_address_book_entry(
    person: GooglePeoplePerson,
) -> Option<AddressBookProviderEntry> {
    let provider_address_book_entry_id = person.resource_name?;
    let display_name = person
        .names
        .unwrap_or_default()
        .into_iter()
        .find_map(|name| non_empty_string(name.display_name));
    let email_addresses = person
        .email_addresses
        .unwrap_or_default()
        .into_iter()
        .filter_map(|email| non_empty_string(email.value))
        .collect();
    let phone_numbers = person
        .phone_numbers
        .unwrap_or_default()
        .into_iter()
        .filter_map(|phone| non_empty_string(phone.value))
        .collect();

    Some(AddressBookProviderEntry {
        provider_address_book_entry_id,
        display_name,
        email_addresses,
        phone_numbers,
        etag: person.etag,
    })
}

fn non_empty_string(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::communications::CommunicationProviderKind;

    #[test]
    fn google_people_payload_supports_phone_only_address_book_entries() {
        let request = AddressBookProviderUpsertRequest {
            account_id: "gmail-account".to_owned(),
            provider_kind: CommunicationProviderKind::Gmail,
            provider_address_book_entry_id: None,
            provider_etag: None,
            display_name: "Phone Only Persona".to_owned(),
            email_address: None,
            phone_numbers: vec![" +1 555 0100 ".to_owned()],
            remote_write_allowed: true,
        };

        validate_contact_channels(&request).expect("phone-only contact channel is valid");
        assert_eq!(google_people_update_fields(&request), "names,phoneNumbers");

        let payload = google_people_contact_payload(&request, None);
        assert_eq!(
            payload.get("names"),
            Some(&json!([{ "unstructuredName": "Phone Only Persona" }]))
        );
        assert_eq!(
            payload.get("phoneNumbers"),
            Some(&json!([{ "value": "+1 555 0100" }]))
        );
        assert!(payload.get("emailAddresses").is_none());
    }

    #[test]
    fn google_people_payload_rejects_address_book_entries_without_contact_channels() {
        let request = AddressBookProviderUpsertRequest {
            account_id: "gmail-account".to_owned(),
            provider_kind: CommunicationProviderKind::Gmail,
            provider_address_book_entry_id: None,
            provider_etag: None,
            display_name: "No Channels".to_owned(),
            email_address: None,
            phone_numbers: Vec::new(),
            remote_write_allowed: true,
        };

        let error = validate_contact_channels(&request)
            .expect_err("contact needs at least one email or phone");
        assert!(matches!(
            error,
            EmailProviderNetworkError::InvalidProviderRequest {
                field: "contact_channels",
                ..
            }
        ));
    }
}
