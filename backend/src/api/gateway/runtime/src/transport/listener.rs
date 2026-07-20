use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use hyper::Request;
use hyper::body::Incoming;
use hyper::service::Service;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Semaphore, watch};
use tokio::task::JoinSet;
use tokio::time::timeout;
use tokio_rustls::{TlsAcceptor, server::TlsStream};

use crate::{
    GatewayHttpResponse, GatewayTransportProfileV1, PairedRemoteProfileV1,
    serve_local_embedded_http1, serve_paired_remote_http2,
};

/// A listener must never turn an unauthenticated connection storm into an
/// unbounded task or file-descriptor allocation. This cap is deliberately
/// transport-wide and owner-neutral; owner routes apply their own request
/// budgets after authentication.
const MAX_CONCURRENT_CONNECTIONS: usize = 128;
const TLS_HANDSHAKE_DEADLINE: Duration = Duration::from_secs(10);

fn connection_budget() -> Arc<Semaphore> {
    Arc::new(Semaphore::new(MAX_CONCURRENT_CONNECTIONS))
}

/// A local application listener. It can bind only loopback and deliberately
/// has no TLS or paired-remote admission path.
pub struct GatewayLoopbackListenerV1 {
    listener: TcpListener,
}

/// Plain HTTP listener reserved for the explicitly enabled private-LAN
/// developer profile. Production and paired-remote listeners never use it.
pub struct GatewayLanDevelopmentListenerV1 {
    listener: TcpListener,
}

impl GatewayLoopbackListenerV1 {
    /// Validates the local profile before binding so an address configuration
    /// error cannot turn this browser/desktop transport into a LAN listener.
    pub async fn bind(address: SocketAddr) -> Result<Self, String> {
        GatewayTransportProfileV1::LocalEmbedded
            .validate_bind(address.ip(), false)
            .map_err(str::to_owned)?;
        let listener = TcpListener::bind(address)
            .await
            .map_err(|error| format!("Gateway loopback listener bind failed: {error}"))?;
        Ok(Self { listener })
    }

    /// Serves one local HTTP/1.1 connection. Lifecycle ownership remains with
    /// the Kernel admission path; this foundation does not start a listener.
    pub async fn serve_once<S>(&self, service: S) -> Result<(), String>
    where
        S: Service<Request<Incoming>, Response = GatewayHttpResponse> + Send + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        S::Future: Send + 'static,
    {
        let (stream, _) = self
            .listener
            .accept()
            .await
            .map_err(|error| format!("Gateway loopback listener accept failed: {error}"))?;
        serve_local_embedded_http1(stream, service).await
    }

    /// Serves loopback connections until the owning process requests shutdown.
    /// Existing connections are aborted on shutdown because client sessions are
    /// Kernel-memory scoped and must reconnect after a Gateway restart.
    pub async fn serve_until_shutdown<S>(
        self,
        service: S,
        shutdown_requested: watch::Receiver<bool>,
    ) -> Result<(), String>
    where
        S: Service<Request<Incoming>, Response = GatewayHttpResponse> + Clone + Send + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        S::Future: Send + 'static,
    {
        serve_plain_until_shutdown(self.listener, service, shutdown_requested, "loopback").await
    }

    pub fn local_address(&self) -> Result<SocketAddr, String> {
        self.listener
            .local_addr()
            .map_err(|error| format!("Gateway loopback listener address is unavailable: {error}"))
    }
}

impl GatewayLanDevelopmentListenerV1 {
    pub async fn bind(address: SocketAddr) -> Result<Self, String> {
        let private = match address.ip() {
            std::net::IpAddr::V4(ip) => ip.is_private() || ip.is_link_local(),
            std::net::IpAddr::V6(ip) => {
                let first = ip.segments()[0];
                (first & 0xfe00) == 0xfc00 || (first & 0xffc0) == 0xfe80
            }
        };
        if !private || address.ip().is_unspecified() {
            return Err("Gateway developer listener must bind one private LAN address".to_owned());
        }
        let listener = TcpListener::bind(address)
            .await
            .map_err(|error| format!("Gateway developer listener bind failed: {error}"))?;
        Ok(Self { listener })
    }

    pub async fn serve_until_shutdown<S>(
        self,
        service: S,
        shutdown_requested: watch::Receiver<bool>,
    ) -> Result<(), String>
    where
        S: Service<Request<Incoming>, Response = GatewayHttpResponse> + Clone + Send + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        S::Future: Send + 'static,
    {
        serve_plain_until_shutdown(self.listener, service, shutdown_requested, "developer").await
    }

