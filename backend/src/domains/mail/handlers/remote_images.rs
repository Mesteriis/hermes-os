use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::Duration;

use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, header};
use axum::response::Response;
use reqwest::Url;
use serde::Deserialize;
use thiserror::Error;

use super::communication_messages::rich_body_html_for_message;
use crate::app::{ApiError, AppState};
use crate::domains::api_support::message_store;

const MAX_REMOTE_IMAGE_BYTES: u64 = 12 * 1024 * 1024;
const REMOTE_IMAGE_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Deserialize)]
pub(crate) struct RemoteImageQuery {
    url: String,
}

pub(crate) async fn get_v1_communication_message_remote_image(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Query(query): Query<RemoteImageQuery>,
) -> Result<Response, ApiError> {
    let image_url = parse_remote_image_url(&query.url)?;
    let Some(message) = message_store(&state)?.message(&message_id).await? else {
        return Err(ApiError::CommunicationMessageNotFound);
    };
    let Some(body_html) = rich_body_html_for_message(&state, &message).await? else {
        return Err(ApiError::CommunicationMessageNotFound);
    };
    if !message_html_references_url(&body_html, image_url.as_str()) {
        return Err(ApiError::InvalidCommunicationQuery(
            "remote image is not referenced by this message",
        ));
    }

    let image = fetch_remote_image(&image_url)
        .await
        .map_err(remote_image_fetch_api_error)?;

    let mut response = Response::new(Body::from(image.body));
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, image.content_type);
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, max-age=600"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    Ok(response)
}

fn parse_remote_image_url(value: &str) -> Result<Url, ApiError> {
    if value.len() > 4096 {
        return Err(ApiError::InvalidCommunicationQuery(
            "remote image URL is too long",
        ));
    }
    let url = Url::parse(value)
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid remote image URL"))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(ApiError::InvalidCommunicationQuery(
            "remote image URL scheme is not allowed",
        ));
    }
    let Some(host) = url.host_str() else {
        return Err(ApiError::InvalidCommunicationQuery(
            "remote image URL host is required",
        ));
    };
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    if host == "localhost" || host.ends_with(".localhost") || host.ends_with(".local") {
        return Err(ApiError::InvalidCommunicationQuery(
            "remote image URL host is not allowed",
        ));
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        ensure_public_ip(ip)?;
    }
    Ok(url)
}

fn message_html_references_url(body_html: &str, image_url: &str) -> bool {
    body_html.contains(image_url)
        || body_html.contains(&image_url.replace('&', "&amp;"))
        || body_html.replace("&amp;", "&").contains(image_url)
}

struct RemoteImage {
    content_type: HeaderValue,
    body: Vec<u8>,
}

#[derive(Debug, Error)]
enum RemoteImageFetchError {
    #[error("remote image host is unavailable")]
    MissingHost,
    #[error("remote image has no public DNS address")]
    NoPublicAddress,
    #[error("remote image client failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("remote image returned non-success status")]
    NonSuccessStatus,
    #[error("remote image content type is not image")]
    NotImage,
    #[error("remote image exceeds size limit")]
    TooLarge,
    #[error("remote image response header is invalid")]
    InvalidHeader,
}

async fn fetch_remote_image(url: &Url) -> Result<RemoteImage, RemoteImageFetchError> {
    let default_client = remote_image_client(None)?;
    match fetch_remote_image_with_client(&default_client, url).await {
        Ok(image) => Ok(image),
        Err(first_error @ RemoteImageFetchError::Http(_)) => {
            let Some(host) = url.host_str() else {
                return Err(RemoteImageFetchError::MissingHost);
            };
            let port = url.port_or_known_default().unwrap_or(443);
            let public_addrs = resolve_public_image_addrs(host, port).await?;
            if public_addrs.is_empty() {
                return Err(first_error);
            }
            let fallback_client = remote_image_client(Some((host, public_addrs.as_slice())))?;
            fetch_remote_image_with_client(&fallback_client, url).await
        }
        Err(error) => Err(error),
    }
}

fn remote_image_client(
    dns_override: Option<(&str, &[SocketAddr])>,
) -> Result<reqwest::Client, RemoteImageFetchError> {
    let mut builder = reqwest::Client::builder()
        .timeout(REMOTE_IMAGE_TIMEOUT)
        .redirect(reqwest::redirect::Policy::limited(4))
        .user_agent("HermesHub-MailImageProxy/0.1");
    if let Some((host, addrs)) = dns_override {
        builder = builder.resolve_to_addrs(host, addrs);
    }
    Ok(builder.build()?)
}

