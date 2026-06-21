use super::telegram_capabilities::{
    TelegramActionClass, TelegramCapabilityState, TelegramOperationCapability,
};

pub(super) fn push_message_capabilities(
    capabilities: &mut Vec<TelegramOperationCapability>,
    qr_ready: bool,
) {
    push_message_read_capabilities(capabilities);
    push_message_write_capabilities(capabilities, qr_ready);
    push_reply_forward_capabilities(capabilities, qr_ready);
    push_reaction_capabilities(capabilities, qr_ready);
    push_participant_capabilities(capabilities, qr_ready);
    push_topic_capabilities(capabilities, qr_ready);
}

fn push_message_read_capabilities(capabilities: &mut Vec<TelegramOperationCapability>) {
    let cat_msg_read = "messages:read";
    capabilities.push(TelegramOperationCapability::new(
        "messages.list",
        cat_msg_read,
        TelegramCapabilityState::Available,
        TelegramActionClass::Read,
        "Projected message list is available.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "messages.get_versions",
        cat_msg_read,
        TelegramCapabilityState::Available,
        TelegramActionClass::Read,
        "Observed Telegram edit versions are available through the lifecycle history endpoints.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "messages.get_raw_evidence",
        cat_msg_read,
        TelegramCapabilityState::Available,
        TelegramActionClass::Read,
        "Sanitized raw provider evidence view is available.",
        false,
        false,
    ));
}

fn push_message_write_capabilities(
    capabilities: &mut Vec<TelegramOperationCapability>,
    qr_ready: bool,
) {
    let cat_msg_write = "messages:write";
    capabilities.push(TelegramOperationCapability::new(
        "messages.send_text",
        cat_msg_write,
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Degraded
        },
        TelegramActionClass::ProviderWrite,
        if qr_ready {
            "Manual text send through TDLib QR runtime is available."
        } else {
            "Manual text send limited to fixture runtime."
        },
        true,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "messages.send_media",
        cat_msg_write,
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Blocked
        },
        TelegramActionClass::ProviderWrite,
        if qr_ready {
            "Provider-side media upload/send is available through local attachment import and durable outbox."
        } else {
            "Media upload/send requires TDLib QR runtime, local attachment import and durable outbox."
        },
        true,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "messages.edit",
        cat_msg_write,
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Degraded
        },
        TelegramActionClass::ProviderWrite,
        if qr_ready {
            "Append-only edit history and TDLib provider edit execution are available through the command executor."
        } else {
            "Append-only edit history is available locally; provider edit execution requires TDLib QR runtime."
        },
        true,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "messages.delete",
        cat_msg_write,
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Degraded
        },
        TelegramActionClass::Destructive,
        if qr_ready {
            "Tombstone recording and TDLib provider delete execution are available through the command executor."
        } else {
            "Tombstone recording is available locally; provider delete execution requires TDLib QR runtime."
        },
        true,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "messages.restore_visibility",
        cat_msg_write,
        TelegramCapabilityState::Available,
        TelegramActionClass::LocalWrite,
        "Local visibility restore writes tombstone history and command audit evidence.",
        true,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "messages.mark_read",
        cat_msg_write,
        if qr_ready {
            TelegramCapabilityState::Degraded
        } else {
            TelegramCapabilityState::Blocked
        },
        TelegramActionClass::ProviderWrite,
        if qr_ready {
            "Provider mark-read uses TDLib viewMessages and now has dedicated message-level API/UI, but mark-unread symmetry and richer read-history remain incomplete."
        } else {
            "Provider mark-read requires TDLib QR runtime; mark-unread symmetry and richer read-history remain incomplete."
        },
        true,
        true,
    ));
}

fn push_reply_forward_capabilities(
    capabilities: &mut Vec<TelegramOperationCapability>,
    qr_ready: bool,
) {
    let cat_reply = "replies_forwards";
    capabilities.push(TelegramOperationCapability::new(
        "messages.reply",
        cat_reply,
        provider_or_unsupported(qr_ready),
        TelegramActionClass::ProviderWrite,
        "TDLib reply send is available for QR-authorized user accounts.",
        true,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "messages.forward",
        cat_reply,
        provider_or_unsupported(qr_ready),
        TelegramActionClass::ProviderWrite,
        "TDLib forward send is available for QR-authorized user accounts.",
        true,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "messages.pin",
        cat_reply,
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Degraded
        },
        TelegramActionClass::ProviderWrite,
        if qr_ready {
            "Local pin projection and TDLib provider pin execution are available through the command executor."
        } else {
            "Local pin projection is available; provider pin execution requires TDLib QR runtime."
        },
        true,
        true,
    ));
}

