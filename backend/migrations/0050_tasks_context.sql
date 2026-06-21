-- Phase 2: Context packs, evidence, relations, checklists, subtasks

CREATE TABLE IF NOT EXISTS task_context_packs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
    summary TEXT,
    source_summary TEXT,
    open_questions JSONB NOT NULL DEFAULT '[]',
    blockers JSONB NOT NULL DEFAULT '[]',
    risks JSONB NOT NULL DEFAULT '[]',
    suggested_next_action TEXT,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    model TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS task_context_packs_task_idx ON task_context_packs (task_id);

CREATE TABLE IF NOT EXISTS task_evidence (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
    source_type TEXT NOT NULL,
    source_id TEXT NOT NULL,
    quote TEXT,
    confidence REAL DEFAULT 1.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT task_evidence_confidence_check CHECK (confidence >= 0.0 AND confidence <= 1.0)
);
CREATE INDEX IF NOT EXISTS task_evidence_task_idx ON task_evidence (task_id);

CREATE TABLE IF NOT EXISTS task_relations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    relation_type TEXT NOT NULL,
    source TEXT DEFAULT 'manual',
    confidence REAL DEFAULT 1.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT task_relations_type_check CHECK (relation_type IN ('blocks','blocked_by','depends_on','relates_to','duplicates','caused_by','derived_from','follow_up_for','parent','subtask'))
);
CREATE INDEX IF NOT EXISTS task_relations_task_idx ON task_relations (task_id);

CREATE TABLE IF NOT EXISTS task_checklists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
    items JSONB NOT NULL DEFAULT '[]',
    source TEXT DEFAULT 'manual',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS task_checklists_task_idx ON task_checklists (task_id);

CREATE TABLE IF NOT EXISTS task_subtasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
    child_task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
    sort_order INT DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(parent_task_id, child_task_id)
);
CREATE INDEX IF NOT EXISTS task_subtasks_parent_idx ON task_subtasks (parent_task_id);
