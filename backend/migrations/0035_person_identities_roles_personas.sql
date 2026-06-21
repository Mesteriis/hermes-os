-- Phase 1: Multi-channel person identity model

-- Extend persons table with type, role, org, timezone
ALTER TABLE persons
    ADD COLUMN IF NOT EXISTS person_type TEXT,
    ADD COLUMN IF NOT EXISTS primary_role TEXT,
    ADD COLUMN IF NOT EXISTS organization_reference TEXT,
    ADD COLUMN IF NOT EXISTS timezone TEXT;

-- Person identities: multi-channel identifiers
CREATE TABLE IF NOT EXISTS person_identities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id TEXT NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE,
    identity_type TEXT NOT NULL,
    identity_value TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'manual',
    confidence REAL NOT NULL DEFAULT 1.0,
    last_verified_at TIMESTAMPTZ,
    status TEXT NOT NULL DEFAULT 'active',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT person_identities_type_check CHECK (identity_type IN (
        'email', 'telegram', 'whatsapp', 'phone',
        'github', 'linkedin', 'website',
        'mastodon', 'x', 'stackoverflow', 'habr',
        'medium', 'orcid', 'google_scholar'
    )),
    CONSTRAINT person_identities_status_check CHECK (status IN (
        'active', 'outdated', 'unreachable', 'blocked'
    )),
    CONSTRAINT person_identities_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE UNIQUE INDEX IF NOT EXISTS person_identities_type_value_idx
    ON person_identities (identity_type, identity_value) WHERE status = 'active';
CREATE INDEX IF NOT EXISTS person_identities_person_id_idx ON person_identities (person_id);

-- Person roles: many-to-many role assignments
CREATE TABLE IF NOT EXISTS person_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id TEXT NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE,
    role TEXT NOT NULL,
    assigned_by TEXT,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT person_roles_unique UNIQUE (person_id, role)
);

CREATE INDEX IF NOT EXISTS person_roles_person_id_idx ON person_roles (person_id);
CREATE INDEX IF NOT EXISTS person_roles_role_idx ON person_roles (role);

-- Person personas: named interaction contexts
CREATE TABLE IF NOT EXISTS person_personas (
    persona_id TEXT PRIMARY KEY,
    person_id TEXT NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    context TEXT,
    default_tone TEXT,
    default_language TEXT,
    preferred_channel TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT person_personas_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT person_personas_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS person_personas_person_id_idx ON person_personas (person_id);

-- Backfill: create an email identity for each existing person
INSERT INTO person_identities (person_id, identity_type, identity_value, source, confidence, status)
SELECT person_id, 'email', email_address, 'import', 1.0, 'active'
FROM persons
WHERE email_address IS NOT NULL
ON CONFLICT (identity_type, identity_value) WHERE status = 'active' DO NOTHING;
