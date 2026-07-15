-- Phase 4: Deadlines and focus blocks
-- Phase 7: Calendar rules

CREATE TABLE IF NOT EXISTS deadline_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_entity_type TEXT,
    source_entity_id TEXT,
    title TEXT NOT NULL,
    due_at TIMESTAMPTZ NOT NULL,
    severity TEXT DEFAULT 'medium',
    status TEXT DEFAULT 'active',
    linked_calendar_event_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT deadline_events_severity_check CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    CONSTRAINT deadline_events_status_check CHECK (status IN ('active', 'completed', 'overdue', 'cancelled'))
);

CREATE INDEX IF NOT EXISTS deadline_events_due_idx ON deadline_events (due_at);
CREATE INDEX IF NOT EXISTS deadline_events_status_idx ON deadline_events (status);

CREATE TABLE IF NOT EXISTS focus_blocks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT NOT NULL,
    start_at TIMESTAMPTZ NOT NULL,
    end_at TIMESTAMPTZ NOT NULL,
    purpose TEXT,
    linked_project_id TEXT,
    protection_level TEXT DEFAULT 'medium',
    status TEXT DEFAULT 'scheduled',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT focus_blocks_protection_check CHECK (protection_level IN ('low', 'medium', 'high', 'locked')),
    CONSTRAINT focus_blocks_status_check CHECK (status IN ('scheduled', 'in_progress', 'completed', 'interrupted', 'cancelled'))
);

CREATE INDEX IF NOT EXISTS focus_blocks_time_idx ON focus_blocks (start_at, end_at);

CREATE TABLE IF NOT EXISTS calendar_rules (
    rule_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    natural_language_description TEXT,
    compiled_dsl JSONB NOT NULL DEFAULT '{}',
    enabled BOOLEAN NOT NULL DEFAULT true,
    approval_mode TEXT NOT NULL DEFAULT 'suggest_only',
    last_run_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT calendar_rules_approval_check CHECK (approval_mode IN ('suggest_only', 'ask_before_execute', 'auto_execute', 'dry_run')),
    CONSTRAINT calendar_rules_dsl_is_object CHECK (jsonb_typeof(compiled_dsl) = 'object')
);

CREATE INDEX IF NOT EXISTS calendar_rules_enabled_idx ON calendar_rules (rule_id) WHERE enabled = true;
