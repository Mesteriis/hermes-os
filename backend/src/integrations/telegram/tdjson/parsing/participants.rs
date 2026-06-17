use serde_json::{Map, Value, json};

use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::tdjson::snapshots::TelegramTdlibChatMemberSnapshot;

pub(crate) fn parse_tdlib_chat_member_list(
    response: &Value,
) -> Result<Vec<TelegramTdlibChatMemberSnapshot>, TelegramError> {
    parse_member_array(response.get("members"), "TDLib chatMembers response")
}

pub(crate) fn parse_tdlib_basic_group_member_list(
    response: &Value,
) -> Result<Vec<TelegramTdlibChatMemberSnapshot>, TelegramError> {
    parse_member_array(response.get("members"), "TDLib basicGroupFullInfo response")
}

fn parse_member_array(
    members: Option<&Value>,
    response_kind: &str,
) -> Result<Vec<TelegramTdlibChatMemberSnapshot>, TelegramError> {
    let members = members.and_then(Value::as_array).ok_or_else(|| {
        TelegramError::TdlibRuntime(format!("{response_kind} missing `members` array"))
    })?;
    members.iter().map(parse_chat_member).collect()
}

fn parse_chat_member(member: &Value) -> Result<TelegramTdlibChatMemberSnapshot, TelegramError> {
    let member_id = member.get("member_id").ok_or_else(|| {
        TelegramError::TdlibRuntime("chatMember missing `member_id` field".to_owned())
    })?;
    let provider_member_id = provider_member_id(member_id)?;
    let status = member.get("status").and_then(Value::as_object);
    let status_kind = status
        .and_then(|value| value.get("@type"))
        .and_then(Value::as_str)
        .unwrap_or("chatMemberStatusUnknown");
    let role = role_from_status(status_kind).to_owned();
    let permissions = status
        .map(status_permissions)
        .unwrap_or_else(|| json!({ "tdlib_status": status_kind }));

    Ok(TelegramTdlibChatMemberSnapshot {
        provider_member_id,
        display_name: member
            .get("display_name")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned),
        username: member
            .get("username")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned),
        role,
        status: status_kind
            .trim_start_matches("chatMemberStatus")
            .to_lowercase(),
        is_admin: matches!(
            status_kind,
            "chatMemberStatusCreator" | "chatMemberStatusAdministrator"
        ),
        is_owner: status_kind == "chatMemberStatusCreator",
        permissions,
        raw: member.clone(),
    })
}

fn provider_member_id(member_id: &Value) -> Result<String, TelegramError> {
    let member_kind = member_id
        .get("@type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    match member_kind {
        "messageSenderUser" => member_id
            .get("user_id")
            .and_then(Value::as_i64)
            .map(|id| format!("user:{id}"))
            .ok_or_else(|| {
                TelegramError::TdlibRuntime(
                    "messageSenderUser chatMember missing `user_id`".to_owned(),
                )
            }),
        "messageSenderChat" => member_id
            .get("chat_id")
            .and_then(Value::as_i64)
            .map(|id| format!("chat:{id}"))
            .ok_or_else(|| {
                TelegramError::TdlibRuntime(
                    "messageSenderChat chatMember missing `chat_id`".to_owned(),
                )
            }),
        other => Err(TelegramError::TdlibRuntime(format!(
            "unsupported chat member sender kind `{other}`"
        ))),
    }
}

fn role_from_status(status_kind: &str) -> &'static str {
    match status_kind {
        "chatMemberStatusCreator" => "owner",
        "chatMemberStatusAdministrator" => "admin",
        "chatMemberStatusRestricted" => "restricted",
        "chatMemberStatusBanned" => "banned",
        "chatMemberStatusLeft" => "left",
        "chatMemberStatusMember" => "member",
        _ => "unknown",
    }
}

fn status_permissions(status: &Map<String, Value>) -> Value {
    let mut permissions = Map::new();
    for (key, value) in status {
        if key == "@type" {
            continue;
        }
        if value.is_boolean() || value.is_number() || value.is_string() || value.is_object() {
            permissions.insert(key.clone(), value.clone());
        }
    }
    Value::Object(permissions)
}
