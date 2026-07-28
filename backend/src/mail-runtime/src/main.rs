//! Mail integration process root for the exact Kernel-inherited runtime contract.

use std::ffi::{OsStr, OsString};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use hermes_mail_runtime::managed::{
    MailDeliveryDispatchErrorV1, MailMessageFlagDispatchErrorV1,
    MailMessageLocationDispatchErrorV1, MailMessagePermanentDeleteDispatchErrorV1,
};
use hermes_mail_runtime::{
    MailRuntimeAdmission,
    attachment_security_outbox::MailAttachmentSecurityOutboxRelayError,
    communications_outbox::MailCommunicationsOutboxRelayError,
    gmail_oauth::{
        CompletedGmailOAuthProviderOperationV1, MailGmailOAuthDispatchErrorV1,
        execute_gmail_oauth_provider_operation,
    },
    managed, settings,
};
use hermes_runtime_protocol::{
    v1::ManagedIntegrationRuntimeConfigurationV1,
    validation::{
        descriptor::{
            decode_settings_schema_v1, decode_settings_snapshot_v1,
            validate_settings_snapshot_against_schema_v1,
        },
        managed_integration_runtime::validate_managed_integration_runtime_configuration,
    },
};
use prost::Message;

struct InheritedPaths {
    descriptor: PathBuf,
    settings_schema: PathBuf,
    settings_snapshot: PathBuf,
    runtime_configuration: PathBuf,
    runtime_instance_id: String,
}

fn main() -> Result<(), String> {
    let mut arguments = std::env::args_os();
    let _binary = arguments.next();
    match arguments.next().as_deref() {
        Some(command) if command == OsStr::new("serve-inherited") => {
            serve_inherited(&mut arguments.peekable())
        }
        _ => Err("Mail runtime command is unavailable".to_owned()),
    }
}

