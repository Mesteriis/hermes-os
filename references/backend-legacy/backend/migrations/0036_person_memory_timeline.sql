-- Phases 2-3: Person memory and relationship timeline

-- Person facts: extracted facts with source and confidence
CREATE TABLE IF NOT EXISTS person_facts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id TEXT NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE,
    fact_type TEXT NOT NULL,
    value TEXT NOT NULL,
    source TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 1.0,
    last_verified_at TIMESTAMPTZ,
    valid_from TIMESTAMPTZ,
    valid_to TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT person_facts_confidence_range CHECK (confidence >= 0 AND confidence <= 1)
);

CREATE INDEX IF NOT EXISTS person_facts_person_id_idx ON person_facts (person_id);
CREATE INDEX IF NOT EXISTS person_facts_type_idx ON person_facts (fact_type);

-- Person memory cards: important things to remember
CREATE TABLE IF NOT EXISTS person_memory_cards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id TEXT NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    source TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 1.0,
    importance SMALLINT NOT NULL DEFAULT 5,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_verified_at TIMESTAMPTZ,

    CONSTRAINT person_memory_cards_confidence_range CHECK (confidence >= 0 AND confidence <= 1),
    CONSTRAINT person_memory_cards_importance_range CHECK (importance >= 1 AND importance <= 10),
    CONSTRAINT person_memory_cards_title_not_empty CHECK (length(trim(title)) > 0)
);

CREATE INDEX IF NOT EXISTS person_memory_cards_person_id_idx ON person_memory_cards (person_id);

-- Person preferences: communication preferences
CREATE TABLE IF NOT EXISTS person_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id TEXT NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE,
    preference_type TEXT NOT NULL,
    value TEXT NOT NULL,
    source TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 1.0,
    last_verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT person_preferences_unique UNIQUE (person_id, preference_type),
    CONSTRAINT person_preferences_confidence_range CHECK (confidence >= 0 AND confidence <= 1)
);

CREATE INDEX IF NOT EXISTS person_preferences_person_id_idx ON person_preferences (person_id);

-- Person snapshots: state at a point in time
CREATE TABLE IF NOT EXISTS person_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id TEXT NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE,
    snapshot_date TIMESTAMPTZ NOT NULL DEFAULT now(),
    data JSONB NOT NULL,
    source TEXT NOT NULL DEFAULT 'manual',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT person_snapshots_data_is_object CHECK (jsonb_typeof(data) = 'object')
);

CREATE INDEX IF NOT EXISTS person_snapshots_person_id_idx ON person_snapshots (person_id);

-- Person knowledge conflicts: detected contradictions
CREATE TABLE IF NOT EXISTS person_knowledge_conflicts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id TEXT NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE,
    field TEXT NOT NULL,
    value_a TEXT NOT NULL,
    value_b TEXT NOT NULL,
    source_a TEXT NOT NULL,
    source_b TEXT NOT NULL,
    detected_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at TIMESTAMPTZ,
    resolution TEXT
);

CREATE INDEX IF NOT EXISTS person_knowledge_conflicts_person_id_idx
    ON person_knowledge_conflicts (person_id);

-- Relationship events: timeline of relationship events
CREATE TABLE IF NOT EXISTS relationship_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id TEXT NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE,
    event_type TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    occurred_at TIMESTAMPTZ NOT NULL,
    source TEXT NOT NULL,
    related_entity_id TEXT,
    related_entity_kind TEXT,
    confidence REAL NOT NULL DEFAULT 1.0,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT relationship_events_confidence_range CHECK (confidence >= 0 AND confidence <= 1),
    CONSTRAINT relationship_events_title_not_empty CHECK (length(trim(title)) > 0),
    CONSTRAINT relationship_events_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS relationship_events_person_id_idx ON relationship_events (person_id);
CREATE INDEX IF NOT EXISTS relationship_events_type_idx ON relationship_events (event_type);
CREATE INDEX IF NOT EXISTS relationship_events_occurred_at_idx ON relationship_events (occurred_at);

-- Phase 4: Communication DNA columns on persons table
ALTER TABLE persons
    ADD COLUMN IF NOT EXISTS communication_style TEXT,
    ADD COLUMN IF NOT EXISTS verbosity TEXT,
    ADD COLUMN IF NOT EXISTS technical_depth TEXT,
    ADD COLUMN IF NOT EXISTS question_frequency TEXT,
    ADD COLUMN IF NOT EXISTS call_preference TEXT,
    ADD COLUMN IF NOT EXISTS response_pattern TEXT,
    ADD COLUMN IF NOT EXISTS active_hours JSONB,
    ADD COLUMN IF NOT EXISTS active_days JSONB;
