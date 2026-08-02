//! Bounded Google People provider adapter owned by the Mail integration.
//!
//! The adapter owns only the provider dialect. It has no persistence, Vault,
//! Event Hub, workflow, Contacts-domain or runtime authority.

use std::{fmt, time::Duration};

use async_native_tls::{Certificate, TlsConnector};
use async_std::net::TcpStream;
use futures_util::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Deserialize, Serialize};

pub const GOOGLE_PEOPLE_API_HOST_V1: &str = "people.googleapis.com";
pub const GOOGLE_PEOPLE_API_PORT_V1: u16 = 443;
pub const GOOGLE_PEOPLE_CONTACTS_SCOPE_V1: &str = "https://www.googleapis.com/auth/contacts";
pub const GOOGLE_PEOPLE_MAX_PAGE_SIZE_V1: u16 = 1_000;
pub const GOOGLE_PEOPLE_MAX_CONTACT_VALUES_V1: usize = 64;

const MAX_RESPONSE_BYTES: usize = 4 * 1024 * 1024;
const MAX_TOKEN_BYTES: usize = 16 * 1024;
const MAX_TEXT_BYTES: usize = 2_048;
const OPERATION_TIMEOUT: Duration = Duration::from_secs(15);
const PERSON_FIELDS: &str = "metadata,names,emailAddresses,phoneNumbers";
const UPDATE_FIELDS: &str = "names,emailAddresses,phoneNumbers";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GooglePeopleClientV1 {
    host: String,
    port: u16,
    ca_certificate_pem: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GooglePeopleContactV1 {
    pub resource_name: String,
    pub etag: String,
    pub display_name: String,
    pub email_addresses: Vec<String>,
    pub phone_numbers: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GooglePeoplePageV1 {
    pub contacts: Vec<GooglePeopleContactV1>,
    pub next_page_token: Option<String>,
    pub next_sync_token: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GooglePeopleUpsertV1 {
    pub resource_name: Option<String>,
    pub expected_etag: Option<String>,
    pub display_name: String,
    pub email_addresses: Vec<String>,
    pub phone_numbers: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GooglePeopleUpsertedV1 {
    pub resource_name: String,
    pub etag: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GooglePeopleAdapterErrorV1 {
    InvalidRequest,
    Unavailable,
    OutcomeUnknown,
    EtagConflict,
    ProviderRejected(u16),
    InvalidResponse,
}

impl fmt::Display for GooglePeopleAdapterErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for GooglePeopleAdapterErrorV1 {}

impl GooglePeopleClientV1 {
    pub fn new() -> Result<Self, GooglePeopleAdapterErrorV1> {
        Self::for_endpoint(GOOGLE_PEOPLE_API_HOST_V1, GOOGLE_PEOPLE_API_PORT_V1, None)
    }

    #[cfg(any(test, feature = "conformance-test-support"))]
    pub fn for_conformance_endpoint(
        host: impl Into<String>,
        port: u16,
        ca_certificate_pem: Option<String>,
    ) -> Result<Self, GooglePeopleAdapterErrorV1> {
        let host = host.into();
        if !matches!(host.as_str(), "127.0.0.1" | "localhost") {
            return Err(GooglePeopleAdapterErrorV1::InvalidRequest);
        }
        Self::for_endpoint(host, port, ca_certificate_pem)
    }

    fn for_endpoint(
        host: impl Into<String>,
        port: u16,
        ca_certificate_pem: Option<String>,
    ) -> Result<Self, GooglePeopleAdapterErrorV1> {
        let host = host.into();
        if !valid_host(&host)
            || port == 0
            || ca_certificate_pem
                .as_deref()
                .is_some_and(|value| value.is_empty() || value.len() > 64 * 1024)
        {
            return Err(GooglePeopleAdapterErrorV1::InvalidRequest);
        }
        Ok(Self {
            host,
            port,
            ca_certificate_pem,
        })
    }

    pub async fn list_connections(
        &self,
        access_token: &str,
        page_token: Option<&str>,
        sync_token: Option<&str>,
        page_size: u16,
    ) -> Result<GooglePeoplePageV1, GooglePeopleAdapterErrorV1> {
        if !(1..=GOOGLE_PEOPLE_MAX_PAGE_SIZE_V1).contains(&page_size)
            || page_token.is_some() && sync_token.is_some()
            || page_token.is_some_and(|value| !valid_cursor(value))
            || sync_token.is_some_and(|value| !valid_cursor(value))
        {
            return Err(GooglePeopleAdapterErrorV1::InvalidRequest);
        }
        let mut path = format!(
            "/v1/people/me/connections?personFields={PERSON_FIELDS}&pageSize={page_size}&requestSyncToken=true&sources=READ_SOURCE_TYPE_CONTACT"
        );
        if let Some(token) = page_token {
            path.push_str("&pageToken=");
            path.push_str(&percent_encode(token)?);
        }
        if let Some(token) = sync_token {
            path.push_str("&syncToken=");
            path.push_str(&percent_encode(token)?);
        }
        let response: ConnectionsResponse =
            self.request_json(access_token, "GET", &path, None).await?;
        let contacts = response
            .connections
            .unwrap_or_default()
            .into_iter()
            .filter(|person| {
                !person
                    .metadata
                    .as_ref()
                    .is_some_and(|metadata| metadata.deleted)
            })
            .map(contact_from_person)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(GooglePeoplePageV1 {
            contacts,
            next_page_token: bounded_optional_cursor(response.next_page_token)?,
            next_sync_token: bounded_optional_cursor(response.next_sync_token)?,
        })
    }

    pub async fn upsert_contact(
        &self,
        access_token: &str,
        request: &GooglePeopleUpsertV1,
    ) -> Result<GooglePeopleUpsertedV1, GooglePeopleAdapterErrorV1> {
        validate_upsert(request)?;
        let (method, path, person) = match (&request.resource_name, &request.expected_etag) {
            (None, None) => (
                "POST",
                format!("/v1/people:createContact?personFields={PERSON_FIELDS}"),
                person_for_write(request, None),
            ),
            (Some(resource_name), Some(etag)) => (
                "PATCH",
                format!(
                    "/v1/{resource_name}:updateContact?updatePersonFields={UPDATE_FIELDS}&personFields={PERSON_FIELDS}"
                ),
                person_for_write(request, Some(source_for_update(resource_name, etag)?)),
            ),
            _ => return Err(GooglePeopleAdapterErrorV1::InvalidRequest),
        };
        let body =
            serde_json::to_vec(&person).map_err(|_| GooglePeopleAdapterErrorV1::InvalidRequest)?;
        let response: Person = self
            .request_json(access_token, method, &path, Some(&body))
            .await?;
        let contact = contact_from_person(response)?;
        Ok(GooglePeopleUpsertedV1 {
            resource_name: contact.resource_name,
            etag: contact.etag,
        })
    }

    async fn request_json<T: for<'de> Deserialize<'de>>(
        &self,
        access_token: &str,
        method: &str,
        path: &str,
        body: Option<&[u8]>,
    ) -> Result<T, GooglePeopleAdapterErrorV1> {
        async_std::future::timeout(
            OPERATION_TIMEOUT,
            self.request_json_inner(access_token, method, path, body),
        )
        .await
        .map_err(|_| GooglePeopleAdapterErrorV1::OutcomeUnknown)?
    }

    async fn request_json_inner<T: for<'de> Deserialize<'de>>(
        &self,
        access_token: &str,
        method: &str,
        path: &str,
        body: Option<&[u8]>,
    ) -> Result<T, GooglePeopleAdapterErrorV1> {
        if !valid_token(access_token)
            || !matches!(method, "GET" | "POST" | "PATCH")
            || !path.starts_with('/')
            || path.contains(['\r', '\n'])
        {
            return Err(GooglePeopleAdapterErrorV1::InvalidRequest);
        }
        let stream = TcpStream::connect((self.host.as_str(), self.port))
            .await
            .map_err(|_| GooglePeopleAdapterErrorV1::Unavailable)?;
        let connector = self
            .ca_certificate_pem
            .as_deref()
            .map(|pem| {
                Certificate::from_pem(pem.as_bytes())
                    .map(|certificate| TlsConnector::new().add_root_certificate(certificate))
                    .map_err(|_| GooglePeopleAdapterErrorV1::InvalidRequest)
            })
            .transpose()?
            .unwrap_or_default();
        let mut stream = connector
            .connect(self.host.as_str(), stream)
            .await
            .map_err(|_| GooglePeopleAdapterErrorV1::Unavailable)?;
        let body = body.unwrap_or_default();
        let request_head = format!(
            "{method} {path} HTTP/1.1\r\nHost: {}\r\nAuthorization: Bearer {access_token}\r\nAccept: application/json\r\nAccept-Encoding: identity\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            self.host,
            body.len(),
        );
        stream
            .write_all(request_head.as_bytes())
            .await
            .map_err(|_| GooglePeopleAdapterErrorV1::OutcomeUnknown)?;
        if !body.is_empty() {
            stream
                .write_all(body)
                .await
                .map_err(|_| GooglePeopleAdapterErrorV1::OutcomeUnknown)?;
        }
        stream
            .flush()
            .await
            .map_err(|_| GooglePeopleAdapterErrorV1::OutcomeUnknown)?;
        let mut response = Vec::new();
        stream
            .take((MAX_RESPONSE_BYTES + 1) as u64)
            .read_to_end(&mut response)
            .await
            .map_err(|_| GooglePeopleAdapterErrorV1::OutcomeUnknown)?;
        parse_response(&response)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Person {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    resource_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    metadata: Option<PersonMetadata>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    names: Vec<Name>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    email_addresses: Vec<Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    phone_numbers: Vec<Value>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct PersonMetadata {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    sources: Vec<PersonSource>,
    #[serde(default, skip_serializing)]
    deleted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PersonSource {
    #[serde(rename = "type")]
    source_type: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    id: String,
    etag: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Name {
    display_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Value {
    value: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConnectionsResponse {
    connections: Option<Vec<Person>>,
    next_page_token: Option<String>,
    next_sync_token: Option<String>,
}

fn validate_upsert(request: &GooglePeopleUpsertV1) -> Result<(), GooglePeopleAdapterErrorV1> {
    if !valid_text(&request.display_name)
        || request.email_addresses.len() > GOOGLE_PEOPLE_MAX_CONTACT_VALUES_V1
        || request.phone_numbers.len() > GOOGLE_PEOPLE_MAX_CONTACT_VALUES_V1
        || request
            .email_addresses
            .iter()
            .any(|value| !valid_text(value))
        || request.phone_numbers.iter().any(|value| !valid_text(value))
        || request.email_addresses.is_empty() && request.phone_numbers.is_empty()
        || request
            .resource_name
            .as_deref()
            .is_some_and(|value| !valid_resource_name(value))
        || request
            .expected_etag
            .as_deref()
            .is_some_and(|value| !valid_etag(value))
    {
        return Err(GooglePeopleAdapterErrorV1::InvalidRequest);
    }
    Ok(())
}

fn person_for_write(request: &GooglePeopleUpsertV1, source: Option<PersonSource>) -> Person {
    Person {
        resource_name: request.resource_name.clone().unwrap_or_default(),
        metadata: source.map(|source| PersonMetadata {
            sources: vec![source],
            deleted: false,
        }),
        names: vec![Name {
            display_name: request.display_name.clone(),
        }],
        email_addresses: request
            .email_addresses
            .iter()
            .cloned()
            .map(|value| Value { value })
            .collect(),
        phone_numbers: request
            .phone_numbers
            .iter()
            .cloned()
            .map(|value| Value { value })
            .collect(),
    }
}

fn source_for_update(
    resource_name: &str,
    etag: &str,
) -> Result<PersonSource, GooglePeopleAdapterErrorV1> {
    let id = resource_name
        .strip_prefix("people/")
        .filter(|value| !value.is_empty())
        .ok_or(GooglePeopleAdapterErrorV1::InvalidRequest)?;
    Ok(PersonSource {
        source_type: "CONTACT".to_owned(),
        id: id.to_owned(),
        etag: etag.to_owned(),
    })
}

fn contact_from_person(
    person: Person,
) -> Result<GooglePeopleContactV1, GooglePeopleAdapterErrorV1> {
    let etag = person
        .metadata
        .as_ref()
        .and_then(|metadata| {
            metadata
                .sources
                .iter()
                .find(|source| source.source_type == "CONTACT")
        })
        .map(|source| source.etag.clone())
        .filter(|value| valid_etag(value))
        .ok_or(GooglePeopleAdapterErrorV1::InvalidResponse)?;
    if !valid_resource_name(&person.resource_name) {
        return Err(GooglePeopleAdapterErrorV1::InvalidResponse);
    }
    let display_name = person
        .names
        .first()
        .map(|name| name.display_name.clone())
        .filter(|value| valid_text(value))
        .ok_or(GooglePeopleAdapterErrorV1::InvalidResponse)?;
    let email_addresses = bounded_values(person.email_addresses)?;
    let phone_numbers = bounded_values(person.phone_numbers)?;
    if email_addresses.is_empty() && phone_numbers.is_empty() {
        return Err(GooglePeopleAdapterErrorV1::InvalidResponse);
    }
    Ok(GooglePeopleContactV1 {
        resource_name: person.resource_name,
        etag,
        display_name,
        email_addresses,
        phone_numbers,
    })
}

fn bounded_values(values: Vec<Value>) -> Result<Vec<String>, GooglePeopleAdapterErrorV1> {
    if values.len() > GOOGLE_PEOPLE_MAX_CONTACT_VALUES_V1
        || values.iter().any(|value| !valid_text(&value.value))
    {
        return Err(GooglePeopleAdapterErrorV1::InvalidResponse);
    }
    Ok(values.into_iter().map(|value| value.value).collect())
}

fn parse_response<T: for<'de> Deserialize<'de>>(
    response: &[u8],
) -> Result<T, GooglePeopleAdapterErrorV1> {
    if response.len() > MAX_RESPONSE_BYTES {
        return Err(GooglePeopleAdapterErrorV1::InvalidResponse);
    }
    let split = response
        .windows(4)
        .position(|value| value == b"\r\n\r\n")
        .ok_or(GooglePeopleAdapterErrorV1::InvalidResponse)?;
    let headers = std::str::from_utf8(&response[..split])
        .map_err(|_| GooglePeopleAdapterErrorV1::InvalidResponse)?;
    let status = headers
        .split_whitespace()
        .nth(1)
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or(GooglePeopleAdapterErrorV1::InvalidResponse)?;
    let body = response_body(headers, &response[split + 4..])?;
    if status == 400
        && body
            .windows(b"failedPrecondition".len())
            .any(|window| window == b"failedPrecondition")
    {
        return Err(GooglePeopleAdapterErrorV1::EtagConflict);
    }
    if !(200..300).contains(&status) {
        return Err(GooglePeopleAdapterErrorV1::ProviderRejected(status));
    }
    serde_json::from_slice(&body).map_err(|_| GooglePeopleAdapterErrorV1::InvalidResponse)
}

fn response_body(headers: &str, body: &[u8]) -> Result<Vec<u8>, GooglePeopleAdapterErrorV1> {
    let transfer_encoding = header_value(headers, "transfer-encoding");
    if transfer_encoding.is_some_and(|value| !value.eq_ignore_ascii_case("chunked")) {
        return Err(GooglePeopleAdapterErrorV1::InvalidResponse);
    }
    if transfer_encoding.is_some() {
        return decode_chunked(body);
    }
    if let Some(length) = header_value(headers, "content-length") {
        let length = length
            .parse::<usize>()
            .map_err(|_| GooglePeopleAdapterErrorV1::InvalidResponse)?;
        if length != body.len() || length > MAX_RESPONSE_BYTES {
            return Err(GooglePeopleAdapterErrorV1::InvalidResponse);
        }
    }
    Ok(body.to_vec())
}

fn header_value<'a>(headers: &'a str, expected: &str) -> Option<&'a str> {
    headers.lines().skip(1).find_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.eq_ignore_ascii_case(expected).then(|| value.trim())
    })
}

fn decode_chunked(mut input: &[u8]) -> Result<Vec<u8>, GooglePeopleAdapterErrorV1> {
    let mut output = Vec::new();
    loop {
        let line_end = input
            .windows(2)
            .position(|window| window == b"\r\n")
            .ok_or(GooglePeopleAdapterErrorV1::InvalidResponse)?;
        let size = std::str::from_utf8(&input[..line_end])
            .ok()
            .and_then(|line| line.split(';').next())
            .and_then(|value| usize::from_str_radix(value.trim(), 16).ok())
            .ok_or(GooglePeopleAdapterErrorV1::InvalidResponse)?;
        input = &input[line_end + 2..];
        if size == 0 {
            return input
                .starts_with(b"\r\n")
                .then_some(output)
                .ok_or(GooglePeopleAdapterErrorV1::InvalidResponse);
        }
        if size > MAX_RESPONSE_BYTES.saturating_sub(output.len())
            || input.len() < size + 2
            || &input[size..size + 2] != b"\r\n"
        {
            return Err(GooglePeopleAdapterErrorV1::InvalidResponse);
        }
        output.extend_from_slice(&input[..size]);
        input = &input[size + 2..];
    }
}

fn valid_host(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 253
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
}

fn valid_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_TOKEN_BYTES
        && !value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace())
}

fn valid_text(value: &str) -> bool {
    !value.trim().is_empty()
        && value.len() <= MAX_TEXT_BYTES
        && !value.chars().any(char::is_control)
}

fn valid_resource_name(value: &str) -> bool {
    value.strip_prefix("people/").is_some_and(|id| {
        !id.is_empty()
            && id.len() <= 256
            && id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    })
}

fn valid_etag(value: &str) -> bool {
    !value.is_empty() && value.len() <= 512 && !value.chars().any(char::is_control)
}

fn valid_cursor(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 4_096
        && !value
            .chars()
            .any(|value| value.is_control() || value.is_whitespace())
}

fn bounded_optional_cursor(
    value: Option<String>,
) -> Result<Option<String>, GooglePeopleAdapterErrorV1> {
    if value.as_deref().is_some_and(|value| !valid_cursor(value)) {
        return Err(GooglePeopleAdapterErrorV1::InvalidResponse);
    }
    Ok(value)
}

fn percent_encode(value: &str) -> Result<String, GooglePeopleAdapterErrorV1> {
    if !valid_cursor(value) {
        return Err(GooglePeopleAdapterErrorV1::InvalidRequest);
    }
    Ok(value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                char::from(byte).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_payload_carries_contact_source_etag() {
        let request = GooglePeopleUpsertV1 {
            resource_name: Some("people/abc-1".to_owned()),
            expected_etag: Some("etag-1".to_owned()),
            display_name: "Ada Lovelace".to_owned(),
            email_addresses: vec!["ada@example.test".to_owned()],
            phone_numbers: Vec::new(),
        };
        validate_upsert(&request).expect("valid update");
        let person = person_for_write(
            &request,
            Some(source_for_update("people/abc-1", "etag-1").expect("source")),
        );
        let json = serde_json::to_string(&person).expect("json");
        assert!(json.contains("\"type\":\"CONTACT\""));
        assert!(json.contains("\"etag\":\"etag-1\""));
        assert!(!json.contains("access_token"));
    }

    #[test]
    fn rejects_create_with_update_only_etag() {
        let request = GooglePeopleUpsertV1 {
            resource_name: None,
            expected_etag: Some("etag-1".to_owned()),
            display_name: "Ada".to_owned(),
            email_addresses: vec!["ada@example.test".to_owned()],
            phone_numbers: Vec::new(),
        };
        let client = GooglePeopleClientV1::new().expect("client");
        assert_eq!(
            async_std::task::block_on(client.upsert_contact("token", &request)),
            Err(GooglePeopleAdapterErrorV1::InvalidRequest),
        );
    }

    #[test]
    fn parses_contact_etag_from_contact_source_only() {
        let person: Person = serde_json::from_str(
            r#"{"resourceName":"people/1","metadata":{"sources":[{"type":"PROFILE","id":"1","etag":"profile"},{"type":"CONTACT","id":"1","etag":"contact"}]},"names":[{"displayName":"Ada"}],"emailAddresses":[{"value":"ada@example.test"}]}"#,
        )
        .expect("person");
        let contact = contact_from_person(person).expect("contact");
        assert_eq!(contact.etag, "contact");
    }

    #[test]
    fn decodes_bounded_chunked_json_body() {
        assert_eq!(
            response_body(
                "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked",
                b"3\r\n{\"a\r\n3\r\n\":1\r\n1\r\n}\r\n0\r\n\r\n",
            ),
            Ok(br#"{"a":1}"#.to_vec()),
        );
    }
}
