use std::{ffi::OsString, fs, path::PathBuf};

use hermes_review_task_candidate_persistence::review_task_candidate_storage_bundle_v1;
use hermes_review_task_candidate_runtime::{
    review_task_candidate_module_descriptor_v1, review_task_candidate_settings_schema_bytes_v1,
};
use prost::Message;

fn main() -> Result<(), String> {
    let mut arguments = std::env::args_os().skip(1);
    let Some(command) = arguments.next() else {
        return Err("Review Task Candidate command is required".to_owned());
    };
    match command.to_str() {
        Some("export-storage-bundle") => export(
            &mut arguments,
            review_task_candidate_storage_bundle_v1().encode_to_vec(),
        ),
        Some("export-module-descriptor") => {
            let build_id = required(&mut arguments, "build id")?;
            export(
                &mut arguments,
                review_task_candidate_module_descriptor_v1(&build_id).encode_to_vec(),
            )
        }
        Some("export-settings-schema") => export(
            &mut arguments,
            review_task_candidate_settings_schema_bytes_v1(),
        ),
        _ => Err("Review Task Candidate command is invalid".to_owned()),
    }
}

fn export<I>(arguments: &mut I, bytes: Vec<u8>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    let output = arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| "Review Task Candidate output path is required".to_owned())?;
    if arguments.next().is_some() {
        return Err("Review Task Candidate arguments are invalid".to_owned());
    }
    fs::write(output, bytes).map_err(|_| "Review Task Candidate export failed".to_owned())
}

fn required<I>(arguments: &mut I, name: &str) -> Result<String, String>
where
    I: Iterator<Item = OsString>,
{
    arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("Review Task Candidate {name} is required"))
}
