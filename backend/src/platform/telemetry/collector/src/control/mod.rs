//! Private authenticated control channel for the Telemetry Collector.

mod diagnostics;
mod framing;
mod handshake;

pub use diagnostics::serve as serve_diagnostics;
#[allow(unused_imports)] // Re-exported for the Collector composition harness.
pub use handshake::describe;