    pub fn local_address(&self) -> Result<SocketAddr, String> {
        self.listener
            .local_addr()
            .map_err(|error| format!("Gateway developer listener address is unavailable: {error}"))
    }
}

async fn serve_plain_until_shutdown<S>(
    listener: TcpListener,
    service: S,
    mut shutdown_requested: watch::Receiver<bool>,
    profile: &'static str,
) -> Result<(), String>
where
    S: Service<Request<Incoming>, Response = GatewayHttpResponse> + Clone + Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    S::Future: Send + 'static,
{
    if *shutdown_requested.borrow() {
        return Ok(());
    }
    let mut connections = JoinSet::new();
    let budget = connection_budget();
    loop {
        tokio::select! {
            completed = connections.join_next(), if !connections.is_empty() => {
                let _ = completed;
            }
            changed = shutdown_requested.changed() => {
                match changed {
                    Ok(()) if *shutdown_requested.borrow() => {
                        connections.abort_all();
                        while connections.join_next().await.is_some() {}
                        return Ok(());
                    }
                    Ok(()) => {}
                    Err(_) => {
                        connections.abort_all();
                        while connections.join_next().await.is_some() {}
                        return Ok(());
                    }
                }
            }
            accepted = listener.accept() => {
                let (stream, _) = accepted
                    .map_err(|error| format!("Gateway {profile} listener accept failed: {error}"))?;
                let Ok(permit) = budget.clone().try_acquire_owned() else {
                    // Drop unauthenticated excess peers immediately. A future
                    // accepted peer can use the released slot.
                    continue;
                };
                let service = service.clone();
                connections.spawn(async move {
                    let _permit = permit;
                    let _ = serve_local_embedded_http1(stream, service).await;
                });
            }
        }
    }
}

/// A local browser listener. Unlike the plaintext embedded profile, it keeps
/// the loopback-only admission rule while completing TLS before any HTTP is
/// exposed. This is the only local profile suitable for an HTTPS WebAuthn
/// origin.
pub struct GatewayLoopbackTlsListenerV1 {
    listener: TcpListener,
    acceptor: TlsAcceptor,
}

impl GatewayLoopbackTlsListenerV1 {
    pub async fn bind(address: SocketAddr, acceptor: TlsAcceptor) -> Result<Self, String> {
        GatewayTransportProfileV1::LocalEmbedded
            .validate_bind(address.ip(), true)
            .map_err(str::to_owned)?;
        let listener = TcpListener::bind(address)
            .await
            .map_err(|error| format!("Gateway loopback TLS listener bind failed: {error}"))?;
        Ok(Self { listener, acceptor })
    }

    /// Serves local HTTPS connections until the Kernel-owned shutdown signal.
    /// TLS failure is isolated to its peer; it cannot end the listener.
    pub async fn serve_until_shutdown<S>(
        self,
        service: S,
        mut shutdown_requested: watch::Receiver<bool>,
    ) -> Result<(), String>
    where
        S: Service<Request<Incoming>, Response = GatewayHttpResponse> + Clone + Send + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        S::Future: Send + 'static,
    {
        if *shutdown_requested.borrow() {
            return Ok(());
        }
        let mut connections = JoinSet::new();
        let budget = connection_budget();
        loop {
            tokio::select! {
                completed = connections.join_next(), if !connections.is_empty() => {
                    let _ = completed;
                }
                changed = shutdown_requested.changed() => {
                    match changed {
                        Ok(()) if *shutdown_requested.borrow() => {
                            connections.abort_all();
                            while connections.join_next().await.is_some() {}
                            return Ok(());
                        }
                        Ok(()) => {}
                        Err(_) => {
                            connections.abort_all();
                            while connections.join_next().await.is_some() {}
                            return Ok(());
                        }
                    }
                }
                accepted = self.listener.accept() => {
                    let (stream, _) = accepted
                        .map_err(|error| format!("Gateway loopback TLS listener accept failed: {error}"))?;
                    let Ok(permit) = budget.clone().try_acquire_owned() else {
                        continue;
                    };
                    let acceptor = self.acceptor.clone();
                    let service = service.clone();
                    connections.spawn(async move {
                        let _permit = permit;
                        if let Ok(Ok(stream)) = timeout(TLS_HANDSHAKE_DEADLINE, acceptor.accept(stream)).await {
                            let _ = serve_local_embedded_http1(stream, service).await;
                        }
                    });
                }
            }
        }
    }

