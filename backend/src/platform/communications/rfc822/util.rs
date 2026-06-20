pub(crate) fn split_address_list(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .collect()
}

pub(crate) fn non_empty_or_default(value: String, default: &str) -> String {
    let value = value.trim().to_owned();
    if value.is_empty() {
        default.to_owned()
    } else {
        value
    }
}

pub(crate) fn non_empty_recipients(recipients: Vec<String>) -> Vec<String> {
    if recipients.is_empty() {
        vec!["unknown@example.invalid".to_owned()]
    } else {
        recipients
    }
}
