//! Structural validation for aggregate-only Telemetry Collector control frames.

use crate::v1::{
    TelemetryRuntimeControlRequestV1, TelemetryRuntimeControlResponseV1,
    telemetry_runtime_control_request_v1::Operation as RequestOperation,
    telemetry_runtime_control_response_v1::Result as ResponseResult,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelemetryRuntimeControlErrorV1 {
    MissingOperation,
    InvalidResponse,
}

pub fn validate_telemetry_runtime_control_request(
    request: &TelemetryRuntimeControlRequestV1,
) -> Result<(), TelemetryRuntimeControlErrorV1> {
    match request.operation {
        Some(RequestOperation::GetDiagnostics(_)) => Ok(()),
        None => Err(TelemetryRuntimeControlErrorV1::MissingOperation),
    }
}

pub fn validate_telemetry_runtime_control_response(
    response: &TelemetryRuntimeControlResponseV1,
) -> Result<(), TelemetryRuntimeControlErrorV1> {
    match (&response.result, response.error_code.is_empty()) {
        (Some(ResponseResult::Diagnostics(_)), true) => Ok(()),
        (None, false) if valid_error_code(&response.error_code) => Ok(()),
        _ => Err(TelemetryRuntimeControlErrorV1::InvalidResponse),
    }
}

fn valid_error_code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}