    pub fn local_address(&self) -> Result<SocketAddr, String> {
        self.listener.local_addr().map_err(|error| {
            format!("Gateway loopback TLS listener address is unavailable: {error}")
        })
    }
}

/// A paired-remote listener that cannot accept plaintext application traffic.
pub struct GatewayTlsListenerV1 {
    listener: TcpListener,
    acceptor: TlsAcceptor,
}

impl GatewayTlsListenerV1 {
    /// Validates the profile before binding and keeps the TLS acceptor adjacent
    /// to the socket, so a caller cannot accidentally expose a plaintext path.
    pub async fn bind(
        address: SocketAddr,
        profile: PairedRemoteProfileV1,
        acceptor: TlsAcceptor,
    ) -> Result<Self, String> {
        GatewayTransportProfileV1::PairedRemote(profile)
            .validate_bind(address.ip(), true)
            .map_err(str::to_owned)?;
        let listener = TcpListener::bind(address)
            .await
            .map_err(|error| format!("Gateway TLS listener bind failed: {error}"))?;
        Ok(Self { listener, acceptor })
    }

    /// Accepts exactly one TCP peer and completes TLS before exposing it to a
    /// protocol adapter.
    pub async fn accept(&self) -> Result<TlsStream<TcpStream>, String> {
        let (stream, _) = self
            .listener
            .accept()
            .await
            .map_err(|error| format!("Gateway TLS listener accept failed: {error}"))?;
        self.acceptor
            .accept(stream)
            .await
            .map_err(|error| format!("Gateway TLS handshake failed: {error}"))
    }

    /// Serves one authenticated peer over HTTP/2. The process/lifecycle owner
    /// remains responsible for deciding when and for how long to call this.
    pub async fn serve_once<S>(&self, service: S) -> Result<(), String>
    where
        S: Service<Request<Incoming>, Response = GatewayHttpResponse> + Send + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        S::Future: Send + 'static,
    {
        serve_paired_remote_http2(self.accept().await?, service).await
    }

    /// Serves paired TLS peers until the owning process requests shutdown.
    /// A bad TLS peer is isolated to its connection and cannot stop the
    /// listener. On shutdown, in-flight requests are aborted so clients must
    /// establish new sessions against the next Gateway run.
    pub async fn serve_until_shutdown<S>(
        self,
        service: S,
        mut shutdown_requested: watch::Receiver<bool>,
    ) -> Result<(), String>
    where
        S: Service<Request<Incoming>, Response = GatewayHttpResponse> + Clone + Send + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        S::Future: Send + 'static,
    {
        if *shutdown_requested.borrow() {
            return Ok(());
        }
        let mut connections = JoinSet::new();
        let budget = connection_budget();
        loop {
            tokio::select! {
                completed = connections.join_next(), if !connections.is_empty() => {
                    let _ = completed;
                }
                changed = shutdown_requested.changed() => {
                    match changed {
                        Ok(()) if *shutdown_requested.borrow() => {
                            connections.abort_all();
                            while connections.join_next().await.is_some() {}
                            return Ok(());
                        }
                        Ok(()) => {}
                        Err(_) => {
                            connections.abort_all();
                            while connections.join_next().await.is_some() {}
                            return Ok(());
                        }
                    }
                }
                accepted = self.listener.accept() => {
                    let (stream, _) = accepted
                        .map_err(|error| format!("Gateway TLS listener accept failed: {error}"))?;
                    let Ok(permit) = budget.clone().try_acquire_owned() else {
                        continue;
                    };
                    let acceptor = self.acceptor.clone();
                    let service = service.clone();
                    connections.spawn(async move {
                        let _permit = permit;
                        if let Ok(Ok(stream)) = timeout(TLS_HANDSHAKE_DEADLINE, acceptor.accept(stream)).await {
                            let _ = serve_paired_remote_http2(stream, service).await;
                        }
                    });
                }
            }
        }
    }

    pub fn local_address(&self) -> Result<SocketAddr, String> {
        self.listener
            .local_addr()
            .map_err(|error| format!("Gateway TLS listener address is unavailable: {error}"))
    }
}
