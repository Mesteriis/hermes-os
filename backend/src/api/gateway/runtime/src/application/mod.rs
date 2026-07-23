//! Detached composition of the narrow pre-owner Gateway HTTP surface.

use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;

use bytes::Bytes;
use hermes_gateway_session_contract::{
    BrowserAuthenticationAuthority, BrowserEnrollmentAuthority, ClientBootstrapAuthority,
};
use http_body_util::Full;
use hyper::Request;
use hyper::body::Body;
use hyper::header::{HOST, ORIGIN};
use hyper::service::Service;
use hyper::{Response, StatusCode};

use crate::{
    BrowserAuthenticationRouter, BrowserBootstrapRouter, BrowserPairingRouter,
    BrowserRealtimeRouter, BrowserRealtimeSubscriptionSource, BrowserSessionStatusRouter,
    CommunicationsQueryRouter, GatewayHttpResponse, GatewayTechnicalRouter,
    ClientBootstrapRouter,
    SharedBrowserGatewaySessionService,
};

const AUTHENTICATION_PREFIX: &str = "/browser/v1/authentication/";
const PAIRING_PREFIX: &str = "/browser/v1/pairing/";
const REALTIME_PATH: &str = "/api/realtime/v1/events";
const SESSION_STATUS_PATH: &str = "/hermes.gateway.v1.BrowserSessionService/GetStatus";
const CLIENT_BOOTSTRAP_PATH: &str = "/hermes.gateway.v1.ClientBootstrapService/GetBootstrap";
const COMMUNICATIONS_QUERY_PATH: &str = "/hermes.communications.query.v1.CommunicationsQueryService/Query";

/// Composes technical health, browser authentication and client-safe realtime
/// without adding an owner API or mounting a listener.
pub struct GatewayApplicationRouter<A, S> {
    technical: GatewayTechnicalRouter,
    browser_authentication: BrowserAuthenticationRouter<A>,
    browser_pairing: Option<BrowserPairingRouter<A>>,
    browser_bootstrap: Option<BrowserBootstrapRouter>,
    browser_session_status: BrowserSessionStatusRouter<A>,
    client_bootstrap: ClientBootstrapRouter<A>,
    communications_query: Option<CommunicationsQueryRouter<A>>,
    browser_realtime: BrowserRealtimeRouter<A, S>,
    lan_development_policy: Option<LanDevelopmentRequestPolicyV1>,
}

#[derive(Clone)]
struct LanDevelopmentRequestPolicyV1 {
    exact_origin: String,
    exact_authority: String,
}

impl<A, S> Clone for GatewayApplicationRouter<A, S> {
    fn clone(&self) -> Self {
        Self {
            technical: self.technical,
            browser_authentication: self.browser_authentication.clone(),
            browser_pairing: self.browser_pairing.clone(),
            browser_bootstrap: self.browser_bootstrap.clone(),
            browser_session_status: self.browser_session_status.clone(),
            client_bootstrap: self.client_bootstrap.clone(),
            communications_query: self.communications_query.clone(),
            browser_realtime: self.browser_realtime.clone(),
            lan_development_policy: self.lan_development_policy.clone(),
        }
    }
}

impl<A, S> GatewayApplicationRouter<A, S>
where
    A: BrowserAuthenticationAuthority + BrowserEnrollmentAuthority + ClientBootstrapAuthority,
    S: BrowserRealtimeSubscriptionSource,
{
    #[must_use]
    pub fn new(ready: bool, service: SharedBrowserGatewaySessionService<A>, source: S) -> Self {
        Self {
            technical: GatewayTechnicalRouter::new(ready),
            browser_authentication: BrowserAuthenticationRouter::from_shared(service.clone()),
            browser_pairing: None,
            browser_bootstrap: None,
            browser_session_status: BrowserSessionStatusRouter::from_shared(service.clone()),
            client_bootstrap: ClientBootstrapRouter::from_shared(service.clone()),
            communications_query: None,
            browser_realtime: BrowserRealtimeRouter::new(service, source),
            lan_development_policy: None,
        }
    }

    #[must_use]
    pub fn with_browser_pairing(mut self, router: BrowserPairingRouter<A>) -> Self {
        self.browser_pairing = Some(router);
        self
    }

    #[must_use]
    pub fn with_browser_bootstrap(mut self, router: BrowserBootstrapRouter) -> Self {
        self.browser_bootstrap = Some(router);
        self
    }

    #[must_use]
    pub fn with_communications_query(mut self, router: CommunicationsQueryRouter<A>) -> Self {
        self.communications_query = Some(router);
        self
    }

    pub fn with_lan_development_policy(mut self, exact_origin: &str) -> Result<Self, &'static str> {
        let exact_authority = exact_origin
            .strip_prefix("http://")
            .filter(|authority| !authority.is_empty() && !authority.contains('/'))
            .ok_or("developer mode origin is invalid")?;
        self.lan_development_policy = Some(LanDevelopmentRequestPolicyV1 {
            exact_origin: exact_origin.to_owned(),
            exact_authority: exact_authority.to_owned(),
        });
        Ok(self)
    }

    pub async fn route<B>(&self, request: Request<B>) -> GatewayHttpResponse
    where
        B: Body<Data = Bytes>,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        let path = request.uri().path();
        let route = route_class(path);
        let method = request.method().clone();
        if let Some(policy) = &self.lan_development_policy
            && !policy.admits(&request)
        {
            println!(
                "developer_gateway_request method={} route={} status={} admission=rejected",
                method,
                route,
                StatusCode::FORBIDDEN.as_u16()
            );
            return forbidden();
        }
        let response = self.route_admitted(request).await;
        if self.lan_development_policy.is_some() {
            println!(
                "developer_gateway_request method={} route={} status={} admission=accepted",
                method,
                route,
                response.status().as_u16()
            );
        }
        response
    }

    async fn route_admitted<B>(&self, request: Request<B>) -> GatewayHttpResponse
    where
        B: Body<Data = Bytes>,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        let path = request.uri().path();
        if request.uri().query().is_some() {
            return self.technical.route(request.method(), "");
        }
        if path == "/" || path.starts_with("/assets/") {
            return match &self.browser_bootstrap {
                Some(router) => router.route(request.method(), path),
                None => self.technical.route(request.method(), path),
            };
        }
        if is_technical_path(path) {
            return self.technical.route(request.method(), path);
        }
        if path == REALTIME_PATH {
            return self.browser_realtime.route(request);
        }
        if path == SESSION_STATUS_PATH {
            return self.browser_session_status.route(request).await;
        }
        if path == CLIENT_BOOTSTRAP_PATH {
            return self.client_bootstrap.route(request).await;
        }
        if path == COMMUNICATIONS_QUERY_PATH
            && let Some(router) = &self.communications_query
        {
            return router.route(request).await;
        }
        if path == COMMUNICATIONS_QUERY_PATH {
            return not_found();
        }
        if path.starts_with(AUTHENTICATION_PREFIX) {
            if self.lan_development_policy.is_some() {
                return self.technical.route(request.method(), path);
            }
            return self.browser_authentication.route(request).await;
        }
        if path.starts_with(PAIRING_PREFIX) {
            if self.lan_development_policy.is_some() {
                return self.technical.route(request.method(), path);
            }
            return match &self.browser_pairing {
                Some(router) => router.route(request).await,
                None => self.technical.route(request.method(), path),
            };
        }
        self.technical.route(request.method(), path)
    }
}

