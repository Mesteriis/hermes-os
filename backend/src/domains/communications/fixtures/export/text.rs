pub(super) fn normalize_body_text(input: &str) -> String {
    input
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_owned()
}

pub(super) fn non_empty_or_default(value: String, fallback: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        fallback.to_owned()
    } else {
        value.to_owned()
    }
}

pub(super) fn non_empty_recipients(recipients: Vec<String>) -> Vec<String> {
    let recipients = recipients
        .into_iter()
        .map(|recipient| recipient.trim().to_owned())
        .filter(|recipient| !recipient.is_empty())
        .collect::<Vec<_>>();
    if recipients.is_empty() {
        vec!["recipient-unknown@example.invalid".to_owned()]
    } else {
        recipients
    }
}
