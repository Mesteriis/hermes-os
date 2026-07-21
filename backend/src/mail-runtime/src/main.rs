//! Mail runtime composition for ADR-0239.

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use hermes_mail_api::{
    BeginImapConnection, CompleteImapConnection, GetConnection, GetOperationStatus, MailConnection,
    MailConnectionId, MailConnectionState, MailOperation, MailOperationId, SyncNow,
};
use hermes_mail_core::{
    ConnectionTracker, SyncPlan, bounded_window, draft_ingress_observation, validate_sync_request,
};
use hermes_mail_imap::synthetic_inbox;
use hermes_mail_persistence::{MailPersistence, PersistedMailConnection};

const COMMUNICATIONS_RUNTIME_PROCESS_ID: &str = "hermes-communications-runtime";
const DEFAULT_COMMUNICATION_INGEST_LIMIT: usize = 3;

struct RuntimeState {
    persistence: MailPersistence,
    tracker: ConnectionTracker,
    operations: HashMap<MailOperationId, MailOperation>,
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

    fn sync_now(&mut self, request: SyncNow) -> Result<String, String> {
        self.sync_now_with_observations(request, false)
    }

    fn sync_now_with_observations(
        &mut self,
        request: SyncNow,
        emit_observations: bool,
    ) -> Result<String, String> {
        let connection = self
            .persistence
            .get_connection(&request.connection_id)
            .ok_or_else(|| "connection is not provisioned".to_owned())?;
        let plan = bounded_window(100, 1).map_err(|_| "invalid sync plan".to_owned())?;
        let operation = MailOperation {
            operation_id: request.operation_id.clone(),
            state: MailConnectionState::Syncing,
            window_size: plan.window,
        };
        self.tracker
            .set_syncing(&request.connection_id, operation.clone());
        self.operations
            .insert(request.operation_id.clone(), operation.clone());
        let synthetic = synthetic_inbox(&connection.host, plan.window, 1);
        if emit_observations {
            self.emit_ingress_observations(
                &request.operation_id,
                &connection.id,
                &synthetic.messages,
            )?;
        } else {
            draft_ingress_observation(
                &request.operation_id,
                "mail-imap",
                connection.id.clone(),
                synthetic.messages.len(),
            )
            .map_err(|_| "invalid ingress draft".to_owned())?;
        }
        self.tracker.set_ready(&request.connection_id);
        Ok(format!(
            "sync_started op={} host={} window={} messages={}",
            request.operation_id,
            connection.host,
            plan.window,
            synthetic.messages.len()
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
        let mut emitted = 0usize;
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
            emitted += 1;
        }
        if emitted == 0 {
            return Err("no observations emitted".to_owned());
        }
        Ok(())
    }

    fn complete_connection(&mut self, request: CompleteImapConnection) -> Result<String, String> {
        self.tracker.set_ready(&request.connection_id);
        if let Some(op) = self.operations.get_mut(&request.operation_id) {
            op.state = MailConnectionState::Ready;
            Ok(format!(
                "connection_ready id={} op={}",
                request.connection_id, request.operation_id
            ))
        } else {
            Err("operation id is unknown".to_owned())
        }
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
}

fn connection_to_persistence(connection: MailConnection) -> PersistedMailConnection {
    PersistedMailConnection {
        id: connection.id,
        host: connection.host,
        port: connection.port,
        username: connection.username,
    }
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
        Some("simulate") => handle_simulate(&mut state),
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
    let port = args
        .next()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(hermes_mail_api::IMAP_PORT);
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
    let emit_observations = if let Some(value) = args.next() {
        match value.as_str() {
            "--emit-observations" => true,
            other => return Err(format!("unknown sync argument: {other}")),
        }
    } else {
        false
    };
    if args.next().is_some() {
        return Err("too many sync arguments".to_owned());
    }
    if state.persistence.get_connection(&connection_id).is_none() {
        state.begin_connection(BeginImapConnection {
            connection_id: connection_id.clone(),
            host: "mail.example.com".to_owned(),
            port: hermes_mail_api::IMAP_PORT,
            username: "auto".to_owned(),
            use_tls: true,
        })?;
    }
    let response = state.sync_now_with_observations(
        SyncNow {
            connection_id,
            operation_id,
        },
        emit_observations,
    )?;
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

fn handle_simulate(state: &mut RuntimeState) -> Result<(), String> {
    let plan = bounded_window(100, 1).map_err(|_| "invalid sync window".to_owned())?;
    run_simulated_sync(plan, state, "conn".to_owned())
}

fn run_simulated_sync(
    plan: SyncPlan,
    state: &mut RuntimeState,
    connection_id: MailConnectionId,
) -> Result<(), String> {
    if plan.window > 500 {
        return Err("window is not supported".to_owned());
    }
    state.begin_connection(BeginImapConnection {
        connection_id: connection_id.clone(),
        host: "mail.example.com".to_owned(),
        port: hermes_mail_api::IMAP_PORT,
        username: "sim-user".to_owned(),
        use_tls: true,
    })?;
    let sync_now = SyncNow {
        connection_id,
        operation_id: "sim-op".to_owned(),
    };
    let response = state.sync_now(sync_now)?;
    println!("{response}");
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
