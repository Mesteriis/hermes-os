-- Phase 3-4: Timeline, templates, portals, procedures, playbooks

CREATE TABLE IF NOT EXISTS organization_timeline_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
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
    CONSTRAINT org_timeline_confidence_range CHECK (confidence >= 0 AND confidence <= 1),
    CONSTRAINT org_timeline_title_not_empty CHECK (length(trim(title)) > 0)
);

CREATE TABLE IF NOT EXISTS organization_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    template_type TEXT NOT NULL DEFAULT 'email',
    subject TEXT,
    body TEXT,
    language TEXT,
    tone TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT org_templates_type_check CHECK (template_type IN ('email', 'document'))
);

CREATE TABLE IF NOT EXISTS organization_portals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    portal_type TEXT NOT NULL DEFAULT 'customer',
    login_hint TEXT,
    secret_reference TEXT,
    last_used_at TIMESTAMPTZ,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT org_portals_type_check CHECK (portal_type IN ('tax', 'customer', 'banking', 'support', 'billing', 'admin', 'app'))
);

CREATE TABLE IF NOT EXISTS organization_procedures (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    steps JSONB NOT NULL DEFAULT '[]'::jsonb,
    source TEXT NOT NULL DEFAULT 'manual',
    confidence REAL NOT NULL DEFAULT 1.0,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT org_procedures_steps_is_array CHECK (jsonb_typeof(steps) = 'array')
);

CREATE TABLE IF NOT EXISTS organization_playbooks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    trigger_condition TEXT,
    steps JSONB NOT NULL DEFAULT '[]'::jsonb,
    approval_mode TEXT NOT NULL DEFAULT 'confirm',
    enabled BOOL NOT NULL DEFAULT false,
    last_run_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT org_playbooks_approval_check CHECK (approval_mode IN ('auto', 'confirm', 'disabled')),
    CONSTRAINT org_playbooks_steps_is_array CHECK (jsonb_typeof(steps) = 'array')
);

CREATE TABLE IF NOT EXISTS organization_quick_actions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    action_type TEXT NOT NULL,
    action_params JSONB NOT NULL DEFAULT '{}'::jsonb,
    sort_order INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS org_timeline_org_id_idx ON organization_timeline_events (organization_id);
CREATE INDEX IF NOT EXISTS org_portals_org_id_idx ON organization_portals (organization_id);
CREATE INDEX IF NOT EXISTS org_procedures_org_id_idx ON organization_procedures (organization_id);
CREATE INDEX IF NOT EXISTS org_playbooks_org_id_idx ON organization_playbooks (organization_id);
