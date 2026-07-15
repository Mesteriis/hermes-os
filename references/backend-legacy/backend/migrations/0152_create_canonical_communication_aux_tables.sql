CREATE TABLE IF NOT EXISTS communication_rules (
    rule_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description_nl TEXT NOT NULL DEFAULT '',
    conditions_json JSONB NOT NULL DEFAULT '[]'::jsonb,
    actions_json JSONB NOT NULL DEFAULT '[]'::jsonb,
    mode TEXT NOT NULL DEFAULT 'suggest',
    enabled BOOLEAN NOT NULL DEFAULT true,
    match_count BIGINT NOT NULL DEFAULT 0,
    last_matched_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_rules_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT communication_rules_mode CHECK (
        mode IN ('suggest', 'ask_before_execute', 'auto_execute', 'dry_run')
    ),
    CONSTRAINT communication_rules_conditions_is_array CHECK (jsonb_typeof(conditions_json) = 'array'),
    CONSTRAINT communication_rules_actions_is_array CHECK (jsonb_typeof(actions_json) = 'array')
);

CREATE TABLE IF NOT EXISTS communication_templates (
    template_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    subject_template TEXT NOT NULL,
    body_template TEXT NOT NULL DEFAULT '',
    variables JSONB NOT NULL DEFAULT '[]'::jsonb,
    language TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_templates_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT communication_templates_subject_not_empty CHECK (length(trim(subject_template)) > 0),
    CONSTRAINT communication_templates_variables_is_array CHECK (jsonb_typeof(variables) = 'array')
);

