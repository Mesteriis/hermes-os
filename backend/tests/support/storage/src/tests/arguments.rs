//! Regression coverage for the fixed Storage inherited-runtime CLI contract.

use std::ffi::OsString;
use std::path::PathBuf;

use crate::storage_runtime_arguments::parse_serve_inherited_arguments;

#[test]
fn optional_settings_schema_preserves_the_required_configuration_argument() {
    let mut arguments = [
        "--descriptor-path",
        "/tmp/descriptor",
        "--configuration-path",
        "/tmp/configuration",
    ]
    .into_iter()
    .map(OsString::from)
    .peekable();

    let paths = parse_serve_inherited_arguments(&mut arguments).expect("parse runtime arguments");

    assert_eq!(paths.descriptor_path, PathBuf::from("/tmp/descriptor"));
    assert_eq!(paths.settings_schema_path, None);
    assert_eq!(
        paths.configuration_path,
        PathBuf::from("/tmp/configuration")
    );
}
