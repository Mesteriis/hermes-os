use std::convert::Infallible;
use std::future::{Ready, ready};

use hyper::service::Service;
use hyper::{Method, Request, Response, StatusCode};

use crate::{GatewayHttpResponse, full_gateway_body};

/// Client-safe technical surface before the first owner contract is admitted.
#[derive(Debug, Clone, Copy)]
pub struct GatewayTechnicalRouter {
    ready: bool,
}

impl GatewayTechnicalRouter {
    #[must_use]
    pub const fn new(ready: bool) -> Self {
        Self { ready }
    }

    /// Routes only health/readiness probes; no owner, provider or generic API
    /// is reachable before its public contract is explicitly admitted.
    #[must_use]
    pub fn route(&self, method: &Method, path: &str) -> GatewayHttpResponse {
        match (method, path) {
            (&Method::GET, "/healthz") => response(StatusCode::OK, "ok\n"),
            (&Method::GET, "/readyz") if self.ready => response(StatusCode::OK, "ready\n"),
            (&Method::GET, "/readyz") => response(StatusCode::SERVICE_UNAVAILABLE, "not ready\n"),
            _ => response(StatusCode::NOT_FOUND, "not found\n"),
        }
    }
}

impl<B> Service<Request<B>> for GatewayTechnicalRouter {
    type Response = GatewayHttpResponse;
    type Error = Infallible;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn call(&self, request: Request<B>) -> Self::Future {
        ready(Ok(self.route(request.method(), request.uri().path())))
    }
}

fn response(status: StatusCode, body: &'static str) -> GatewayHttpResponse {
    Response::builder()
        .status(status)
        .body(full_gateway_body(body))
        .expect("Gateway technical response is valid")
}
