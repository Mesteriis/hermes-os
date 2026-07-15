-- Durable provider folder/label discovery and canonical Communications mappings.

CREATE TABLE IF NOT EXISTS communication_mail_provider_resources (
    mapping_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES communication_accounts(account_id) ON DELETE CASCADE,
    resource_kind TEXT NOT NULL,
    provider_resource_id TEXT NOT NULL,
    display_name TEXT NOT NULL,
    semantic_role TEXT,
    local_folder_id TEXT REFERENCES communication_folders(folder_id) ON DELETE SET NULL,
    selectable BOOLEAN NOT NULL DEFAULT true,
    writable BOOLEAN NOT NULL DEFAULT true,
    mapping_source TEXT NOT NULL DEFAULT 'discovered',
    capabilities JSONB NOT NULL DEFAULT '{}'::jsonb,
    observed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_mail_provider_resources_id_not_empty
        CHECK (length(trim(mapping_id)) > 0),
    CONSTRAINT communication_mail_provider_resources_kind
        CHECK (resource_kind IN ('folder', 'label')),
    CONSTRAINT communication_mail_provider_resources_provider_id_not_empty
        CHECK (length(trim(provider_resource_id)) > 0),
    CONSTRAINT communication_mail_provider_resources_display_name_not_empty
        CHECK (length(trim(display_name)) > 0),
    CONSTRAINT communication_mail_provider_resources_semantic_role
        CHECK (
            semantic_role IS NULL OR semantic_role IN (
                'inbox', 'sent', 'drafts', 'archive', 'trash', 'junk',
                'all', 'flagged', 'important', 'user'
            )
        ),
    CONSTRAINT communication_mail_provider_resources_mapping_source
        CHECK (mapping_source IN ('discovered', 'manual')),
    CONSTRAINT communication_mail_provider_resources_capabilities_is_object
        CHECK (jsonb_typeof(capabilities) = 'object'),
    CONSTRAINT communication_mail_provider_resources_provider_unique
        UNIQUE (account_id, resource_kind, provider_resource_id)
);

CREATE UNIQUE INDEX IF NOT EXISTS communication_mail_provider_resources_role_unique
    ON communication_mail_provider_resources (account_id, resource_kind, semantic_role)
    WHERE semantic_role IS NOT NULL AND semantic_role <> 'user';

CREATE INDEX IF NOT EXISTS communication_mail_provider_resources_account_idx
    ON communication_mail_provider_resources (
        account_id,
        resource_kind,
        lower(display_name),
        provider_resource_id
    );

-- Gmail system labels have stable provider IDs and are safe to seed before the
-- first authenticated discovery request.
INSERT INTO communication_mail_provider_resources (
    mapping_id,
    account_id,
    resource_kind,
    provider_resource_id,
    display_name,
    semantic_role,
    capabilities
)
SELECT
    'mail-provider-resource:v1:gmail:' || communication_accounts.account_id || ':' || gmail_labels.label_id,
    communication_accounts.account_id,
    'label',
    gmail_labels.label_id,
    gmail_labels.display_name,
    gmail_labels.semantic_role,
    '{"system":true}'::jsonb
FROM communication_accounts
CROSS JOIN (
    VALUES
        ('INBOX', 'Inbox', 'inbox'),
        ('SENT', 'Sent', 'sent'),
        ('DRAFT', 'Drafts', 'drafts'),
        ('TRASH', 'Trash', 'trash'),
        ('SPAM', 'Spam', 'junk'),
        ('STARRED', 'Starred', 'flagged'),
        ('IMPORTANT', 'Important', 'important')
) AS gmail_labels(label_id, display_name, semantic_role)
WHERE provider_kind = 'gmail'
ON CONFLICT (account_id, resource_kind, provider_resource_id) DO NOTHING;

-- INBOX is the only universally named IMAP mailbox. Other roles are seeded
-- only from explicit account configuration and later refined by SPECIAL-USE.
INSERT INTO communication_mail_provider_resources (
    mapping_id,
    account_id,
    resource_kind,
    provider_resource_id,
    display_name,
    semantic_role,
    capabilities
)
SELECT
    'mail-provider-resource:v1:imap:' || account_id || ':INBOX',
    account_id,
    'folder',
    'INBOX',
    'INBOX',
    'inbox',
    '{"seeded":true}'::jsonb
FROM communication_accounts
WHERE provider_kind IN ('icloud', 'imap')
ON CONFLICT (account_id, resource_kind, provider_resource_id) DO NOTHING;

INSERT INTO communication_mail_provider_resources (
    mapping_id,
    account_id,
    resource_kind,
    provider_resource_id,
    display_name,
    semantic_role,
    capabilities
)
SELECT
    'mail-provider-resource:v1:configured:' || account_id || ':' || semantic_role,
    account_id,
    'folder',
    mailbox_name,
    mailbox_name,
    semantic_role,
    '{"configured":true}'::jsonb
FROM communication_accounts
CROSS JOIN LATERAL (
    VALUES
        ('archive', NULLIF(trim(config ->> 'archive_mailbox'), '')),
        ('trash', NULLIF(trim(config ->> 'trash_mailbox'), '')),
        ('junk', NULLIF(trim(config ->> 'spam_mailbox'), '')),
        ('sent', NULLIF(trim(config ->> 'sent_mailbox'), '')),
        ('drafts', NULLIF(trim(config ->> 'drafts_mailbox'), ''))
) AS configured_mailboxes(semantic_role, mailbox_name)
WHERE provider_kind IN ('icloud', 'imap')
  AND mailbox_name IS NOT NULL
ON CONFLICT (account_id, resource_kind, provider_resource_id) DO NOTHING;
