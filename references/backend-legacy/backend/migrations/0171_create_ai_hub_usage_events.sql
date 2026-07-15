CREATE TABLE IF NOT EXISTS ai_hub_usage_events (
    usage_event_id TEXT PRIMARY KEY,
    provider_id TEXT REFERENCES ai_provider_accounts(provider_id) ON DELETE SET NULL,
    model_key TEXT NOT NULL,
    route_slot TEXT NOT NULL,
    operation TEXT NOT NULL,
    status TEXT NOT NULL,
    prompt_chars INTEGER NOT NULL DEFAULT 0,
    output_chars INTEGER,
    estimated_input_tokens INTEGER NOT NULL DEFAULT 0,
    estimated_output_tokens INTEGER,
    total_duration_ns BIGINT,
    latency_ms BIGINT NOT NULL DEFAULT 0,
    error_summary TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT ai_hub_usage_events_model_key_not_empty CHECK (length(trim(model_key)) > 0),
    CONSTRAINT ai_hub_usage_events_route_slot_check CHECK (
        route_slot IN (
            'default_chat',
            'reasoning',
            'summarization',
            'mail_intelligence',
            'reply_draft',
            'extraction',
            'embeddings',
            'meeting_prep'
        )
    ),
    CONSTRAINT ai_hub_usage_events_operation_check CHECK (operation IN ('chat', 'embed')),
    CONSTRAINT ai_hub_usage_events_status_check CHECK (status IN ('completed', 'failed')),
    CONSTRAINT ai_hub_usage_events_prompt_chars_non_negative CHECK (prompt_chars >= 0),
    CONSTRAINT ai_hub_usage_events_output_chars_non_negative CHECK (output_chars IS NULL OR output_chars >= 0),
    CONSTRAINT ai_hub_usage_events_input_tokens_non_negative CHECK (estimated_input_tokens >= 0),
    CONSTRAINT ai_hub_usage_events_output_tokens_non_negative CHECK (
        estimated_output_tokens IS NULL OR estimated_output_tokens >= 0
    ),
    CONSTRAINT ai_hub_usage_events_latency_non_negative CHECK (latency_ms >= 0)
);

CREATE INDEX IF NOT EXISTS ai_hub_usage_events_provider_created_idx
    ON ai_hub_usage_events (provider_id, created_at DESC);

CREATE INDEX IF NOT EXISTS ai_hub_usage_events_route_created_idx
    ON ai_hub_usage_events (route_slot, created_at DESC);

CREATE INDEX IF NOT EXISTS ai_hub_usage_events_created_idx
    ON ai_hub_usage_events (created_at DESC);
