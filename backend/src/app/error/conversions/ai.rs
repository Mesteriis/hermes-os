use super::super::types::ApiError;
use crate::ai::control_center::AiControlCenterError;
use crate::ai::core::AiError;
use crate::ai::hub::AiHubError;
use crate::platform::ai_runtime::AiRuntimePortError;

impl From<AiError> for ApiError {
    fn from(error: AiError) -> Self {
        match error {
            AiError::RunNotFound => Self::AiRunNotFound,
            _ => Self::Ai(error),
        }
    }
}

impl From<AiControlCenterError> for ApiError {
    fn from(error: AiControlCenterError) -> Self {
        Self::AiControlCenter(error)
    }
}

impl From<crate::integrations::ollama::client::error::OllamaError> for ApiError {
    fn from(error: crate::integrations::ollama::client::error::OllamaError) -> Self {
        Self::Ai(AiError::Runtime(AiHubError::Runtime(
            AiRuntimePortError::provider("ollama", error.to_string()),
        )))
    }
}

impl From<crate::integrations::omniroute::client::error::OmniRouteError> for ApiError {
    fn from(error: crate::integrations::omniroute::client::error::OmniRouteError) -> Self {
        Self::Ai(AiError::Runtime(AiHubError::Runtime(
            AiRuntimePortError::provider("omniroute", error.to_string()),
        )))
    }
}

impl From<crate::integrations::ai_runtime::AiRuntimeError> for ApiError {
    fn from(error: crate::integrations::ai_runtime::AiRuntimeError) -> Self {
        let port_error: AiRuntimePortError = error.into();
        Self::Ai(AiError::Runtime(AiHubError::Runtime(port_error)))
    }
}
