//! WhatsApp process loop for the admitted host-owned WebView bridge.

use std::os::unix::net::UnixStream;

use crate::{WhatsAppProviderTransport, WhatsAppRuntime, client_transport};
use hermes_whatsapp_persistence::WhatsAppDurablePersistence;

pub struct WhatsAppProcessLoop<T> {
    runtime: WhatsAppRuntime<T>,
}

impl<T: WhatsAppProviderTransport> WhatsAppProcessLoop<T> {
    #[must_use]
    pub fn new(runtime: WhatsAppRuntime<T>) -> Self {
        Self { runtime }
    }

    pub fn serve_client_connection(
        &mut self,
        stream: UnixStream,
    ) -> Result<(), client_transport::WhatsAppClientTransportError> {
        client_transport::serve_connection(stream, &mut self.runtime)
    }

    pub fn serve_client_connection_durable(
        &mut self,
        stream: UnixStream,
        durable: &WhatsAppDurablePersistence,
        handle: &tokio::runtime::Handle,
    ) -> Result<(), client_transport::WhatsAppClientTransportError> {
        client_transport::serve_connection_durable(stream, &mut self.runtime, durable, handle)
    }
}
