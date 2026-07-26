//! Public WhatsApp operational replay contract. This is a distinct capability
//! from bounded operational reads.

use crate::{WhatsAppProviderEvent, provider_event_account_id, validate_event, validate_id};

pub const MAX_OPERATIONAL_REPLAY_LIMIT: u32 = 500;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatsAppOperationalReplayRequestV1 {
    pub account_id: String,
    pub after_sequence: u64,
    pub limit: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatsAppOperationalReplayFrameV1 {
    pub sequence: u64,
    pub event: WhatsAppProviderEvent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatsAppOperationalReplayResponseV1 {
    pub account_id: String,
    pub earliest_available_sequence: Option<u64>,
    pub latest_available_sequence: Option<u64>,
    pub frames: Vec<WhatsAppOperationalReplayFrameV1>,
    pub next_sequence: u64,
    pub reset_required: bool,
}

pub fn validate_operational_replay_request(
    request: &WhatsAppOperationalReplayRequestV1,
) -> Result<(), &'static str> {
    validate_id(&request.account_id).map_err(|_| "account_id")?;
    if request.limit == 0 || request.limit > MAX_OPERATIONAL_REPLAY_LIMIT {
        return Err("limit");
    }
    if request.after_sequence > i64::MAX as u64 {
        return Err("after_sequence");
    }
    Ok(())
}

pub fn validate_operational_replay_response(
    response: &WhatsAppOperationalReplayResponseV1,
) -> Result<(), &'static str> {
    validate_id(&response.account_id).map_err(|_| "account_id")?;
    match (
        response.earliest_available_sequence,
        response.latest_available_sequence,
    ) {
        (None, None) => {}
        (Some(earliest), Some(latest)) if earliest > 0 && earliest <= latest => {}
        _ => return Err("availability"),
    }
    if response.reset_required {
        if !response.frames.is_empty() || response.next_sequence != 0 {
            return Err("reset");
        }
        return Ok(());
    }
    if response.frames.len() > MAX_OPERATIONAL_REPLAY_LIMIT as usize {
        return Err("frames");
    }
    let mut previous = None;
    for frame in &response.frames {
        if frame.sequence == 0 || previous.is_some_and(|value| frame.sequence <= value) {
            return Err("sequence");
        }
        validate_event(&frame.event).map_err(|_| "event")?;
        if provider_event_account_id(&frame.event) != response.account_id {
            return Err("account_id");
        }
        previous = Some(frame.sequence);
    }
    if let Some(last) = response.frames.last() {
        if response.next_sequence != last.sequence {
            return Err("next_sequence");
        }
    } else if response.latest_available_sequence.is_none() && response.next_sequence != 0 {
        return Err("next_sequence");
    }
    if let Some(latest) = response.latest_available_sequence
        && (response.next_sequence > latest
            || response
                .frames
                .last()
                .is_some_and(|frame| frame.sequence > latest))
    {
        return Err("next_sequence");
    }
    if let Some(earliest) = response.earliest_available_sequence
        && response
            .frames
            .first()
            .is_some_and(|frame| frame.sequence < earliest)
    {
        return Err("sequence");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_request_is_bounded() {
        let mut request = WhatsAppOperationalReplayRequestV1 {
            account_id: "wa-1".into(),
            after_sequence: 0,
            limit: MAX_OPERATIONAL_REPLAY_LIMIT,
        };
        assert_eq!(validate_operational_replay_request(&request), Ok(()));
        request.limit += 1;
        assert_eq!(validate_operational_replay_request(&request), Err("limit"));
    }

    #[test]
    fn reset_response_cannot_smuggle_frames_or_a_cursor() {
        let response = WhatsAppOperationalReplayResponseV1 {
            account_id: "wa-1".into(),
            earliest_available_sequence: Some(10),
            latest_available_sequence: Some(20),
            frames: Vec::new(),
            next_sequence: 10,
            reset_required: true,
        };
        assert_eq!(
            validate_operational_replay_response(&response),
            Err("reset")
        );
    }
}
