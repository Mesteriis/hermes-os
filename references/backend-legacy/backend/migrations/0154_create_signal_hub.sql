CREATE TABLE IF NOT EXISTS signal_sources (
    id UUID PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    category TEXT NOT NULL,
    source_kind TEXT NOT NULL,
    default_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    supports_connections BOOLEAN NOT NULL DEFAULT FALSE,
    supports_runtime BOOLEAN NOT NULL DEFAULT FALSE,
    supports_replay BOOLEAN NOT NULL DEFAULT FALSE,
    supports_pause BOOLEAN NOT NULL DEFAULT FALSE,
    supports_mute BOOLEAN NOT NULL DEFAULT FALSE,
    capability_schema_version INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT signal_sources_code_not_empty CHECK (length(trim(code)) > 0),
    CONSTRAINT signal_sources_display_name_not_empty CHECK (length(trim(display_name)) > 0),
    CONSTRAINT signal_sources_category_not_empty CHECK (length(trim(category)) > 0),
    CONSTRAINT signal_sources_kind_not_empty CHECK (length(trim(source_kind)) > 0),
    CONSTRAINT signal_sources_capability_schema_version_positive CHECK (capability_schema_version > 0)
);

CREATE INDEX IF NOT EXISTS signal_sources_category_idx
    ON signal_sources (category, code);

CREATE TABLE IF NOT EXISTS signal_connections (
    id UUID PRIMARY KEY,
    source_code TEXT NOT NULL REFERENCES signal_sources(code),
    display_name TEXT NOT NULL,
    status TEXT NOT NULL,
    profile TEXT,
    settings JSONB NOT NULL DEFAULT '{}'::jsonb,
    secret_ref TEXT,
    connected_at TIMESTAMPTZ,
    last_seen_at TIMESTAMPTZ,
    last_signal_at TIMESTAMPTZ,
    last_sync_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT signal_connections_display_name_not_empty CHECK (length(trim(display_name)) > 0),
    CONSTRAINT signal_connections_status_not_empty CHECK (length(trim(status)) > 0),
    CONSTRAINT signal_connections_settings_is_object CHECK (jsonb_typeof(settings) = 'object')
);

CREATE INDEX IF NOT EXISTS signal_connections_source_status_idx
    ON signal_connections (source_code, status);

