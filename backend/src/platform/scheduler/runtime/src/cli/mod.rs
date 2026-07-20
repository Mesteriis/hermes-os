//! Parses fixed Scheduler inherited-runtime arguments.

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

pub(crate) struct ServeInheritedPaths {
    pub(crate) descriptor_path: PathBuf,
    pub(crate) settings_schema_path: Option<PathBuf>,
    pub(crate) configuration_path: PathBuf,
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
        return Err("Scheduler runtime arguments are invalid".to_owned());
    }
    Ok(ServeInheritedPaths {
        descriptor_path,
        settings_schema_path,
        configuration_path,
    })
}

fn required_path<I>(arguments: &mut std::iter::Peekable<I>, name: &str) -> Result<PathBuf, String>
where
    I: Iterator<Item = OsString>,
{
    if arguments.next().as_deref() != Some(OsStr::new(name)) {
        return Err("Scheduler runtime arguments are invalid".to_owned());
    }
    arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| "Scheduler runtime arguments are invalid".to_owned())
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
            .ok_or_else(|| "Scheduler runtime arguments are invalid".to_owned());
    }
    Ok(None)
}
