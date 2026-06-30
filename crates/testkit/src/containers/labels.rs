use std::process;

pub const SESSION_ID_ENV: &str = "HERMES_TEST_SESSION_ID";
pub const TESTKIT_LABEL: &str = "com.hermes.testkit";
pub const TESTKIT_SERVICE_LABEL: &str = "com.hermes.testkit.service";
pub const TESTKIT_SESSION_LABEL: &str = "com.hermes.testkit.session";

pub fn session_id_label_value() -> String {
    std::env::var(SESSION_ID_ENV)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| format!("hermes-test-standalone-{}", process::id()))
}

pub fn testkit_labels(service: &'static str, session_id: &str) -> [(&'static str, String); 3] {
    [
        (TESTKIT_LABEL, "true".to_owned()),
        (TESTKIT_SERVICE_LABEL, service.to_owned()),
        (TESTKIT_SESSION_LABEL, session_id.to_owned()),
    ]
}
