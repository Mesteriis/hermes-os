//! Core Gateway HTTP transport foundation.
//!
//! This package owns only transport-profile validation and HTTP/2 serving over
//! an already-authenticated TLS stream. Route authorization and client
//! contracts remain separate Gateway concerns.

mod application;
mod browser;
mod realtime;
mod routes;
mod transport;

pub use application::GatewayApplicationRouter;
pub use browser::{
    BrowserAuthenticationRouter, BrowserBootstrapRouter, BrowserPairingRouter, BrowserSessionStatusRouter,
    ClientBootstrapRouter, ClientRpcRouteErrorV1, ClientRpcRouteHandler, ClientRpcRouteV1,
    ClientRpcRouter, SharedBrowserGatewaySessionService, SharedBrowserPairingManager,
};
pub use realtime::{
    BrowserRealtimePublisherV1, BrowserRealtimeRouter, BrowserRealtimeSubscriptionSource,
    ClientRealtimeSubscriptionV1, InMemoryBrowserRealtimeSource,
};
pub use routes::GatewayTechnicalRouter;
pub use transport::{
    GatewayHttp3ListenerV1, GatewayHttpResponse, GatewayLanDevelopmentListenerV1,
    GatewayLoopbackListenerV1, GatewayLoopbackTlsListenerV1, GatewayTlsListenerV1,
    GatewayTransportProfileV1, PairedRemoteProfileV1, full_gateway_body,
    serve_local_embedded_http1, serve_paired_remote_http2,
};
