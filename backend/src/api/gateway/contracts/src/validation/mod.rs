//! Validation for client-safe realtime frames before they enter the Gateway.

use crate::v1::{
    ClientRealtimeFrameV1, ClientRealtimeStreamStateKindV1, client_realtime_frame_v1::Frame,
};

const MAX_CURSOR_BYTES: usize = 512;
const MAX_NAME_BYTES: usize = 256;
const MAX_ID_BYTES: usize = 128;
const MAX_PAYLOAD_BYTES: usize = 64 * 1024;

pub fn validate_client_realtime_frame(frame: &ClientRealtimeFrameV1) -> Result<(), String> {
    match frame.frame.as_ref() {
        Some(Frame::Event(event))
            if valid_bytes(&event.event_id, MAX_ID_BYTES)
                && valid_cursor(&event.cursor)
                && valid_text(&event.contract_name, MAX_NAME_BYTES)
                && event.contract_version > 0
                && valid_text(&event.event_kind, MAX_NAME_BYTES)
                && event.occurred_at_unix_millis > 0
                && optional_text(&event.causation_id)
                && optional_text(&event.correlation_id)
                && optional_text(&event.trace_id)
                && event.payload.len() <= MAX_PAYLOAD_BYTES =>
        {
            Ok(())
        }
        Some(Frame::ReplayGap(gap))
            if valid_text(&gap.reason_code, MAX_NAME_BYTES)
                && optional_cursor(&gap.requested_cursor)
                && optional_cursor(&gap.earliest_available_cursor) =>
        {
            Ok(())
        }
        Some(Frame::StreamState(state))
            if ClientRealtimeStreamStateKindV1::try_from(state.state)
                .ok()
                .is_some_and(|kind| {
                    kind
                        != ClientRealtimeStreamStateKindV1::ClientRealtimeStreamStateKindUnspecified
                })
                && optional_cursor(&state.cursor) =>
        {
            Ok(())
        }
        _ => Err("client realtime frame is invalid".to_owned()),
    }
}

fn valid_bytes(value: &[u8], maximum: usize) -> bool {
    !value.is_empty() && value.len() <= maximum
}

fn valid_text(value: &str, maximum: usize) -> bool {
    !value.is_empty() && value.len() <= maximum && value.is_char_boundary(value.len())
}

fn optional_text(value: &str) -> bool {
    value.len() <= MAX_ID_BYTES && value.is_char_boundary(value.len())
}

// Cursors are emitted as SSE `id` fields, so they must be safe as a single
// protocol line as well as bounded. This prevents a source from injecting a
// second SSE field.
fn valid_cursor(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_CURSOR_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_graphic() && byte != b'\\' && byte != b'\"')
}

fn optional_cursor(value: &str) -> bool {
    value.is_empty() || valid_cursor(value)
}