async fn fetch_remote_image_with_client(
    client: &reqwest::Client,
    url: &Url,
) -> Result<RemoteImage, RemoteImageFetchError> {
    let response = client.get(url.clone()).send().await?;
    if !response.status().is_success() {
        return Err(RemoteImageFetchError::NonSuccessStatus);
    }
    if response.content_length().unwrap_or(0) > MAX_REMOTE_IMAGE_BYTES {
        return Err(RemoteImageFetchError::TooLarge);
    }
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .cloned()
        .ok_or(RemoteImageFetchError::InvalidHeader)?;
    let content_type_text = content_type
        .to_str()
        .map_err(|_| RemoteImageFetchError::InvalidHeader)?
        .to_ascii_lowercase();
    if !content_type_text.starts_with("image/") {
        return Err(RemoteImageFetchError::NotImage);
    }
    let body = response.bytes().await?;
    if body.len() as u64 > MAX_REMOTE_IMAGE_BYTES {
        return Err(RemoteImageFetchError::TooLarge);
    }
    Ok(RemoteImage {
        content_type,
        body: body.to_vec(),
    })
}

#[derive(Deserialize)]
struct GoogleDnsResponse {
    #[serde(rename = "Answer")]
    answer: Option<Vec<GoogleDnsAnswer>>,
}

#[derive(Deserialize)]
struct GoogleDnsAnswer {
    #[serde(rename = "type")]
    record_type: u16,
    data: String,
}

async fn resolve_public_image_addrs(
    host: &str,
    port: u16,
) -> Result<Vec<SocketAddr>, RemoteImageFetchError> {
    let resolver = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;
    let response = resolver
        .get("https://dns.google/resolve")
        .query(&[("name", host), ("type", "A")])
        .send()
        .await?
        .error_for_status()?;
    let dns = response.json::<GoogleDnsResponse>().await?;
    let addrs = dns
        .answer
        .unwrap_or_default()
        .into_iter()
        .filter(|answer| answer.record_type == 1)
        .filter_map(|answer| answer.data.parse::<Ipv4Addr>().ok())
        .map(IpAddr::V4)
        .filter(|ip| is_public_ip(*ip))
        .map(|ip| SocketAddr::new(ip, port))
        .collect::<Vec<_>>();
    if addrs.is_empty() {
        return Err(RemoteImageFetchError::NoPublicAddress);
    }
    Ok(addrs)
}

fn ensure_public_ip(ip: IpAddr) -> Result<(), ApiError> {
    if is_public_ip(ip) {
        return Ok(());
    }
    Err(ApiError::InvalidCommunicationQuery(
        "remote image URL address is not allowed",
    ))
}

fn is_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => is_public_ipv4(ip),
        IpAddr::V6(ip) => is_public_ipv6(ip),
    }
}

fn is_public_ipv4(ip: Ipv4Addr) -> bool {
    !(ip.is_private()
        || ip.is_loopback()
        || ip.is_link_local()
        || ip.is_broadcast()
        || ip.is_documentation()
        || ip.octets()[0] == 0
        || ip.octets()[0] >= 224
        || (ip.octets()[0] == 100 && (64..=127).contains(&ip.octets()[1])))
}

fn is_public_ipv6(ip: Ipv6Addr) -> bool {
    !(ip.is_loopback()
        || ip.is_unspecified()
        || ip.is_unique_local()
        || ((ip.segments()[0] & 0xffc0) == 0xfe80)
        || ((ip.segments()[0] & 0xff00) == 0xff00))
}

fn remote_image_fetch_api_error(error: RemoteImageFetchError) -> ApiError {
    match error {
        RemoteImageFetchError::TooLarge => {
            ApiError::InvalidCommunicationQuery("remote image exceeds size limit")
        }
        RemoteImageFetchError::NotImage => {
            ApiError::InvalidCommunicationQuery("remote asset is not an image")
        }
        RemoteImageFetchError::NoPublicAddress => {
            ApiError::InvalidCommunicationQuery("remote image host has no public address")
        }
        _ => ApiError::InvalidCommunicationQuery("remote image unavailable"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_private_and_local_image_hosts() {
        assert!(parse_remote_image_url("https://127.0.0.1/a.png").is_err());
        assert!(parse_remote_image_url("https://192.168.1.20/a.png").is_err());
        assert!(parse_remote_image_url("https://localhost/a.png").is_err());
        assert!(parse_remote_image_url("cid:part-1").is_err());
        assert!(parse_remote_image_url("https://image.email.feverup.com/a.png").is_ok());
    }

    #[test]
    fn accepts_escaped_message_image_references() {
        let html = r#"<img src="https://img.example.test/a.png?x=1&amp;y=2">"#;
        assert!(message_html_references_url(
            html,
            "https://img.example.test/a.png?x=1&y=2"
        ));
        assert!(!message_html_references_url(
            html,
            "https://img.example.test/other.png"
        ));
    }

    #[test]
    fn classifies_public_ips_for_ssrf_guard() {
        assert!(is_public_ip(IpAddr::V4(Ipv4Addr::new(13, 224, 83, 8))));
        assert!(!is_public_ip(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))));
        assert!(!is_public_ip(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
        assert!(!is_public_ip(IpAddr::V6(Ipv6Addr::LOCALHOST)));
    }
}
