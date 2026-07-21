use std::ffi::OsString;
use std::path::PathBuf;

use crate::runtime_cli::parse_recovery_arguments;

#[test]
fn scheduler_recovery_cli_accepts_only_fixed_non_secret_arguments() {
    let mut arguments = [
        "--host",
        "127.0.0.1",
        "--port",
        "5432",
        "--database",
        "hermes",
        "--username",
        "recovery",
        "--ssl-mode",
        "verify-full",
        "--password-file",
        "/private/password",
        "--storage-bundle",
        "/private/storage-bundle.pb",
    ]
    .into_iter()
    .map(OsString::from)
    .peekable();
    let parsed: crate::runtime_cli::RecoveryArguments =
        parse_recovery_arguments(&mut arguments).expect("recovery arguments");
    assert_eq!(parsed.host, "127.0.0.1");
    assert_eq!(parsed.port, 5432);
    assert_eq!(parsed.password_file, PathBuf::from("/private/password"));
    assert_eq!(
        parsed.storage_bundle,
        PathBuf::from("/private/storage-bundle.pb")
    );
}

#[test]
fn scheduler_recovery_bundle_export_requires_one_absolute_output() {
    let mut valid = ["--output", "/private/storage-bundle.pb"]
        .into_iter()
        .map(OsString::from)
        .peekable();
    assert_eq!(
        crate::runtime_cli::parse_export_bundle_arguments(&mut valid),
        Ok(PathBuf::from("/private/storage-bundle.pb"))
    );
    let mut relative = ["--output", "storage-bundle.pb"]
        .into_iter()
        .map(OsString::from)
        .peekable();
    assert!(crate::runtime_cli::parse_export_bundle_arguments(&mut relative).is_err());
}

#[test]
fn scheduler_inherited_cli_remains_available_in_the_shared_parser() {
    let mut arguments = [
        "--descriptor-path",
        "/private/descriptor.pb",
        "--settings-schema-path",
        "/private/settings.pb",
        "--configuration-path",
        "/private/configuration.pb",
    ]
    .into_iter()
    .map(OsString::from)
    .peekable();
    let parsed = crate::runtime_cli::parse_serve_inherited_arguments(&mut arguments)
        .expect("inherited arguments");
    assert_eq!(
        parsed.descriptor_path,
        PathBuf::from("/private/descriptor.pb")
    );
    assert_eq!(
        parsed.settings_schema_path,
        Some(PathBuf::from("/private/settings.pb"))
    );
    assert_eq!(
        parsed.configuration_path,
        PathBuf::from("/private/configuration.pb")
    );
}

#[test]
fn scheduler_recovery_cli_rejects_passwords_and_relative_files() {
    let mut secret = [
        "--host",
        "user:secret@localhost",
        "--port",
        "5432",
        "--database",
        "hermes",
        "--username",
        "recovery",
        "--ssl-mode",
        "disable",
        "--password-file",
        "/private/password",
        "--storage-bundle",
        "/private/storage-bundle.pb",
    ]
    .into_iter()
    .map(OsString::from)
    .peekable();
    assert!(parse_recovery_arguments(&mut secret).is_err());

    let mut relative = [
        "--host",
        "localhost",
        "--port",
        "5432",
        "--database",
        "hermes",
        "--username",
        "recovery",
        "--ssl-mode",
        "disable",
        "--password-file",
        "password",
        "--storage-bundle",
        "/private/storage-bundle.pb",
    ]
    .into_iter()
    .map(OsString::from)
    .peekable();
    assert!(parse_recovery_arguments(&mut relative).is_err());
}
