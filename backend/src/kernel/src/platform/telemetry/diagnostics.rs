//! Owner-authorized, aggregate-only diagnostics relay for Telemetry Collector.

use crate::platform::telemetry::binding::TELEMETRY_PROCESS_ID;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

const REQUEST: &[u8] = b"hermes.telemetry.diagnostics.v1";
const HEADER: &str = "hermes.telemetry.diagnostics.v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TelemetryDiagnostics {
    segment_count: u32,
    total_bytes: u64,
}

impl TelemetryDiagnostics {
    #[must_use]
    pub const fn segment_count(self) -> u32 {
        self.segment_count
    }

    #[must_use]
    pub const fn total_bytes(self) -> u64 {
        self.total_bytes
    }
}

pub fn read(supervisor: &ManagedRuntimeSupervisor) -> Result<TelemetryDiagnostics, String> {
    let response = supervisor.relay(TELEMETRY_PROCESS_ID, REQUEST.to_vec())?;
    parse(&response)
}

pub(crate) fn parse(response: &[u8]) -> Result<TelemetryDiagnostics, String> {
    let text = std::str::from_utf8(response)
        .map_err(|_| "Telemetry diagnostics response is invalid".to_owned())?;
    let fields = text.split('|').collect::<Vec<_>>();
    let [header, segment_count, total_bytes] = fields.as_slice() else {
        return Err("Telemetry diagnostics response is invalid".to_owned());
    };
    if *header != HEADER {
        return Err("Telemetry diagnostics response is invalid".to_owned());
    }
    Ok(TelemetryDiagnostics {
        segment_count: segment_count
            .parse()
            .map_err(|_| "Telemetry diagnostics response is invalid".to_owned())?,
        total_bytes: total_bytes
            .parse()
            .map_err(|_| "Telemetry diagnostics response is invalid".to_owned())?,
    })
}
