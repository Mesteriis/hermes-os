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
fn ingest_command_works() {
    let binary = runtime_binary_name("hermes-communications-runtime");
    let output = Command::new(binary)
        .args([
            "ingest",
            "op-1",
            "mail-imap",
            "source-1",
            "preview body",
            "true",
            "true",
        ])
        .output()
        .expect("run runtime");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("communications_runtime_ingest ok"));
}

#[test]
fn status_command_works() {
    let binary = runtime_binary_name("hermes-communications-runtime");
    let output = Command::new(binary)
        .args(["status"])
        .output()
        .expect("run runtime");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("communications_runtime status ok"));
}
