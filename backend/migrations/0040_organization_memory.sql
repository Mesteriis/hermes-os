-- Phase 2: Organization memory

CREATE TABLE IF NOT EXISTS organization_facts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    fact_type TEXT NOT NULL,
    value TEXT NOT NULL,
    source TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 1.0,
    last_verified_at TIMESTAMPTZ,
    valid_from TIMESTAMPTZ,
    valid_to TIMESTAMPTZ,
    is_active BOOL NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT org_facts_confidence_range CHECK (confidence >= 0 AND confidence <= 1)
);

CREATE TABLE IF NOT EXISTS organization_memory_cards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    source TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 1.0,
    importance SMALLINT NOT NULL DEFAULT 5,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_verified_at TIMESTAMPTZ,
    CONSTRAINT org_memory_cards_confidence_range CHECK (confidence >= 0 AND confidence <= 1),
    CONSTRAINT org_memory_cards_importance_range CHECK (importance >= 1 AND importance <= 10)
);

CREATE TABLE IF NOT EXISTS organization_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    preference_type TEXT NOT NULL,
    value TEXT NOT NULL,
    source TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 1.0,
    last_verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT org_preferences_unique UNIQUE (organization_id, preference_type),
    CONSTRAINT org_preferences_confidence_range CHECK (confidence >= 0 AND confidence <= 1)
);

CREATE TABLE IF NOT EXISTS organization_required_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    document_type TEXT NOT NULL,
    description TEXT,
    source TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 0.5,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT org_reqdocs_confidence_range CHECK (confidence >= 0 AND confidence <= 1)
);

CREATE TABLE IF NOT EXISTS organization_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    snapshot_date TIMESTAMPTZ NOT NULL DEFAULT now(),
    data JSONB NOT NULL,
    source TEXT NOT NULL DEFAULT 'manual',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT org_snapshots_data_is_object CHECK (jsonb_typeof(data) = 'object')
);

CREATE TABLE IF NOT EXISTS organization_knowledge_conflicts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    field TEXT NOT NULL,
    value_a TEXT NOT NULL,
    value_b TEXT NOT NULL,
    source_a TEXT NOT NULL,
    source_b TEXT NOT NULL,
    detected_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at TIMESTAMPTZ,
    resolution TEXT
);

CREATE INDEX IF NOT EXISTS org_facts_org_id_idx ON organization_facts (organization_id);
CREATE INDEX IF NOT EXISTS org_memory_cards_org_id_idx ON organization_memory_cards (organization_id);
CREATE INDEX IF NOT EXISTS org_preferences_org_id_idx ON organization_preferences (organization_id);
CREATE INDEX IF NOT EXISTS org_snapshots_org_id_idx ON organization_snapshots (organization_id);
CREATE INDEX IF NOT EXISTS org_conflicts_org_id_idx ON organization_knowledge_conflicts (organization_id);
