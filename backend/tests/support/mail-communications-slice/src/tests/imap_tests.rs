use hermes_mail_imap::{ImapMessage, supports_read_only_sync, synthetic_inbox};

#[test]
fn synthetic_inbox_is_deterministic() {
    let first = synthetic_inbox("mail.example.com", 10, 2);
    let second = synthetic_inbox("mail.example.com", 10, 2);
    assert_eq!(first.messages.len(), second.messages.len());
    assert_eq!(first.messages, second.messages);
    assert_eq!(first.messages.first(), second.messages.first());
    assert!(!first.has_more);
    assert_eq!(first.attempts, 1);
    assert!(
        first
            .messages
            .iter()
            .all(|message| std::mem::size_of_val(message) > 0)
    );
}

#[test]
fn supports_only_read_windows() {
    assert!(supports_read_only_sync(100));
    assert!(!supports_read_only_sync(0));
    assert!(!supports_read_only_sync(501));
    assert!(
        ImapMessage {
            uid: 1,
            subject: "s".to_owned(),
            snippet: "p".to_owned(),
            has_plain_text: true,
        } != ImapMessage {
            uid: 2,
            subject: "s".to_owned(),
            snippet: "p".to_owned(),
            has_plain_text: true,
        }
    );
}
