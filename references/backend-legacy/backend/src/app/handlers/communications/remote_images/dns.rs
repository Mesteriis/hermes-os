use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use serde::Deserialize;

use super::errors::RemoteImageFetchError;
use super::url_policy::is_public_ip;

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

pub(super) async fn resolve_public_image_addrs(
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
