use super::super::types::ApiError;
use crate::ai::control_center::AiControlCenterError;
use crate::ai::core::AiError;

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

impl From<crate::integrations::ollama::client::OllamaError> for ApiError {
    fn from(error: crate::integrations::ollama::client::OllamaError) -> Self {
        Self::Ai(AiError::Runtime(
            crate::integrations::ai_runtime::AiRuntimeError::Ollama(error),
        ))
    }
}

impl From<crate::integrations::omniroute::client::OmniRouteError> for ApiError {
    fn from(error: crate::integrations::omniroute::client::OmniRouteError) -> Self {
        Self::Ai(AiError::Runtime(
            crate::integrations::ai_runtime::AiRuntimeError::OmniRoute(error),
        ))
    }
}

impl From<crate::integrations::ai_runtime::AiRuntimeError> for ApiError {
    fn from(error: crate::integrations::ai_runtime::AiRuntimeError) -> Self {
        Self::Ai(AiError::Runtime(error))
    }
}
