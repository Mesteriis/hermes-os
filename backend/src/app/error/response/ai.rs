mod control_center;
mod runtime;

use crate::ai::control_center::AiControlCenterError;
use crate::ai::core::AiError;

use super::ErrorParts;

pub(super) fn ai_run_not_found_parts() -> ErrorParts {
    runtime::ai_run_not_found_parts()
}

pub(super) fn ai_error_parts(error: AiError) -> ErrorParts {
    runtime::ai_error_parts(error)
}

pub(super) fn control_center_error_parts(error: AiControlCenterError) -> ErrorParts {
    control_center::control_center_error_parts(error)
}
