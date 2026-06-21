ALTER TABLE secret_references
    DROP CONSTRAINT secret_references_store_kind;

ALTER TABLE secret_references
    ADD CONSTRAINT secret_references_store_kind CHECK (
        store_kind IN (
            'os_keychain',
            'encrypted_vault',
            'database_encrypted_vault',
            'external_vault',
            'test_double'
        )
    );

CREATE TABLE IF NOT EXISTS encrypted_secret_vault_entries (
    secret_ref TEXT PRIMARY KEY REFERENCES secret_references(secret_ref) ON DELETE RESTRICT,
    kdf TEXT NOT NULL,
    salt TEXT NOT NULL,
    nonce TEXT NOT NULL,
    ciphertext TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT encrypted_secret_vault_entries_ref_not_empty CHECK (length(trim(secret_ref)) > 0),
    CONSTRAINT encrypted_secret_vault_entries_kdf CHECK (kdf IN ('argon2id:v1')),
    CONSTRAINT encrypted_secret_vault_entries_salt_not_empty CHECK (length(trim(salt)) > 0),
    CONSTRAINT encrypted_secret_vault_entries_nonce_not_empty CHECK (length(trim(nonce)) > 0),
    CONSTRAINT encrypted_secret_vault_entries_ciphertext_not_empty CHECK (length(trim(ciphertext)) > 0)
);

CREATE INDEX IF NOT EXISTS encrypted_secret_vault_entries_updated_idx
    ON encrypted_secret_vault_entries (updated_at);
