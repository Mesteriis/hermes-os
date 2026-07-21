use std::process::Command;
use std::{env, path::PathBuf};

fn runtime_binary_name(name: &str) -> String {
    if let Ok(binary) = env::var(format!("CARGO_BIN_EXE_{name}")) {
        return binary;
    }
    let exe = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_owned()
    };
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../../target/debug");
    path.push(exe);
    path.to_string_lossy().into_owned()
}

#[test]
fn status_command_is_printable() {
    let binary = runtime_binary_name("hermes-mail-runtime");
    let output = Command::new(binary)
        .args(["status"])
        .output()
        .expect("run runtime");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("mail_runtime_status"));
}

#[test]
fn begin_and_sync_commands_are_printable() {
    let binary = runtime_binary_name("hermes-mail-runtime");
    let output = Command::new(&binary)
        .args(["begin", "conn-1", "mail.example.com", "alice"])
        .output()
        .expect("run runtime");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("begin_ok"));

    let output = Command::new(&binary)
        .args(["sync", "conn-1", "op-1"])
        .output()
        .expect("run runtime");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("sync_started op=op-1"));
}

#[test]
fn sync_supports_emit_observations_flag() {
    let mail_binary = runtime_binary_name("hermes-mail-runtime");
    let communications_binary = runtime_binary_name("hermes-communications-runtime");

    let output = Command::new(&mail_binary)
        .args(["begin", "conn-observations", "mail.example.com", "alice"])
        .output()
        .expect("run runtime");
    assert!(output.status.success());

    let output = Command::new(mail_binary)
        .arg("sync")
        .arg("conn-observations")
        .arg("op-observations")
        .arg("--emit-observations")
        .env("HERMES_ENABLE_MAIL_COMMUNICATION_INGEST", "1")
        .env(
            "CARGO_BIN_EXE_hermes_communications_runtime",
            &communications_binary,
        )
        .output()
        .expect("run runtime");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("sync_started op=op-observations"));
}

#[test]
fn sync_rejects_unknown_or_excess_arguments() {
    let binary = runtime_binary_name("hermes-mail-runtime");
    let output = Command::new(&binary)
        .args(["sync", "conn-1", "op-2", "--bad-flag"])
        .output()
        .expect("run runtime");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let output_text = format!("{stderr}{stdout}");
    assert!(output_text.contains("unknown sync argument"));

    let output = Command::new(&binary)
        .args([
            "sync",
            "conn-1",
            "op-2",
            "--emit-observations",
            "--too-many",
        ])
        .output()
        .expect("run runtime");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let output_text = format!("{stderr}{stdout}");
    assert!(output_text.contains("too many sync arguments"));
}

#[test]
fn simulate_requires_valid_command_state() {
    let binary = runtime_binary_name("hermes-mail-runtime");
    let output = Command::new(binary)
        .args(["simulate"])
        .output()
        .expect("run runtime");
    assert!(output.status.success());
}
