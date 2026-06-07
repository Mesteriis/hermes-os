-- Phases 5-7: Enrichment engine, expertise, trust and risk

-- Enrichment results: tracking enrichment attempts from external sources
CREATE TABLE IF NOT EXISTS enrichment_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id TEXT NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE,
    source TEXT NOT NULL,
    url TEXT,
    data JSONB NOT NULL DEFAULT '{}'::jsonb,
    confidence REAL NOT NULL DEFAULT 0.5,
    status TEXT NOT NULL DEFAULT 'pending',
    last_checked_at TIMESTAMPTZ,
    applied_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT enrichment_results_confidence_range CHECK (confidence >= 0 AND confidence <= 1),
    CONSTRAINT enrichment_results_status_check CHECK (status IN ('pending', 'applied', 'rejected', 'conflict')),
    CONSTRAINT enrichment_results_data_is_object CHECK (jsonb_typeof(data) = 'object')
);

CREATE INDEX IF NOT EXISTS enrichment_results_person_id_idx ON enrichment_results (person_id);
CREATE INDEX IF NOT EXISTS enrichment_results_status_idx ON enrichment_results (person_id, status);

-- Person expertise: skills and domains
CREATE TABLE IF NOT EXISTS person_expertise (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id TEXT NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE,
    skill TEXT NOT NULL,
    domain TEXT,
    evidence TEXT,
    source TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 0.5,
    last_verified_at TIMESTAMPTZ,
    endorsed_by_person_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT person_expertise_confidence_range CHECK (confidence >= 0 AND confidence <= 1),
    CONSTRAINT person_expertise_skill_not_empty CHECK (length(trim(skill)) > 0)
);

CREATE INDEX IF NOT EXISTS person_expertise_person_id_idx ON person_expertise (person_id);
CREATE INDEX IF NOT EXISTS person_expertise_skill_idx ON person_expertise (skill);

-- Person promises: tracked promises and commitments
CREATE TABLE IF NOT EXISTS person_promises (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id TEXT NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE,
    description TEXT NOT NULL,
    source_message_id TEXT,
    promised_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    due_at TIMESTAMPTZ,
    fulfilled_at TIMESTAMPTZ,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT person_promises_status_check CHECK (status IN ('pending', 'fulfilled', 'broken', 'forgiven')),
    CONSTRAINT person_promises_desc_not_empty CHECK (length(trim(description)) > 0)
);

CREATE INDEX IF NOT EXISTS person_promises_person_id_idx ON person_promises (person_id);
CREATE INDEX IF NOT EXISTS person_promises_status_idx ON person_promises (person_id, status);

-- Person risks: risk tracking
CREATE TABLE IF NOT EXISTS person_risks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id TEXT NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE,
    risk_type TEXT NOT NULL,
    description TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'medium',
    source TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 0.5,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at TIMESTAMPTZ,
    resolution TEXT,

    CONSTRAINT person_risks_severity_check CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    CONSTRAINT person_risks_confidence_range CHECK (confidence >= 0 AND confidence <= 1)
);

CREATE INDEX IF NOT EXISTS person_risks_person_id_idx ON person_risks (person_id);

-- Phase 8 prep: health columns on persons
ALTER TABLE persons
    ADD COLUMN IF NOT EXISTS health_status TEXT DEFAULT 'healthy',
    ADD COLUMN IF NOT EXISTS last_health_check TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS communication_gap_days INT DEFAULT 0,
    ADD COLUMN IF NOT EXISTS watchlist BOOLEAN NOT NULL DEFAULT false;

ALTER TABLE persons
    ADD CONSTRAINT persons_health_status_check CHECK (health_status IN ('healthy', 'needs_attention', 'at_risk', 'dormant'));

CREATE INDEX IF NOT EXISTS persons_watchlist_idx ON persons (person_id) WHERE watchlist = true;
