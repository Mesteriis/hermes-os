use std::collections::HashMap;

use crate::integrations::telegram::client::models::messages::{
    TelegramReaction, TelegramReactionGroup, TelegramReactionSummary,
};

pub(crate) fn canonical_reaction_summary(
    message_id: &str,
    reactions: &[TelegramReaction],
) -> TelegramReactionSummary {
    let total_reactions = reactions.len() as i64;
    let mut groups: HashMap<String, Vec<String>> = HashMap::new();
    for reaction in reactions {
        groups
            .entry(reaction.reaction_emoji.clone())
            .or_default()
            .push(
                reaction
                    .sender_display_name
                    .clone()
                    .unwrap_or_else(|| reaction.sender_id.clone()),
            );
    }
    let grouped_reactions = groups
        .into_iter()
        .map(|(reaction_emoji, senders)| TelegramReactionGroup {
            reaction_emoji,
            count: senders.len() as i64,
            senders,
        })
        .collect();
    TelegramReactionSummary {
        message_id: message_id.to_owned(),
        total_reactions,
        active_reactions: total_reactions,
        reactions: grouped_reactions,
    }
}
