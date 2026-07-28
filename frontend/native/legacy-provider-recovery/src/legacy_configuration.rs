use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use zeroize::Zeroizing;

use crate::error::{LegacyProviderRecoveryErrorV1, LegacyProviderRecoveryResultV1};

const PROVIDER_KEYS: [&str; 3] = [
    "HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_PATH",
    "HERMES_TELEGRAM_API_HASH",
    "HERMES_TELEGRAM_API_ID",
];
const DATABASE_KEYS: [&str; 3] = [
    "HERMES_POSTGRES_DB",
    "HERMES_POSTGRES_PASSWORD",
    "HERMES_POSTGRES_USER",
];

pub(crate) struct LegacyProviderSourceConfigurationV1 {
    pub google_oauth_client_path: PathBuf,
    pub telegram_api_id: i64,
    pub telegram_api_hash: Zeroizing<String>,
}

#[cfg(feature = "prepare")]
pub(crate) struct LegacyDatabaseSourceConfigurationV1 {
    pub database: String,
    pub username: String,
    pub password: Zeroizing<String>,
}

pub(crate) fn parse_provider_configuration(
    bytes: &[u8],
) -> LegacyProviderRecoveryResultV1<LegacyProviderSourceConfigurationV1> {
    let assignments = parse_assignments(bytes, &PROVIDER_KEYS.into_iter().collect())?;
    let google_oauth_client_path = PathBuf::from(required(&assignments, PROVIDER_KEYS[0])?);
    let telegram_api_hash = Zeroizing::new(required(&assignments, PROVIDER_KEYS[1])?.to_owned());
    let telegram_api_id = required(&assignments, PROVIDER_KEYS[2])?
        .parse::<i64>()
        .map_err(|_| LegacyProviderRecoveryErrorV1::InvalidConfiguration)?;
    if !google_oauth_client_path.is_absolute()
        || telegram_api_id <= 0
        || telegram_api_hash.is_empty()
        || telegram_api_hash.len() > 4096
        || !telegram_api_hash.is_ascii()
    {
        return Err(LegacyProviderRecoveryErrorV1::InvalidConfiguration);
    }
    Ok(LegacyProviderSourceConfigurationV1 {
        google_oauth_client_path,
        telegram_api_id,
        telegram_api_hash,
    })
}

#[cfg(feature = "prepare")]
pub(crate) fn parse_database_configuration(
    bytes: &[u8],
) -> LegacyProviderRecoveryResultV1<LegacyDatabaseSourceConfigurationV1> {
    let allowed = [
        "HERMES_BACKEND_BIND",
        "HERMES_BACKEND_PORT",
        "HERMES_DEV_KEY_PATH",
        "HERMES_DEV_MODE",
        "HERMES_FRONTEND_BIND",
        "HERMES_FRONTEND_PORT",
        "HERMES_HOST_VAULT_HOME",
        "HERMES_LOCAL_API_SECRET",
        "HERMES_OLLAMA_BASE_URL",
        "HERMES_OLLAMA_CHAT_MODEL",
        "HERMES_OLLAMA_EMBED_MODEL",
        "HERMES_OLLAMA_TIMEOUT_SECONDS",
        "HERMES_POSTGRES_BIND",
        "HERMES_POSTGRES_DB",
        "HERMES_POSTGRES_PASSWORD",
        "HERMES_POSTGRES_PORT",
        "HERMES_POSTGRES_USER",
        "HERMES_SECRET_VAULT_KEY",
        "HERMES_VAULT_HOME",
    ]
    .into_iter()
    .collect();
    let assignments = parse_assignments(bytes, &allowed)?;
    Ok(LegacyDatabaseSourceConfigurationV1 {
        database: required(&assignments, DATABASE_KEYS[0])?.to_owned(),
        password: Zeroizing::new(required(&assignments, DATABASE_KEYS[1])?.to_owned()),
        username: required(&assignments, DATABASE_KEYS[2])?.to_owned(),
    })
}

fn parse_assignments(
    bytes: &[u8],
    allowed: &BTreeSet<&str>,
) -> LegacyProviderRecoveryResultV1<BTreeMap<String, String>> {
    let input = std::str::from_utf8(bytes)
        .map_err(|_| LegacyProviderRecoveryErrorV1::InvalidConfiguration)?;
    if input.len() > 128 * 1024 {
        return Err(LegacyProviderRecoveryErrorV1::InvalidConfiguration);
    }
    let mut assignments = BTreeMap::new();
    for raw_line in input.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let assignment = line.strip_prefix("export ").unwrap_or(line);
        let (name, value) = assignment
            .split_once('=')
            .ok_or(LegacyProviderRecoveryErrorV1::InvalidConfiguration)?;
        let name = name.trim();
        if !allowed.contains(name) || assignments.contains_key(name) {
            return Err(LegacyProviderRecoveryErrorV1::InvalidConfiguration);
        }
        let value = literal_value(value.trim())?;
        if value.is_empty() || value.len() > 16 * 1024 {
            return Err(LegacyProviderRecoveryErrorV1::InvalidConfiguration);
        }
        assignments.insert(name.to_owned(), value);
    }
    Ok(assignments)
}

fn literal_value(value: &str) -> LegacyProviderRecoveryResultV1<String> {
    let unquoted = if let Some(value) = value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
    {
        value
    } else if let Some(value) = value
        .strip_prefix('\'')
        .and_then(|value| value.strip_suffix('\''))
    {
        value
    } else {
        value
    };
    if unquoted.chars().any(char::is_control) {
        return Err(LegacyProviderRecoveryErrorV1::InvalidConfiguration);
    }
    Ok(unquoted.to_owned())
}

fn required<'a>(
    assignments: &'a BTreeMap<String, String>,
    key: &str,
) -> LegacyProviderRecoveryResultV1<&'a str> {
    assignments
        .get(key)
        .map(String::as_str)
        .ok_or(LegacyProviderRecoveryErrorV1::InvalidConfiguration)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_parser_accepts_only_exact_literal_keys() {
        let parsed = parse_provider_configuration(
            br#"
            export HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_PATH="/private/oauth.json"
            export HERMES_TELEGRAM_API_HASH='private-hash'
            export HERMES_TELEGRAM_API_ID=12345
            "#,
        )
        .expect("parse exact provider configuration");
        assert_eq!(parsed.telegram_api_id, 12345);
        assert_eq!(parsed.telegram_api_hash.as_str(), "private-hash");
    }

    #[test]
    fn provider_parser_rejects_shell_evaluation_and_unknown_keys() {
        assert_eq!(
            parse_provider_configuration(
                b"HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_PATH=$(echo bad)\nHERMES_TELEGRAM_API_HASH=x\nHERMES_TELEGRAM_API_ID=1",
            )
            .err(),
            Some(LegacyProviderRecoveryErrorV1::InvalidConfiguration),
        );
        assert_eq!(
            parse_provider_configuration(
                b"HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_PATH=/private/oauth.json\nHERMES_TELEGRAM_API_HASH=x\nHERMES_TELEGRAM_API_ID=1\nUNKNOWN=value",
            )
            .err(),
            Some(LegacyProviderRecoveryErrorV1::InvalidConfiguration),
        );
    }
}
