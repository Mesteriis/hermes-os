CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE IF NOT EXISTS ai_agent_runs (
    run_id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    status TEXT NOT NULL,
    chat_model TEXT NOT NULL,
    embedding_model TEXT NOT NULL,
    prompt_template_version TEXT NOT NULL,
    model_config JSONB NOT NULL DEFAULT '{}'::jsonb,
    query TEXT NOT NULL,
    answer TEXT,
    citations JSONB NOT NULL DEFAULT '[]'::jsonb,
    error_summary TEXT,
    actor_id TEXT NOT NULL,
    causation_id TEXT,
    correlation_id TEXT,
    requested_event_id TEXT,
    completed_event_id TEXT,
    failed_event_id TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    duration_ms BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT ai_agent_runs_status_check
        CHECK (status IN ('requested', 'completed', 'failed')),
    CONSTRAINT ai_agent_runs_agent_id_not_empty CHECK (length(trim(agent_id)) > 0),
    CONSTRAINT ai_agent_runs_chat_model_not_empty CHECK (length(trim(chat_model)) > 0),
    CONSTRAINT ai_agent_runs_embedding_model_not_empty CHECK (length(trim(embedding_model)) > 0),
    CONSTRAINT ai_agent_runs_prompt_template_version_not_empty
        CHECK (length(trim(prompt_template_version)) > 0),
    CONSTRAINT ai_agent_runs_query_not_empty CHECK (length(trim(query)) > 0),
    CONSTRAINT ai_agent_runs_actor_id_not_empty CHECK (length(trim(actor_id)) > 0),
    CONSTRAINT ai_agent_runs_model_config_is_object CHECK (jsonb_typeof(model_config) = 'object'),
    CONSTRAINT ai_agent_runs_citations_is_array CHECK (jsonb_typeof(citations) = 'array')
);

CREATE INDEX IF NOT EXISTS ai_agent_runs_started_at_idx
    ON ai_agent_runs (started_at DESC, run_id);

CREATE INDEX IF NOT EXISTS ai_agent_runs_agent_status_idx
    ON ai_agent_runs (agent_id, status, started_at DESC);

CREATE TABLE IF NOT EXISTS semantic_embeddings (
    semantic_embedding_id TEXT PRIMARY KEY,
    source_kind TEXT NOT NULL,
    source_id TEXT NOT NULL,
    title TEXT NOT NULL,
    source_text TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    embedding_model TEXT NOT NULL,
    embedding_dimension INTEGER NOT NULL,
    embedding halfvec(2560) NOT NULL,
    graph_node_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT semantic_embeddings_source_kind_check
        CHECK (source_kind IN ('message', 'document', 'project', 'task', 'contact')),
    CONSTRAINT semantic_embeddings_source_id_not_empty CHECK (length(trim(source_id)) > 0),
    CONSTRAINT semantic_embeddings_title_not_empty CHECK (length(trim(title)) > 0),
    CONSTRAINT semantic_embeddings_source_text_not_empty CHECK (length(trim(source_text)) > 0),
    CONSTRAINT semantic_embeddings_content_hash_not_empty CHECK (length(trim(content_hash)) > 0),
    CONSTRAINT semantic_embeddings_model_not_empty CHECK (length(trim(embedding_model)) > 0),
    CONSTRAINT semantic_embeddings_dimension_check CHECK (embedding_dimension = 2560),
    UNIQUE (source_kind, source_id, embedding_model)
);

CREATE INDEX IF NOT EXISTS semantic_embeddings_source_idx
    ON semantic_embeddings (source_kind, source_id);

CREATE INDEX IF NOT EXISTS semantic_embeddings_model_idx
    ON semantic_embeddings (embedding_model, updated_at DESC);

CREATE INDEX IF NOT EXISTS semantic_embeddings_embedding_hnsw_idx
    ON semantic_embeddings
    USING hnsw (embedding halfvec_cosine_ops);

ALTER TABLE task_candidates
    ADD COLUMN IF NOT EXISTS agent_run_id TEXT
        REFERENCES ai_agent_runs(run_id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS task_candidates_agent_run_idx
    ON task_candidates (agent_run_id)
    WHERE agent_run_id IS NOT NULL;
