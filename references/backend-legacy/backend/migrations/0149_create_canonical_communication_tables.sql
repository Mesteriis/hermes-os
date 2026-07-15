-- Canonical Communications tables.
--
-- Historical provider-prefixed tables remain for upgrade compatibility, but
-- communication-owned durable state is mirrored into provider-neutral tables.

CREATE TABLE IF NOT EXISTS communication_accounts (
    account_id TEXT PRIMARY KEY,
    provider_kind TEXT NOT NULL,
    display_name TEXT NOT NULL,
    external_account_id TEXT NOT NULL,
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_accounts_id_not_empty CHECK (length(trim(account_id)) > 0),
    CONSTRAINT communication_accounts_provider_kind_not_empty CHECK (length(trim(provider_kind)) > 0),
    CONSTRAINT communication_accounts_display_name_not_empty CHECK (length(trim(display_name)) > 0),
    CONSTRAINT communication_accounts_external_id_not_empty CHECK (length(trim(external_account_id)) > 0),
    CONSTRAINT communication_accounts_config_is_object CHECK (jsonb_typeof(config) = 'object'),
    CONSTRAINT communication_accounts_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS communication_accounts_provider_idx
    ON communication_accounts (provider_kind, created_at DESC);

CREATE TABLE IF NOT EXISTS communication_channels (
    channel_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES communication_accounts(account_id) ON DELETE CASCADE,
    channel_kind TEXT NOT NULL,
    provider_kind TEXT NOT NULL,
    display_name TEXT NOT NULL,
    runtime_state TEXT NOT NULL DEFAULT 'metadata_only',
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_channels_id_not_empty CHECK (length(trim(channel_id)) > 0),
    CONSTRAINT communication_channels_kind_not_empty CHECK (length(trim(channel_kind)) > 0),
    CONSTRAINT communication_channels_provider_not_empty CHECK (length(trim(provider_kind)) > 0),
    CONSTRAINT communication_channels_display_name_not_empty CHECK (length(trim(display_name)) > 0),
    CONSTRAINT communication_channels_runtime_state_not_empty CHECK (length(trim(runtime_state)) > 0),
    CONSTRAINT communication_channels_config_is_object CHECK (jsonb_typeof(config) = 'object'),
    CONSTRAINT communication_channels_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS communication_channels_account_idx
    ON communication_channels (account_id, channel_kind);

CREATE TABLE IF NOT EXISTS communication_identities (
    identity_id TEXT PRIMARY KEY,
    account_id TEXT REFERENCES communication_accounts(account_id) ON DELETE CASCADE,
    channel_id TEXT REFERENCES communication_channels(channel_id) ON DELETE CASCADE,
    identity_kind TEXT NOT NULL,
    provider_identity_id TEXT NOT NULL,
    display_name TEXT,
    address TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_identities_id_not_empty CHECK (length(trim(identity_id)) > 0),
    CONSTRAINT communication_identities_kind_not_empty CHECK (length(trim(identity_kind)) > 0),
    CONSTRAINT communication_identities_provider_id_not_empty CHECK (length(trim(provider_identity_id)) > 0),
    CONSTRAINT communication_identities_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object'),
    CONSTRAINT communication_identities_provider_unique UNIQUE (account_id, identity_kind, provider_identity_id)
);

CREATE TABLE IF NOT EXISTS communication_conversations (
    conversation_id TEXT PRIMARY KEY,
    account_id TEXT REFERENCES communication_accounts(account_id) ON DELETE CASCADE,
    channel_id TEXT REFERENCES communication_channels(channel_id) ON DELETE SET NULL,
    channel_kind TEXT NOT NULL,
    provider_conversation_id TEXT,
    title TEXT,
    last_message_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_conversations_id_not_empty CHECK (length(trim(conversation_id)) > 0),
    CONSTRAINT communication_conversations_channel_kind_not_empty CHECK (length(trim(channel_kind)) > 0),
    CONSTRAINT communication_conversations_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS communication_conversations_account_idx
    ON communication_conversations (account_id, last_message_at DESC NULLS LAST);

CREATE TABLE IF NOT EXISTS communication_conversation_participants (
    participant_id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL REFERENCES communication_conversations(conversation_id) ON DELETE CASCADE,
    identity_id TEXT REFERENCES communication_identities(identity_id) ON DELETE SET NULL,
    person_id TEXT REFERENCES persons(person_id) ON DELETE SET NULL,
    role TEXT NOT NULL,
    display_name TEXT,
    address TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_conversation_participants_id_not_empty CHECK (length(trim(participant_id)) > 0),
    CONSTRAINT communication_conversation_participants_role_not_empty CHECK (length(trim(role)) > 0),
    CONSTRAINT communication_conversation_participants_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS communication_conversation_participants_conversation_idx
    ON communication_conversation_participants (conversation_id, role);

CREATE TABLE IF NOT EXISTS communication_message_versions (
    version_id TEXT PRIMARY KEY,
    message_id TEXT NOT NULL REFERENCES communication_messages(message_id) ON DELETE CASCADE,
    account_id TEXT NOT NULL REFERENCES communication_accounts(account_id) ON DELETE CASCADE,
    provider_message_id TEXT NOT NULL,
    provider_conversation_id TEXT,
    version_number INTEGER NOT NULL,
    body_text TEXT,
    edited_at TIMESTAMPTZ NOT NULL,
    source_event TEXT,
    diff_payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    provenance JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_message_versions_version_positive CHECK (version_number > 0),
    CONSTRAINT communication_message_versions_diff_is_object CHECK (jsonb_typeof(diff_payload) = 'object'),
    CONSTRAINT communication_message_versions_provenance_is_object CHECK (jsonb_typeof(provenance) = 'object'),
    CONSTRAINT communication_message_versions_unique UNIQUE (message_id, version_number)
);

CREATE TABLE IF NOT EXISTS communication_message_tombstones (
    tombstone_id TEXT PRIMARY KEY,
    message_id TEXT NOT NULL REFERENCES communication_messages(message_id) ON DELETE CASCADE,
    account_id TEXT NOT NULL REFERENCES communication_accounts(account_id) ON DELETE CASCADE,
    provider_message_id TEXT NOT NULL,
    provider_conversation_id TEXT,
    reason_class TEXT NOT NULL,
    actor_class TEXT NOT NULL,
    observed_at TIMESTAMPTZ NOT NULL,
    source_event TEXT,
    is_provider_delete BOOLEAN NOT NULL DEFAULT false,
    is_local_visible BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    provenance JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_message_tombstones_reason_not_empty CHECK (length(trim(reason_class)) > 0),
    CONSTRAINT communication_message_tombstones_actor_not_empty CHECK (length(trim(actor_class)) > 0),
    CONSTRAINT communication_message_tombstones_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object'),
    CONSTRAINT communication_message_tombstones_provenance_is_object CHECK (jsonb_typeof(provenance) = 'object')
);

CREATE TABLE IF NOT EXISTS communication_message_reactions (
    reaction_id TEXT PRIMARY KEY,
    message_id TEXT NOT NULL REFERENCES communication_messages(message_id) ON DELETE CASCADE,
    account_id TEXT NOT NULL REFERENCES communication_accounts(account_id) ON DELETE CASCADE,
    provider_message_id TEXT NOT NULL,
    provider_conversation_id TEXT,
    sender_identity_id TEXT,
    sender_display_name TEXT,
    reaction TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    observed_at TIMESTAMPTZ NOT NULL,
    source_event TEXT,
    provider_actor_id TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    provenance JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_message_reactions_reaction_not_empty CHECK (length(trim(reaction)) > 0),
    CONSTRAINT communication_message_reactions_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object'),
    CONSTRAINT communication_message_reactions_provenance_is_object CHECK (jsonb_typeof(provenance) = 'object')
);

CREATE INDEX IF NOT EXISTS communication_message_reactions_message_idx
    ON communication_message_reactions (message_id, is_active, created_at DESC);

CREATE TABLE IF NOT EXISTS communication_message_refs (
    message_ref_id TEXT PRIMARY KEY,
    ref_kind TEXT NOT NULL,
    source_message_id TEXT NOT NULL REFERENCES communication_messages(message_id) ON DELETE CASCADE,
    target_message_id TEXT REFERENCES communication_messages(message_id) ON DELETE SET NULL,
    account_id TEXT NOT NULL REFERENCES communication_accounts(account_id) ON DELETE CASCADE,
    provider_conversation_id TEXT,
    source_provider_id TEXT,
    target_provider_id TEXT,
    depth INTEGER NOT NULL DEFAULT 1,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    provenance JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_message_refs_kind CHECK (ref_kind IN ('reply', 'forward')),
    CONSTRAINT communication_message_refs_depth_positive CHECK (depth > 0),
    CONSTRAINT communication_message_refs_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object'),
    CONSTRAINT communication_message_refs_provenance_is_object CHECK (jsonb_typeof(provenance) = 'object')
);

CREATE INDEX IF NOT EXISTS communication_message_refs_source_idx
    ON communication_message_refs (source_message_id, ref_kind, created_at DESC);

CREATE TABLE IF NOT EXISTS communication_folders (
    folder_id TEXT PRIMARY KEY,
    account_id TEXT REFERENCES communication_accounts(account_id) ON DELETE CASCADE,
    channel_kind TEXT,
    name TEXT NOT NULL,
    description TEXT,
    color TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_folders_id_not_empty CHECK (length(trim(folder_id)) > 0),
    CONSTRAINT communication_folders_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT communication_folders_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE TABLE IF NOT EXISTS communication_folder_messages (
    folder_id TEXT NOT NULL REFERENCES communication_folders(folder_id) ON DELETE CASCADE,
    message_id TEXT NOT NULL REFERENCES communication_messages(message_id) ON DELETE CASCADE,
    added_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_operation TEXT NOT NULL DEFAULT 'copy',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    PRIMARY KEY (folder_id, message_id)
);

CREATE TABLE IF NOT EXISTS communication_saved_searches (
    saved_search_id TEXT PRIMARY KEY,
    account_id TEXT REFERENCES communication_accounts(account_id) ON DELETE CASCADE,
    channel_kind TEXT,
    name TEXT NOT NULL,
    description TEXT,
    query_text TEXT NOT NULL DEFAULT '',
    workflow_state TEXT,
    local_state TEXT NOT NULL DEFAULT 'active',
    is_smart_folder BOOLEAN NOT NULL DEFAULT false,
    sort_order INTEGER NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_saved_searches_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT communication_saved_searches_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE TABLE IF NOT EXISTS communication_drafts (
    draft_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES communication_accounts(account_id) ON DELETE CASCADE,
    channel_kind TEXT NOT NULL DEFAULT 'mail',
    identity_id TEXT REFERENCES communication_identities(identity_id) ON DELETE SET NULL,
    to_participants JSONB NOT NULL DEFAULT '[]'::jsonb,
    cc_participants JSONB NOT NULL DEFAULT '[]'::jsonb,
    bcc_participants JSONB NOT NULL DEFAULT '[]'::jsonb,
    subject TEXT NOT NULL DEFAULT '',
    body_text TEXT NOT NULL DEFAULT '',
    body_html TEXT,
    in_reply_to TEXT,
    message_refs JSONB NOT NULL DEFAULT '[]'::jsonb,
    status TEXT NOT NULL DEFAULT 'draft',
    scheduled_send_at TIMESTAMPTZ,
    send_attempts INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_drafts_to_is_array CHECK (jsonb_typeof(to_participants) = 'array'),
    CONSTRAINT communication_drafts_cc_is_array CHECK (jsonb_typeof(cc_participants) = 'array'),
    CONSTRAINT communication_drafts_bcc_is_array CHECK (jsonb_typeof(bcc_participants) = 'array'),
    CONSTRAINT communication_drafts_message_refs_is_array CHECK (jsonb_typeof(message_refs) = 'array'),
    CONSTRAINT communication_drafts_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE TABLE IF NOT EXISTS communication_outbox (
    outbox_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES communication_accounts(account_id) ON DELETE CASCADE,
    channel_kind TEXT NOT NULL DEFAULT 'mail',
    draft_id TEXT REFERENCES communication_drafts(draft_id) ON DELETE SET NULL,
    to_participants JSONB NOT NULL DEFAULT '[]'::jsonb,
    cc_participants JSONB NOT NULL DEFAULT '[]'::jsonb,
    bcc_participants JSONB NOT NULL DEFAULT '[]'::jsonb,
    subject TEXT NOT NULL DEFAULT '',
    body_text TEXT NOT NULL DEFAULT '',
    body_html TEXT,
    status TEXT NOT NULL,
    scheduled_send_at TIMESTAMPTZ,
    undo_deadline_at TIMESTAMPTZ,
    send_attempts INTEGER NOT NULL DEFAULT 0,
    claimed_at TIMESTAMPTZ,
    sent_at TIMESTAMPTZ,
    provider_message_id TEXT,
    last_error TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_outbox_to_is_array CHECK (jsonb_typeof(to_participants) = 'array'),
    CONSTRAINT communication_outbox_cc_is_array CHECK (jsonb_typeof(cc_participants) = 'array'),
    CONSTRAINT communication_outbox_bcc_is_array CHECK (jsonb_typeof(bcc_participants) = 'array'),
    CONSTRAINT communication_outbox_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS communication_outbox_account_status_idx
    ON communication_outbox (account_id, status, created_at DESC);

CREATE TABLE IF NOT EXISTS communication_provider_commands (
    command_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES communication_accounts(account_id) ON DELETE CASCADE,
    channel_kind TEXT NOT NULL,
    command_kind TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    provider_conversation_id TEXT,
    provider_message_id TEXT,
    target_ref JSONB NOT NULL DEFAULT '{}'::jsonb,
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    capability_state TEXT NOT NULL,
    action_class TEXT NOT NULL,
    confirmation_decision TEXT NOT NULL DEFAULT 'pending',
    status TEXT NOT NULL DEFAULT 'queued',
    retry_count INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 3,
    last_error TEXT,
    result_payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    audit_metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    actor_id TEXT NOT NULL,
    happened_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_provider_commands_target_is_object CHECK (jsonb_typeof(target_ref) = 'object'),
    CONSTRAINT communication_provider_commands_payload_is_object CHECK (jsonb_typeof(payload) = 'object'),
    CONSTRAINT communication_provider_commands_result_is_object CHECK (jsonb_typeof(result_payload) = 'object'),
    CONSTRAINT communication_provider_commands_audit_is_object CHECK (jsonb_typeof(audit_metadata) = 'object'),
    CONSTRAINT communication_provider_commands_idempotency_unique UNIQUE (account_id, idempotency_key)
);

CREATE TABLE IF NOT EXISTS communication_sync_runs (
    run_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES communication_accounts(account_id) ON DELETE CASCADE,
    channel_kind TEXT NOT NULL,
    trigger TEXT NOT NULL,
    status TEXT NOT NULL,
    phase TEXT NOT NULL,
    progress JSONB NOT NULL DEFAULT '{}'::jsonb,
    checkpoint_before JSONB,
    checkpoint_after JSONB,
    checkpoint_saved BOOLEAN NOT NULL DEFAULT false,
    error_code TEXT,
    error_message TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    next_run_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_sync_runs_progress_is_object CHECK (jsonb_typeof(progress) = 'object')
);

CREATE INDEX IF NOT EXISTS communication_sync_runs_account_started_idx
    ON communication_sync_runs (account_id, started_at DESC);

CREATE TABLE IF NOT EXISTS communication_sync_checkpoints (
    account_id TEXT NOT NULL REFERENCES communication_accounts(account_id) ON DELETE CASCADE,
    channel_kind TEXT NOT NULL,
    stream_id TEXT NOT NULL,
    checkpoint JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_sync_checkpoints_checkpoint_is_object CHECK (jsonb_typeof(checkpoint) = 'object'),
    PRIMARY KEY (account_id, channel_kind, stream_id)
);

CREATE TABLE IF NOT EXISTS communication_raw_payloads (
    raw_payload_id TEXT PRIMARY KEY,
    raw_record_id TEXT NOT NULL REFERENCES communication_raw_records(raw_record_id) ON DELETE CASCADE,
    account_id TEXT NOT NULL REFERENCES communication_accounts(account_id) ON DELETE CASCADE,
    channel_kind TEXT NOT NULL,
    provider_record_id TEXT NOT NULL,
    record_kind TEXT NOT NULL,
    payload JSONB NOT NULL,
    provenance JSONB NOT NULL DEFAULT '{}'::jsonb,
    captured_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_raw_payloads_payload_is_object CHECK (jsonb_typeof(payload) = 'object'),
    CONSTRAINT communication_raw_payloads_provenance_is_object CHECK (jsonb_typeof(provenance) = 'object'),
    CONSTRAINT communication_raw_payloads_record_unique UNIQUE (raw_record_id)
);

INSERT INTO communication_accounts (
    account_id, provider_kind, display_name, external_account_id, config, metadata, created_at, updated_at
)
SELECT
    account_id,
    provider_kind,
    display_name,
    external_account_id,
    config,
    jsonb_build_object('source_table', 'communication_provider_accounts'),
    created_at,
    updated_at
FROM communication_provider_accounts
ON CONFLICT (account_id) DO NOTHING;

INSERT INTO communication_channels (
    channel_id, account_id, channel_kind, provider_kind, display_name, config, metadata, created_at, updated_at
)
SELECT
    'channel:' || account_id,
    account_id,
    CASE
        WHEN provider_kind IN ('gmail', 'icloud', 'imap') THEN 'mail'
        WHEN provider_kind IN ('telegram_user', 'telegram_bot') THEN 'telegram'
        WHEN provider_kind = 'whatsapp_web' THEN 'whatsapp'
        ELSE provider_kind
    END,
    provider_kind,
    display_name,
    config,
    jsonb_build_object('source_table', 'communication_provider_accounts'),
    created_at,
    updated_at
FROM communication_provider_accounts
ON CONFLICT (channel_id) DO NOTHING;

INSERT INTO communication_identities (
    identity_id, account_id, channel_id, identity_kind, provider_identity_id, display_name, address, metadata, created_at, updated_at
)
SELECT
    'identity:account:' || account_id,
    account_id,
    'channel:' || account_id,
    'account',
    external_account_id,
    display_name,
    external_account_id,
    jsonb_build_object('provider_kind', provider_kind),
    created_at,
    updated_at
FROM communication_provider_accounts
ON CONFLICT (identity_id) DO NOTHING;

INSERT INTO communication_conversations (
    conversation_id, account_id, channel_id, channel_kind, provider_conversation_id, title, last_message_at, metadata, created_at, updated_at
)
SELECT
    telegram_chat_id,
    account_id,
    'channel:' || account_id,
    'telegram',
    provider_chat_id,
    title,
    last_message_at,
    jsonb_build_object('source_table', 'telegram_chats', 'chat_kind', chat_kind, 'metadata', metadata),
    created_at,
    updated_at
FROM telegram_chats
ON CONFLICT (conversation_id) DO NOTHING;

INSERT INTO communication_conversations (
    conversation_id, account_id, channel_id, channel_kind, provider_conversation_id, title, last_message_at, metadata, created_at, updated_at
)
SELECT
    session_id,
    account_id,
    'channel:' || account_id,
    'whatsapp',
    session_id,
    device_name,
    last_sync_at,
    jsonb_build_object('source_table', 'whatsapp_web_sessions', 'metadata', metadata),
    created_at,
    updated_at
FROM whatsapp_web_sessions
ON CONFLICT (conversation_id) DO NOTHING;

INSERT INTO communication_conversations (
    conversation_id, account_id, channel_id, channel_kind, provider_conversation_id, title, last_message_at, metadata, created_at, updated_at
)
SELECT
    conversation_id,
    MIN(account_id),
    'channel:' || MIN(account_id),
    COALESCE(NULLIF(MIN(channel_kind), ''), 'mail'),
    conversation_id,
    NULLIF(MIN(subject), ''),
    MAX(occurred_at),
    jsonb_build_object('source_table', 'communication_messages'),
    MIN(projected_at),
    MAX(projected_at)
FROM communication_messages
WHERE conversation_id IS NOT NULL AND length(trim(conversation_id)) > 0
GROUP BY conversation_id
ON CONFLICT (conversation_id) DO NOTHING;

INSERT INTO communication_conversation_participants (
    participant_id, conversation_id, person_id, role, display_name, address, metadata, created_at, updated_at
)
SELECT DISTINCT ON (m.conversation_id, p.person_id, p.role, p.email_address)
    'participant:' || md5(m.conversation_id || ':' || p.person_id || ':' || p.role || ':' || p.email_address),
    m.conversation_id,
    p.person_id,
    p.role,
    p.display_name,
    p.email_address,
    jsonb_build_object('source_table', 'communication_message_participants', 'source', p.source),
    MIN(p.created_at) OVER (PARTITION BY m.conversation_id, p.person_id, p.role, p.email_address),
    MAX(p.updated_at) OVER (PARTITION BY m.conversation_id, p.person_id, p.role, p.email_address)
FROM communication_message_participants p
JOIN communication_messages m ON m.message_id = p.message_id
WHERE m.conversation_id IS NOT NULL AND length(trim(m.conversation_id)) > 0
ON CONFLICT (participant_id) DO NOTHING;

INSERT INTO communication_message_versions (
    version_id, message_id, account_id, provider_message_id, provider_conversation_id, version_number,
    body_text, edited_at, source_event, diff_payload, provenance, created_at
)
SELECT
    v.version_id,
    v.message_id,
    v.account_id,
    v.provider_message_id,
    v.provider_chat_id,
    v.version_number,
    v.body_text,
    v.edit_timestamp,
    v.source_event,
    v.raw_diff_payload,
    v.provenance,
    v.created_at
FROM telegram_message_versions v
JOIN communication_messages m ON m.message_id = v.message_id
ON CONFLICT (version_id) DO NOTHING;

INSERT INTO communication_message_tombstones (
    tombstone_id, message_id, account_id, provider_message_id, provider_conversation_id,
    reason_class, actor_class, observed_at, source_event, is_provider_delete, is_local_visible,
    metadata, provenance, created_at
)
SELECT
    t.tombstone_id,
    t.message_id,
    t.account_id,
    t.provider_message_id,
    t.provider_chat_id,
    t.reason_class,
    t.actor_class,
    t.observed_at,
    t.source_event,
    t.is_provider_delete,
    t.is_local_visible,
    t.metadata,
    t.provenance,
    t.created_at
FROM telegram_message_tombstones t
JOIN communication_messages m ON m.message_id = t.message_id
ON CONFLICT (tombstone_id) DO NOTHING;

INSERT INTO communication_message_reactions (
    reaction_id, message_id, account_id, provider_message_id, provider_conversation_id,
    sender_identity_id, sender_display_name, reaction, is_active, observed_at, source_event,
    provider_actor_id, metadata, provenance, created_at, updated_at
)
SELECT
    r.reaction_id,
    r.message_id,
    r.account_id,
    r.provider_message_id,
    r.provider_chat_id,
    r.sender_id,
    r.sender_display_name,
    r.reaction_emoji,
    r.is_active,
    r.observed_at,
    r.source_event,
    r.provider_actor_id,
    r.metadata,
    r.provenance,
    r.created_at,
    r.updated_at
FROM telegram_message_reactions r
JOIN communication_messages m ON m.message_id = r.message_id
ON CONFLICT (reaction_id) DO NOTHING;

INSERT INTO communication_message_refs (
    message_ref_id, ref_kind, source_message_id, target_message_id, account_id,
    provider_conversation_id, source_provider_id, target_provider_id, depth, metadata, provenance, created_at
)
SELECT
    reply_ref_id,
    'reply',
    source_message_id,
    target_message_id,
    account_id,
    provider_chat_id,
    source_provider_id,
    target_provider_id,
    reply_depth,
    metadata,
    provenance,
    created_at
FROM telegram_message_reply_refs
WHERE EXISTS (SELECT 1 FROM communication_messages m WHERE m.message_id = source_message_id)
  AND EXISTS (SELECT 1 FROM communication_messages m WHERE m.message_id = target_message_id)
ON CONFLICT (message_ref_id) DO NOTHING;

INSERT INTO communication_message_refs (
    message_ref_id, ref_kind, source_message_id, account_id, provider_conversation_id,
    source_provider_id, depth, metadata, provenance, created_at
)
SELECT
    forward_ref_id,
    'forward',
    source_message_id,
    account_id,
    provider_chat_id,
    source_provider_id,
    forward_depth,
    jsonb_build_object(
        'forward_origin_chat_id', forward_origin_chat_id,
        'forward_origin_message_id', forward_origin_message_id,
        'forward_origin_sender_id', forward_origin_sender_id,
        'forward_origin_sender_name', forward_origin_sender_name,
        'forward_date', forward_date,
        'metadata', metadata
    ),
    provenance,
    created_at
FROM telegram_message_forward_refs
WHERE EXISTS (SELECT 1 FROM communication_messages m WHERE m.message_id = source_message_id)
ON CONFLICT (message_ref_id) DO NOTHING;

INSERT INTO communication_folders (
    folder_id, account_id, channel_kind, name, description, color, sort_order, metadata, created_at, updated_at
)
SELECT
    folder_id,
    account_id,
    'mail',
    name,
    description,
    color,
    sort_order,
    jsonb_build_object('source_table', 'mail_folders'),
    created_at,
    updated_at
FROM mail_folders
ON CONFLICT (folder_id) DO NOTHING;

INSERT INTO communication_folder_messages (
    folder_id, message_id, added_at, last_operation, metadata
)
SELECT
    folder_id,
    message_id,
    added_at,
    last_operation,
    jsonb_build_object('source_table', 'mail_folder_messages')
FROM mail_folder_messages
ON CONFLICT (folder_id, message_id) DO NOTHING;

INSERT INTO communication_saved_searches (
    saved_search_id, account_id, channel_kind, name, description, query_text, workflow_state,
    local_state, is_smart_folder, sort_order, metadata, created_at, updated_at
)
SELECT
    saved_search_id,
    account_id,
    COALESCE(channel_kind, 'mail'),
    name,
    description,
    query_text,
    workflow_state,
    local_state,
    is_smart_folder,
    sort_order,
    jsonb_build_object('source_table', 'mail_saved_searches'),
    created_at,
    updated_at
FROM mail_saved_searches
ON CONFLICT (saved_search_id) DO NOTHING;

INSERT INTO communication_drafts (
    draft_id, account_id, channel_kind, to_participants, cc_participants, bcc_participants,
    subject, body_text, body_html, in_reply_to, message_refs, status, scheduled_send_at,
    send_attempts, last_error, metadata, created_at, updated_at
)
SELECT
    draft_id,
    account_id,
    'mail',
    to_recipients,
    cc_recipients,
    bcc_recipients,
    subject,
    body_text,
    body_html,
    in_reply_to,
    message_references,
    status,
    scheduled_send_at,
    send_attempts,
    last_error,
    metadata || jsonb_build_object('source_table', 'email_drafts'),
    created_at,
    updated_at
FROM email_drafts
ON CONFLICT (draft_id) DO NOTHING;

INSERT INTO communication_outbox (
    outbox_id, account_id, channel_kind, draft_id, to_participants, cc_participants, bcc_participants,
    subject, body_text, body_html, status, scheduled_send_at, undo_deadline_at, send_attempts,
    claimed_at, sent_at, provider_message_id, last_error, metadata, created_at, updated_at
)
SELECT
    outbox_id,
    account_id,
    'mail',
    draft_id,
    to_recipients,
    cc_recipients,
    bcc_recipients,
    subject,
    body_text,
    body_html,
    status,
    scheduled_send_at,
    undo_deadline_at,
    send_attempts,
    claimed_at,
    sent_at,
    provider_message_id,
    last_error,
    metadata || jsonb_build_object('source_table', 'email_outbox_tracking'),
    created_at,
    updated_at
FROM email_outbox_tracking
ON CONFLICT (outbox_id) DO NOTHING;

INSERT INTO communication_provider_commands (
    command_id, account_id, channel_kind, command_kind, idempotency_key,
    provider_conversation_id, provider_message_id, target_ref, payload, capability_state,
    action_class, confirmation_decision, status, retry_count, max_retries, last_error,
    result_payload, audit_metadata, actor_id, happened_at, completed_at, created_at, updated_at
)
SELECT
    command_id,
    account_id,
    'telegram',
    command_kind,
    idempotency_key,
    provider_chat_id,
    provider_message_id,
    target_ref,
    payload,
    capability_state,
    action_class,
    confirmation_decision,
    status,
    retry_count,
    max_retries,
    last_error,
    result_payload,
    audit_metadata,
    actor_id,
    happened_at,
    completed_at,
    created_at,
    updated_at
FROM telegram_provider_write_commands
ON CONFLICT (command_id) DO NOTHING;

INSERT INTO communication_sync_runs (
    run_id, account_id, channel_kind, trigger, status, phase, progress,
    checkpoint_before, checkpoint_after, checkpoint_saved, error_code, error_message,
    started_at, completed_at, next_run_at, created_at, updated_at
)
SELECT
    run_id,
    account_id,
    'mail',
    trigger,
    status,
    phase,
    jsonb_build_object(
        'progress_mode', progress_mode,
        'progress_percent', progress_percent,
        'processed_messages', processed_messages,
        'estimated_total_messages', estimated_total_messages,
        'current_batch_size', current_batch_size,
        'fetched_messages', fetched_messages,
        'projected_messages', projected_messages,
        'upserted_persons', upserted_persons,
        'upserted_organizations', upserted_organizations
    ),
    checkpoint_before,
    checkpoint_after,
    checkpoint_saved,
    error_code,
    error_message,
    started_at,
    completed_at,
    next_run_at,
    created_at,
    updated_at
FROM communication_mail_sync_runs
ON CONFLICT (run_id) DO NOTHING;

INSERT INTO communication_sync_checkpoints (
    account_id, channel_kind, stream_id, checkpoint, updated_at
)
SELECT
    c.account_id,
    CASE
        WHEN a.provider_kind IN ('gmail', 'icloud', 'imap') THEN 'mail'
        WHEN a.provider_kind IN ('telegram_user', 'telegram_bot') THEN 'telegram'
        WHEN a.provider_kind = 'whatsapp_web' THEN 'whatsapp'
        ELSE a.provider_kind
    END,
    c.stream_id,
    c.checkpoint,
    c.updated_at
FROM communication_ingestion_checkpoints c
JOIN communication_provider_accounts a ON a.account_id = c.account_id
ON CONFLICT (account_id, channel_kind, stream_id) DO NOTHING;

INSERT INTO communication_raw_payloads (
    raw_payload_id, raw_record_id, account_id, channel_kind, provider_record_id,
    record_kind, payload, provenance, captured_at
)
SELECT
    'raw_payload:' || r.raw_record_id,
    r.raw_record_id,
    r.account_id,
    CASE
        WHEN a.provider_kind IN ('gmail', 'icloud', 'imap') THEN 'mail'
        WHEN a.provider_kind IN ('telegram_user', 'telegram_bot') THEN 'telegram'
        WHEN a.provider_kind = 'whatsapp_web' THEN 'whatsapp'
        ELSE a.provider_kind
    END,
    r.provider_record_id,
    r.record_kind,
    r.payload,
    r.provenance,
    r.captured_at
FROM communication_raw_records r
JOIN communication_provider_accounts a ON a.account_id = r.account_id
ON CONFLICT (raw_payload_id) DO NOTHING;
