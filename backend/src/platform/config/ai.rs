use super::errors::ConfigError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiRuntimeProvider {
    Ollama,
    OmniRoute,
}

impl AiRuntimeProvider {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ollama => "ollama",
            Self::OmniRoute => "omniroute",
        }
    }
}

impl TryFrom<&str> for AiRuntimeProvider {
    type Error = ConfigError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim().to_lowercase().as_str() {
            "ollama" => Ok(Self::Ollama),
            "omniroute" | "omni_route" | "omni-route" => Ok(Self::OmniRoute),
            _ => Err(ConfigError::InvalidAiProvider {
                value: value.to_owned(),
            }),
        }
    }
}
