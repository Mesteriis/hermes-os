use super::errors::ConfigError;

pub(crate) fn required_trimmed(
    field: &'static str,
    value: Option<String>,
) -> Result<String, ConfigError> {
    let Some(value) = value else {
        return Err(ConfigError::InvalidGoogleOAuthClientConfig {
            field,
            message: "must be present",
        });
    };
    let value = value.trim();
    if value.is_empty() {
        return Err(ConfigError::InvalidGoogleOAuthClientConfig {
            field,
            message: "must not be empty",
        });
    }
    Ok(value.to_owned())
}

pub(crate) fn parse_bool_env(name: &'static str, value: &str) -> Result<bool, ConfigError> {
    match value {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        other => Err(ConfigError::InvalidBoolEnv {
            name,
            value: other.to_owned(),
        }),
    }
}
