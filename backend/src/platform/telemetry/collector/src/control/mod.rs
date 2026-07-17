//! Private authenticated control channel for the Telemetry Collector.

mod diagnostics;
mod framing;
mod handshake;

pub use diagnostics::serve as serve_diagnostics;
pub use handshake::describe;
