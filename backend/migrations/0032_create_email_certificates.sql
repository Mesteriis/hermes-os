CREATE TABLE IF NOT EXISTS email_certificates (
    cert_id TEXT PRIMARY KEY, owner_name TEXT NOT NULL, issuer TEXT NOT NULL DEFAULT '',
    serial_number TEXT, fingerprint_sha256 TEXT,
    valid_from TIMESTAMPTZ, valid_until TIMESTAMPTZ,
    cert_type TEXT NOT NULL DEFAULT 'unknown',
    provider TEXT NOT NULL DEFAULT 'other',
    storage_kind TEXT NOT NULL DEFAULT 'encrypted_vault',
    storage_ref TEXT,
    trust_status TEXT NOT NULL DEFAULT 'untrusted',
    is_revoked BOOLEAN NOT NULL DEFAULT false,
    usage JSONB NOT NULL DEFAULT '[]'::jsonb,
    linked_message_id TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(), updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT email_certs_type CHECK (cert_type IN ('smime','pgp','pdf_sign','cades','xades','gost_sign','unknown')),
    CONSTRAINT email_certs_provider CHECK (provider IN ('fnmt','dnie','cryptopro','gost','apple_keychain','pkcs12','yubikey','usb_token','other')),
    CONSTRAINT email_certs_storage CHECK (storage_kind IN ('os_keychain','encrypted_vault','pkcs12_file','pfx_file','smart_card','usb_token','external_vault')),
    CONSTRAINT email_certs_trust CHECK (trust_status IN ('trusted','untrusted','expired','revoked','pending_verification','self_signed'))
);
CREATE INDEX IF NOT EXISTS email_certs_expiry_idx ON email_certificates (valid_until) WHERE valid_until IS NOT NULL AND is_revoked = false;
