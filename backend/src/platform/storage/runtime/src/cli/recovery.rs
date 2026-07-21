//! Parses the fixed offline PostgreSQL recovery contract.

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PostgresConnectionArguments {
    pub(crate) host: String,
    pub(crate) port: String,
    pub(crate) database: String,
    pub(crate) username: String,
    pub(crate) ssl_mode: String,
    pub(crate) password_file: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExportPostgresBackupArguments {
    pub(crate) pg_dump: PathBuf,
    pub(crate) connection: PostgresConnectionArguments,
    pub(crate) output: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RestorePostgresBackupArguments {
    pub(crate) pg_restore: PathBuf,
    pub(crate) psql: PathBuf,
    pub(crate) connection: PostgresConnectionArguments,
    pub(crate) input: PathBuf,
}

pub(crate) enum OfflineRecoveryCommand {
    Export(ExportPostgresBackupArguments),
    Restore(RestorePostgresBackupArguments),
}

pub(crate) fn parse_offline_recovery_command<I>(
    command: &OsStr,
    arguments: &mut std::iter::Peekable<I>,
) -> Result<OfflineRecoveryCommand, String>
where
    I: Iterator<Item = OsString>,
{
    let command = match command.to_str() {
        Some("export-backup") => OfflineRecoveryCommand::Export(parse_export(arguments)?),
        Some("restore-backup") => OfflineRecoveryCommand::Restore(parse_restore(arguments)?),
        _ => return Err(invalid_arguments()),
    };
    if arguments.next().is_some() {
        return Err(invalid_arguments());
    }
    Ok(command)
}

fn parse_export<I>(
    arguments: &mut std::iter::Peekable<I>,
) -> Result<ExportPostgresBackupArguments, String>
where
    I: Iterator<Item = OsString>,
{
    Ok(ExportPostgresBackupArguments {
        pg_dump: required_absolute_path(arguments, "--pg-dump")?,
        connection: parse_connection(arguments)?,
        output: required_absolute_path(arguments, "--output")?,
    })
}

fn parse_restore<I>(
    arguments: &mut std::iter::Peekable<I>,
) -> Result<RestorePostgresBackupArguments, String>
where
    I: Iterator<Item = OsString>,
{
    Ok(RestorePostgresBackupArguments {
        pg_restore: required_absolute_path(arguments, "--pg-restore")?,
        psql: required_absolute_path(arguments, "--psql")?,
        connection: parse_connection(arguments)?,
        input: required_absolute_path(arguments, "--input")?,
    })
}

fn parse_connection<I>(
    arguments: &mut std::iter::Peekable<I>,
) -> Result<PostgresConnectionArguments, String>
where
    I: Iterator<Item = OsString>,
{
    let host = required_token(arguments, "--host", 253)?;
    let port = required_port(arguments)?;
    let database = required_token(arguments, "--database", 127)?;
    let username = required_token(arguments, "--username", 127)?;
    let ssl_mode = required_ssl_mode(arguments)?;
    let password_file = required_absolute_path(arguments, "--password-file")?;
    Ok(PostgresConnectionArguments {
        host,
        port,
        database,
        username,
        ssl_mode,
        password_file,
    })
}

fn required_absolute_path<I>(
    arguments: &mut std::iter::Peekable<I>,
    name: &str,
) -> Result<PathBuf, String>
where
    I: Iterator<Item = OsString>,
{
    let value = required_value(arguments, name)?;
    let path = PathBuf::from(value);
    path.is_absolute()
        .then_some(path)
        .ok_or_else(invalid_arguments)
}

fn required_token<I>(
    arguments: &mut std::iter::Peekable<I>,
    name: &str,
    maximum_length: usize,
) -> Result<String, String>
where
    I: Iterator<Item = OsString>,
{
    let value = required_value(arguments, name)?;
    let value = value.into_string().map_err(|_| invalid_arguments())?;
    let valid = !value.is_empty()
        && value.len() <= maximum_length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-'));
    valid.then_some(value).ok_or_else(invalid_arguments)
}

fn required_port<I>(arguments: &mut std::iter::Peekable<I>) -> Result<String, String>
where
    I: Iterator<Item = OsString>,
{
    let value = required_value(arguments, "--port")?
        .into_string()
        .map_err(|_| invalid_arguments())?;
    value
        .parse::<u16>()
        .ok()
        .filter(|port| *port > 0)
        .map(|_| value)
        .ok_or_else(invalid_arguments)
}

fn required_ssl_mode<I>(arguments: &mut std::iter::Peekable<I>) -> Result<String, String>
where
    I: Iterator<Item = OsString>,
{
    let value = required_value(arguments, "--ssl-mode")?
        .into_string()
        .map_err(|_| invalid_arguments())?;
    matches!(
        value.as_str(),
        "disable" | "allow" | "prefer" | "require" | "verify-ca" | "verify-full"
    )
    .then_some(value)
    .ok_or_else(invalid_arguments)
}

fn required_value<I>(arguments: &mut std::iter::Peekable<I>, name: &str) -> Result<OsString, String>
where
    I: Iterator<Item = OsString>,
{
    if arguments.next().as_deref() != Some(OsStr::new(name)) {
        return Err(invalid_arguments());
    }
    arguments.next().ok_or_else(invalid_arguments)
}

fn invalid_arguments() -> String {
    "Storage offline recovery arguments are invalid".to_owned()
}
