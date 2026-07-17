//! Aggregate-only diagnostics served over an authenticated inherited control FD.

use std::os::unix::net::UnixStream;

use crate::storage::TelemetrySegmentStore;

use super::framing::{read_frame, write_frame};

const REQUEST: &[u8] = b"hermes.telemetry.diagnostics.v1";
const HEADER: &str = "hermes.telemetry.diagnostics.v1";

pub fn serve(mut stream: UnixStream, store: TelemetrySegmentStore) -> Result<(), String> {
    while let Ok(request) = read_frame(&mut stream) {
        let response = response_for(&request, &store)?;
        write_frame(&mut stream, response.as_bytes())?;
    }
    Ok(())
}

fn response_for(request: &[u8], store: &TelemetrySegmentStore) -> Result<String, String> {
    if request != REQUEST {
        return Ok(format!("{HEADER}|denied"));
    }
    let diagnostics = store.diagnostics()?;
    Ok(format!(
        "{HEADER}|{}|{}",
        diagnostics.segment_count(),
        diagnostics.total_bytes(),
    ))
}
