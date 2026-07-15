-- Phase 4: Rules and templates

CREATE TABLE IF NOT EXISTS task_rules (
    rule_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    natural_language_description TEXT,
    compiled_dsl JSONB NOT NULL DEFAULT '{}',
    enabled BOOLEAN NOT NULL DEFAULT true,
    approval_mode TEXT NOT NULL DEFAULT 'suggest_only',
    last_run_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT task_rules_approval_check CHECK (approval_mode IN ('suggest_only','ask_before_execute','auto_execute','dry_run'))
);

CREATE TABLE IF NOT EXISTS task_templates (
    template_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    default_fields JSONB NOT NULL DEFAULT '{}',
    default_checklist JSONB NOT NULL DEFAULT '[]',
    default_priority TEXT DEFAULT 'medium',
    default_energy_type TEXT,
    required_documents JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO task_templates (template_id, name, description, default_fields, default_checklist, default_priority) VALUES
    ('bug','Bug Report','Standard bug report template','{"source_type":"manual","area":"engineering"}','["Steps to reproduce","Expected result","Actual result","Environment details"]','high'),
    ('feature','Feature Request','New feature specification','{"source_type":"manual","area":"engineering"}','["Requirements","Design doc","Implementation plan","Tests"]','medium'),
    ('research','Research Task','Investigation template','{"source_type":"manual","area":"research"}','["Define question","Gather sources","Document findings","Make decision"]','medium'),
    ('contract_review','Contract Review','Legal document review','{"source_type":"manual","area":"legal"}','["Check parties","Check amounts","Check deadlines","Check signatures","Check terms","Create summary"]','high'),
    ('aeat_response','AEAT Response','Spanish tax agency response','{"source_type":"manual","area":"tax"}','["Check documents","Check certificado digital","Download PDFs","Check deadline","Prepare response","Submit"]','critical'),
    ('client_followup','Client Follow-up','Post-meeting client follow-up','{"source_type":"meeting","area":"client"}','["Send follow-up email","Update project status","Create tasks from decisions","Schedule next check-in"]','medium'),
    ('invoice_review','Invoice Review','Invoice verification','{"source_type":"manual","area":"finance"}','["Check amount","Check VAT","Check dates","Check provider details","Approve or flag"]','high'),
    ('code_review','Code Review','Code review task','{"source_type":"manual","area":"engineering"}','["Review diff","Check tests","Check docs","Add comments","Approve or request changes"]','medium')
ON CONFLICT (template_id) DO NOTHING;

CREATE TABLE IF NOT EXISTS task_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
    snapshot_date TIMESTAMPTZ NOT NULL DEFAULT now(),
    data JSONB NOT NULL,
    source TEXT DEFAULT 'system',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS task_snapshots_task_idx ON task_snapshots (task_id);
