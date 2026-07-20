//! Parses the fixed inherited Blob service arguments.

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

pub(crate) struct ServeInheritedPaths {
    pub(crate) descriptor_path: PathBuf,
    pub(crate) settings_schema_path: Option<PathBuf>,
    pub(crate) configuration_path: PathBuf,
}

pub(crate) enum OfflineRecoveryCommand {
    Export {
        data_dir: PathBuf,
        destination: PathBuf,
    },
    Verify {
        source: PathBuf,
    },
    Restore {
        source: PathBuf,
        data_dir: PathBuf,
    },
}

pub(crate) fn parse_serve_inherited_arguments<I>(
    arguments: &mut std::iter::Peekable<I>,
) -> Result<ServeInheritedPaths, String>
where
    I: Iterator<Item = OsString>,
{
    let descriptor_path = required_path(arguments, "--descriptor-path")?;
    let settings_schema_path = optional_path(arguments, "--settings-schema-path")?;
    let configuration_path = required_path(arguments, "--configuration-path")?;
    if arguments.next().is_some() {
        return Err("Blob service arguments are invalid".to_owned());
    }
    Ok(ServeInheritedPaths {
        descriptor_path,
        settings_schema_path,
        configuration_path,
    })
}

pub(crate) fn parse_offline_recovery_command<I>(
    command: &OsStr,
    arguments: &mut std::iter::Peekable<I>,
) -> Result<OfflineRecoveryCommand, String>
where
    I: Iterator<Item = OsString>,
{
    let parsed = match command.to_str() {
        Some("export-backup") => OfflineRecoveryCommand::Export {
            data_dir: required_absolute_path(arguments, "--data-dir")?,
            destination: required_absolute_path(arguments, "--destination")?,
        },
        Some("verify-backup") => OfflineRecoveryCommand::Verify {
            source: required_absolute_path(arguments, "--source")?,
        },
        Some("restore-backup") => OfflineRecoveryCommand::Restore {
            source: required_absolute_path(arguments, "--source")?,
            data_dir: required_absolute_path(arguments, "--data-dir")?,
        },
        _ => return Err(invalid_recovery_arguments()),
    };
    if arguments.next().is_some() {
        return Err(invalid_recovery_arguments());
    }
    Ok(parsed)
}

fn required_path<I>(arguments: &mut std::iter::Peekable<I>, name: &str) -> Result<PathBuf, String>
where
    I: Iterator<Item = OsString>,
{
    if arguments.next().as_deref() != Some(OsStr::new(name)) {
        return Err("Blob service arguments are invalid".to_owned());
    }
    arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| "Blob service arguments are invalid".to_owned())
}

fn required_absolute_path<I>(
    arguments: &mut std::iter::Peekable<I>,
    name: &str,
) -> Result<PathBuf, String>
where
    I: Iterator<Item = OsString>,
{
    let path = required_path(arguments, name).map_err(|_| invalid_recovery_arguments())?;
    path.is_absolute()
        .then_some(path)
        .ok_or_else(invalid_recovery_arguments)
}

fn optional_path<I>(
    arguments: &mut std::iter::Peekable<I>,
    name: &str,
) -> Result<Option<PathBuf>, String>
where
    I: Iterator<Item = OsString>,
{
    if arguments
        .peek()
        .is_some_and(|argument| argument == OsStr::new(name))
    {
        let _ = arguments.next();
        return arguments
            .next()
            .map(PathBuf::from)
            .map(Some)
            .ok_or_else(|| "Blob service arguments are invalid".to_owned());
    }
    Ok(None)
}

fn invalid_recovery_arguments() -> String {
    "Blob offline recovery arguments are invalid".to_owned()
}
