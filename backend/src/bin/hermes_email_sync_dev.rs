use std::env;
use std::path::PathBuf;

use hermes_hub_backend::communications::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount,
};
use hermes_hub_backend::config::AppConfig;
use hermes_hub_backend::email_provider_network::{ImapFetchOptions, ImapNetworkClient};
use hermes_hub_backend::email_sync::imap_mailbox_stream_id;
use hermes_hub_backend::email_sync_pipeline::{
    EmailSyncPipelineReport, project_email_sync_batch_with_mail_blobs,
};
use hermes_hub_backend::mail_storage::LocalMailBlobStore;
use hermes_hub_backend::secrets::ResolvedSecret;
use hermes_hub_backend::storage::Database;
use serde::Serialize;
use serde_json::json;
use thiserror::Error;

const DEFAULT_ICLOUD_IMAP_HOST: &str = "imap.mail.me.com";
const DEFAULT_IMAP_PORT: u16 = 993;
const DEFAULT_MAILBOX: &str = "INBOX";
const DEFAULT_MAX_MESSAGES: usize = 25;
const DEFAULT_BLOB_ROOT: &str = "docker/data/mail";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    hermes_hub_backend::init_tracing();

    let config = DevEmailSyncConfig::from_env()?;
    let app_config = AppConfig::from_env()?;
    let database_url = app_config
        .database_url()
        .ok_or(DevEmailSyncConfigError::MissingDatabaseUrl)?;
    let database = Database::connect(Some(database_url)).await?;
    let pool = database
        .pool()
        .ok_or(DevEmailSyncConfigError::MissingDatabaseUrl)?
        .clone();

    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let account = NewProviderAccount::new(
        &config.account_id,
        config.provider_kind,
        &config.display_name,
        &config.external_account_id,
    )
    .config(json!({
        "host": config.host,
        "port": config.port,
        "tls": config.tls,
        "mailbox": config.mailbox
    }));
    communication_store
        .upsert_provider_account(&account)
        .await?;

    let stream_id = imap_mailbox_stream_id(&config.mailbox);
    let mut fetch_options = ImapFetchOptions::new(
        config.host,
        config.port,
        config.tls,
        config.mailbox.clone(),
        config.username,
    )
    .provider_kind(config.provider_kind)
    .max_messages(config.max_messages)
    .latest_messages();

    if let Some(checkpoint) = communication_store
        .checkpoint(&config.account_id, &stream_id)
        .await?
        .and_then(|checkpoint| checkpoint.checkpoint["last_seen_uid"].as_u64())
        .and_then(|last_seen_uid| u32::try_from(last_seen_uid).ok())
    {
        fetch_options = fetch_options.last_seen_uid(checkpoint);
    }

    let batch = ImapNetworkClient::new()
        .fetch_raw_messages(&ResolvedSecret::new(config.password)?, &fetch_options)
        .await?;
    let fetched_messages = batch.messages.len();
    let checkpoint = batch.checkpoint.clone();
    let blob_store = LocalMailBlobStore::new(&config.blob_root);
    let pipeline = project_email_sync_batch_with_mail_blobs(
        pool,
        &blob_store,
        &config.account_id,
        &config.import_batch_id,
        &batch,
    )
    .await?;

    println!(
        "{}",
        serde_json::to_string_pretty(&DevEmailSyncReport {
            account_id: &config.account_id,
            provider: config.provider_kind.as_str(),
            mailbox: &config.mailbox,
            fetched_messages,
            blob_root: config.blob_root.display().to_string(),
            checkpoint,
            pipeline,
        })?
    );

    Ok(())
}

struct DevEmailSyncConfig {
    account_id: String,
    display_name: String,
    external_account_id: String,
    provider_kind: EmailProviderKind,
    username: String,
    password: String,
    host: String,
    port: u16,
    tls: bool,
    mailbox: String,
    max_messages: usize,
    blob_root: PathBuf,
    import_batch_id: String,
}

