//! Mail runtime composition for ADR-0239.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

use hermes_mail_api::{
    BeginImapConnection, CompleteImapConnection, DEFAULT_WINDOW, GetConnection, GetOperationStatus,
    MailConnection, MailConnectionState, MailOperation, MailOperationId, SyncNow,
};
use hermes_mail_core::{
    ConnectionTracker, bounded_window, draft_ingress_observation, validate_sync_request,
};
use hermes_mail_persistence::{MailPersistence, PersistedMailConnection};

const COMMUNICATIONS_RUNTIME_PROCESS_ID: &str = "hermes-communications-runtime";
const DEFAULT_COMMUNICATION_INGEST_LIMIT: usize = 3;

struct RuntimeState {
    persistence: MailPersistence,
    tracker: ConnectionTracker,
    operations: HashMap<MailOperationId, MailOperation>,
}

struct SyncNowRequestContext<'a> {
    request: SyncNow,
    host: Option<String>,
    username: Option<String>,
    port: Option<u16>,
    password: &'a str,
    window: u32,
    windows: u32,
    emit_observations: bool,
}

impl RuntimeState {
    fn new() -> Self {
        Self {
            persistence: MailPersistence::new(),
            tracker: ConnectionTracker::new(),
            operations: HashMap::new(),
        }
    }

    fn command_status(&self) -> String {
        format!(
            "mail_runtime_status max_sync_windows={}",
            self.persistence.policy().max_sync_windows
        )
    }

    fn begin_connection(&mut self, request: BeginImapConnection) -> Result<MailConnection, String> {
        if !request.use_tls {
            return Err("connection must use tls".to_owned());
        }
        validate_sync_request(&request.host, request.port, 0)
            .map_err(|_| "invalid connection request".to_owned())?;
        let connection = MailConnection {
            id: request.connection_id.clone(),
            host: request.host,
            port: request.port,
            username: request.username,
            state: MailConnectionState::Provisioning,
            operation_id: None,
        };
        self.persistence
            .put_connection(connection_to_persistence(connection.clone()));
        self.tracker.register_connection(&connection);
        self.tracker.set_ready(&connection.id);
        Ok(connection)
    }

    fn sync_now_with_observations(
        &mut self,
        context: SyncNowRequestContext<'_>,
    ) -> Result<String, String> {
        let connection = if let Some(connection) = self
            .persistence
            .get_connection(&context.request.connection_id)
        {
            connection.clone()
        } else {
            let host = context
                .host
                .ok_or_else(|| "connection is not provisioned".to_owned())?;
            let username = context
                .username
                .ok_or_else(|| "connection is not provisioned".to_owned())?;
            let port = context.port.unwrap_or(hermes_mail_api::IMAP_PORT);
            PersistedMailConnection {
                id: context.request.connection_id.clone(),
                host,
                username,
                port,
            }
        };
        let windows = context.windows.max(1);
        let plan =
            bounded_window(context.window, windows).map_err(|_| "invalid sync plan".to_owned())?;
        let operation = MailOperation {
            operation_id: context.request.operation_id.clone(),
            state: MailConnectionState::Syncing,
            window_size: plan.window,
        };
        self.tracker
            .set_syncing(&context.request.connection_id, operation.clone());
        self.operations
            .insert(context.request.operation_id.clone(), operation.clone());
        let sync_plan = plan.windows;
        let sync_result = hermes_mail_imap::sync_inbox(
            &connection.host,
            connection.port,
            &connection.username,
            Some(context.password),
            plan.window,
            sync_plan,
        )?;
        if context.emit_observations {
            self.emit_ingress_observations(
                &context.request.operation_id,
                &connection.id,
                &sync_result.messages,
            )?;
        } else {
            draft_ingress_observation(
                &context.request.operation_id,
                "mail-imap",
                connection.id.clone(),
                sync_result.messages.len(),
                None,
            )
            .map_err(|_| "invalid ingress draft".to_owned())?;
        }
        self.tracker.set_ready(&context.request.connection_id);
        Ok(format!(
            "sync_started op={} host={} window={} messages={}",
            context.request.operation_id,
            connection.host,
            plan.window,
            sync_result.messages.len()
        ))
    }

