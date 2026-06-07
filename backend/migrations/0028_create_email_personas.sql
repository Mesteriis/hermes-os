CREATE TABLE IF NOT EXISTS email_personas (
    persona_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE CASCADE,
    display_name TEXT NOT NULL,
    signature TEXT NOT NULL DEFAULT '',
    default_language TEXT,
    default_tone TEXT,
    is_default BOOLEAN NOT NULL DEFAULT false,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT email_personas_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT email_personas_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);
CREATE UNIQUE INDEX IF NOT EXISTS email_personas_one_default_per_account
    ON email_personas (account_id) WHERE is_default = true;
