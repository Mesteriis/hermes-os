-- Phase 1: Event participants, relations, context, agendas, checklists
-- Phase 3: Meeting notes, outcomes, recordings, transcripts in 0046 below

CREATE TABLE IF NOT EXISTS event_participants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id TEXT NOT NULL REFERENCES calendar_events(event_id) ON DELETE CASCADE,
    person_id TEXT,
    email TEXT NOT NULL,
    display_name TEXT,
    role TEXT DEFAULT 'attendee',
    response_status TEXT DEFAULT 'needs_action',
    organization_id TEXT,
    timezone TEXT,
    confidence REAL DEFAULT 0.7,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT event_participants_role_check CHECK (role IN ('organizer', 'required', 'optional', 'attendee', 'speaker')),
    CONSTRAINT event_participants_response_check CHECK (response_status IN ('needs_action', 'accepted', 'declined', 'tentative', 'no_response'))
);

CREATE INDEX IF NOT EXISTS event_participants_event_idx ON event_participants (event_id);
CREATE INDEX IF NOT EXISTS event_participants_person_idx ON event_participants (person_id);

CREATE TABLE IF NOT EXISTS event_relations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id TEXT NOT NULL REFERENCES calendar_events(event_id) ON DELETE CASCADE,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    relation_type TEXT NOT NULL,
    source TEXT DEFAULT 'manual',
    confidence REAL DEFAULT 1.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT event_relations_entity_type_check CHECK (entity_type IN ('person', 'organization', 'project', 'document', 'task', 'email', 'note', 'decision', 'obligation', 'recording'))
);

CREATE INDEX IF NOT EXISTS event_relations_event_idx ON event_relations (event_id);
CREATE INDEX IF NOT EXISTS event_relations_entity_idx ON event_relations (entity_type, entity_id);

CREATE TABLE IF NOT EXISTS event_context_packs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id TEXT NOT NULL REFERENCES calendar_events(event_id) ON DELETE CASCADE,
    summary TEXT,
    participants_summary TEXT,
    documents JSONB NOT NULL DEFAULT '[]',
    tasks JSONB NOT NULL DEFAULT '[]',
    open_questions JSONB NOT NULL DEFAULT '[]',
    risks JSONB NOT NULL DEFAULT '[]',
    suggested_agenda JSONB NOT NULL DEFAULT '[]',
    suggested_actions JSONB NOT NULL DEFAULT '[]',
    generated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    model TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT event_context_packs_docs_is_array CHECK (jsonb_typeof(documents) = 'array'),
    CONSTRAINT event_context_packs_tasks_is_array CHECK (jsonb_typeof(tasks) = 'array'),
    CONSTRAINT event_context_packs_questions_is_array CHECK (jsonb_typeof(open_questions) = 'array'),
    CONSTRAINT event_context_packs_risks_is_array CHECK (jsonb_typeof(risks) = 'array'),
    CONSTRAINT event_context_packs_agenda_is_array CHECK (jsonb_typeof(suggested_agenda) = 'array'),
    CONSTRAINT event_context_packs_actions_is_array CHECK (jsonb_typeof(suggested_actions) = 'array')
);

CREATE INDEX IF NOT EXISTS event_context_packs_event_idx ON event_context_packs (event_id);

CREATE TABLE IF NOT EXISTS event_agendas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id TEXT NOT NULL REFERENCES calendar_events(event_id) ON DELETE CASCADE,
    items JSONB NOT NULL DEFAULT '[]',
    source TEXT DEFAULT 'manual',
    created_by TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT event_agendas_items_is_array CHECK (jsonb_typeof(items) = 'array')
);

CREATE INDEX IF NOT EXISTS event_agendas_event_idx ON event_agendas (event_id);

CREATE TABLE IF NOT EXISTS event_checklists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id TEXT NOT NULL REFERENCES calendar_events(event_id) ON DELETE CASCADE,
    items JSONB NOT NULL DEFAULT '[]',
    source TEXT DEFAULT 'manual',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT event_checklists_items_is_array CHECK (jsonb_typeof(items) = 'array')
);

CREATE INDEX IF NOT EXISTS event_checklists_event_idx ON event_checklists (event_id);

-- Phase 3 tables (in same migration for simplicity)

CREATE TABLE IF NOT EXISTS meeting_notes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id TEXT NOT NULL REFERENCES calendar_events(event_id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    format TEXT DEFAULT 'markdown',
    source TEXT DEFAULT 'manual',
    linked_note_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS meeting_notes_event_idx ON meeting_notes (event_id);

CREATE TABLE IF NOT EXISTS meeting_outcomes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id TEXT NOT NULL REFERENCES calendar_events(event_id) ON DELETE CASCADE,
    outcome_type TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    owner_person_id TEXT,
    due_date TIMESTAMPTZ,
    source TEXT DEFAULT 'manual',
    confidence REAL DEFAULT 1.0,
    linked_entity_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT meeting_outcomes_type_check CHECK (outcome_type IN ('decision', 'task', 'promise', 'risk', 'question', 'document_request', 'follow_up', 'agreement', 'blocker'))
);

CREATE INDEX IF NOT EXISTS meeting_outcomes_event_idx ON meeting_outcomes (event_id);
CREATE INDEX IF NOT EXISTS meeting_outcomes_owner_idx ON meeting_outcomes (owner_person_id);

CREATE TABLE IF NOT EXISTS event_recordings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id TEXT NOT NULL REFERENCES calendar_events(event_id) ON DELETE CASCADE,
    file_path TEXT,
    source TEXT DEFAULT 'manual',
    duration_seconds INTEGER,
    transcript_id UUID,
    processing_status TEXT DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT event_recordings_status_check CHECK (processing_status IN ('pending', 'transcribing', 'transcribed', 'failed'))
);

CREATE INDEX IF NOT EXISTS event_recordings_event_idx ON event_recordings (event_id);

CREATE TABLE IF NOT EXISTS event_transcripts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id TEXT NOT NULL REFERENCES calendar_events(event_id) ON DELETE CASCADE,
    text TEXT NOT NULL,
    language TEXT DEFAULT 'en',
    summary TEXT,
    model TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS event_transcripts_event_idx ON event_transcripts (event_id);