impl LanDevelopmentRequestPolicyV1 {
    fn admits<B>(&self, request: &Request<B>) -> bool {
        const FORWARDED_HEADERS: [&str; 7] = [
            "forwarded",
            "x-forwarded-for",
            "x-forwarded-host",
            "x-forwarded-proto",
            "cf-connecting-ip",
            "true-client-ip",
            "x-real-ip",
        ];
        let headers = request.headers();
        if FORWARDED_HEADERS
            .iter()
            .any(|name| headers.contains_key(*name))
        {
            return false;
        }
        let header_authority = headers.get(HOST).and_then(|value| value.to_str().ok());
        let uri_authority = request.uri().authority().map(|value| value.as_str());
        if header_authority
            .into_iter()
            .chain(uri_authority)
            .any(|authority| authority != self.exact_authority)
            || (header_authority.is_none() && uri_authority.is_none())
        {
            return false;
        }
        if headers
            .get(ORIGIN)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|origin| origin != self.exact_origin)
        {
            return false;
        }
        headers
            .get("sec-fetch-site")
            .and_then(|value| value.to_str().ok())
            .is_none_or(|site| matches!(site, "same-origin" | "none"))
    }
}

fn route_class(path: &str) -> &'static str {
    match path {
        "/" => "browser_bootstrap",
        "/healthz" => "health",
        "/readyz" => "readiness",
        REALTIME_PATH => "client_realtime",
        SESSION_STATUS_PATH => "browser_session_status",
        CLIENT_BOOTSTRAP_PATH => "client_bootstrap",
        path if path.starts_with(AUTHENTICATION_PREFIX) => "browser_authentication",
        path if path.starts_with(PAIRING_PREFIX) => "browser_pairing",
        path if path == COMMUNICATIONS_QUERY_PATH => "communications_query",
        _ => "unknown",
    }
}

fn not_found() -> GatewayHttpResponse {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("cache-control", "no-store")
        .body(crate::full_gateway_body(Bytes::from_static(b"unimplemented\n")))
        .expect("Gateway not found response is valid")
}

fn forbidden() -> GatewayHttpResponse {
    Response::builder()
        .status(StatusCode::FORBIDDEN)
        .header("cache-control", "no-store")
        .body(crate::full_gateway_body(Bytes::from_static(
            b"developer LAN admission rejected\n",
        )))
        .expect("Gateway rejection response is valid")
}

impl<A, S> Service<Request<hyper::body::Incoming>> for GatewayApplicationRouter<A, S>
where
    A: BrowserAuthenticationAuthority
        + BrowserEnrollmentAuthority
        + ClientBootstrapAuthority
        + Send
        + Sync
        + 'static,
    S: BrowserRealtimeSubscriptionSource + 'static,
{
    type Response = GatewayHttpResponse;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, request: Request<hyper::body::Incoming>) -> Self::Future {
        let router = self.clone();
        Box::pin(async move { Ok(router.route(request).await) })
    }
}

/// HTTP/3 buffers its bounded request body before invoking the same Gateway
/// router. Keeping this adapter body explicit avoids treating QUIC streams as
/// a second owner API surface.
impl<A, S> Service<Request<Full<Bytes>>> for GatewayApplicationRouter<A, S>
where
    A: BrowserAuthenticationAuthority
        + BrowserEnrollmentAuthority
        + ClientBootstrapAuthority
        + Send
        + Sync
        + 'static,
    S: BrowserRealtimeSubscriptionSource + 'static,
{
    type Response = GatewayHttpResponse;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, request: Request<Full<Bytes>>) -> Self::Future {
        let router = self.clone();
        Box::pin(async move { Ok(router.route(request).await) })
    }
}

fn is_technical_path(path: &str) -> bool {
    matches!(path, "/healthz" | "/readyz")
}