CREATE TABLE IF NOT EXISTS communication_personas (
    persona_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    account_id TEXT NOT NULL REFERENCES communication_accounts(account_id) ON DELETE CASCADE,
    display_name TEXT NOT NULL,
    signature TEXT NOT NULL DEFAULT '',
    default_language TEXT,
    default_tone TEXT,
    is_default BOOLEAN NOT NULL DEFAULT false,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_personas_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT communication_personas_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE UNIQUE INDEX IF NOT EXISTS communication_personas_one_default_per_account
    ON communication_personas (account_id)
    WHERE is_default = true;

CREATE TABLE IF NOT EXISTS communication_invoices (
    invoice_id TEXT PRIMARY KEY,
    message_id TEXT,
    amount DOUBLE PRECISION,
    currency TEXT,
    invoice_number TEXT,
    issue_date TIMESTAMPTZ,
    due_date TIMESTAMPTZ,
    counterparty TEXT,
    tax_id TEXT,
    status TEXT NOT NULL DEFAULT 'received',
    linked_project_id TEXT,
    linked_person_id TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_invoices_status CHECK (
        status IN ('received', 'recognized', 'needs_review', 'approved', 'paid', 'closed', 'rejected')
    ),
    CONSTRAINT communication_invoices_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS communication_invoices_status_idx
    ON communication_invoices (status, due_date);

CREATE INDEX IF NOT EXISTS communication_invoices_linked_person_idx
    ON communication_invoices (linked_person_id);

CREATE TABLE IF NOT EXISTS communication_legal_documents (
    document_id TEXT PRIMARY KEY,
    message_id TEXT,
    document_type TEXT NOT NULL DEFAULT 'other',
    title TEXT NOT NULL,
    parties JSONB NOT NULL DEFAULT '[]'::jsonb,
    effective_date TIMESTAMPTZ,
    expiry_date TIMESTAMPTZ,
    amount DOUBLE PRECISION,
    currency TEXT,
    status TEXT NOT NULL DEFAULT 'draft',
    linked_project_id TEXT,
    risks JSONB NOT NULL DEFAULT '[]'::jsonb,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_legal_docs_title_not_empty CHECK (length(trim(title)) > 0),
    CONSTRAINT communication_legal_docs_type CHECK (
        document_type IN (
            'contract', 'nda', 'msa', 'dpa', 'agreement', 'legal_notice',
            'claim', 'court_document', 'tax_notice', 'government_doc', 'other'
        )
    ),
    CONSTRAINT communication_legal_docs_status CHECK (
        status IN ('active', 'expired', 'pending_review', 'signed', 'terminated', 'draft')
    ),
    CONSTRAINT communication_legal_docs_parties_is_array CHECK (jsonb_typeof(parties) = 'array'),
    CONSTRAINT communication_legal_docs_risks_is_array CHECK (jsonb_typeof(risks) = 'array'),
    CONSTRAINT communication_legal_docs_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE TABLE IF NOT EXISTS communication_certificates (
    cert_id TEXT PRIMARY KEY,
    owner_name TEXT NOT NULL,
    issuer TEXT NOT NULL DEFAULT '',
    serial_number TEXT,
    fingerprint_sha256 TEXT,
    valid_from TIMESTAMPTZ,
    valid_until TIMESTAMPTZ,
    cert_type TEXT NOT NULL DEFAULT 'unknown',
    provider TEXT NOT NULL DEFAULT 'other',
    storage_kind TEXT NOT NULL DEFAULT 'encrypted_vault',
    storage_ref TEXT,
    trust_status TEXT NOT NULL DEFAULT 'untrusted',
    is_revoked BOOLEAN NOT NULL DEFAULT false,
    usage JSONB NOT NULL DEFAULT '[]'::jsonb,
    linked_message_id TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT communication_certs_type CHECK (
        cert_type IN ('smime', 'pgp', 'pdf_sign', 'cades', 'xades', 'gost_sign', 'unknown')
    ),
    CONSTRAINT communication_certs_provider CHECK (
        provider IN ('fnmt', 'dnie', 'cryptopro', 'gost', 'apple_keychain', 'pkcs12', 'yubikey', 'usb_token', 'other')
    ),
    CONSTRAINT communication_certs_storage CHECK (
        storage_kind IN ('os_keychain', 'encrypted_vault', 'pkcs12_file', 'pfx_file', 'smart_card', 'usb_token', 'external_vault')
    ),
    CONSTRAINT communication_certs_trust CHECK (
        trust_status IN ('trusted', 'untrusted', 'expired', 'revoked', 'pending_verification', 'self_signed')
    ),
    CONSTRAINT communication_certs_usage_is_array CHECK (jsonb_typeof(usage) = 'array'),
    CONSTRAINT communication_certs_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS communication_certs_expiry_idx
    ON communication_certificates (valid_until)
    WHERE valid_until IS NOT NULL AND is_revoked = false;

INSERT INTO communication_rules (
    rule_id, name, description_nl, conditions_json, actions_json, mode, enabled,
    match_count, last_matched_at, created_at, updated_at
)
SELECT
    rule_id, name, description_nl, conditions_json, actions_json, mode, enabled,
    match_count, last_matched_at, created_at, updated_at
FROM email_rules
ON CONFLICT (rule_id) DO NOTHING;

INSERT INTO communication_templates (
    template_id, name, subject_template, body_template, variables, language, created_at, updated_at
)
SELECT
    template_id, name, subject_template, body_template, variables, language, created_at, updated_at
FROM email_templates
ON CONFLICT (template_id) DO NOTHING;

INSERT INTO communication_accounts (
    account_id, provider_kind, display_name, external_account_id, config, metadata, created_at, updated_at
)
SELECT
    account_id,
    provider_kind,
    display_name,
    external_account_id,
    config,
    '{}'::jsonb,
    created_at,
    updated_at
FROM communication_provider_accounts
WHERE EXISTS (
    SELECT 1
    FROM email_personas persona
    WHERE persona.account_id = communication_provider_accounts.account_id
)
ON CONFLICT (account_id) DO NOTHING;

INSERT INTO communication_personas (
    persona_id, name, account_id, display_name, signature, default_language, default_tone,
    is_default, metadata, created_at, updated_at
)
SELECT
    persona_id, name, account_id, display_name, signature, default_language, default_tone,
    is_default, metadata || jsonb_build_object('source_table', 'email_personas'), created_at, updated_at
FROM email_personas
ON CONFLICT (persona_id) DO NOTHING;

INSERT INTO communication_invoices (
    invoice_id, message_id, amount, currency, invoice_number, issue_date, due_date,
    counterparty, tax_id, status, linked_project_id, linked_person_id, metadata, created_at, updated_at
)
SELECT
    invoice_id,
    message_id,
    amount,
    currency,
    invoice_number,
    issue_date,
    due_date,
    counterparty,
    tax_id,
    status,
    linked_project_id,
    COALESCE(linked_person_id, linked_contact_id),
    metadata || jsonb_build_object('source_table', 'email_invoices'),
    created_at,
    updated_at
FROM email_invoices
ON CONFLICT (invoice_id) DO NOTHING;

INSERT INTO communication_legal_documents (
    document_id, message_id, document_type, title, parties, effective_date, expiry_date,
    amount, currency, status, linked_project_id, risks, metadata, created_at, updated_at
)
SELECT
    document_id, message_id, document_type, title, parties, effective_date, expiry_date,
    amount, currency, status, linked_project_id, risks,
    metadata || jsonb_build_object('source_table', 'email_legal_documents'),
    created_at, updated_at
FROM email_legal_documents
ON CONFLICT (document_id) DO NOTHING;

INSERT INTO communication_certificates (
    cert_id, owner_name, issuer, serial_number, fingerprint_sha256, valid_from, valid_until,
    cert_type, provider, storage_kind, storage_ref, trust_status, is_revoked, usage,
    linked_message_id, metadata, created_at, updated_at
)
SELECT
    cert_id, owner_name, issuer, serial_number, fingerprint_sha256, valid_from, valid_until,
    cert_type, provider, storage_kind, storage_ref, trust_status, is_revoked, usage,
    linked_message_id,
    metadata || jsonb_build_object('source_table', 'email_certificates'),
    created_at, updated_at
FROM email_certificates
ON CONFLICT (cert_id) DO NOTHING;
