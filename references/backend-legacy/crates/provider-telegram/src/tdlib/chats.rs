use serde_json::{Value, json};

pub fn load_chats(limit: i32, extra: &str) -> Value {
    json!({"@type": "loadChats", "chat_list": null, "limit": page_limit(limit), "@extra": extra.trim()})
}

pub fn get_chats(limit: i32, extra: &str) -> Value {
    json!({"@type": "getChats", "chat_list": null, "limit": page_limit(limit), "@extra": extra.trim()})
}

pub fn get_chat(chat_id: i64, extra: &str) -> Value {
    json!({"@type": "getChat", "chat_id": chat_id, "@extra": extra.trim()})
}

pub fn get_basic_group(basic_group_id: i64, extra: &str) -> Value {
    json!({"@type": "getBasicGroup", "basic_group_id": basic_group_id, "@extra": extra.trim()})
}

pub fn get_basic_group_full_info(basic_group_id: i64, extra: &str) -> Value {
    json!({"@type": "getBasicGroupFullInfo", "basic_group_id": basic_group_id, "@extra": extra.trim()})
}

pub fn get_chat_folder(chat_folder_id: i64, extra: &str) -> Value {
    json!({"@type": "getChatFolder", "chat_folder_id": chat_folder_id, "@extra": extra.trim()})
}

pub fn get_chat_history(
    chat_id: i64,
    from_message_id: Option<i64>,
    limit: i32,
    only_local: bool,
    extra: &str,
) -> Value {
    json!({
        "@type": "getChatHistory", "chat_id": chat_id,
        "from_message_id": from_message_id.unwrap_or(0), "offset": 0, "limit": page_limit(limit),
        "only_local": only_local, "@extra": extra.trim()
    })
}

pub fn download_file(file_id: i64, priority: i32, extra: &str) -> Value {
    json!({
        "@type": "downloadFile", "file_id": file_id, "priority": priority.clamp(1, 32),
        "offset": 0, "limit": 0, "synchronous": true, "@extra": extra.trim()
    })
}

pub fn toggle_marked_as_unread(chat_id: i64, is_marked_as_unread: bool, extra: &str) -> Value {
    json!({
        "@type": "toggleChatIsMarkedAsUnread", "chat_id": chat_id,
        "is_marked_as_unread": is_marked_as_unread, "@extra": extra.trim()
    })
}

pub fn add_chat_to_list(chat_id: i64, archived: bool, extra: &str) -> Value {
    let chat_list_type = if archived {
        "chatListArchive"
    } else {
        "chatListMain"
    };
    json!({
        "@type": "addChatToList", "chat_id": chat_id,
        "chat_list": {"@type": chat_list_type}, "@extra": extra.trim()
    })
}

pub fn add_chat_to_folder(chat_id: i64, chat_folder_id: i64, extra: &str) -> Value {
    json!({
        "@type": "addChatToList", "chat_id": chat_id,
        "chat_list": {"@type": "chatListFolder", "chat_folder_id": chat_folder_id},
        "@extra": extra.trim()
    })
}

pub fn set_chat_mute(chat_id: i64, muted: bool, extra: &str) -> Value {
    json!({
        "@type": "setChatNotificationSettings", "chat_id": chat_id,
        "notification_settings": {
            "@type": "chatNotificationSettings", "use_default_mute_for": !muted,
            "mute_for": if muted { 31_708_800 } else { 0 }, "use_default_sound": true,
            "sound_id": 0, "use_default_show_preview": true, "show_preview": true,
            "use_default_mute_stories": true, "mute_stories": false,
            "use_default_story_sound": true, "story_sound_id": 0,
            "use_default_show_story_poster": true, "show_story_poster": true,
            "use_default_disable_pinned_message_notifications": true,
            "disable_pinned_message_notifications": false,
            "use_default_disable_mention_notifications": true,
            "disable_mention_notifications": false
        }, "@extra": extra.trim()
    })
}

pub fn join_chat(chat_id: i64, extra: &str) -> Value {
    json!({"@type": "joinChat", "chat_id": chat_id, "@extra": extra.trim()})
}

pub fn leave_chat(chat_id: i64, extra: &str) -> Value {
    json!({"@type": "leaveChat", "chat_id": chat_id, "@extra": extra.trim()})
}

pub fn search_messages(query: &str, limit: i32, extra: &str) -> Value {
    json!({
        "@type": "searchMessages", "chat_list": {"@type": "chatListMain"}, "query": query.trim(),
        "offset_date": 0, "offset_chat_id": 0, "offset_message_id": 0, "limit": page_limit(limit),
        "filter": {"@type": "searchMessagesFilterEmpty"}, "@extra": extra.trim()
    })
}

pub fn search_chat_messages(chat_id: i64, query: &str, limit: i32, extra: &str) -> Value {
    json!({
        "@type": "searchChatMessages", "chat_id": chat_id, "query": query.trim(),
        "sender_id": null, "from_message_id": 0, "offset": 0, "limit": page_limit(limit),
        "filter": {"@type": "searchMessagesFilterEmpty"}, "@extra": extra.trim()
    })
}

pub fn get_supergroup_members(supergroup_id: i64, offset: i32, limit: i32, extra: &str) -> Value {
    get_supergroup_members_with_filter(
        supergroup_id,
        "supergroupMembersFilterRecent",
        offset,
        limit,
        extra,
    )
}

pub fn get_supergroup_administrators(
    supergroup_id: i64,
    offset: i32,
    limit: i32,
    extra: &str,
) -> Value {
    get_supergroup_members_with_filter(
        supergroup_id,
        "supergroupMembersFilterAdministrators",
        offset,
        limit,
        extra,
    )
}

fn get_supergroup_members_with_filter(
    supergroup_id: i64,
    filter_type: &str,
    offset: i32,
    limit: i32,
    extra: &str,
) -> Value {
    json!({
        "@type": "getSupergroupMembers", "supergroup_id": supergroup_id,
        "filter": {"@type": filter_type}, "offset": offset.max(0), "limit": page_limit(limit),
        "@extra": extra.trim()
    })
}

fn page_limit(limit: i32) -> i32 {
    limit.clamp(1, 100)
}
