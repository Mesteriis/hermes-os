mod http3;
mod listener;
mod profile;

pub use http3::GatewayHttp3ListenerV1;
pub use listener::{
    GatewayLanDevelopmentListenerV1, GatewayLoopbackListenerV1, GatewayLoopbackTlsListenerV1,
    GatewayTlsListenerV1,
};
pub use profile::{GatewayTransportProfileV1, PairedRemoteProfileV1};

use std::convert::Infallible;
use std::time::Duration;

use bytes::Bytes;
use http_body_util::{BodyExt, Full, combinators::BoxBody};
use hyper::service::Service;
use hyper::{Request, Response, body::Incoming};
use hyper_util::rt::{TokioExecutor, TokioIo, TokioTimer};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_rustls::server::TlsStream;

pub type GatewayHttpResponse = Response<BoxBody<Bytes, Infallible>>;

pub fn full_gateway_body(body: impl Into<Bytes>) -> BoxBody<Bytes, Infallible> {
    Full::new(body.into()).boxed()
}

/// Serves one loopback-only local client connection. Local transport is a
/// distinct profile: it is never used for paired remote access, which remains
/// TLS plus HTTP/2 below.
pub async fn serve_local_embedded_http1<T, S>(stream: T, service: S) -> Result<(), String>
where
    T: AsyncRead + AsyncWrite + Send + Unpin + 'static,
    S: Service<Request<Incoming>, Response = GatewayHttpResponse> + Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    S::Future: Send + 'static,
{
    let mut builder = hyper::server::conn::http1::Builder::new();
    builder
        .timer(TokioTimer::new())
        .header_read_timeout(Duration::from_secs(10));
    builder
        .serve_connection(TokioIo::new(stream), service)
        .await
        .map_err(|error| format!("Gateway local HTTP/1.1 connection failed: {error}"))
}

/// Serves one already-negotiated TLS connection using the mandatory HTTP/2
/// remote baseline. The caller owns listener admission and TLS identity.
pub async fn serve_paired_remote_http2<T, S>(stream: TlsStream<T>, service: S) -> Result<(), String>
where
    T: AsyncRead + AsyncWrite + Send + Unpin + 'static,
    S: Service<Request<Incoming>, Response = GatewayHttpResponse> + Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    S::Future: Send + 'static,
{
    hyper::server::conn::http2::Builder::new(TokioExecutor::new())
        .serve_connection(TokioIo::new(stream), service)
        .await
        .map_err(|error| format!("Gateway HTTP/2 connection failed: {error}"))
}
