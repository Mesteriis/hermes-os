CREATE TABLE IF NOT EXISTS observation_kind_definitions (
    kind_definition_id TEXT PRIMARY KEY,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    category TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT observation_kind_definitions_id_not_empty CHECK (length(trim(kind_definition_id)) > 0),
    CONSTRAINT observation_kind_definitions_code_not_empty CHECK (length(trim(code)) > 0),
    CONSTRAINT observation_kind_definitions_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT observation_kind_definitions_version_positive CHECK (version > 0),
    CONSTRAINT observation_kind_definitions_category_not_empty CHECK (length(trim(category)) > 0),
    CONSTRAINT observation_kind_definitions_code_upper CHECK (code = upper(code)),
    UNIQUE (code, version)
);

INSERT INTO observation_kind_definitions (
    kind_definition_id,
    code,
    name,
    version,
    category,
    description
)
VALUES
    (
        'observation_kind:v1:communication_message',
        'COMMUNICATION_MESSAGE',
        'Communication message',
        1,
        'communication',
        'Provider or manual communication message captured as evidence.'
    ),
    (
        'observation_kind:v1:communication_message_deleted',
        'COMMUNICATION_MESSAGE_DELETED',
        'Communication message deleted',
        1,
        'communication',
        'Observed provider-side deletion or disappearance of a communication message.'
    ),
    (
        'observation_kind:v1:communication_attachment',
        'COMMUNICATION_ATTACHMENT',
        'Communication attachment',
        1,
        'communication',
        'Attachment captured from a communication source.'
    ),
    (
        'observation_kind:v1:meeting',
        'MEETING',
        'Meeting',
        1,
        'meeting',
        'Meeting occurrence captured as evidence.'
    ),
    (
        'observation_kind:v1:meeting_recording',
        'MEETING_RECORDING',
        'Meeting recording',
        1,
        'meeting',
        'Meeting audio or video recording reference captured as evidence.'
    ),
    (
        'observation_kind:v1:meeting_transcript',
        'MEETING_TRANSCRIPT',
        'Meeting transcript',
        1,
        'meeting',
        'Meeting transcript captured as evidence.'
    ),
    (
        'observation_kind:v1:document',
        'DOCUMENT',
        'Document',
        1,
        'document',
        'Imported or manually created document evidence.'
    ),
    (
        'observation_kind:v1:voice_recording',
        'VOICE_RECORDING',
        'Voice recording',
        1,
        'voice',
        'Voice memo or recording captured as evidence.'
    ),
    (
        'observation_kind:v1:browser_capture',
        'BROWSER_CAPTURE',
        'Browser capture',
        1,
        'browser',
        'User-driven browser capture evidence.'
    ),
    (
        'observation_kind:v1:contact_record',
        'CONTACT_RECORD',
        'Contact record',
        1,
        'identity',
        'Provider contact or address book record captured as evidence.'
    ),
    (
        'observation_kind:v1:calendar_event',
        'CALENDAR_EVENT',
        'Calendar event',
        1,
        'calendar',
        'Calendar event captured as evidence.'
    ),
    (
        'observation_kind:v1:calendar_event_deleted',
        'CALENDAR_EVENT_DELETED',
        'Calendar event deleted',
        1,
        'calendar',
        'Observed deletion or archival removal of a calendar event.'
    )
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();

CREATE TABLE IF NOT EXISTS observations (
    observation_id TEXT PRIMARY KEY,
    kind_definition_id TEXT NOT NULL REFERENCES observation_kind_definitions(kind_definition_id),
    origin_kind TEXT NOT NULL,
    vault_source_id TEXT,
    observed_at TIMESTAMPTZ NOT NULL,
    captured_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    payload JSONB NOT NULL,
    confidence DOUBLE PRECISION NOT NULL,
    content_hash TEXT NOT NULL,
    source_ref TEXT NOT NULL,
    provenance JSONB NOT NULL DEFAULT '{}'::jsonb,

    CONSTRAINT observations_id_not_empty CHECK (length(trim(observation_id)) > 0),
    CONSTRAINT observations_origin_kind CHECK (
        origin_kind IN (
            'vault_source',
            'manual',
            'browser_capture',
            'voice_memo',
            'file_import',
            'local_runtime',
            'test_fixture'
        )
    ),
    CONSTRAINT observations_vault_source_id_not_empty CHECK (
        vault_source_id IS NULL OR length(trim(vault_source_id)) > 0
    ),
    CONSTRAINT observations_payload_is_object CHECK (jsonb_typeof(payload) = 'object'),
    CONSTRAINT observations_confidence_range CHECK (confidence >= 0.0 AND confidence <= 1.0),
    CONSTRAINT observations_content_hash_not_empty CHECK (length(trim(content_hash)) > 0),
    CONSTRAINT observations_source_ref_not_empty CHECK (length(trim(source_ref)) > 0),
    CONSTRAINT observations_provenance_is_object CHECK (jsonb_typeof(provenance) = 'object')
);

