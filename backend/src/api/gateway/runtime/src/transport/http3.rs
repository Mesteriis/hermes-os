use std::net::SocketAddr;

use bytes::{Buf, Bytes, BytesMut};
use h3::server::Connection;
use http_body_util::{BodyExt, Full};
use hyper::service::Service;
use hyper::{Request, Response, StatusCode};
use quinn::{Endpoint, ServerConfig, VarInt};
use tokio::sync::watch;
use tokio::task::JoinSet;

use crate::{
    GatewayHttpResponse, GatewayTransportProfileV1, PairedRemoteProfileV1, full_gateway_body,
};

/// Request bodies are buffered only at the HTTP/3 adapter boundary. Gateway
/// routes currently need bounded protocol messages, never an unbounded upload.
const MAX_HTTP3_REQUEST_BODY_BYTES: usize = 1024 * 1024;

/// A paired-remote HTTP/3 listener. It accepts only HTTP/3 over a Quinn server
/// endpoint; raw QUIC is never exposed to Gateway routes.
pub struct GatewayHttp3ListenerV1 {
    endpoint: Endpoint,
}

impl GatewayHttp3ListenerV1 {
    /// Binds a TLS-authenticated QUIC endpoint only when the paired-remote
    /// profile explicitly enables HTTP/3. The caller owns certificate loading
    /// and public-origin admission.
    pub fn bind(
        address: SocketAddr,
        profile: PairedRemoteProfileV1,
        server_config: ServerConfig,
    ) -> Result<Self, String> {
        GatewayTransportProfileV1::PairedRemote(profile)
            .validate_bind(address.ip(), true)
            .map_err(str::to_owned)?;
        if !profile.http3_enabled() {
            return Err(
                "Gateway HTTP/3 listener requires an HTTP/3-enabled remote profile".to_owned(),
            );
        }
        let endpoint = Endpoint::server(server_config, address)
            .map_err(|error| format!("Gateway HTTP/3 listener bind failed: {error}"))?;
        Ok(Self { endpoint })
    }

    /// Serves HTTP/3 connections until the owning process requests shutdown.
    /// Connection failures stay local to their peer and cannot terminate the
    /// listener. Shutdown closes the endpoint and aborts active handlers, so
    /// sessions must reconnect after a Gateway restart.
    pub async fn serve_until_shutdown<S>(
        self,
        service: S,
        mut shutdown_requested: watch::Receiver<bool>,
    ) -> Result<(), String>
    where
        S: Service<Request<Full<Bytes>>, Response = GatewayHttpResponse> + Clone + Send + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        S::Future: Send + 'static,
    {
        if *shutdown_requested.borrow() {
            return Ok(());
        }
        let mut connections = JoinSet::new();
        loop {
            tokio::select! {
                changed = shutdown_requested.changed() => {
                    match changed {
                        Ok(()) if *shutdown_requested.borrow() => {
                            self.endpoint.close(VarInt::from_u32(0), b"gateway shutdown");
                            connections.abort_all();
                            while connections.join_next().await.is_some() {}
                            return Ok(());
                        }
                        Ok(()) => {}
                        Err(_) => {
                            self.endpoint.close(VarInt::from_u32(0), b"gateway shutdown");
                            connections.abort_all();
                            while connections.join_next().await.is_some() {}
                            return Ok(());
                        }
                    }
                }
                incoming = self.endpoint.accept() => {
                    let Some(incoming) = incoming else {
                        return Ok(());
                    };
                    let service = service.clone();
                    connections.spawn(async move {
                        let Ok(connection) = incoming.await else {
                            return;
                        };
                        let _ = serve_http3_connection(connection, service).await;
                    });
                }
            }
        }
    }

    pub fn local_address(&self) -> Result<SocketAddr, String> {
        self.endpoint
            .local_addr()
            .map_err(|error| format!("Gateway HTTP/3 listener address is unavailable: {error}"))
    }
}

async fn serve_http3_connection<S>(connection: quinn::Connection, service: S) -> Result<(), String>
where
    S: Service<Request<Full<Bytes>>, Response = GatewayHttpResponse> + Clone + Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    S::Future: Send + 'static,
{
    let mut connection = Connection::new(h3_quinn::Connection::new(connection))
        .await
        .map_err(|error| format!("Gateway HTTP/3 setup failed: {error}"))?;
    let mut requests = JoinSet::new();
    while let Some(resolver) = connection
        .accept()
        .await
        .map_err(|error| format!("Gateway HTTP/3 accept failed: {error}"))?
    {
        let service = service.clone();
        requests.spawn(async move {
            let Ok((request, stream)) = resolver.resolve_request().await else {
                return;
            };
            let _ = serve_http3_request(request, stream, service).await;
        });
    }
    while requests.join_next().await.is_some() {}
    Ok(())
}

async fn serve_http3_request<S>(
    request: Request<()>,
    mut stream: h3::server::RequestStream<h3_quinn::BidiStream<Bytes>, Bytes>,
    service: S,
) -> Result<(), String>
where
    S: Service<Request<Full<Bytes>>, Response = GatewayHttpResponse>,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    let mut body = BytesMut::new();
    while let Some(mut data) = stream
        .recv_data()
        .await
        .map_err(|error| format!("Gateway HTTP/3 request body failed: {error}"))?
    {
        let remaining = data.remaining();
        if body.len().saturating_add(remaining) > MAX_HTTP3_REQUEST_BODY_BYTES {
            return write_http3_response(
                &mut stream,
                Response::builder()
                    .status(StatusCode::PAYLOAD_TOO_LARGE)
                    .body(full_gateway_body(Bytes::new()))
                    .expect("bounded HTTP/3 response"),
            )
            .await;
        }
        body.extend_from_slice(&data.copy_to_bytes(remaining));
    }
    let (parts, ()) = request.into_parts();
    let request = Request::from_parts(parts, Full::new(body.freeze()));
    let response = service
        .call(request)
        .await
        .map_err(|error| format!("Gateway HTTP/3 route failed: {}", error.into()))?;
    write_http3_response(&mut stream, response).await
}

async fn write_http3_response(
    stream: &mut h3::server::RequestStream<h3_quinn::BidiStream<Bytes>, Bytes>,
    response: GatewayHttpResponse,
) -> Result<(), String> {
    let (parts, mut body) = response.into_parts();
    stream
        .send_response(Response::from_parts(parts, ()))
        .await
        .map_err(|error| format!("Gateway HTTP/3 response headers failed: {error}"))?;
    while let Some(frame) = body.frame().await {
        let frame = match frame {
            Ok(frame) => frame,
            Err(error) => match error {},
        };
        if let Ok(data) = frame.into_data() {
            stream
                .send_data(data)
                .await
                .map_err(|error| format!("Gateway HTTP/3 response body failed: {error}"))?;
        }
    }
    stream
        .finish()
        .await
        .map_err(|error| format!("Gateway HTTP/3 response finish failed: {error}"))
}
