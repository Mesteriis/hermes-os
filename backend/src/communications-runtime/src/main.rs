//! Communications runtime composition root for ADR-0239.

use hermes_communications_api::CommunicationSummary;
use hermes_communications_domain::{canonicalize_communication, promote_draft_to_summary};
use hermes_communications_ingress::{SourceEnvelope, new_communication_observation_draft};
use hermes_communications_persistence::CommunicationsPersistence;

fn main() -> Result<(), String> {
    let mut args = std::env::args();
    let _ = args.next();

    match args.next().as_deref() {
        Some("ingest") => {
            let operation_id = args
                .next()
                .ok_or_else(|| "operation_id is required".to_owned())?;
            let source_kind = args.next().unwrap_or_else(|| "mail-imap".to_owned());
            let source_id = args
                .next()
                .ok_or_else(|| "source_id is required".to_owned())?;
            let preview = args.next().unwrap_or_default();
            let has_body =
                parse_bool_flag(args.next().unwrap_or_else(|| "true".to_owned()).as_str())?;
            let is_final_window =
                parse_bool_flag(args.next().unwrap_or_else(|| "true".to_owned()).as_str())?;
            let draft = new_communication_observation_draft(
                operation_id,
                SourceEnvelope {
                    source_kind,
                    source_id,
                },
                if preview.is_empty() {
                    None
                } else {
                    Some(preview)
                },
                has_body,
                is_final_window,
            )
            .map_err(|_| "invalid communication draft".to_owned())?;
            let summary = promote_draft_to_summary(draft).map_err(|_| "draft to summary failed")?;
            let canonical = canonicalize_communication(&summary);
            store_summary(summary)?;
            println!("communications_runtime_ingest ok id={}", canonical.id);
            Ok(())
        }
        Some("status") => {
            println!("communications_runtime status ok");
            Ok(())
        }
        Some(command) => Err(format!(
            "communications runtime command is unavailable: {command}"
        )),
        None => Err("communications runtime command is unavailable".to_owned()),
    }
}

fn parse_bool_flag(value: &str) -> Result<bool, String> {
    match value {
        "1" | "true" | "TRUE" | "True" => Ok(true),
        "0" | "false" | "FALSE" | "False" => Ok(false),
        _ => Err(format!("invalid boolean flag: {value}")),
    }
}

fn store_summary(summary: CommunicationSummary) -> Result<(), String> {
    let mut persistence = CommunicationsPersistence::new();
    persistence
        .persist(&summary)
        .map_err(|_| "persistence rejected duplicate operation".to_owned())?;
    Ok(())
}
