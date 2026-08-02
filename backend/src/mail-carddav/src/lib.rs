//! Bounded read-only CardDAV adapter owned by the Mail integration.
//!
//! Provider credentials are borrowed by the caller and never retained. Remote
//! write is intentionally outside this adapter's admitted contract.

use std::{fmt, time::Duration};

use async_native_tls::{Certificate, TlsConnector};
use async_std::net::TcpStream;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use futures_util::io::{AsyncReadExt, AsyncWriteExt};
use quick_xml::escape::unescape;

pub const ICLOUD_CARDDAV_HOST_V1: &str = "contacts.icloud.com";
pub const ICLOUD_CARDDAV_PORT_V1: u16 = 443;
pub const ICLOUD_CARDDAV_CREDENTIAL_PURPOSE_V1: &str = "mail_icloud_carddav_password";
pub const CARDDAV_MAX_CONTACTS_V1: usize = 10_000;

const MAX_RESPONSE_BYTES: usize = 16 * 1024 * 1024;
const MAX_CREDENTIAL_BYTES: usize = 16 * 1024;
const MAX_FIELD_BYTES: usize = 2_048;
const OPERATION_TIMEOUT: Duration = Duration::from_secs(30);
const DISCOVERY_BODY: &str = "<?xml version=\"1.0\"?><propfind xmlns=\"DAV:\"><prop><current-user-principal/><addressbook-home-set xmlns=\"urn:ietf:params:xml:ns:carddav\"/></prop></propfind>";
const ALLPROP_BODY: &str = "<?xml version=\"1.0\"?><propfind xmlns=\"DAV:\"><allprop/></propfind>";
const ADDRESSBOOK_QUERY_BODY: &str = "<?xml version=\"1.0\"?><card:addressbook-query xmlns:d=\"DAV:\" xmlns:card=\"urn:ietf:params:xml:ns:carddav\"><d:prop><d:getetag/><d:href/><card:address-data/></d:prop></card:addressbook-query>";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CardDavClientV1 {
    host: String,
    port: u16,
    base_path: String,
    ca_certificate_pem: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CardDavContactV1 {
    pub href: String,
    pub etag: String,
    pub display_name: String,
    pub email_addresses: Vec<String>,
    pub phone_numbers: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CardDavAdapterErrorV1 {
    InvalidRequest,
    Unavailable,
    ProviderRejected(u16),
    InvalidResponse,
    DiscoveryFailed,
    ReadOnlyProvider,
}

impl fmt::Display for CardDavAdapterErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for CardDavAdapterErrorV1 {}

impl CardDavClientV1 {
    pub fn new() -> Result<Self, CardDavAdapterErrorV1> {
        Self::for_endpoint(ICLOUD_CARDDAV_HOST_V1, ICLOUD_CARDDAV_PORT_V1, "/", None)
    }

    #[cfg(any(test, feature = "conformance-test-support"))]
    pub fn for_conformance_endpoint(
        host: impl Into<String>,
        port: u16,
        base_path: impl Into<String>,
        ca_certificate_pem: Option<String>,
    ) -> Result<Self, CardDavAdapterErrorV1> {
        let host = host.into();
        if !matches!(host.as_str(), "127.0.0.1" | "localhost") {
            return Err(CardDavAdapterErrorV1::InvalidRequest);
        }
        Self::for_endpoint(host, port, base_path, ca_certificate_pem)
    }

    fn for_endpoint(
        host: impl Into<String>,
        port: u16,
        base_path: impl Into<String>,
        ca_certificate_pem: Option<String>,
    ) -> Result<Self, CardDavAdapterErrorV1> {
        let host = host.into();
        let base_path = base_path.into();
        if !valid_host(&host)
            || port == 0
            || !valid_path(&base_path)
            || ca_certificate_pem
                .as_deref()
                .is_some_and(|value| value.is_empty() || value.len() > 64 * 1024)
        {
            return Err(CardDavAdapterErrorV1::InvalidRequest);
        }
        Ok(Self {
            host,
            port,
            base_path,
            ca_certificate_pem,
        })
    }

    #[must_use]
    pub const fn supports_remote_write(&self) -> bool {
        false
    }

    pub fn reject_remote_write(&self) -> Result<(), CardDavAdapterErrorV1> {
        Err(CardDavAdapterErrorV1::ReadOnlyProvider)
    }

    pub async fn list_contacts(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Vec<CardDavContactV1>, CardDavAdapterErrorV1> {
        if !valid_credential(username) || !valid_credential(password) {
            return Err(CardDavAdapterErrorV1::InvalidRequest);
        }
        async_std::future::timeout(
            OPERATION_TIMEOUT,
            self.list_contacts_inner(username, password),
        )
        .await
        .map_err(|_| CardDavAdapterErrorV1::Unavailable)?
    }

    async fn list_contacts_inner(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Vec<CardDavContactV1>, CardDavAdapterErrorV1> {
        let discovery = self
            .xml_request(
                "PROPFIND",
                &self.base_path,
                "1",
                DISCOVERY_BODY,
                username,
                password,
            )
            .await?;
        let home_href = match first_property_href(&discovery, "addressbook-home-set") {
            Some(value) => value,
            None => {
                let principal = first_property_href(&discovery, "current-user-principal")
                    .ok_or(CardDavAdapterErrorV1::DiscoveryFailed)?;
                let principal = self
                    .xml_request(
                        "PROPFIND",
                        &resolve_path(&self.host, self.port, &self.base_path, &principal)?,
                        "1",
                        DISCOVERY_BODY,
                        username,
                        password,
                    )
                    .await?;
                first_property_href(&principal, "addressbook-home-set")
                    .ok_or(CardDavAdapterErrorV1::DiscoveryFailed)?
            }
        };
        let home_path = resolve_path(&self.host, self.port, &self.base_path, &home_href)?;
        let home = self
            .xml_request(
                "PROPFIND",
                &home_path,
                "1",
                ALLPROP_BODY,
                username,
                password,
            )
            .await?;
        let address_book_href = response_blocks(&home)
            .into_iter()
            .find(|block| find_named_open_tag(block, "addressbook").is_some())
            .and_then(|block| first_tag_text(block, "href"))
            .ok_or(CardDavAdapterErrorV1::DiscoveryFailed)?;
        let response = self
            .xml_request(
                "REPORT",
                &resolve_path(&self.host, self.port, &self.base_path, &address_book_href)?,
                "1",
                ADDRESSBOOK_QUERY_BODY,
                username,
                password,
            )
            .await?;
        let contacts = response_blocks(&response)
            .into_iter()
            .filter_map(carddav_contact)
            .collect::<Vec<_>>();
        if contacts.len() > CARDDAV_MAX_CONTACTS_V1 {
            return Err(CardDavAdapterErrorV1::InvalidResponse);
        }
        Ok(contacts)
    }

    async fn xml_request(
        &self,
        method: &str,
        path: &str,
        depth: &str,
        body: &str,
        username: &str,
        password: &str,
    ) -> Result<String, CardDavAdapterErrorV1> {
        if !matches!(method, "PROPFIND" | "REPORT") || !valid_path(path) {
            return Err(CardDavAdapterErrorV1::InvalidRequest);
        }
        let stream = TcpStream::connect((self.host.as_str(), self.port))
            .await
            .map_err(|_| CardDavAdapterErrorV1::Unavailable)?;
        let connector = self
            .ca_certificate_pem
            .as_deref()
            .map(|pem| {
                Certificate::from_pem(pem.as_bytes())
                    .map(|certificate| TlsConnector::new().add_root_certificate(certificate))
                    .map_err(|_| CardDavAdapterErrorV1::InvalidRequest)
            })
            .transpose()?
            .unwrap_or_default();
        let mut stream = connector
            .connect(self.host.as_str(), stream)
            .await
            .map_err(|_| CardDavAdapterErrorV1::Unavailable)?;
        let authorization = STANDARD.encode(format!("{username}:{password}"));
        let request = format!(
            "{method} {path} HTTP/1.1\r\nHost: {}\r\nAuthorization: Basic {authorization}\r\nDepth: {depth}\r\nAccept: application/xml\r\nAccept-Encoding: identity\r\nContent-Type: application/xml; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            self.host,
            body.len(),
        );
        stream
            .write_all(request.as_bytes())
            .await
            .map_err(|_| CardDavAdapterErrorV1::Unavailable)?;
        stream
            .flush()
            .await
            .map_err(|_| CardDavAdapterErrorV1::Unavailable)?;
        let mut response = Vec::new();
        stream
            .take((MAX_RESPONSE_BYTES + 1) as u64)
            .read_to_end(&mut response)
            .await
            .map_err(|_| CardDavAdapterErrorV1::Unavailable)?;
        parse_xml_response(&response)
    }
}

fn parse_xml_response(response: &[u8]) -> Result<String, CardDavAdapterErrorV1> {
    if response.len() > MAX_RESPONSE_BYTES {
        return Err(CardDavAdapterErrorV1::InvalidResponse);
    }
    let split = response
        .windows(4)
        .position(|value| value == b"\r\n\r\n")
        .ok_or(CardDavAdapterErrorV1::InvalidResponse)?;
    let headers = std::str::from_utf8(&response[..split])
        .map_err(|_| CardDavAdapterErrorV1::InvalidResponse)?;
    let status = headers
        .split_whitespace()
        .nth(1)
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or(CardDavAdapterErrorV1::InvalidResponse)?;
    if status != 207 && !(200..300).contains(&status) {
        return Err(CardDavAdapterErrorV1::ProviderRejected(status));
    }
    let body = response_body(headers, &response[split + 4..])?;
    let body = std::str::from_utf8(&body).map_err(|_| CardDavAdapterErrorV1::InvalidResponse)?;
    if body.is_empty() || body.len() > MAX_RESPONSE_BYTES {
        return Err(CardDavAdapterErrorV1::InvalidResponse);
    }
    Ok(body.to_owned())
}

fn response_body(headers: &str, body: &[u8]) -> Result<Vec<u8>, CardDavAdapterErrorV1> {
    let transfer_encoding = header_value(headers, "transfer-encoding");
    if transfer_encoding.is_some_and(|value| !value.eq_ignore_ascii_case("chunked")) {
        return Err(CardDavAdapterErrorV1::InvalidResponse);
    }
    if transfer_encoding.is_some() {
        return decode_chunked(body);
    }
    if let Some(length) = header_value(headers, "content-length") {
        let length = length
            .parse::<usize>()
            .map_err(|_| CardDavAdapterErrorV1::InvalidResponse)?;
        if length != body.len() || length > MAX_RESPONSE_BYTES {
            return Err(CardDavAdapterErrorV1::InvalidResponse);
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

fn decode_chunked(mut input: &[u8]) -> Result<Vec<u8>, CardDavAdapterErrorV1> {
    let mut output = Vec::new();
    loop {
        let line_end = input
            .windows(2)
            .position(|window| window == b"\r\n")
            .ok_or(CardDavAdapterErrorV1::InvalidResponse)?;
        let size = std::str::from_utf8(&input[..line_end])
            .ok()
            .and_then(|line| line.split(';').next())
            .and_then(|value| usize::from_str_radix(value.trim(), 16).ok())
            .ok_or(CardDavAdapterErrorV1::InvalidResponse)?;
        input = &input[line_end + 2..];
        if size == 0 {
            return input
                .starts_with(b"\r\n")
                .then_some(output)
                .ok_or(CardDavAdapterErrorV1::InvalidResponse);
        }
        if size > MAX_RESPONSE_BYTES.saturating_sub(output.len())
            || input.len() < size + 2
            || &input[size..size + 2] != b"\r\n"
        {
            return Err(CardDavAdapterErrorV1::InvalidResponse);
        }
        output.extend_from_slice(&input[..size]);
        input = &input[size + 2..];
    }
}

fn carddav_contact(response: &str) -> Option<CardDavContactV1> {
    let href = first_tag_text(response, "href")?;
    let etag = first_tag_text(response, "getetag")?;
    let card = first_tag_text(response, "address-data")?;
    let vcard = unescape(&card).ok()?.into_owned();
    let display_name = vcard_property(&vcard, "FN")?;
    let email_addresses = vcard_properties(&vcard, "EMAIL");
    let phone_numbers = vcard_properties(&vcard, "TEL");
    if !valid_path(&href)
        || !valid_field(&etag)
        || !valid_field(&display_name)
        || email_addresses.is_empty() && phone_numbers.is_empty()
    {
        return None;
    }
    Some(CardDavContactV1 {
        href,
        etag,
        display_name,
        email_addresses,
        phone_numbers,
    })
}

fn response_blocks(xml: &str) -> Vec<&str> {
    let mut blocks = Vec::new();
    let mut remainder = xml;
    while let Some(start) = find_named_open_tag(remainder, "response") {
        let Some((tag, content_start)) = open_tag_name_and_content_start(&remainder[start..])
        else {
            break;
        };
        let close = format!("</{tag}>");
        let Some(end) = remainder[start + content_start..].find(&close) else {
            break;
        };
        let end = start + content_start + end + close.len();
        blocks.push(&remainder[start..end]);
        remainder = &remainder[end..];
    }
    blocks
}

fn first_property_href(xml: &str, property: &str) -> Option<String> {
    let start = find_named_open_tag(xml, property)?;
    let (_, content_start) = open_tag_name_and_content_start(&xml[start..])?;
    first_tag_text(&xml[start + content_start..], "href")
}

fn first_tag_text(xml: &str, name: &str) -> Option<String> {
    let start = find_named_open_tag(xml, name)?;
    let (tag, content_start) = open_tag_name_and_content_start(&xml[start..])?;
    let content_start = start + content_start;
    let close = format!("</{tag}>");
    let content_end = xml[content_start..].find(&close)? + content_start;
    Some(xml[content_start..content_end].trim().to_owned())
}

fn find_named_open_tag(xml: &str, name: &str) -> Option<usize> {
    let mut offset = 0;
    while let Some(found) = xml[offset..].find('<') {
        let start = offset + found;
        let Some((tag, _)) = open_tag_name_and_content_start(&xml[start..]) else {
            offset = start + 1;
            continue;
        };
        if tag == name || tag.ends_with(&format!(":{name}")) {
            return Some(start);
        }
        offset = start + 1;
    }
    None
}

fn open_tag_name_and_content_start(xml: &str) -> Option<(String, usize)> {
    let end = xml.find('>')?;
    let raw = xml.get(1..end)?.trim_start();
    if raw.starts_with(['/', '?', '!']) {
        return None;
    }
    let tag = raw
        .split_whitespace()
        .next()?
        .trim_end_matches('/')
        .to_owned();
    Some((tag, end + 1))
}

fn vcard_property(vcard: &str, name: &str) -> Option<String> {
    vcard_properties(vcard, name).into_iter().next()
}

fn vcard_properties(vcard: &str, name: &str) -> Vec<String> {
    unfold_vcard_lines(vcard)
        .lines()
        .filter_map(|line| {
            let (key, value) = line.split_once(':')?;
            key.split(';')
                .next()?
                .eq_ignore_ascii_case(name)
                .then(|| unescape_vcard_text(value.trim()))
                .filter(|value| valid_field(value))
        })
        .take(64)
        .collect()
}

fn unfold_vcard_lines(vcard: &str) -> String {
    let mut unfolded = String::with_capacity(vcard.len());
    for line in vcard.replace("\r\n", "\n").split('\n') {
        if line.starts_with([' ', '\t']) {
            unfolded.push_str(line.trim_start_matches([' ', '\t']));
        } else {
            if !unfolded.is_empty() {
                unfolded.push('\n');
            }
            unfolded.push_str(line);
        }
    }
    unfolded
}

fn unescape_vcard_text(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            output.push(ch);
            continue;
        }
        match chars.next() {
            Some('n' | 'N') => output.push('\n'),
            Some(',') => output.push(','),
            Some(';') => output.push(';'),
            Some('\\') => output.push('\\'),
            Some(other) => {
                output.push('\\');
                output.push(other);
            }
            None => output.push('\\'),
        }
    }
    output
}

fn resolve_path(
    host: &str,
    port: u16,
    base_path: &str,
    href: &str,
) -> Result<String, CardDavAdapterErrorV1> {
    let href = href.trim();
    let default_origin = format!("https://{host}");
    let explicit_origin = format!("https://{host}:{port}");
    let href = if let Some(path) = href.strip_prefix(&explicit_origin) {
        path
    } else if port == 443 {
        href.strip_prefix(&default_origin).unwrap_or(href)
    } else {
        href
    };
    if href.starts_with("https://") || href.starts_with("http://") {
        return Err(CardDavAdapterErrorV1::InvalidResponse);
    }
    let path = if href.starts_with('/') {
        href.to_owned()
    } else {
        format!("{}{href}", base_path.trim_end_matches('/').to_owned() + "/")
    };
    valid_path(&path)
        .then_some(path)
        .ok_or(CardDavAdapterErrorV1::InvalidResponse)
}

fn valid_host(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 253
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
}

fn valid_path(value: &str) -> bool {
    value.starts_with('/')
        && value.len() <= 4_096
        && !value.contains("..")
        && !value
            .chars()
            .any(|value| value.is_control() || value.is_whitespace())
}

fn valid_credential(value: &str) -> bool {
    !value.is_empty() && value.len() <= MAX_CREDENTIAL_BYTES && !value.chars().any(char::is_control)
}

fn valid_field(value: &str) -> bool {
    !value.trim().is_empty()
        && value.len() <= MAX_FIELD_BYTES
        && !value.chars().any(char::is_control)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_namespaced_folded_and_escaped_vcard_contact() {
        let response = r#"<d:response xmlns:d="DAV:" xmlns:card="urn:ietf:params:xml:ns:carddav"><d:href>/card/grace.vcf</d:href><d:getetag>"v2"</d:getetag><card:address-data>BEGIN:VCARD&#10;FN:Grace\, Hopper&#10;EMAIL;TYPE=INTERNET:grace@exam&#10; ple.test&#10;TEL;TYPE=CELL:+456&#10;END:VCARD</card:address-data></d:response>"#;
        let contact = carddav_contact(response).expect("CardDAV contact");
        assert_eq!(contact.href, "/card/grace.vcf");
        assert_eq!(contact.display_name, "Grace, Hopper");
        assert_eq!(contact.email_addresses, ["grace@example.test"]);
        assert_eq!(contact.phone_numbers, ["+456"]);
        assert_eq!(contact.etag, "\"v2\"");
    }

    #[test]
    fn remote_write_is_explicitly_read_only() {
        let client = CardDavClientV1::new().expect("client");
        assert!(!client.supports_remote_write());
        assert_eq!(
            client.reject_remote_write(),
            Err(CardDavAdapterErrorV1::ReadOnlyProvider),
        );
    }

    #[test]
    fn refuses_cross_host_discovery_urls() {
        assert_eq!(
            resolve_path(
                ICLOUD_CARDDAV_HOST_V1,
                ICLOUD_CARDDAV_PORT_V1,
                "/",
                "https://attacker.invalid/addressbook",
            ),
            Err(CardDavAdapterErrorV1::InvalidResponse),
        );
    }

    #[test]
    fn accepts_pinned_same_origin_discovery_url() {
        assert_eq!(
            resolve_path(
                ICLOUD_CARDDAV_HOST_V1,
                ICLOUD_CARDDAV_PORT_V1,
                "/",
                "https://contacts.icloud.com/addressbooks/user/default/",
            ),
            Ok("/addressbooks/user/default/".to_owned()),
        );
    }

    #[test]
    fn decodes_bounded_chunked_xml_body() {
        assert_eq!(
            response_body(
                "HTTP/1.1 207 Multi-Status\r\nTransfer-Encoding: chunked",
                b"4\r\n<xml\r\n2\r\n/>\r\n0\r\n\r\n",
            ),
            Ok(b"<xml/>".to_vec()),
        );
    }
}