fn serve_inherited<I>(arguments: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    let paths = parse_paths(arguments)?;
    let descriptor = read_contract(&paths.descriptor)?;
    let schema_bytes = read_contract(&paths.settings_schema)?;
    let schema = decode_settings_schema_v1(&schema_bytes)
        .map_err(|_| "Mail runtime settings schema is invalid".to_owned())?;
    let selected_snapshot_bytes = read_contract(&paths.settings_snapshot)?;
    let selected_snapshot = decode_settings_snapshot_v1(&selected_snapshot_bytes)
        .map_err(|_| "Mail runtime settings snapshot is invalid".to_owned())?;
    validate_settings_snapshot_against_schema_v1(&schema, &selected_snapshot)
        .map_err(|_| "Mail runtime settings snapshot is invalid".to_owned())?;
    let configuration = ManagedIntegrationRuntimeConfigurationV1::decode(
        read_contract(&paths.runtime_configuration)?.as_slice(),
    )
    .map_err(|_| "Mail runtime configuration is invalid".to_owned())?;
    validate_managed_integration_runtime_configuration(&configuration)
        .map_err(|_| "Mail runtime configuration is invalid".to_owned())?;
    if configuration.runtime_instance_id != paths.runtime_instance_id {
        return Err("Mail runtime configuration is stale".to_owned());
    }
    let snapshots = if configuration.configuration_instances.is_empty() {
        vec![selected_snapshot]
    } else {
        let selected = configuration
            .configuration_instances
            .iter()
            .find(|instance| {
                instance.configuration_instance_id == configuration.configuration_instance_id
            })
            .ok_or_else(|| "Mail runtime settings catalog is invalid".to_owned())?;
        if selected.settings_snapshot_bytes != selected_snapshot_bytes {
            return Err("Mail runtime settings catalog is stale".to_owned());
        }
        configuration
            .configuration_instances
            .iter()
            .map(|instance| {
                let snapshot = decode_settings_snapshot_v1(&instance.settings_snapshot_bytes)
                    .map_err(|_| "Mail runtime settings catalog is invalid".to_owned())?;
                validate_settings_snapshot_against_schema_v1(&schema, &snapshot)
                    .map_err(|_| "Mail runtime settings catalog is invalid".to_owned())?;
                Ok(snapshot)
            })
            .collect::<Result<Vec<_>, String>>()?
    };
    let storage = configuration
        .storage
        .clone()
        .ok_or_else(|| "Mail runtime configuration is invalid".to_owned())?;
    let admissions = snapshots
        .into_iter()
        .map(|snapshot| {
            let settings = settings::decode(&snapshot)?;
            Ok(MailRuntimeAdmission {
                logical_owner_id: configuration.logical_owner_id.clone(),
                configuration_instance_id: snapshot.target_id,
                module_registration_id: configuration.registration_id.clone(),
                runtime_instance_id: configuration.runtime_instance_id.clone(),
                runtime_generation: configuration.runtime_generation,
                grant_epoch: configuration.grant_epoch,
                vault_runtime_generation: storage.vault_runtime_generation,
                settings_revision: snapshot.revision,
                account: settings.account,
                gmail_oauth: settings.gmail_oauth,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let control_channel = inherited_control_channel()?;
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|_| "Mail runtime executor is unavailable".to_owned())?;
    let mut admitted = runtime
        .block_on(managed::open_admitted_runtime_catalog(
            control_channel,
            descriptor,
            schema_bytes,
            &admissions,
            storage,
            &configuration.event_hub_endpoint,
            configuration.event_credential_revision,
        ))
        .map_err(|error| {
            developer_diagnostic(&format!("developer_mail_admission_error={error:?}"));
            "Mail runtime admission was rejected".to_owned()
        })?;
    let mut gmail_oauth_provider_operation: Option<
        tokio::task::JoinHandle<CompletedGmailOAuthProviderOperationV1>,
    > = None;
    loop {
        runtime
            .block_on(admitted.try_handle_client_delivery())
            .map_err(|error| {
                developer_diagnostic(&format!("developer_mail_client_delivery_error={error:?}"));
                "Mail runtime client delivery failed".to_owned()
            })?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "Mail runtime clock is unavailable".to_owned())?;
        let now = i64::try_from(now.as_secs())
            .map_err(|_| "Mail runtime clock is unavailable".to_owned())?;
        if gmail_oauth_provider_operation
            .as_ref()
            .is_some_and(tokio::task::JoinHandle::is_finished)
        {
            let completed = runtime
                .block_on(
                    gmail_oauth_provider_operation
                        .take()
                        .expect("finished Gmail OAuth provider operation"),
                )
                .map_err(|_| "Mail runtime Gmail OAuth provider worker failed".to_owned())?;
            let connection_id = completed.connection_id().to_owned();
            admitted
                .select_account(&connection_id)
                .map_err(|_| "Mail runtime Gmail OAuth account selection failed".to_owned())?;
            handle_gmail_oauth_dispatch_result(
                runtime.block_on(admitted.finalize_gmail_oauth_provider_operation(completed, now)),
            )?;
        }
        if gmail_oauth_provider_operation.is_none() {
            for connection_id in admitted.connection_ids() {
                admitted
                    .select_account(&connection_id)
                    .map_err(|_| "Mail runtime Gmail OAuth account selection failed".to_owned())?;
                match runtime
                    .block_on(admitted.prepare_next_gmail_oauth_provider_operation(now, now))
                {
                    Ok(Some(prepared)) => {
                        gmail_oauth_provider_operation =
                            Some(runtime.spawn(execute_gmail_oauth_provider_operation(prepared)));
                        break;
                    }
                    Ok(None) => {}
                    Err(error) => handle_gmail_oauth_dispatch_result(Err(error))?,
                }
            }
        }
        for connection_id in admitted.connection_ids() {
            admitted
                .select_account(&connection_id)
                .map_err(|_| "Mail runtime account selection failed".to_owned())?;
            execute_account_queues(&runtime, &mut admitted, now)?;
        }
        runtime
            .block_on(admitted.try_consume_attachment_anchor_handoff(now))
            .map_err(|_| {
                developer_diagnostic("developer_mail_attachment_anchor_handoff_failed");
                "Mail runtime attachment-anchor handoff failed".to_owned()
            })?;
        runtime
            .block_on(admitted.try_consume_attachment_safety_state(now))
            .map_err(|_| {
                developer_diagnostic("developer_mail_attachment_safety_projection_failed");
                "Mail runtime attachment safety projection failed".to_owned()
            })?;
        match runtime.block_on(admitted.relay_communications_outbox(now)) {
            Ok(_) => {}
            Err(MailCommunicationsOutboxRelayError::Unavailable) => {
                developer_diagnostic("developer_mail_outbox_relay_unavailable");
            }
            Err(MailCommunicationsOutboxRelayError::Persistence(_)) => {
                developer_diagnostic("developer_mail_outbox_persistence_failed");
                return Err("Mail runtime outbox persistence failed".to_owned());
            }
        }
        match runtime.block_on(admitted.relay_attachment_security_outbox(now)) {
            Ok(_) => {}
            Err(MailAttachmentSecurityOutboxRelayError::Unavailable) => {
                developer_diagnostic("developer_mail_attachment_security_outbox_relay_unavailable");
            }
            Err(MailAttachmentSecurityOutboxRelayError::Persistence(_)) => {
                developer_diagnostic(
                    "developer_mail_attachment_security_outbox_persistence_failed",
                );
                return Err("Mail runtime Attachment Security outbox persistence failed".to_owned());
            }
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

fn execute_account_queues(
    runtime: &tokio::runtime::Runtime,
    admitted: &mut managed::MailAdmittedRuntime,
    now: i64,
) -> Result<(), String> {
    match runtime.block_on(admitted.execute_next_delivery(now, now)) {
        Ok(_) => {}
        Err(MailDeliveryDispatchErrorV1::ProviderRejected) => {
            developer_diagnostic("developer_mail_delivery_rejected");
        }
        Err(MailDeliveryDispatchErrorV1::AttachmentRejected) => {
            developer_diagnostic("developer_mail_delivery_attachment_rejected");
        }
        Err(MailDeliveryDispatchErrorV1::ProviderOutcomeUnknown) => {
            developer_diagnostic("developer_mail_delivery_outcome_unknown");
        }
        Err(MailDeliveryDispatchErrorV1::InvalidStoredCommand) => {
            developer_diagnostic("developer_mail_delivery_command_invalid");
            return Err("Mail runtime delivery command is invalid".to_owned());
        }
        Err(MailDeliveryDispatchErrorV1::Persistence) => {
            developer_diagnostic("developer_mail_delivery_persistence_failed");
            return Err("Mail runtime delivery persistence failed".to_owned());
        }
    }
    match runtime.block_on(admitted.execute_next_message_flag_command(now)) {
        Ok(_) => {}
        Err(MailMessageFlagDispatchErrorV1::ProviderRejected) => {
            developer_diagnostic("developer_mail_message_flag_rejected");
        }
        Err(MailMessageFlagDispatchErrorV1::ProviderOutcomeUnknown) => {
            developer_diagnostic("developer_mail_message_flag_outcome_unknown");
        }
        Err(MailMessageFlagDispatchErrorV1::InvalidStoredCommand) => {
            developer_diagnostic("developer_mail_message_flag_command_invalid");
            return Err("Mail runtime message flag command is invalid".to_owned());
        }
        Err(MailMessageFlagDispatchErrorV1::Persistence) => {
            developer_diagnostic("developer_mail_message_flag_persistence_failed");
            return Err("Mail runtime message flag persistence failed".to_owned());
        }
    }
    match runtime.block_on(admitted.execute_next_message_location_command(now)) {
        Ok(_) => {}
        Err(MailMessageLocationDispatchErrorV1::ProviderRejected) => {
            developer_diagnostic("developer_mail_message_location_rejected");
        }
        Err(MailMessageLocationDispatchErrorV1::ProviderUnsupported) => {
            developer_diagnostic("developer_mail_message_location_unsupported");
        }
        Err(MailMessageLocationDispatchErrorV1::ProviderOutcomeUnknown) => {
            developer_diagnostic("developer_mail_message_location_outcome_unknown");
        }
        Err(MailMessageLocationDispatchErrorV1::InvalidStoredCommand) => {
            developer_diagnostic("developer_mail_message_location_command_invalid");
            return Err("Mail runtime message location command is invalid".to_owned());
        }
        Err(MailMessageLocationDispatchErrorV1::Persistence) => {
            developer_diagnostic("developer_mail_message_location_persistence_failed");
            return Err("Mail runtime message location persistence failed".to_owned());
        }
    }
    match runtime.block_on(admitted.execute_next_message_permanent_delete_command(now)) {
        Ok(_) => {}
        Err(MailMessagePermanentDeleteDispatchErrorV1::ProviderRejected) => {
            developer_diagnostic("developer_mail_message_permanent_delete_rejected");
        }
        Err(MailMessagePermanentDeleteDispatchErrorV1::ProviderUnsupported) => {
            developer_diagnostic("developer_mail_message_permanent_delete_unsupported");
        }
        Err(MailMessagePermanentDeleteDispatchErrorV1::ReauthorizationRequired) => {
            developer_diagnostic(
                "developer_mail_message_permanent_delete_reauthorization_required",
            );
        }
        Err(MailMessagePermanentDeleteDispatchErrorV1::ProviderOutcomeUnknown) => {
            developer_diagnostic("developer_mail_message_permanent_delete_outcome_unknown");
        }
        Err(MailMessagePermanentDeleteDispatchErrorV1::InvalidStoredCommand) => {
            developer_diagnostic("developer_mail_message_permanent_delete_command_invalid");
            return Err("Mail runtime permanent delete command is invalid".to_owned());
        }
        Err(MailMessagePermanentDeleteDispatchErrorV1::Persistence) => {
            developer_diagnostic("developer_mail_message_permanent_delete_persistence_failed");
            return Err("Mail runtime permanent delete persistence failed".to_owned());
        }
    }
    Ok(())
}

fn developer_diagnostic(message: &str) {
    if std::env::var_os("HERMES_DEVELOPER_VERBOSE").is_some() {
        eprintln!("{message}");
    }
}

fn handle_gmail_oauth_dispatch_result(
    result: Result<(), MailGmailOAuthDispatchErrorV1>,
) -> Result<(), String> {
    match result {
        Ok(()) => Ok(()),
        Err(MailGmailOAuthDispatchErrorV1::Rejected) => {
            developer_diagnostic("developer_mail_gmail_oauth_rejected");
            Ok(())
        }
        Err(MailGmailOAuthDispatchErrorV1::OutcomeUnknown) => {
            developer_diagnostic("developer_mail_gmail_oauth_outcome_unknown");
            Ok(())
        }
        Err(MailGmailOAuthDispatchErrorV1::InvalidStoredOperation) => {
            developer_diagnostic("developer_mail_gmail_oauth_operation_invalid");
            Err("Mail runtime Gmail OAuth operation is invalid".to_owned())
        }
        Err(MailGmailOAuthDispatchErrorV1::Persistence) => {
            developer_diagnostic("developer_mail_gmail_oauth_persistence_failed");
            Err("Mail runtime Gmail OAuth persistence failed".to_owned())
        }
    }
}

fn parse_paths<I>(arguments: &mut std::iter::Peekable<I>) -> Result<InheritedPaths, String>
where
    I: Iterator<Item = OsString>,
{
    let descriptor = required_path(arguments, "--descriptor-path")?;
    let settings_schema = required_path(arguments, "--settings-schema-path")?;
    let settings_snapshot = required_path(arguments, "--settings-snapshot-path")?;
    let runtime_configuration = required_path(arguments, "--runtime-configuration-path")?;
    let runtime_instance_id = required_string(arguments, "--runtime-instance-id")?;
    if arguments.next().is_some() || runtime_instance_id.trim().is_empty() {
        return Err("Mail runtime arguments are invalid".to_owned());
    }
    Ok(InheritedPaths {
        descriptor,
        settings_schema,
        settings_snapshot,
        runtime_configuration,
        runtime_instance_id,
    })
}

fn required_path<I>(arguments: &mut I, name: &str) -> Result<PathBuf, String>
where
    I: Iterator<Item = OsString>,
{
    required_string(arguments, name).map(PathBuf::from)
}

fn required_string<I>(arguments: &mut I, name: &str) -> Result<String, String>
where
    I: Iterator<Item = OsString>,
{
    if arguments.next().as_deref() != Some(OsStr::new(name)) {
        return Err("Mail runtime arguments are invalid".to_owned());
    }
    arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or_else(|| "Mail runtime arguments are invalid".to_owned())
}

fn inherited_control_channel() -> Result<UnixStream, String> {
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 {
        return Err("Mail runtime inherited control channel is unavailable".to_owned());
    }
    Ok(unsafe { UnixStream::from_raw_fd(duplicated) })
}

fn read_contract(path: &Path) -> Result<Vec<u8>, String> {
    const MAX_CONTRACT_BYTES: u64 = 512 * 1024;
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "Mail runtime contract is unavailable".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_CONTRACT_BYTES
    {
        return Err("Mail runtime contract is unavailable".to_owned());
    }
    std::fs::read(path).map_err(|_| "Mail runtime contract is unavailable".to_owned())
}
