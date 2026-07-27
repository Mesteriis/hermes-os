//! Public Zulip-owned operational replay contract.

use crate::operational::{ZulipOperationalContractErrorV1, ZulipOperationalEventV1};

pub const MAX_OPERATIONAL_REPLAY_SIZE: u32 = 200;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipOperationalReplayRequestV1 {
    pub account_id: String,
    pub after_sequence: u64,
    pub limit: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipOperationalReplayFrameV1 {
    pub sequence: u64,
    pub event: ZulipOperationalEventV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipOperationalReplayResponseV1 {
    pub earliest_available_sequence: Option<u64>,
    pub latest_available_sequence: Option<u64>,
    pub frames: Vec<ZulipOperationalReplayFrameV1>,
    pub next_sequence: u64,
    pub reset_required: bool,
    pub account_id: String,
}

pub fn validate_operational_replay_request(
    request: &ZulipOperationalReplayRequestV1,
) -> Result<(), ZulipOperationalContractErrorV1> {
    if request.account_id.trim().is_empty()
        || request.account_id.len() > 512
        || request.account_id.contains(['\0', '\r', '\n'])
    {
        return Err(ZulipOperationalContractErrorV1::InvalidId);
    }
    if request.limit == 0 || request.limit > MAX_OPERATIONAL_REPLAY_SIZE {
        return Err(ZulipOperationalContractErrorV1::InvalidLimit);
    }
    Ok(())
}

pub fn validate_operational_replay_response(
    response: &ZulipOperationalReplayResponseV1,
) -> Result<(), ZulipOperationalContractErrorV1> {
    validate_operational_replay_request(&ZulipOperationalReplayRequestV1 {
        account_id: response.account_id.clone(),
        after_sequence: response.next_sequence,
        limit: 1,
    })?;
    if response.frames.len() > MAX_OPERATIONAL_REPLAY_SIZE as usize
        || response
            .frames
            .windows(2)
            .any(|frames| frames[0].sequence >= frames[1].sequence)
        || response
            .frames
            .iter()
            .any(|frame| frame.event.account_id != response.account_id)
        || response
            .frames
            .last()
            .is_some_and(|frame| frame.sequence != response.next_sequence)
        || (response.reset_required && !response.frames.is_empty())
    {
        return Err(ZulipOperationalContractErrorV1::InvalidCursor);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_request_is_bounded() {
        assert_eq!(
            validate_operational_replay_request(&ZulipOperationalReplayRequestV1 {
                account_id: "account".into(),
                after_sequence: 0,
                limit: MAX_OPERATIONAL_REPLAY_SIZE,
            }),
            Ok(())
        );
        assert_eq!(
            validate_operational_replay_request(&ZulipOperationalReplayRequestV1 {
                account_id: "account".into(),
                after_sequence: 0,
                limit: MAX_OPERATIONAL_REPLAY_SIZE + 1,
            }),
            Err(ZulipOperationalContractErrorV1::InvalidLimit)
        );
    }
}
