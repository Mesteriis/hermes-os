use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use reqwest::Url;

use crate::app::error::types::ApiError;

const MAX_REMOTE_IMAGE_URL_BYTES: usize = 4096;

pub(super) fn parse_remote_image_url(value: &str) -> Result<Url, ApiError> {
    if value.len() > MAX_REMOTE_IMAGE_URL_BYTES {
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

fn ensure_public_ip(ip: IpAddr) -> Result<(), ApiError> {
    if is_public_ip(ip) {
        return Ok(());
    }
    Err(ApiError::InvalidCommunicationQuery(
        "remote image URL address is not allowed",
    ))
}

pub(super) fn is_public_ip(ip: IpAddr) -> bool {
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
    fn classifies_public_ips_for_ssrf_guard() {
        assert!(is_public_ip(IpAddr::V4(Ipv4Addr::new(13, 224, 83, 8))));
        assert!(!is_public_ip(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))));
        assert!(!is_public_ip(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
        assert!(!is_public_ip(IpAddr::V6(Ipv6Addr::LOCALHOST)));
    }
}