CREATE INDEX IF NOT EXISTS observations_kind_captured_idx
    ON observations (kind_definition_id, captured_at DESC);

CREATE INDEX IF NOT EXISTS observations_vault_source_idx
    ON observations (vault_source_id)
    WHERE vault_source_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS observations_source_ref_idx
    ON observations (source_ref, captured_at DESC);

CREATE INDEX IF NOT EXISTS observations_content_hash_idx
    ON observations (content_hash);

CREATE OR REPLACE FUNCTION prevent_observations_mutation()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    RAISE EXCEPTION 'observations are append-only; create a new observation instead'
        USING ERRCODE = '55000';
END;
$$;

DROP TRIGGER IF EXISTS observations_append_only_update ON observations;
CREATE TRIGGER observations_append_only_update
    BEFORE UPDATE ON observations
    FOR EACH ROW
    EXECUTE FUNCTION prevent_observations_mutation();

DROP TRIGGER IF EXISTS observations_append_only_delete ON observations;
CREATE TRIGGER observations_append_only_delete
    BEFORE DELETE ON observations
    FOR EACH ROW
    EXECUTE FUNCTION prevent_observations_mutation();

CREATE TABLE IF NOT EXISTS observation_links (
    observation_id TEXT NOT NULL REFERENCES observations(observation_id),
    domain TEXT NOT NULL,
    entity_kind TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    relationship_kind TEXT NOT NULL DEFAULT 'evidence_for',
    confidence DOUBLE PRECISION NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (observation_id, domain, entity_kind, entity_id, relationship_kind),

    CONSTRAINT observation_links_domain_not_empty CHECK (length(trim(domain)) > 0),
    CONSTRAINT observation_links_entity_kind_not_empty CHECK (length(trim(entity_kind)) > 0),
    CONSTRAINT observation_links_entity_id_not_empty CHECK (length(trim(entity_id)) > 0),
    CONSTRAINT observation_links_relationship_kind_not_empty CHECK (length(trim(relationship_kind)) > 0),
    CONSTRAINT observation_links_confidence_range CHECK (confidence >= 0.0 AND confidence <= 1.0),
    CONSTRAINT observation_links_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS observation_links_entity_idx
    ON observation_links (domain, entity_kind, entity_id);

CREATE TABLE IF NOT EXISTS observation_ingestion_runs (
    ingestion_run_id TEXT PRIMARY KEY,
    observation_id TEXT NOT NULL REFERENCES observations(observation_id),
    pipeline TEXT NOT NULL,
    status TEXT NOT NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    finished_at TIMESTAMPTZ,
    output JSONB NOT NULL DEFAULT '{}'::jsonb,
    error_message TEXT,

    CONSTRAINT observation_ingestion_runs_id_not_empty CHECK (length(trim(ingestion_run_id)) > 0),
    CONSTRAINT observation_ingestion_runs_pipeline_not_empty CHECK (length(trim(pipeline)) > 0),
    CONSTRAINT observation_ingestion_runs_status CHECK (
        status IN ('running', 'succeeded', 'failed', 'skipped')
    ),
    CONSTRAINT observation_ingestion_runs_output_is_object CHECK (jsonb_typeof(output) = 'object'),
    CONSTRAINT observation_ingestion_runs_error_not_empty CHECK (
        error_message IS NULL OR length(trim(error_message)) > 0
    )
);

CREATE INDEX IF NOT EXISTS observation_ingestion_runs_observation_idx
    ON observation_ingestion_runs (observation_id, started_at DESC);

CREATE TABLE IF NOT EXISTS review_items (
    review_item_id TEXT PRIMARY KEY,
    item_kind TEXT NOT NULL,
    title TEXT NOT NULL,
    summary TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'new',
    target_domain TEXT,
    target_entity_kind TEXT,
    target_entity_id TEXT,
    confidence DOUBLE PRECISION NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT review_items_id_not_empty CHECK (length(trim(review_item_id)) > 0),
    CONSTRAINT review_items_item_kind CHECK (
        item_kind IN (
            'new_person',
            'new_organization',
            'potential_task',
            'potential_obligation',
            'potential_decision',
            'potential_relationship',
            'potential_project',
            'knowledge_candidate'
        )
    ),
    CONSTRAINT review_items_title_not_empty CHECK (length(trim(title)) > 0),
    CONSTRAINT review_items_summary_not_empty CHECK (length(trim(summary)) > 0),
    CONSTRAINT review_items_status CHECK (
        status IN ('new', 'in_review', 'approved', 'promoted', 'dismissed', 'archived')
    ),
    CONSTRAINT review_items_target_domain_not_empty CHECK (
        target_domain IS NULL OR length(trim(target_domain)) > 0
    ),
    CONSTRAINT review_items_target_entity_kind_not_empty CHECK (
        target_entity_kind IS NULL OR length(trim(target_entity_kind)) > 0
    ),
    CONSTRAINT review_items_target_entity_id_not_empty CHECK (
        target_entity_id IS NULL OR length(trim(target_entity_id)) > 0
    ),
    CONSTRAINT review_items_target_complete CHECK (
        (target_domain IS NULL AND target_entity_kind IS NULL AND target_entity_id IS NULL)
        OR
        (target_domain IS NOT NULL AND target_entity_kind IS NOT NULL AND target_entity_id IS NOT NULL)
    ),
    CONSTRAINT review_items_confidence_range CHECK (confidence >= 0.0 AND confidence <= 1.0),
    CONSTRAINT review_items_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS review_items_status_idx
    ON review_items (status, updated_at DESC);

CREATE INDEX IF NOT EXISTS review_items_kind_status_idx
    ON review_items (item_kind, status, updated_at DESC);

CREATE INDEX IF NOT EXISTS review_items_target_idx
    ON review_items (target_domain, target_entity_kind, target_entity_id)
    WHERE target_domain IS NOT NULL;

CREATE TABLE IF NOT EXISTS review_item_evidence (
    review_item_id TEXT NOT NULL REFERENCES review_items(review_item_id) ON DELETE CASCADE,
    observation_id TEXT NOT NULL REFERENCES observations(observation_id),
    evidence_role TEXT NOT NULL DEFAULT 'primary',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (review_item_id, observation_id, evidence_role),

    CONSTRAINT review_item_evidence_role_not_empty CHECK (length(trim(evidence_role)) > 0),
    CONSTRAINT review_item_evidence_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS review_item_evidence_observation_idx
    ON review_item_evidence (observation_id);

CREATE TABLE IF NOT EXISTS context_packs (
    context_pack_id TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    subject_id TEXT NOT NULL,
    content JSONB NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    rebuildable BOOLEAN NOT NULL DEFAULT TRUE,
    built_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT context_packs_id_not_empty CHECK (length(trim(context_pack_id)) > 0),
    CONSTRAINT context_packs_kind CHECK (
        kind IN ('persona', 'meeting', 'task', 'calendar', 'project')
    ),
    CONSTRAINT context_packs_subject_id_not_empty CHECK (length(trim(subject_id)) > 0),
    CONSTRAINT context_packs_content_is_object CHECK (jsonb_typeof(content) = 'object'),
    CONSTRAINT context_packs_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object'),
    UNIQUE (kind, subject_id)
);

CREATE INDEX IF NOT EXISTS context_packs_kind_subject_idx
    ON context_packs (kind, subject_id);

CREATE TABLE IF NOT EXISTS context_pack_sources (
    context_pack_id TEXT NOT NULL REFERENCES context_packs(context_pack_id) ON DELETE CASCADE,
    source_kind TEXT NOT NULL,
    source_id TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'source',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (context_pack_id, source_kind, source_id, role),

    CONSTRAINT context_pack_sources_kind CHECK (
        source_kind IN (
            'observation',
            'domain_entity',
            'knowledge',
            'relationship',
            'decision',
            'task',
            'obligation',
            'document',
            'calendar_event',
            'project'
        )
    ),
    CONSTRAINT context_pack_sources_source_id_not_empty CHECK (length(trim(source_id)) > 0),
    CONSTRAINT context_pack_sources_role_not_empty CHECK (length(trim(role)) > 0),
    CONSTRAINT context_pack_sources_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS context_pack_sources_source_idx
    ON context_pack_sources (source_kind, source_id);
