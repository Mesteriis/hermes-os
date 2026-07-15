use crate::application::communication_fixture_error::CommunicationFixtureIngestError;
use crate::platform::calls::models::{CallDirection, CallState};

pub(crate) fn normalized_media_storage_kind(value: &str) -> String {
    match value.trim() {
        "local_blob" => "local_fs".to_owned(),
        other => other.to_owned(),
    }
}
pub(crate) fn normalized_media_sha256(value: &str) -> String {
    let value = value.trim();
    if value.starts_with("sha256:") {
        value.to_owned()
    } else {
        format!("sha256:{value}")
    }
}
pub(crate) fn call_direction(
    value: &str,
) -> Result<CallDirection, CommunicationFixtureIngestError> {
    match value.trim() {
        "incoming" => Ok(CallDirection::Incoming),
        "outgoing" => Ok(CallDirection::Outgoing),
        other => Err(CommunicationFixtureIngestError::SignalControlBlocked(
            format!("unsupported whatsapp call direction `{other}`"),
        )),
    }
}
pub(crate) fn call_state(value: &str) -> Result<CallState, CommunicationFixtureIngestError> {
    match value.trim() {
        "ringing" => Ok(CallState::Ringing),
        "active" => Ok(CallState::Active),
        "ended" => Ok(CallState::Ended),
        "missed" => Ok(CallState::Missed),
        "declined" => Ok(CallState::Declined),
        "failed" => Ok(CallState::Failed),
        other => Err(CommunicationFixtureIngestError::SignalControlBlocked(
            format!("unsupported whatsapp call state `{other}`"),
        )),
    }
}
