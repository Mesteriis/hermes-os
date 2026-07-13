use hermes_provider_telegram::tdlib::{edit_message_text, send_reply, send_text_message};

#[test]
fn send_and_reply_use_tdlib_formatted_text_contract() {
    let send = send_text_message(10, " hello ", "send-1").expect("send request");
    assert_eq!(send["input_message_content"]["text"]["text"], "hello");
    assert_eq!(send["input_message_content"]["clear_draft"], true);

    let reply = send_reply(10, 11, "reply", "reply-1").expect("reply request");
    assert_eq!(reply["reply_to"]["message_id"], 11);
}

#[test]
fn edit_rejects_empty_text_without_a_provider_call() {
    assert!(edit_message_text(10, 11, " ", "edit-1").is_err());
}
