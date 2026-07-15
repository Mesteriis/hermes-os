use crate::integrations::telegram::tdjson::snapshots::TelegramTdlibMessageSnapshot;

pub(in crate::integrations::telegram::runtime) fn oldest_tdlib_message_id(
    snapshots: &[TelegramTdlibMessageSnapshot],
) -> Option<i64> {
    snapshots
        .iter()
        .filter_map(|snapshot| snapshot.provider_message_id.trim().parse::<i64>().ok())
        .min()
}

pub(super) fn short_thread_suffix(account_id: &str) -> String {
    let sanitized = account_id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_owned();
    if sanitized.is_empty() {
        "account".to_owned()
    } else {
        sanitized.chars().take(32).collect()
    }
}
