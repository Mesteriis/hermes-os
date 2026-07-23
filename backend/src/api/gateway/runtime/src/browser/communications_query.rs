use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use hermes_gateway_session_contract::BrowserAuthenticationAuthority;
use http_body_util::{BodyExt, Limited};
use hyper::body::Body;
use hyper::header::{CACHE_CONTROL, CONTENT_TYPE, COOKIE, HeaderName};
use hyper::{Method, Request, Response, StatusCode};
use tokio::task;
use tokio::time::{Instant, timeout_at};

use crate::{
    GatewayHttpResponse,
    SharedBrowserGatewaySessionService,
    full_gateway_body,
};

const QUERY_PATH: &str = "/hermes.communications.query.v1.CommunicationsQueryService/Query";
const MAX_REQUEST_BYTES: usize = 16_384;
const MAX_REQUEST_DEADLINE: Duration = Duration::from_secs(10);
const CONNECT_PROTOCOL_VERSION: HeaderName = HeaderName::from_static("connect-protocol-version");
const CONNECT_ERROR_CODE: HeaderName = HeaderName::from_static("connect-error-code");
const CONNECT_TIMEOUT_MS: HeaderName = HeaderName::from_static("connect-timeout-ms");

pub struct CommunicationsQueryRouter<A> {
    service: SharedBrowserGatewaySessionService<A>,
    handler: CommunicationsQueryRouteHandler,
}

pub type CommunicationsQueryRouteHandler =
    Arc<dyn Fn(&str, &[u8]) -> Result<Vec<u8>, CommunicationsQueryRouteErrorV1> + Send + Sync>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsQueryRouteErrorV1 {
    InvalidArgument,
    NotFound,
    Unavailable,
    Internal,
}

impl<A> CommunicationsQueryRouter<A>
where
    A: BrowserAuthenticationAuthority,
{
    #[must_use]
    pub fn new(
        service: SharedBrowserGatewaySessionService<A>,
        handler: CommunicationsQueryRouteHandler,
    ) -> Self {
        Self {
            service,
            handler,
        }
    }

    pub async fn route<B>(&self, request: Request<B>) -> GatewayHttpResponse
    where
        B: Body<Data = Bytes>,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        let (parts, body) = request.into_parts();
        if parts.method != Method::POST || parts.uri.path() != QUERY_PATH || parts.uri.query().is_some() {
            return not_found();
        }
        if !is_protobuf(&parts.headers) {
            return invalid_argument();
        }
        let cookie = parts
            .headers
            .get(COOKIE)
            .and_then(|value| value.to_str().ok());
        let session = match self.service.authorize_request(cookie) {
            Ok(session) => session,
            Err(_) => return unauthenticated(),
        };
        let timeout = match request_timeout(&parts.headers) {
            Ok(timeout) => timeout,
            Err(()) => return invalid_argument(),
        };
        let deadline = Instant::now() + timeout;
        let body = match timeout_at(deadline, Limited::new(body, MAX_REQUEST_BYTES).collect()).await {
            Ok(Ok(collected)) => collected.to_bytes(),
            Ok(Err(_)) => return invalid_argument(),
            Err(_) => return deadline_exceeded(),
        };
        if body.is_empty() {
            return invalid_argument();
        }
        let owner_id = session.owner_id().to_owned();
        let handler = Arc::clone(&self.handler);
        let response_payload = match timeout_at(
            deadline,
            task::spawn_blocking(move || handler(&owner_id, &body)),
        )
        .await
        {
            Ok(Ok(Ok(response))) => response,
            Ok(Ok(Err(error))) => return query_error(error),
            Ok(Err(_)) => return internal(),
            Err(_) => return deadline_exceeded(),
        };
        if response_payload.is_empty() {
            return unavailable();
        }
        protobuf_response(response_payload)
    }
}

impl<A> Clone for CommunicationsQueryRouter<A> {
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
            handler: Arc::clone(&self.handler),
        }
    }
}

fn query_error(error: CommunicationsQueryRouteErrorV1) -> GatewayHttpResponse {
    match error {
        CommunicationsQueryRouteErrorV1::InvalidArgument => invalid_argument(),
        CommunicationsQueryRouteErrorV1::NotFound => not_found(),
        CommunicationsQueryRouteErrorV1::Unavailable => unavailable(),
        CommunicationsQueryRouteErrorV1::Internal => internal(),
    }
}

fn is_protobuf(headers: &hyper::HeaderMap) -> bool {
    headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .is_some_and(|value| {
            matches!(
                value.trim(),
                "application/proto" | "application/connect+proto"
            )
        })
}

fn request_timeout(headers: &hyper::HeaderMap) -> Result<Duration, ()> {
    let Some(value) = headers.get(CONNECT_TIMEOUT_MS) else {
        return Ok(MAX_REQUEST_DEADLINE);
    };
    let millis = value
        .to_str()
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| (1..=MAX_REQUEST_DEADLINE.as_millis() as u64).contains(value))
        .ok_or(())?;
    Ok(Duration::from_millis(millis))
}

fn protobuf_response(response_payload: Vec<u8>) -> GatewayHttpResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "application/proto")
        .header(CACHE_CONTROL, "no-store")
        .header(CONNECT_PROTOCOL_VERSION, "1")
        .body(
            full_gateway_body(response_payload),
        )
        .expect("Gateway Query response is valid")
}

fn invalid_argument() -> GatewayHttpResponse {
    connect_error(StatusCode::BAD_REQUEST, "invalid_argument")
}

fn unauthenticated() -> GatewayHttpResponse {
    connect_error(StatusCode::UNAUTHORIZED, "unauthenticated")
}

fn not_found() -> GatewayHttpResponse {
    connect_error(StatusCode::NOT_FOUND, "unimplemented")
}

fn unavailable() -> GatewayHttpResponse {
    connect_error(StatusCode::SERVICE_UNAVAILABLE, "unavailable")
}

fn internal() -> GatewayHttpResponse {
    connect_error(StatusCode::INTERNAL_SERVER_ERROR, "internal")
}

fn deadline_exceeded() -> GatewayHttpResponse {
    connect_error(StatusCode::GATEWAY_TIMEOUT, "deadline_exceeded")
}

fn connect_error(status: StatusCode, code: &'static str) -> GatewayHttpResponse {
    Response::builder()
        .status(status)
        .header(CACHE_CONTROL, "no-store")
        .header(CONNECT_PROTOCOL_VERSION, "1")
        .header(CONNECT_ERROR_CODE, code)
        .body(full_gateway_body(Bytes::new()))
        .expect("Gateway Connect error is valid")
}