    fn emit_ingress_observations(
        &self,
        operation_id: &str,
        connection_id: &str,
        messages: &[hermes_mail_imap::ImapMessage],
    ) -> Result<(), String> {
        if !std::env::var("HERMES_ENABLE_MAIL_COMMUNICATION_INGEST").is_ok() {
            return Ok(());
        }

        let binary = runtime_binary(COMMUNICATIONS_RUNTIME_PROCESS_ID)?;
        for message in messages.iter().take(DEFAULT_COMMUNICATION_INGEST_LIMIT) {
            let body_is_readable = if message.has_plain_text {
                message.snippet.len()
            } else {
                0
            };
            let observation = draft_ingress_observation(
                &format!("{operation_id}:{}", message.uid),
                "mail-imap",
                connection_id.to_owned(),
                body_is_readable,
                Some(message.snippet.clone()),
            )
            .map_err(|_| "invalid ingress draft".to_owned())?;
            let preview = observation.text_preview.unwrap_or_default();
            let status = Command::new(&binary)
                .args([
                    "ingest".to_owned(),
                    observation.operation_id.to_owned(),
                    observation.source_kind,
                    observation.source_id,
                    preview,
                    observation.has_body.to_string(),
                    observation.is_final_window.to_string(),
                ])
                .output()
                .map_err(|_| "communication ingress command unavailable".to_owned())?;
            if !status.status.success() {
                return Err("communication ingress command failed".to_owned());
            }
        }
        Ok(())
    }

    fn complete_connection(&mut self, request: CompleteImapConnection) -> Result<String, String> {
        self.tracker.set_ready(&request.connection_id);
        if let Some(op) = self.operations.get_mut(&request.operation_id) {
            op.state = MailConnectionState::Ready;
        }
        Ok(format!(
            "connection_ready id={} op={}",
            request.connection_id, request.operation_id
        ))
    }

    fn get_connection(&self, request: GetConnection) -> Result<String, String> {
        self.persistence
            .get_connection(&request.connection_id)
            .ok_or_else(|| "connection is unknown".to_owned())
            .map(|connection| {
                format!(
                    "connection_id={} host={} state={:?} username={}",
                    connection.id,
                    connection.host,
                    self.tracker.status_of(&connection.id),
                    connection.username
                )
            })
    }

    fn get_operation_status(&self, request: GetOperationStatus) -> Result<String, String> {
        self.operations
            .get(&request.operation_id)
            .map(|operation| {
                format!(
                    "operation_id={} state={:?} window={}",
                    operation.operation_id, operation.state, operation.window_size
                )
            })
            .ok_or_else(|| "operation is unknown".to_owned())
    }

    fn sync_credentials(&self, password_file: Option<&str>) -> Result<String, String> {
        if let Some(path) = password_file {
            let password = read_trimmed_secret_file(path)?;
            if !password.is_empty() {
                return Ok(password);
            }
        }

        Err("mail sync requires password".to_owned())
    }
}

fn connection_to_persistence(connection: MailConnection) -> PersistedMailConnection {
    PersistedMailConnection {
        id: connection.id,
        host: connection.host,
        port: connection.port,
        username: connection.username,
    }
}

fn read_trimmed_secret_file(path: &str) -> Result<String, String> {
    let secret = fs::read_to_string(path).map_err(|_| "password file is unavailable".to_owned())?;
    validate_secret(secret).map_err(|_| "mail password file is invalid".to_owned())
}

fn parse_imap_port(raw_port: Option<String>) -> Result<u16, String> {
    raw_port
        .map(|value| {
            value
                .parse::<u16>()
                .map_err(|_| "port is invalid".to_owned())
        })
        .unwrap_or(Ok(hermes_mail_api::IMAP_PORT))
}

fn validate_secret(mut value: String) -> Result<String, io::Error> {
    if value.ends_with('\n') {
        let _ = value.pop();
        if value.ends_with('\r') {
            let _ = value.pop();
        }
    }
    if value.is_empty() || value.len() > 1024 || value.contains(['\0', '\r', '\n']) {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid secret"));
    }
    Ok(value)
}

fn main() -> Result<(), String> {
    let mut state = RuntimeState::new();
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("status") => handle_status(&state),
        Some("begin") => handle_begin(&mut state, &mut args),
        Some("sync") => handle_sync(&mut state, &mut args),
        Some("complete") => handle_complete(&mut state, &mut args),
        Some("get-connection") => handle_get_connection(&state, &mut args),
        Some("get-operation") => handle_get_operation(&state, &mut args),
        Some(command) => Err(format!("mail runtime command is unavailable: {command}")),
        None => Err("mail runtime command is unavailable".to_owned()),
    }
}

fn next_arg<I>(args: &mut I, name: &str) -> Result<String, String>
where
    I: Iterator<Item = String>,
{
    args.next().ok_or_else(|| format!("{name} is required"))
}

