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
    ClientBootstrapRouter, GatewayHttpResponse, GatewayTechnicalRouter,
    SharedBrowserGatewaySessionService,
};

mod mail;

const AUTHENTICATION_PREFIX: &str = "/browser/v1/authentication/";
const PAIRING_PREFIX: &str = "/browser/v1/pairing/";
const REALTIME_PATH: &str = "/api/realtime/v1/events";
const SESSION_STATUS_PATH: &str = "/hermes.gateway.v1.BrowserSessionService/GetStatus";
const CLIENT_BOOTSTRAP_PATH: &str = "/hermes.gateway.v1.ClientBootstrapService/GetBootstrap";
const MAIL_COMMUNICATIONS_LIST_PATH: &str = "/api/v1/communications/email/accounts";
const MAIL_INTEGRATIONS_PATH_PREFIX: &str = "/api/v1/integrations/mail/";
const TELEGRAM_INTEGRATIONS_PATH_PREFIX: &str = "/api/v1/integrations/telegram/";
const WHATSAPP_INTEGRATIONS_PATH_PREFIX: &str = "/api/v1/integrations/whatsapp/";
const TELEGRAM_INTEGRATIONS_PATH_ROOT: &str = "/api/v1/integrations/telegram";
const WHATSAPP_INTEGRATIONS_PATH_ROOT: &str = "/api/v1/integrations/whatsapp";
const MAIL_GMAIL_OAUTH_CALLBACK_PATH: &str =
    "/api/v1/integrations/mail/accounts/gmail/oauth/callback";

/// Composes technical health, browser authentication and client-safe realtime
/// without adding an owner API or mounting a listener.
pub struct GatewayApplicationRouter<A, S> {
    technical: GatewayTechnicalRouter,
    browser_authentication: BrowserAuthenticationRouter<A>,
    browser_pairing: Option<BrowserPairingRouter<A>>,
    browser_bootstrap: Option<BrowserBootstrapRouter>,
    browser_session_status: BrowserSessionStatusRouter<A>,
    client_bootstrap: ClientBootstrapRouter<A>,
    browser_realtime: BrowserRealtimeRouter<A, S>,
    mail: mail::MailGatewayIntegrationRouter,
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
            browser_realtime: self.browser_realtime.clone(),
            mail: self.mail.clone(),
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
            browser_realtime: BrowserRealtimeRouter::new(service, source),
            mail: mail::MailGatewayIntegrationRouter::new(),
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
        if let Some(provider) = unsupported_integration_provider(path) {
            return unsupported_integration_response(provider);
        }
        if request.uri().query().is_some() && path != MAIL_GMAIL_OAUTH_CALLBACK_PATH {
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
        if path.starts_with(MAIL_INTEGRATIONS_PATH_PREFIX) {
            return self.mail.route(request).await;
        }
        if path == MAIL_COMMUNICATIONS_LIST_PATH {
            return self.mail.route(request).await;
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
        path if path.starts_with(MAIL_INTEGRATIONS_PATH_PREFIX) => "mail",
        path if path == MAIL_COMMUNICATIONS_LIST_PATH => "mail",
        path if path == TELEGRAM_INTEGRATIONS_PATH_ROOT => {
            unsupported_integration_response_class(path, "telegram")
        }
        path if path == WHATSAPP_INTEGRATIONS_PATH_ROOT => {
            unsupported_integration_response_class(path, "whatsapp")
        }
        path if path.starts_with(TELEGRAM_INTEGRATIONS_PATH_PREFIX) => {
            unsupported_integration_response_class(path, "telegram")
        }
        path if path.starts_with(WHATSAPP_INTEGRATIONS_PATH_PREFIX) => {
            unsupported_integration_response_class(path, "whatsapp")
        }
        _ => "unknown",
    }
}

fn unsupported_integration_provider(path: &str) -> Option<&'static str> {
    if path == TELEGRAM_INTEGRATIONS_PATH_ROOT || path.starts_with(TELEGRAM_INTEGRATIONS_PATH_PREFIX) {
        Some("telegram")
    } else if path == WHATSAPP_INTEGRATIONS_PATH_ROOT
        || path.starts_with(WHATSAPP_INTEGRATIONS_PATH_PREFIX)
    {
        Some("whatsapp")
    } else {
        None
    }
}

fn unsupported_integration_response(provider: &str) -> GatewayHttpResponse {
    Response::builder()
        .status(StatusCode::NOT_IMPLEMENTED)
        .header("cache-control", "no-store")
        .body(crate::full_gateway_body(Bytes::from(format!(
            "integration {provider} is not migrated in clean-room backend yet; endpoint remains reserved for external provider runtime\n"
        ))))
        .expect("Gateway response for unsupported provider integration is valid")
}

fn unsupported_integration_response_class(path: &str, provider: &str) -> &'static str {
    if path == TELEGRAM_INTEGRATIONS_PATH_ROOT || path == WHATSAPP_INTEGRATIONS_PATH_ROOT {
        "communications_integrations_legacy"
    } else {
        provider
    }
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