fn push_reaction_capabilities(capabilities: &mut Vec<TelegramOperationCapability>, qr_ready: bool) {
    let cat_reactions = "reactions";
    capabilities.push(TelegramOperationCapability::new(
        "reactions.add",
        cat_reactions,
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Degraded
        },
        TelegramActionClass::ProviderWrite,
        if qr_ready {
            "Local reaction projection and TDLib provider reaction execution are available through the command executor."
        } else {
            "Local reaction projection is available; provider reaction execution requires TDLib QR runtime."
        },
        false,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "reactions.remove",
        cat_reactions,
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Degraded
        },
        TelegramActionClass::ProviderWrite,
        if qr_ready {
            "Local reaction removal and TDLib provider reaction execution are available through the command executor."
        } else {
            "Local reaction removal is available; provider reaction execution requires TDLib QR runtime."
        },
        false,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "reactions.sync",
        cat_reactions,
        TelegramCapabilityState::Available,
        TelegramActionClass::Read,
        "TDLib interaction updates and history sync project provider reaction aggregates and reconcile self reaction commands.",
        false,
        false,
    ));
}

fn push_participant_capabilities(
    capabilities: &mut Vec<TelegramOperationCapability>,
    qr_ready: bool,
) {
    let cat_participants = "participants";
    capabilities.push(TelegramOperationCapability::new(
        "participants.sync",
        cat_participants,
        provider_or_unsupported(qr_ready),
        TelegramActionClass::Read,
        if qr_ready {
            "TDLib provider member roster sync is available for supergroups/channels with recent-member pagination plus administrator snapshots, for basic groups through getBasicGroup/getBasicGroupFullInfo, and for private/saved-message chats through TDLib chat metadata."
        } else {
            "Provider member roster sync requires TDLib QR runtime."
        },
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "participants.join",
        cat_participants,
        provider_or_unsupported(qr_ready),
        TelegramActionClass::ProviderWrite,
        if qr_ready {
            "TDLib chat join is available through the durable provider-write outbox with roster/service-message reconciliation."
        } else {
            "Chat join requires TDLib QR runtime and provider reconciliation."
        },
        true,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "participants.leave",
        cat_participants,
        provider_or_unsupported(qr_ready),
        TelegramActionClass::Destructive,
        if qr_ready {
            "TDLib chat leave is available through the durable provider-write outbox with service-message and inactive-roster reconciliation."
        } else {
            "Chat leave requires TDLib QR runtime and provider reconciliation."
        },
        true,
        true,
    ));
}

fn push_topic_capabilities(capabilities: &mut Vec<TelegramOperationCapability>, qr_ready: bool) {
    let cat_topics = "topics";
    capabilities.push(TelegramOperationCapability::new(
        "topics.list",
        cat_topics,
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Degraded
        },
        TelegramActionClass::Read,
        if qr_ready {
            "Topic projection, topic search and topic-scoped timeline reads are available."
        } else {
            "Topic projection reads are available from local state, but live TDLib refresh requires QR-authorized runtime."
        },
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "topics.create",
        cat_topics,
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Blocked
        },
        TelegramActionClass::ProviderWrite,
        if qr_ready {
            "Topic create uses the durable provider-write outbox and TDLib forum topic creation."
        } else {
            "Topic create requires TDLib QR runtime before provider-write execution is available."
        },
        true,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "topics.close",
        cat_topics,
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Blocked
        },
        TelegramActionClass::ProviderWrite,
        if qr_ready {
            "Topic close/reopen uses the durable provider-write outbox and provider-observed reconciliation."
        } else {
            "Topic close/reopen requires TDLib QR runtime before provider-write execution is available."
        },
        true,
        true,
    ));
}

fn provider_or_unsupported(qr_ready: bool) -> TelegramCapabilityState {
    if qr_ready {
        TelegramCapabilityState::Available
    } else {
        TelegramCapabilityState::Unsupported
    }
}
