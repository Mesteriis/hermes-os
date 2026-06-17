use serde_json::{Value, json};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::integrations::telegram) struct TdlibProviderReaction {
    pub(in crate::integrations::telegram) sender_id: String,
    pub(in crate::integrations::telegram) reaction_emoji: String,
    pub(in crate::integrations::telegram) is_outgoing: bool,
}

pub(in crate::integrations::telegram) fn derive_tdlib_reaction_summary_metadata(
    raw: &Value,
) -> Option<Value> {
    let reactions = raw
        .get("interaction_info")
        .and_then(|value| value.get("reactions"))
        .and_then(|value| value.get("reactions"))
        .and_then(Value::as_array)?;

    let groups = reactions
        .iter()
        .filter_map(tdlib_reaction_group)
        .collect::<Vec<_>>();
    let custom_groups = reactions
        .iter()
        .filter_map(tdlib_custom_reaction_group)
        .collect::<Vec<_>>();
    if groups.is_empty() && custom_groups.is_empty() {
        return None;
    }

    let total_reactions = groups
        .iter()
        .chain(custom_groups.iter())
        .filter_map(|group| group.get("count").and_then(Value::as_i64))
        .sum::<i64>();

    Some(json!({
        "source": "tdlib_interaction_info",
        "total_reactions": total_reactions,
        "active_reactions": total_reactions,
        "reactions": groups,
        "custom_reactions": custom_groups,
    }))
}

