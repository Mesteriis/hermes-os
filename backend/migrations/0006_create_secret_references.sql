CREATE TABLE IF NOT EXISTS secret_references (
    secret_ref TEXT PRIMARY KEY,
    secret_kind TEXT NOT NULL,
    store_kind TEXT NOT NULL,
    label TEXT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT secret_references_kind CHECK (
        secret_kind IN ('oauth_token', 'app_password', 'password', 'api_token', 'private_key', 'other')
    ),
    CONSTRAINT secret_references_store_kind CHECK (
        store_kind IN ('os_keychain', 'encrypted_vault', 'external_vault', 'test_double')
    ),
    CONSTRAINT secret_references_ref_not_empty CHECK (length(trim(secret_ref)) > 0),
    CONSTRAINT secret_references_label_not_empty CHECK (length(trim(label)) > 0),
    CONSTRAINT secret_references_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS secret_references_kind_idx
    ON secret_references (secret_kind, created_at);

CREATE TABLE IF NOT EXISTS communication_provider_account_secret_refs (
    account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    secret_purpose TEXT NOT NULL,
    secret_ref TEXT NOT NULL REFERENCES secret_references(secret_ref) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_provider_account_secret_purpose CHECK (
        secret_purpose IN ('oauth_token', 'imap_password', 'smtp_password')
    ),
    CONSTRAINT communication_provider_account_secret_ref_not_empty CHECK (length(trim(secret_ref)) > 0),
    PRIMARY KEY (account_id, secret_purpose)
);

CREATE INDEX IF NOT EXISTS communication_provider_account_secret_refs_secret_idx
    ON communication_provider_account_secret_refs (secret_ref, updated_at);
