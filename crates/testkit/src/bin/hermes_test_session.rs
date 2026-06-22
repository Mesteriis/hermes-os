use std::env;
use std::process::{Command, Stdio};

use testkit::containers::postgres::{
    PostgresContainer, SESSION_ID_ENV, SESSION_POSTGRES_HOST_PORT_ENV,
};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let command_args = env::args().skip(1).collect::<Vec<_>>();
    if command_args.is_empty() {
        eprintln!("usage: hermes-test-session <command> [args...]");
        std::process::exit(2);
    }

    let session_id = format!("hermes-test-{}", Uuid::new_v4());
    let container = PostgresContainer::start_owned().await;
    let status = Command::new(&command_args[0])
        .args(&command_args[1..])
        .env(SESSION_ID_ENV, &session_id)
        .env(
            SESSION_POSTGRES_HOST_PORT_ENV,
            container.host_port().to_string(),
        )
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .unwrap_or_else(|error| {
            panic!(
                "failed to run test session command '{}': {error}",
                command_args[0]
            )
        });

    drop(container);
    std::process::exit(status.code().unwrap_or(1));
}