pub(in crate::integrations::telegram) fn derive_tdlib_provider_reactions(
    raw: &Value,
) -> Vec<TdlibProviderReaction> {
    raw.get("interaction_info")
        .and_then(|value| value.get("reactions"))
        .and_then(|value| value.get("recent_reactions"))
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(tdlib_provider_reaction)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub(in crate::integrations::telegram) fn derive_tdlib_chosen_reaction_emojis(
    raw: &Value,
) -> Vec<String> {
    let mut chosen = raw
        .get("interaction_info")
        .and_then(|value| value.get("reactions"))
        .and_then(|value| value.get("reactions"))
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter(|reaction| {
                    reaction
                        .get("is_chosen")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                })
                .filter_map(tdlib_reaction_emoji)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    chosen.sort();
    chosen.dedup();
    chosen
}

fn tdlib_provider_reaction(reaction: &Value) -> Option<TdlibProviderReaction> {
    Some(TdlibProviderReaction {
        sender_id: tdlib_sender_id(reaction.get("sender_id")?)?,
        reaction_emoji: tdlib_reaction_emoji(reaction)?,
        is_outgoing: reaction
            .get("is_outgoing")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

fn tdlib_reaction_group(reaction: &Value) -> Option<Value> {
    let emoji = tdlib_reaction_emoji(reaction)?;
    let count = reaction
        .get("total_count")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    if count <= 0 {
        return None;
    }

    Some(json!({
        "reaction_emoji": emoji,
        "count": count,
        "senders": [],
        "is_chosen": reaction.get("is_chosen").and_then(Value::as_bool).unwrap_or(false),
        "source": "tdlib_interaction_info",
    }))
}

fn tdlib_custom_reaction_group(reaction: &Value) -> Option<Value> {
    let custom_emoji_id = reaction
        .get("type")
        .and_then(|value| value.get("@type"))
        .and_then(Value::as_str)
        .filter(|reaction_type| *reaction_type == "reactionTypeCustomEmoji")
        .and_then(|_| reaction.get("type"))
        .and_then(|value| {
            value
                .get("custom_emoji_id")
                .and_then(Value::as_i64)
                .map(|number| number.to_string())
                .or_else(|| {
                    value
                        .get("custom_emoji_id")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned)
                })
        })
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())?;
    let count = reaction
        .get("total_count")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    if count <= 0 {
        return None;
    }

    Some(json!({
        "custom_emoji_id": custom_emoji_id,
        "count": count,
        "senders": [],
        "is_chosen": reaction.get("is_chosen").and_then(Value::as_bool).unwrap_or(false),
        "source": "tdlib_interaction_info",
    }))
}

fn tdlib_reaction_emoji(reaction: &Value) -> Option<String> {
    reaction
        .get("type")
        .and_then(|value| value.get("@type"))
        .and_then(Value::as_str)
        .filter(|reaction_type| *reaction_type == "reactionTypeEmoji")
        .and_then(|_| reaction.get("type"))
        .and_then(|value| value.get("emoji"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn tdlib_sender_id(sender: &Value) -> Option<String> {
    match sender.get("@type").and_then(Value::as_str)? {
        "messageSenderUser" => tdlib_id(sender, "user_id").map(|id| format!("user:{id}")),
        "messageSenderChat" => tdlib_id(sender, "chat_id").map(|id| format!("chat:{id}")),
        _ => None,
    }
}

fn tdlib_id(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(|value| {
            value
                .as_i64()
                .map(|number| number.to_string())
                .or_else(|| value.as_str().map(ToOwned::to_owned))
        })
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .filter(|value| value != "0")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        derive_tdlib_chosen_reaction_emojis, derive_tdlib_provider_reactions,
        derive_tdlib_reaction_summary_metadata,
    };

    #[test]
    fn derives_tdlib_emoji_reaction_summary_from_interaction_info() {
        let summary = derive_tdlib_reaction_summary_metadata(&json!({
            "@type": "message",
            "interaction_info": {
                "@type": "messageInteractionInfo",
                "reactions": {
                    "@type": "messageReactions",
                    "reactions": [
                        {
                            "@type": "messageReaction",
                            "type": {
                                "@type": "reactionTypeEmoji",
                                "emoji": "👍"
                            },
                            "total_count": 3,
                            "is_chosen": true
                        },
                        {
                            "@type": "messageReaction",
                            "type": {
                                "@type": "reactionTypeEmoji",
                                "emoji": "🔥"
                            },
                            "total_count": 2
                        }
                    ]
                }
            }
        }))
        .expect("reaction summary");

        assert_eq!(summary["source"], "tdlib_interaction_info");
        assert_eq!(summary["total_reactions"], 5);
        assert_eq!(summary["active_reactions"], 5);
        assert_eq!(summary["reactions"][0]["reaction_emoji"], "👍");
        assert_eq!(summary["reactions"][0]["count"], 3);
        assert_eq!(summary["reactions"][0]["is_chosen"], true);
        assert_eq!(summary["reactions"][1]["reaction_emoji"], "🔥");
        assert_eq!(summary["reactions"][1]["senders"], json!([]));
        assert_eq!(summary["custom_reactions"], json!([]));
    }

    #[test]
    fn preserves_custom_tdlib_reaction_summary_without_faking_emoji_contract() {
        let summary = derive_tdlib_reaction_summary_metadata(&json!({
            "@type": "message",
            "interaction_info": {
                "reactions": {
                    "reactions": [
                        {
                            "type": {
                                "@type": "reactionTypeCustomEmoji",
                                "custom_emoji_id": 42
                            },
                            "total_count": 10,
                            "is_chosen": true
                        },
                        {
                            "type": {
                                "@type": "reactionTypeEmoji",
                                "emoji": " "
                            },
                            "total_count": 1
                        },
                        {
                            "type": {
                                "@type": "reactionTypeEmoji",
                                "emoji": "👍"
                            },
                            "total_count": 0
                        }
                    ]
                }
            }
        }));

        let summary = summary.expect("custom reaction summary");
        assert_eq!(summary["total_reactions"], 10);
        assert_eq!(summary["reactions"], json!([]));
        assert_eq!(summary["custom_reactions"][0]["custom_emoji_id"], "42");
        assert_eq!(summary["custom_reactions"][0]["count"], 10);
        assert_eq!(summary["custom_reactions"][0]["is_chosen"], true);
    }

    #[test]
    fn derives_sender_level_tdlib_recent_reactions() {
        let reactions = derive_tdlib_provider_reactions(&json!({
            "@type": "message",
            "interaction_info": {
                "reactions": {
                    "recent_reactions": [
                        {
                            "@type": "messageReaction",
                            "sender_id": {
                                "@type": "messageSenderUser",
                                "user_id": 777
                            },
                            "type": {
                                "@type": "reactionTypeEmoji",
                                "emoji": "👍"
                            },
                            "is_outgoing": true
                        },
                        {
                            "@type": "messageReaction",
                            "sender_id": {
                                "@type": "messageSenderChat",
                                "chat_id": -10042
                            },
                            "type": {
                                "@type": "reactionTypeEmoji",
                                "emoji": "🔥"
                            }
                        },
                        {
                            "@type": "messageReaction",
                            "sender_id": {
                                "@type": "messageSenderUser",
                                "user_id": 999
                            },
                            "type": {
                                "@type": "reactionTypeCustomEmoji",
                                "custom_emoji_id": 42
                            }
                        }
                    ]
                }
            }
        }));

        assert_eq!(reactions.len(), 2);
        assert_eq!(reactions[0].sender_id, "user:777");
        assert_eq!(reactions[0].reaction_emoji, "👍");
        assert!(reactions[0].is_outgoing);
        assert_eq!(reactions[1].sender_id, "chat:-10042");
        assert_eq!(reactions[1].reaction_emoji, "🔥");
        assert!(!reactions[1].is_outgoing);
    }

    #[test]
    fn derives_chosen_emoji_reactions_for_current_actor() {
        let chosen = derive_tdlib_chosen_reaction_emojis(&json!({
            "@type": "message",
            "interaction_info": {
                "reactions": {
                    "reactions": [
                        {
                            "type": {
                                "@type": "reactionTypeEmoji",
                                "emoji": "🔥"
                            },
                            "is_chosen": false
                        },
                        {
                            "type": {
                                "@type": "reactionTypeEmoji",
                                "emoji": "👍"
                            },
                            "is_chosen": true
                        },
                        {
                            "type": {
                                "@type": "reactionTypeCustomEmoji",
                                "custom_emoji_id": 42
                            },
                            "is_chosen": true
                        },
                        {
                            "type": {
                                "@type": "reactionTypeEmoji",
                                "emoji": "👍"
                            },
                            "is_chosen": true
                        }
                    ]
                }
            }
        }));

        assert_eq!(chosen, vec!["👍".to_owned()]);
    }
}
