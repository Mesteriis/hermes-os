//! Fixed argument contract for Scheduler's stopped-instance recovery command.

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RecoveryArguments {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) database: String,
    pub(crate) username: String,
    pub(crate) ssl_mode: String,
    pub(crate) password_file: PathBuf,
    pub(crate) storage_bundle: PathBuf,
}

pub(crate) fn parse_recovery_arguments<I>(
    arguments: &mut std::iter::Peekable<I>,
) -> Result<RecoveryArguments, String>
where
    I: Iterator<Item = OsString>,
{
    let parsed = RecoveryArguments {
        host: required_token(arguments, "--host", 253)?,
        port: required_port(arguments)?,
        database: required_token(arguments, "--database", 127)?,
        username: required_token(arguments, "--username", 127)?,
        ssl_mode: required_ssl_mode(arguments)?,
        password_file: required_absolute_path(arguments, "--password-file")?,
        storage_bundle: required_absolute_path(arguments, "--storage-bundle")?,
    };
    if arguments.next().is_some() {
        return Err(invalid_arguments());
    }
    Ok(parsed)
}

pub(crate) fn parse_export_bundle_arguments<I>(
    arguments: &mut std::iter::Peekable<I>,
) -> Result<PathBuf, String>
where
    I: Iterator<Item = OsString>,
{
    let output = required_absolute_path(arguments, "--output")?;
    if arguments.next().is_some() {
        return Err(invalid_arguments());
    }
    Ok(output)
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
    let value = required_value(arguments, name)?
        .into_string()
        .map_err(|_| invalid_arguments())?;
    let valid = !value.is_empty()
        && value.len() <= maximum_length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-'));
    valid.then_some(value).ok_or_else(invalid_arguments)
}

fn required_port<I>(arguments: &mut std::iter::Peekable<I>) -> Result<u16, String>
where
    I: Iterator<Item = OsString>,
{
    required_value(arguments, "--port")?
        .into_string()
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .filter(|port| *port > 0)
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
    "Scheduler offline recovery arguments are invalid".to_owned()
}
