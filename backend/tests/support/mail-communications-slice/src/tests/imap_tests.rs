use hermes_mail_api::IMAP_PORT;
use hermes_mail_api::MAX_WINDOW;
use hermes_mail_imap::{ImapMessage, supports_read_only_sync, sync_inbox};

#[test]
fn sync_requires_password() {
    let result = sync_inbox("mail.example.com", IMAP_PORT, "user", None, MAX_WINDOW, 1);
    assert!(matches!(result, Err(error) if error == "imap password is required"));
}

#[test]
fn supports_only_read_windows() {
    assert!(supports_read_only_sync(MAX_WINDOW));
    assert!(!supports_read_only_sync(0));
    assert!(!supports_read_only_sync(MAX_WINDOW + 1));
    assert!(
        ImapMessage {
            uid: 1,
            subject: "s".to_owned(),
            snippet: "p".to_owned(),
            has_plain_text: true,
            plain_text_body: None,
            attachments: Vec::new(),
        } != ImapMessage {
            uid: 2,
            subject: "s".to_owned(),
            snippet: "p".to_owned(),
            has_plain_text: true,
            plain_text_body: None,
            attachments: Vec::new(),
        }
    );
}
