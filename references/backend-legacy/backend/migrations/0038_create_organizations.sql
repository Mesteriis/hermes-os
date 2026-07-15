-- Phase 0: Organizations core table

CREATE TABLE IF NOT EXISTS organizations (
    organization_id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    legal_name TEXT,
    org_type TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    country TEXT,
    city TEXT,
    address TEXT,
    website TEXT,
    industry TEXT,
    description TEXT,
    primary_language TEXT,
    timezone TEXT,
    trust_score SMALLINT,
    health_status TEXT DEFAULT 'healthy',
    priority TEXT DEFAULT 'medium',
    notes TEXT,
    tags JSONB NOT NULL DEFAULT '[]'::jsonb,
    org_metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    last_interaction_at TIMESTAMPTZ,
    interaction_count INT NOT NULL DEFAULT 0,
    -- Legal identity columns
    registration_number TEXT,
    country_of_registration TEXT,
    vat TEXT,
    cif TEXT,
    nif TEXT,
    tax_id TEXT,
    legal_address TEXT,
    registry_source TEXT,
    registry_last_verified TIMESTAMPTZ,
    -- DNA columns (populated in Phase 3)
    communication_style TEXT,
    verbosity TEXT,
    formality TEXT,
    secondary_languages JSONB,
    preferred_tone TEXT,
    official_style_required BOOL DEFAULT false,
    -- Health columns (populated in Phase 7)
    last_health_check TIMESTAMPTZ,
    watchlist BOOL NOT NULL DEFAULT false,
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT organizations_display_name_not_empty CHECK (length(trim(display_name)) > 0),
    CONSTRAINT organizations_status_check CHECK (status IN ('active', 'inactive', 'archived', 'watchlist', 'blocked', 'unknown')),
    CONSTRAINT organizations_priority_check CHECK (priority IN ('low', 'medium', 'high', 'critical')),
    CONSTRAINT organizations_trust_score_range CHECK (trust_score IS NULL OR (trust_score >= 0 AND trust_score <= 100)),
    CONSTRAINT organizations_tags_is_array CHECK (jsonb_typeof(tags) = 'array'),
    CONSTRAINT organizations_metadata_is_object CHECK (jsonb_typeof(org_metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS organizations_type_idx ON organizations (org_type);
CREATE INDEX IF NOT EXISTS organizations_status_idx ON organizations (status);
CREATE INDEX IF NOT EXISTS organizations_vat_idx ON organizations (vat) WHERE vat IS NOT NULL;
CREATE INDEX IF NOT EXISTS organizations_domain_idx ON organizations (website);
CREATE INDEX IF NOT EXISTS organizations_watchlist_idx ON organizations (organization_id) WHERE watchlist = true;
CREATE INDEX IF NOT EXISTS organizations_trust_score_idx ON organizations (trust_score DESC NULLS LAST) WHERE trust_score IS NOT NULL;
