use std::collections::BTreeMap;

use bytes::Bytes;
use hyper::header::{CACHE_CONTROL, CONTENT_SECURITY_POLICY, CONTENT_TYPE};
use hyper::{Method, Response, StatusCode};

use crate::{GatewayHttpResponse, full_gateway_body};

const MAX_BOOTSTRAP_BYTES: usize = 512 * 1024;
const MAX_ASSET_BYTES: usize = 1536 * 1024;
const MAX_TOTAL_ASSET_BYTES: usize = 4 * 1024 * 1024;

/// Immutable bytes loaded only after the installed release verifier has bound
/// them to a signed browser-bootstrap artifact.
#[derive(Clone)]
pub struct BrowserBootstrapRouter {
    document: Bytes,
    assets: BTreeMap<String, BrowserBootstrapAsset>,
}

#[derive(Clone)]
struct BrowserBootstrapAsset {
    bytes: Bytes,
    content_type: &'static str,
}

impl BrowserBootstrapRouter {
    pub fn new(document: Vec<u8>) -> Result<Self, String> {
        (!document.is_empty()
            && document.len() <= MAX_BOOTSTRAP_BYTES
            && std::str::from_utf8(&document).is_ok())
        .then_some(Self {
            document: Bytes::from(document),
            assets: BTreeMap::new(),
        })
        .ok_or_else(|| "signed browser bootstrap is invalid".to_owned())
    }

    /// Exact signed assets only: no filesystem lookup or directory fallback.
    pub fn with_assets<I>(mut self, assets: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = (String, Vec<u8>)>,
    {
        let mut total = 0usize;
        for (path, bytes) in assets {
            let content_type = asset_content_type(&path)
                .ok_or_else(|| "signed browser asset path is invalid".to_owned())?;
            if bytes.is_empty() || bytes.len() > MAX_ASSET_BYTES || self.assets.contains_key(&path)
            {
                return Err("signed browser asset is invalid".to_owned());
            }
            total = total
                .checked_add(bytes.len())
                .ok_or_else(|| "signed browser asset inventory is invalid".to_owned())?;
            if total > MAX_TOTAL_ASSET_BYTES {
                return Err("signed browser asset inventory is too large".to_owned());
            }
            self.assets.insert(
                path,
                BrowserBootstrapAsset {
                    bytes: Bytes::from(bytes),
                    content_type,
                },
            );
        }
        Ok(self)
    }

    #[must_use]
    pub fn route(&self, method: &Method, path: &str) -> GatewayHttpResponse {
        match (method, path) {
            (&Method::GET, "/") => Response::builder()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, "text/html; charset=utf-8")
                .header(CACHE_CONTROL, "no-store")
                .header(
                    CONTENT_SECURITY_POLICY,
                    "default-src 'self'; base-uri 'none'; form-action 'self'; frame-ancestors 'none'",
                )
                .body(full_gateway_body(self.document.clone()))
                .expect("Gateway browser bootstrap response is valid"),
            (&Method::GET, path) if self.assets.contains_key(path) => {
                let asset = self.assets.get(path).expect("checked browser asset");
                Response::builder()
                    .status(StatusCode::OK)
                    .header(CONTENT_TYPE, asset.content_type)
                    .header(CACHE_CONTROL, "public, max-age=31536000, immutable")
                    .header(
                        CONTENT_SECURITY_POLICY,
                        "default-src 'self'; base-uri 'none'; form-action 'self'; frame-ancestors 'none'",
                    )
                    .body(full_gateway_body(asset.bytes.clone()))
                    .expect("Gateway browser bootstrap response is valid")
            }
            _ => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header(CACHE_CONTROL, "no-store")
                .body(full_gateway_body("not found\n"))
                .expect("Gateway browser bootstrap response is valid"),
        }
    }
}

fn asset_content_type(path: &str) -> Option<&'static str> {
    if !path.starts_with("/assets/") || path.contains("//") || path.contains("..") {
        return None;
    }
    if path.ends_with(".js") {
        return Some("text/javascript; charset=utf-8");
    }
    if path.ends_with(".css") {
        return Some("text/css; charset=utf-8");
    }
    if path.ends_with(".svg") {
        return Some("image/svg+xml");
    }
    if path.ends_with(".png") {
        return Some("image/png");
    }
    if path.ends_with(".webp") {
        return Some("image/webp");
    }
    None
}
