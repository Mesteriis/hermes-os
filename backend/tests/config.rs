use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;

use hermes_hub_backend::config::{AppConfig, ConfigError};

#[test]
fn default_config_binds_to_localhost_without_database_url() {
    let config = AppConfig::default();

    assert_eq!(
        config.http_addr(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080)
    );
    assert_eq!(config.service_name(), "hermes-hub-backend");
    assert_eq!(config.database_url(), None);
    assert_eq!(config.local_api_token(), None);
    assert_eq!(config.secret_vault_path(), None);
    assert_eq!(config.secret_vault_key(), None);
}

#[test]
fn config_from_pairs_overrides_http_addr_database_url_and_local_api_token() {
    let config = AppConfig::from_pairs([
        ("HERMES_HTTP_ADDR", "127.0.0.1:9090"),
        (
            "DATABASE_URL",
            "postgres://hermes:local-dev-password@postgres:5432/hermes_hub",
        ),
        ("HERMES_LOCAL_API_TOKEN", "local-dev-api-token"),
    ])
    .expect("valid config");

    assert_eq!(
        config.http_addr(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9090)
    );
    assert_eq!(
        config.database_url(),
        Some("postgres://hermes:local-dev-password@postgres:5432/hermes_hub")
    );
    assert_eq!(config.local_api_token(), Some("local-dev-api-token"));
}

#[test]
fn config_from_pairs_accepts_secret_vault_path_and_key() {
    let config = AppConfig::from_pairs([
        (
            "HERMES_SECRET_VAULT_PATH",
            "docker/data/secrets/hermes.vault.json",
        ),
        ("HERMES_SECRET_VAULT_KEY", "local-vault-key"),
    ])
    .expect("valid secret vault config");

    assert_eq!(
        config.secret_vault_path(),
        Some(Path::new("docker/data/secrets/hermes.vault.json"))
    );
    assert_eq!(
        config
            .secret_vault_key()
            .expect("vault key")
            .expose_for_runtime(),
        "local-vault-key"
    );
    assert_eq!(
        format!("{:?}", config.secret_vault_key().expect("vault key")),
        "ResolvedSecret { value: \"<redacted>\" }"
    );
}

#[test]
fn config_from_pairs_accepts_legacy_local_write_token_as_fallback() {
    let config = AppConfig::from_pairs([("HERMES_LOCAL_WRITE_TOKEN", "legacy-write-token")])
        .expect("valid legacy local write token");

    assert_eq!(config.local_api_token(), Some("legacy-write-token"));
}

#[test]
fn config_from_pairs_prefers_local_api_token_over_legacy_write_token() {
    let config = AppConfig::from_pairs([
        ("HERMES_LOCAL_WRITE_TOKEN", "legacy-write-token"),
        ("HERMES_LOCAL_API_TOKEN", "local-api-token"),
    ])
    .expect("valid local API token");

    assert_eq!(config.local_api_token(), Some("local-api-token"));
}

#[test]
fn config_from_pairs_rejects_invalid_http_addr() {
    let error = AppConfig::from_pairs([("HERMES_HTTP_ADDR", "not-a-socket")])
        .expect_err("invalid socket address must fail");

    assert!(matches!(error, ConfigError::InvalidHttpAddr { .. }));
}

#[test]
fn config_from_pairs_rejects_empty_database_url() {
    let error =
        AppConfig::from_pairs([("DATABASE_URL", "   ")]).expect_err("empty database URL must fail");

    assert!(matches!(error, ConfigError::EmptyDatabaseUrl));
}

#[test]
fn config_from_pairs_rejects_empty_local_write_token() {
    let error = AppConfig::from_pairs([("HERMES_LOCAL_WRITE_TOKEN", "   ")])
        .expect_err("empty local write token must fail");

    assert!(matches!(error, ConfigError::EmptyLocalWriteToken));
}

#[test]
fn config_from_pairs_rejects_empty_local_api_token() {
    let error = AppConfig::from_pairs([("HERMES_LOCAL_API_TOKEN", "   ")])
        .expect_err("empty local API token must fail");

    assert!(matches!(error, ConfigError::EmptyLocalApiToken));
}

#[test]
fn config_from_pairs_rejects_empty_secret_vault_path() {
    let error = AppConfig::from_pairs([("HERMES_SECRET_VAULT_PATH", "   ")])
        .expect_err("empty secret vault path must fail");

    assert!(matches!(error, ConfigError::EmptySecretVaultPath));
}

#[test]
fn config_from_pairs_rejects_empty_secret_vault_key() {
    let error = AppConfig::from_pairs([("HERMES_SECRET_VAULT_KEY", "   ")])
        .expect_err("empty secret vault key must fail");

    assert!(matches!(error, ConfigError::EmptySecretVaultKey));
}
