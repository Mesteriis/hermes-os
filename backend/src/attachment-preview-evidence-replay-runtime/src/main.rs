use std::{ffi::OsString, fs, path::PathBuf};

use hermes_attachment_preview_evidence_replay_persistence::attachment_preview_evidence_replay_storage_bundle_v1;
use hermes_attachment_preview_evidence_replay_runtime::{
    attachment_preview_evidence_replay_module_descriptor_v1,
    attachment_preview_evidence_replay_settings_schema_bytes_v1,
};
use prost::Message;

fn main() -> Result<(), String> {
    let mut arguments = std::env::args_os().skip(1);
    let Some(command) = arguments.next().and_then(|value| value.into_string().ok()) else {
        return Err("Replay workflow command is required".to_owned());
    };
    match command.as_str() {
        "export-storage-bundle" => export(
            &mut arguments,
            attachment_preview_evidence_replay_storage_bundle_v1().encode_to_vec(),
        ),
        "export-module-descriptor" => {
            let build_id = required_string(&mut arguments, "build id")?;
            export(
                &mut arguments,
                attachment_preview_evidence_replay_module_descriptor_v1(&build_id).encode_to_vec(),
            )
        }
        "export-settings-schema" => export(
            &mut arguments,
            attachment_preview_evidence_replay_settings_schema_bytes_v1(),
        ),
        _ => Err("Replay workflow command is invalid".to_owned()),
    }
}

fn export<I>(arguments: &mut I, bytes: Vec<u8>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    let output = arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| "Replay workflow output path is required".to_owned())?;
    if arguments.next().is_some() {
        return Err("Replay workflow arguments are invalid".to_owned());
    }
    fs::write(output, bytes).map_err(|_| "Replay workflow export failed".to_owned())
}

fn required_string<I>(arguments: &mut I, name: &str) -> Result<String, String>
where
    I: Iterator<Item = OsString>,
{
    arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("Replay workflow {name} is required"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_exact_export_arguments() {
        let mut missing = Vec::<OsString>::new().into_iter();
        assert!(required_string(&mut missing, "build id").is_err());
        let mut exact = vec![OsString::from("build-1")].into_iter();
        assert_eq!(
            required_string(&mut exact, "build id"),
            Ok("build-1".to_owned())
        );
    }
}
