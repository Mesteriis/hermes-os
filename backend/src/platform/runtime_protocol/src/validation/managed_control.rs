//! Structural validation for the correlated private managed-control transport.

use crate::v1::{ManagedRuntimeControlFrameV2, managed_runtime_control_frame_v2::Frame};

pub const MANAGED_CONTROL_TRANSPORT_MAJOR_V2: u32 = 2;
pub const MANAGED_CONTROL_CORRELATION_ID_BYTES: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManagedControlFrameValidationErrorV2 {
    InvalidVersion,
    InvalidCorrelationId,
    MissingFrame,
    EmptyRequest,
    EmptyResponse,
}

pub fn validate_managed_control_frame_v2(
    frame: &ManagedRuntimeControlFrameV2,
) -> Result<(), ManagedControlFrameValidationErrorV2> {
    if frame.transport_major != MANAGED_CONTROL_TRANSPORT_MAJOR_V2 {
        return Err(ManagedControlFrameValidationErrorV2::InvalidVersion);
    }
    if frame.correlation_id.len() != MANAGED_CONTROL_CORRELATION_ID_BYTES
        || frame.correlation_id.iter().all(|byte| *byte == 0)
    {
        return Err(ManagedControlFrameValidationErrorV2::InvalidCorrelationId);
    }
    match &frame.frame {
        Some(Frame::Request(request)) if request.operation.is_some() => Ok(()),
        Some(Frame::Request(_)) => Err(ManagedControlFrameValidationErrorV2::EmptyRequest),
        Some(Frame::Response(response))
            if response.result.is_some() || !response.error_code.is_empty() =>
        {
            Ok(())
        }
        Some(Frame::Response(_)) => Err(ManagedControlFrameValidationErrorV2::EmptyResponse),
        None => Err(ManagedControlFrameValidationErrorV2::MissingFrame),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v1::{
        ManagedRuntimeControlRequestV1, ManagedRuntimeControlResponseV1,
        managed_runtime_control_request_v1::Operation,
        managed_runtime_control_response_v1::Result as ResponseResult,
    };

    #[test]
    fn accepts_a_non_zero_correlated_typed_request() {
        let frame = ManagedRuntimeControlFrameV2 {
            transport_major: MANAGED_CONTROL_TRANSPORT_MAJOR_V2,
            correlation_id: vec![1; MANAGED_CONTROL_CORRELATION_ID_BYTES],
            frame: Some(Frame::Request(ManagedRuntimeControlRequestV1 {
                operation: Some(Operation::Ready(Default::default())),
            })),
        };

        assert_eq!(validate_managed_control_frame_v2(&frame), Ok(()));
    }

    #[test]
    fn rejects_an_uncorrelated_or_empty_response() {
        let frame = ManagedRuntimeControlFrameV2 {
            transport_major: MANAGED_CONTROL_TRANSPORT_MAJOR_V2,
            correlation_id: vec![0; MANAGED_CONTROL_CORRELATION_ID_BYTES],
            frame: Some(Frame::Response(ManagedRuntimeControlResponseV1 {
                result: Some(ResponseResult::Describe(Default::default())),
                error_code: String::new(),
            })),
        };
        assert_eq!(
            validate_managed_control_frame_v2(&frame),
            Err(ManagedControlFrameValidationErrorV2::InvalidCorrelationId)
        );
    }
}