fn handle_status(state: &RuntimeState) -> Result<(), String> {
    println!("{}", state.command_status());
    Ok(())
}

fn handle_begin<I>(state: &mut RuntimeState, args: &mut I) -> Result<(), String>
where
    I: Iterator<Item = String>,
{
    let connection_id = next_arg(args, "connection_id")?;
    let host = next_arg(args, "host")?;
    let username = next_arg(args, "username")?;
    let port = parse_imap_port(args.next())?;
    let connection = state.begin_connection(BeginImapConnection {
        connection_id,
        host,
        port,
        username,
        use_tls: true,
    })?;
    println!(
        "begin_ok connection_id={} host={} state={:?}",
        connection.id, connection.host, connection.state
    );
    Ok(())
}

fn handle_sync<I>(state: &mut RuntimeState, args: &mut I) -> Result<(), String>
where
    I: Iterator<Item = String>,
{
    let connection_id = next_arg(args, "connection_id")?;
    let operation_id = next_arg(args, "operation_id")?;
    let mut emit_observations = false;
    let mut window = None;
    let mut windows = None;
    let mut password_file = None;
    let mut host = None;
    let mut username = None;
    let mut port = None;
    while let Some(value) = args.next() {
        match value.as_str() {
            "--host" => {
                host = Some(
                    args.next()
                        .ok_or_else(|| "--host requires a value".to_owned())?,
                );
            }
            "--username" => {
                username = Some(
                    args.next()
                        .ok_or_else(|| "--username requires a value".to_owned())?,
                );
            }
            "--port" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--port requires a value".to_owned())?;
                let parsed = value
                    .parse::<u16>()
                    .map_err(|_| "sync port is invalid".to_owned())?;
                port = Some(parsed);
            }
            "--emit-observations" => {
                emit_observations = true;
            }
            "--full-resync" => {}
            "--password-file" => {
                password_file = Some(
                    args.next()
                        .ok_or_else(|| "--password-file requires a value".to_owned())?,
                );
            }
            "--window" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--window requires a value".to_owned())?;
                let parsed = value
                    .parse::<u32>()
                    .map_err(|_| "sync window is invalid".to_owned())?;
                window = Some(parsed);
            }
            "--windows" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--windows requires a value".to_owned())?;
                let parsed = value
                    .parse::<u32>()
                    .map_err(|_| "sync windows is invalid".to_owned())?;
                windows = Some(parsed);
            }
            other => return Err(format!("unknown sync argument: {other}")),
        }
    }

    let password = state.sync_credentials(password_file.as_deref())?;
    let response = state.sync_now_with_observations(SyncNowRequestContext {
        request: SyncNow {
            connection_id,
            operation_id,
        },
        host,
        username,
        port,
        password: &password,
        window: window.unwrap_or(DEFAULT_WINDOW),
        windows: windows.unwrap_or_else(|| state.persistence.policy().max_sync_windows.max(1)),
        emit_observations,
    })?;
    println!("{response}");
    Ok(())
}

fn handle_complete<I>(state: &mut RuntimeState, args: &mut I) -> Result<(), String>
where
    I: Iterator<Item = String>,
{
    let connection_id = next_arg(args, "connection_id")?;
    let operation_id = next_arg(args, "operation_id")?;
    println!(
        "{}",
        state.complete_connection(CompleteImapConnection {
            connection_id,
            operation_id,
        })?
    );
    Ok(())
}

fn handle_get_connection<I>(state: &RuntimeState, args: &mut I) -> Result<(), String>
where
    I: Iterator<Item = String>,
{
    let connection_id = next_arg(args, "connection_id")?;
    println!("{}", state.get_connection(GetConnection { connection_id })?);
    Ok(())
}

fn handle_get_operation<I>(state: &RuntimeState, args: &mut I) -> Result<(), String>
where
    I: Iterator<Item = String>,
{
    let operation_id = next_arg(args, "operation_id")?;
    println!(
        "{}",
        state.get_operation_status(GetOperationStatus { operation_id })?
    );
    Ok(())
}

