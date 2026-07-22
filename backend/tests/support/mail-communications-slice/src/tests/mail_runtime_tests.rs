use hermes_mail_api::MAX_WINDOWS;
use hermes_mail_imap::MAX_ATTEMPTS;
use std::fs;
use std::io::Write;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
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
    let password_file = write_mail_test_password_file("secret");
    let binary = runtime_binary_name("hermes-mail-runtime");
    let output = Command::new(&binary)
        .args(["begin", "conn-1", "mail.example.com", "alice"])
        .output()
        .expect("run runtime");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("begin_ok"));

    let output = Command::new(&binary)
        .args([
            "sync",
            "conn-1",
            "op-1",
            "--host",
            "mail.example.com",
            "--username",
            "alice",
            "--port",
            "993",
            "--password-file",
            password_file.as_str(),
        ])
        .output()
        .expect("run runtime");
    assert!(!output.status.success());
    let output_text = {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        format!("{stderr}{stdout}")
    };
    assert!(!output_text.trim().is_empty());
    let _ = fs::remove_file(password_file);
}

#[test]
fn sync_supports_emit_observations_flag() {
    let mail_binary = runtime_binary_name("hermes-mail-runtime");
    let communications_binary = runtime_binary_name("hermes-communications-runtime");
    let password_file = write_mail_test_password_file("secret");

    let output = Command::new(&mail_binary)
        .args(["begin", "conn-observations", "mail.example.com", "alice"])
        .output()
        .expect("run runtime");
    assert!(output.status.success());

    let output = Command::new(mail_binary)
        .arg("sync")
        .arg("conn-observations")
        .arg("op-observations")
        .arg("--host")
        .arg("mail.example.com")
        .arg("--username")
        .arg("alice")
        .arg("--port")
        .arg("993")
        .arg("--emit-observations")
        .arg("--password-file")
        .arg(password_file.as_str())
        .env("HERMES_ENABLE_MAIL_COMMUNICATION_INGEST", "1")
        .env(
            "CARGO_BIN_EXE_hermes_communications_runtime",
            &communications_binary,
        )
        .output()
        .expect("run runtime");
    assert!(!output.status.success());
    let output_text = {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        format!("{stderr}{stdout}")
    };
    assert!(!output_text.trim().is_empty());
    let _ = fs::remove_file(password_file);
}

#[test]
fn sync_rejects_unknown_or_excess_arguments() {
    let binary = runtime_binary_name("hermes-mail-runtime");
    let password_file = write_mail_test_password_file("secret");
    let output = Command::new(&binary)
        .args([
            "sync",
            "conn-1",
            "op-2",
            "--password-file",
            password_file.as_str(),
            "--bad-flag",
        ])
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
            "--password-file",
            password_file.as_str(),
            "--emit-observations",
            "--too-many",
        ])
        .output()
        .expect("run runtime");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let output_text = format!("{stderr}{stdout}");
    assert!(output_text.contains("unknown sync argument"));
    let _ = fs::remove_file(&password_file);
}

#[test]
fn sync_rejects_windows_above_policy_limit() {
    let binary = runtime_binary_name("hermes-mail-runtime");
    let password_file = write_mail_test_password_file("secret");
    let windows_over_limit = (MAX_WINDOWS + 1).to_string();
    let output = Command::new(&binary)
        .args([
            "sync",
            "conn-1",
            "op-window-overflow",
            "--host",
            "mail.example.com",
            "--username",
            "alice",
            "--port",
            "993",
            "--windows",
            windows_over_limit.as_str(),
            "--password-file",
            password_file.as_str(),
        ])
        .output()
        .expect("run runtime");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let output_text = format!("{stderr}{stdout}");
    assert!(output_text.contains("invalid sync plan"));
    let _ = fs::remove_file(&password_file);
}

#[test]
fn sync_windows_is_clamped_to_minimum_one() {
    let binary = runtime_binary_name("hermes-mail-runtime");
    let password_file = write_mail_test_password_file("secret");
    let output = Command::new(&binary)
        .args([
            "sync",
            "conn-1",
            "op-window-zero",
            "--host",
            "mail.example.com",
            "--username",
            "alice",
            "--port",
            "993",
            "--windows",
            "0",
            "--password-file",
            password_file.as_str(),
        ])
        .output()
        .expect("run runtime");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let output_text = format!("{stderr}{stdout}");
    assert!(output_text.contains("imap sync attempt 1 failed"));
    let _ = fs::remove_file(&password_file);
}

#[test]
fn sync_command_rejects_legacy_password_argument() {
    let binary = runtime_binary_name("hermes-mail-runtime");
    let password_file = write_mail_test_password_file("secret");
    let output = Command::new(&binary)
        .args([
            "sync",
            "conn-1",
            "op-legacy",
            "--password",
            "legacy-secret",
            "--password-file",
            password_file.as_str(),
        ])
        .output()
        .expect("run runtime");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let output_text = format!("{stderr}{stdout}");
    assert!(output_text.contains("unknown sync argument"));
    let _ = fs::remove_file(password_file);
}

#[test]
fn sync_retries_until_imap_attempt_limit() {
    let binary = runtime_binary_name("hermes-mail-runtime");
    let password_file = write_mail_test_password_file("secret");
    let output = Command::new(&binary)
        .args([
            "sync",
            "conn-1",
            "op-attempts",
            "--host",
            "127.0.0.1",
            "--username",
            "alice",
            "--port",
            "1",
            "--password-file",
            password_file.as_str(),
        ])
        .output()
        .expect("run runtime");
    assert!(!output.status.success());

    let output_text = {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        format!("{stderr}{stdout}")
    };
    assert!(output_text.contains("imap sync failed:"));
    assert!(output_text.contains("imap sync attempt 1 failed"));
    assert!(output_text.contains(&format!("imap sync attempt {} failed", MAX_ATTEMPTS - 1)));
    assert!(output_text.contains(&format!("imap sync attempt {} failed", MAX_ATTEMPTS)));
    let _ = fs::remove_file(password_file);
}

fn write_mail_test_password_file(secret: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let path = format!("/tmp/hermes-mail-test-secret-{nanos}.txt");
    let mut file = fs::File::create(&path).expect("create test password file");
    file.write_all(secret.as_bytes())
        .expect("write test password");
    path
}

#[test]
fn simulate_command_is_no_longer_supported() {
    let binary = runtime_binary_name("hermes-mail-runtime");
    let output = Command::new(binary)
        .args(["simulate"])
        .output()
        .expect("run runtime");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let output_text = format!("{stderr}{stdout}");
    assert!(output_text.contains("mail runtime command is unavailable"));
}