impl DevEmailSyncConfig {
    fn from_env() -> Result<Self, DevEmailSyncConfigError> {
        let provider_kind = parse_provider_kind(
            optional_env("HERMES_EMAIL_SYNC_PROVIDER")
                .unwrap_or_else(|| "icloud".to_owned())
                .as_str(),
        )?;
        if provider_kind == EmailProviderKind::Gmail {
            return Err(DevEmailSyncConfigError::UnsupportedProviderForDevSync);
        }

        let username = first_env([
            "HERMES_EMAIL_SYNC_USERNAME",
            "HERMES_IMAP_FIXTURE_USERNAME",
            "ICLOUD_LOGIN",
        ])?;
        let external_account_id = optional_env("HERMES_EMAIL_SYNC_EXTERNAL_ACCOUNT_ID")
            .unwrap_or_else(|| username.clone());

        Ok(Self {
            account_id: optional_env("HERMES_EMAIL_SYNC_ACCOUNT_ID")
                .unwrap_or_else(|| format!("dev-{}-mail-cache", provider_kind.as_str())),
            display_name: optional_env("HERMES_EMAIL_SYNC_DISPLAY_NAME")
                .unwrap_or_else(|| "Dev Mail Cache".to_owned()),
            external_account_id,
            provider_kind,
            username,
            password: first_env([
                "HERMES_EMAIL_SYNC_PASSWORD",
                "HERMES_IMAP_FIXTURE_PASSWORD",
                "ICLOUD_2FA",
            ])?,
            host: optional_env("HERMES_EMAIL_SYNC_HOST")
                .unwrap_or_else(|| default_host(provider_kind).to_owned()),
            port: optional_env("HERMES_EMAIL_SYNC_PORT")
                .map(|value| parse_port("HERMES_EMAIL_SYNC_PORT", &value))
                .transpose()?
                .unwrap_or(DEFAULT_IMAP_PORT),
            tls: optional_env("HERMES_EMAIL_SYNC_TLS")
                .map(|value| parse_bool("HERMES_EMAIL_SYNC_TLS", &value))
                .transpose()?
                .unwrap_or(true),
            mailbox: optional_env("HERMES_EMAIL_SYNC_MAILBOX")
                .unwrap_or_else(|| DEFAULT_MAILBOX.to_owned()),
            max_messages: optional_env("HERMES_EMAIL_SYNC_MAX_MESSAGES")
                .map(|value| parse_usize("HERMES_EMAIL_SYNC_MAX_MESSAGES", &value))
                .transpose()?
                .unwrap_or(DEFAULT_MAX_MESSAGES),
            blob_root: PathBuf::from(
                optional_env("HERMES_EMAIL_SYNC_BLOB_ROOT")
                    .unwrap_or_else(|| DEFAULT_BLOB_ROOT.to_owned()),
            ),
            import_batch_id: optional_env("HERMES_EMAIL_SYNC_IMPORT_BATCH_ID")
                .unwrap_or_else(|| format!("email-sync-dev-{}", chrono::Utc::now().timestamp())),
        })
    }
}

#[derive(Serialize)]
struct DevEmailSyncReport<'a> {
    account_id: &'a str,
    provider: &'a str,
    mailbox: &'a str,
    fetched_messages: usize,
    blob_root: String,
    checkpoint: Option<serde_json::Value>,
    pipeline: EmailSyncPipelineReport,
}

fn parse_provider_kind(value: &str) -> Result<EmailProviderKind, DevEmailSyncConfigError> {
    EmailProviderKind::try_from(value.trim())
        .map_err(|_| DevEmailSyncConfigError::InvalidProviderKind(value.to_owned()))
}

fn default_host(provider_kind: EmailProviderKind) -> &'static str {
    match provider_kind {
        EmailProviderKind::Icloud => DEFAULT_ICLOUD_IMAP_HOST,
        EmailProviderKind::Imap => "localhost",
        EmailProviderKind::Gmail => "",
    }
}

fn first_env<const N: usize>(names: [&'static str; N]) -> Result<String, DevEmailSyncConfigError> {
    for name in names {
        if let Some(value) = optional_env(name) {
            return Ok(value);
        }
    }
    Err(DevEmailSyncConfigError::MissingEnv(names.join(" or ")))
}

fn optional_env(name: &'static str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn parse_port(name: &'static str, value: &str) -> Result<u16, DevEmailSyncConfigError> {
    let port = parse_u16(name, value)?;
    if port == 0 {
        return Err(DevEmailSyncConfigError::InvalidEnv {
            name,
            value: value.to_owned(),
            message: "must be greater than zero",
        });
    }
    Ok(port)
}

fn parse_bool(name: &'static str, value: &str) -> Result<bool, DevEmailSyncConfigError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" => Ok(true),
        "0" | "false" | "no" => Ok(false),
        _ => Err(DevEmailSyncConfigError::InvalidEnv {
            name,
            value: value.to_owned(),
            message: "expected one of true/false/yes/no/1/0",
        }),
    }
}

fn parse_usize(name: &'static str, value: &str) -> Result<usize, DevEmailSyncConfigError> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| DevEmailSyncConfigError::InvalidEnv {
            name,
            value: value.to_owned(),
            message: "expected positive integer",
        })?;
    if parsed == 0 {
        return Err(DevEmailSyncConfigError::InvalidEnv {
            name,
            value: value.to_owned(),
            message: "must be greater than zero",
        });
    }
    Ok(parsed)
}

fn parse_u16(name: &'static str, value: &str) -> Result<u16, DevEmailSyncConfigError> {
    value
        .parse::<u16>()
        .map_err(|_| DevEmailSyncConfigError::InvalidEnv {
            name,
            value: value.to_owned(),
            message: "expected u16 integer",
        })
}

#[derive(Debug, Error)]
enum DevEmailSyncConfigError {
    #[error("DATABASE_URL is required for email sync dev command")]
    MissingDatabaseUrl,

    #[error("missing required environment variable: {0}")]
    MissingEnv(String),

    #[error("invalid HERMES_EMAIL_SYNC_PROVIDER `{0}`; expected `icloud` or `imap`")]
    InvalidProviderKind(String),

    #[error("Gmail dev sync is not supported by this IMAP-only command")]
    UnsupportedProviderForDevSync,

    #[error("invalid {name} value `{value}`: {message}")]
    InvalidEnv {
        name: &'static str,
        value: String,
        message: &'static str,
    },
}
