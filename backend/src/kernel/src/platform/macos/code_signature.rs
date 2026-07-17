//! macOS code-signature verification for a release-bound native artifact.

use std::path::Path;
use std::process::Command;

pub fn verify(path: &Path, expected_team_id: &str) -> Result<(), String> {
    if !cfg!(target_os = "macos") {
        return Err("macOS code-signature verification is unavailable on this platform".to_owned());
    }
    if !path.is_absolute() {
        return Err("signed artifact path must be absolute".to_owned());
    }
    let metadata = std::fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("signed artifact must be a regular non-symlink file".to_owned());
    }
    let verified = Command::new("/usr/bin/codesign")
        .args(["--verify", "--strict", "--verbose=4"])
        .arg(path)
        .output()
        .map_err(|_| "macOS code-signature verifier is unavailable".to_owned())?;
    if !verified.status.success() {
        return Err("macOS code-signature verification failed".to_owned());
    }
    let details = Command::new("/usr/bin/codesign")
        .args(["-dvv"])
        .arg(path)
        .output()
        .map_err(|_| "macOS code-signature verifier is unavailable".to_owned())?;
    if !details.status.success() || !has_expected_team_id(&details.stderr, expected_team_id) {
        return Err("macOS code-signature team identity does not match".to_owned());
    }
    Ok(())
}

pub fn has_expected_team_id(details: &[u8], expected_team_id: &str) -> bool {
    std::str::from_utf8(details).is_ok_and(|details| {
        details
            .lines()
            .any(|line| line == format!("TeamIdentifier={expected_team_id}"))
    })
}