fn runtime_binary(process_id: &str) -> Result<String, String> {
    if let Ok(binary) = std::env::var(format!("CARGO_BIN_EXE_{}", process_id.replace('-', "_"))) {
        return Ok(binary);
    }
    let exe = if cfg!(windows) {
        format!("{process_id}.exe")
    } else {
        process_id.to_owned()
    };
    let fallback = std::env::current_exe()
        .ok()
        .and_then(|path| {
            path.parent()
                .map(|parent| parent.join(&exe).to_string_lossy().into_owned())
        })
        .unwrap_or(exe);
    if Path::new(&fallback).exists() {
        return Ok(fallback);
    }
    Err("communication runtime binary missing".to_owned())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    use hermes_mail_api::MAX_WINDOWS;

    use super::{parse_imap_port, validate_secret};

    #[test]
    fn validate_secret_rejects_invalid_chars_and_long_values() {
        assert!(validate_secret("bad\nsecret".to_owned()).is_err());
        assert!(validate_secret("bad\rsecret".to_owned()).is_err());
        let mut long_secret = String::new();
        long_secret.push_str(&"a".repeat(1025));
        assert!(validate_secret(long_secret).is_err());
    }

    #[test]
    fn parse_imap_port_requires_valid_number_or_defaults() {
        assert_eq!(
            parse_imap_port(None).expect("default"),
            hermes_mail_api::IMAP_PORT
        );
        assert_eq!(
            parse_imap_port(Some("1143".to_owned())).expect("explicit"),
            1143
        );
        assert!(parse_imap_port(Some("invalid-port".to_owned())).is_err());
    }

    #[test]
    fn sync_credentials_rejects_invalid_file_secret() {
        let state = super::RuntimeState::new();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let path = format!("/tmp/hermes-mail-secret-{nanos}.txt");
        {
            let mut file = fs::File::create(&path).expect("create secret file");
            file.write_all(b"line\nbreak").expect("write secret file");
        }
        assert!(state.sync_credentials(Some(&path)).is_err());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn handle_sync_rejects_invalid_window_argument() {
        let mut state = super::RuntimeState::new();
        let args = vec![
            "account-id".to_owned(),
            "op-id".to_owned(),
            "--window".to_owned(),
            "invalid".to_owned(),
        ];
        let mut args = args.into_iter();
        let err = super::handle_sync(&mut state, &mut args).unwrap_err();
        assert_eq!(err, "sync window is invalid");
    }

    #[test]
    fn handle_sync_rejects_unknown_argument() {
        let mut state = super::RuntimeState::new();
        let args = vec![
            "account-id".to_owned(),
            "op-id".to_owned(),
            "--mystery-flag".to_owned(),
            "value".to_owned(),
        ];
        let mut args = args.into_iter();
        let err = super::handle_sync(&mut state, &mut args).unwrap_err();
        assert!(err.contains("unknown sync argument"));
    }

    #[test]
    fn handle_sync_rejects_invalid_windows_argument() {
        let mut state = super::RuntimeState::new();
        let args = vec![
            "account-id".to_owned(),
            "op-id".to_owned(),
            "--windows".to_owned(),
            "invalid".to_owned(),
        ];
        let mut args = args.into_iter();
        let err = super::handle_sync(&mut state, &mut args).unwrap_err();
        assert_eq!(err, "sync windows is invalid");
    }

    #[test]
    fn handle_sync_accepts_max_windows_but_rejects_windows_over_limit() {
        let mut state = super::RuntimeState::new();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let path = format!("/tmp/hermes-mail-secret-{}-accept.txt", nanos);
        {
            let mut file = fs::File::create(&path).expect("create secret file");
            file.write_all(b"secret-pass").expect("write secret file");
        }

        let args = vec![
            "account-id".to_owned(),
            "op-id".to_owned(),
            "--windows".to_owned(),
            MAX_WINDOWS.to_string(),
            "--password-file".to_owned(),
            path.clone(),
        ];
        let mut args = args.into_iter();
        assert_eq!(
            super::handle_sync(&mut state, &mut args).unwrap_err(),
            "connection is not provisioned"
        );

        let begin_result = state.begin_connection(super::BeginImapConnection {
            connection_id: "account-id".to_owned(),
            host: "localhost".to_owned(),
            port: 993,
            username: "alice".to_owned(),
            use_tls: true,
        });
        assert!(begin_result.is_ok());

        let args = vec![
            "account-id".to_owned(),
            "op-id".to_owned(),
            "--windows".to_owned(),
            (MAX_WINDOWS + 1).to_string(),
            "--password-file".to_owned(),
            path.clone(),
        ];
        let mut args = args.into_iter();
        let err = super::handle_sync(&mut state, &mut args).unwrap_err();
        assert_eq!(err, "invalid sync plan");

        let _ = fs::remove_file(&path);
    }
}