CREATE TABLE IF NOT EXISTS signal_capabilities (
    id UUID PRIMARY KEY,
    source_code TEXT NOT NULL REFERENCES signal_sources(code),
    connection_id UUID REFERENCES signal_connections(id),
    capability TEXT NOT NULL,
    state TEXT NOT NULL,
    reason TEXT,
    requires_confirmation BOOLEAN NOT NULL DEFAULT FALSE,
    action_class TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT signal_capabilities_capability_not_empty CHECK (length(trim(capability)) > 0),
    CONSTRAINT signal_capabilities_state_not_empty CHECK (length(trim(state)) > 0),
    CONSTRAINT signal_capabilities_action_class_not_empty CHECK (length(trim(action_class)) > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS signal_capabilities_identity_idx
    ON signal_capabilities (source_code, COALESCE(connection_id, '00000000-0000-0000-0000-000000000000'::uuid), capability);

CREATE TABLE IF NOT EXISTS signal_runtime_states (
    id UUID PRIMARY KEY,
    source_code TEXT NOT NULL REFERENCES signal_sources(code),
    connection_id UUID REFERENCES signal_connections(id),
    runtime_kind TEXT NOT NULL,
    state TEXT NOT NULL,
    last_started_at TIMESTAMPTZ,
    last_stopped_at TIMESTAMPTZ,
    last_heartbeat_at TIMESTAMPTZ,
    last_error_at TIMESTAMPTZ,
    last_error_code TEXT,
    last_error_message_redacted TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT signal_runtime_states_runtime_kind_not_empty CHECK (length(trim(runtime_kind)) > 0),
    CONSTRAINT signal_runtime_states_state_not_empty CHECK (length(trim(state)) > 0),
    CONSTRAINT signal_runtime_states_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS signal_runtime_states_source_state_idx
    ON signal_runtime_states (source_code, state);

CREATE TABLE IF NOT EXISTS signal_health (
    id UUID PRIMARY KEY,
    source_code TEXT NOT NULL REFERENCES signal_sources(code),
    connection_id UUID REFERENCES signal_connections(id),
    level TEXT NOT NULL,
    summary TEXT NOT NULL,
    last_ok_at TIMESTAMPTZ,
    last_failure_at TIMESTAMPTZ,
    failure_count INTEGER NOT NULL DEFAULT 0,
    consecutive_failure_count INTEGER NOT NULL DEFAULT 0,
    next_retry_at TIMESTAMPTZ,
    evidence JSONB NOT NULL DEFAULT '{}'::jsonb,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT signal_health_level_not_empty CHECK (length(trim(level)) > 0),
    CONSTRAINT signal_health_summary_not_empty CHECK (length(trim(summary)) > 0),
    CONSTRAINT signal_health_failure_count_non_negative CHECK (failure_count >= 0),
    CONSTRAINT signal_health_consecutive_failure_count_non_negative CHECK (consecutive_failure_count >= 0),
    CONSTRAINT signal_health_evidence_is_object CHECK (jsonb_typeof(evidence) = 'object')
);

CREATE INDEX IF NOT EXISTS signal_health_source_level_idx
    ON signal_health (source_code, level);

CREATE TABLE IF NOT EXISTS signal_policies (
    id UUID PRIMARY KEY,
    scope TEXT NOT NULL,
    source_code TEXT REFERENCES signal_sources(code),
    connection_id UUID REFERENCES signal_connections(id),
    event_pattern TEXT,
    mode TEXT NOT NULL,
    reason TEXT NOT NULL,
    created_by TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,

    CONSTRAINT signal_policies_scope_not_empty CHECK (length(trim(scope)) > 0),
    CONSTRAINT signal_policies_mode_not_empty CHECK (length(trim(mode)) > 0),
    CONSTRAINT signal_policies_reason_not_empty CHECK (length(trim(reason)) > 0),
    CONSTRAINT signal_policies_created_by_not_empty CHECK (length(trim(created_by)) > 0),
    CONSTRAINT signal_policies_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS signal_policies_active_idx
    ON signal_policies (scope, source_code, connection_id, event_pattern, mode, expires_at);

CREATE TABLE IF NOT EXISTS signal_profiles (
    id UUID PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT NOT NULL,
    source_policies JSONB NOT NULL DEFAULT '[]'::jsonb,
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT signal_profiles_code_not_empty CHECK (length(trim(code)) > 0),
    CONSTRAINT signal_profiles_display_name_not_empty CHECK (length(trim(display_name)) > 0),
    CONSTRAINT signal_profiles_description_not_empty CHECK (length(trim(description)) > 0),
    CONSTRAINT signal_profiles_source_policies_is_array CHECK (jsonb_typeof(source_policies) = 'array')
);

CREATE INDEX IF NOT EXISTS signal_profiles_system_idx
    ON signal_profiles (is_system, code);

CREATE TABLE IF NOT EXISTS signal_paused_events (
    id UUID PRIMARY KEY,
    event_id TEXT NOT NULL UNIQUE,
    source_code TEXT NOT NULL REFERENCES signal_sources(code),
    connection_id UUID REFERENCES signal_connections(id),
    raw_event_type TEXT NOT NULL,
    event_envelope JSONB NOT NULL,
    reason TEXT NOT NULL,
    paused_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    released_at TIMESTAMPTZ,

    CONSTRAINT signal_paused_events_raw_event_type_not_empty CHECK (length(trim(raw_event_type)) > 0),
    CONSTRAINT signal_paused_events_event_envelope_is_object CHECK (jsonb_typeof(event_envelope) = 'object'),
    CONSTRAINT signal_paused_events_reason_not_empty CHECK (length(trim(reason)) > 0)
);

CREATE INDEX IF NOT EXISTS signal_paused_events_source_paused_idx
    ON signal_paused_events (source_code, paused_at)
    WHERE released_at IS NULL;

CREATE TABLE IF NOT EXISTS signal_replay_requests (
    id UUID PRIMARY KEY,
    source_code TEXT REFERENCES signal_sources(code),
    connection_id UUID REFERENCES signal_connections(id),
    event_pattern TEXT,
    status TEXT NOT NULL,
    requested_by TEXT NOT NULL,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    last_error_redacted TEXT,
    replayed_count INTEGER NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,

    CONSTRAINT signal_replay_requests_status_not_empty CHECK (length(trim(status)) > 0),
    CONSTRAINT signal_replay_requests_requested_by_not_empty CHECK (length(trim(requested_by)) > 0),
    CONSTRAINT signal_replay_requests_replayed_count_non_negative CHECK (replayed_count >= 0),
    CONSTRAINT signal_replay_requests_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS signal_replay_requests_status_idx
    ON signal_replay_requests (status, requested_at);
