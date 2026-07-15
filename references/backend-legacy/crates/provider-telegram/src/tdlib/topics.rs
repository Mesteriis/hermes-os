use serde_json::{Value, json};

use super::types::TdlibProtocolError;

pub fn get_forum_topics(chat_id: i64, limit: i32, extra: &str) -> Value {
    json!({
        "@type": "getForumTopics", "chat_id": chat_id, "query": "", "offset_date": 0,
        "offset_message_id": 0, "offset_message_thread_id": 0, "limit": limit.clamp(1, 100),
        "@extra": extra.trim()
    })
}

pub fn create_forum_topic(
    chat_id: i64,
    title: &str,
    extra: &str,
) -> Result<Value, TdlibProtocolError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(TdlibProtocolError::InvalidCommand(
            "forum topic title must not be empty",
        ));
    }
    Ok(json!({
        "@type": "createForumTopic", "chat_id": chat_id, "name": title,
        "icon_custom_emoji_id": 0, "@extra": extra.trim()
    }))
}

pub fn toggle_forum_topic_closed(
    chat_id: i64,
    message_thread_id: i64,
    is_closed: bool,
    extra: &str,
) -> Value {
    json!({
        "@type": "toggleForumTopicIsClosed", "chat_id": chat_id,
        "message_thread_id": message_thread_id, "is_closed": is_closed, "@extra": extra.trim()
    })
}
