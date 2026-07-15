-- Phase 0: Calendar core tables

CREATE TABLE IF NOT EXISTS calendar_accounts (
    account_id TEXT PRIMARY KEY,
    provider TEXT NOT NULL,
    account_name TEXT NOT NULL,
    email TEXT,
    credentials_reference TEXT,
    sync_status TEXT NOT NULL DEFAULT 'idle',
    capabilities JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT calendar_accounts_provider_check CHECK (provider IN ('google', 'microsoft', 'exchange', 'apple', 'caldav', 'ics', 'local')),
    CONSTRAINT calendar_accounts_sync_status_check CHECK (sync_status IN ('idle', 'syncing', 'synced', 'error', 'disabled')),
    CONSTRAINT calendar_accounts_caps_is_object CHECK (jsonb_typeof(capabilities) = 'object')
);

CREATE INDEX IF NOT EXISTS calendar_accounts_provider_idx ON calendar_accounts (provider);

CREATE TABLE IF NOT EXISTS calendar_sources (
    source_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES calendar_accounts(account_id) ON DELETE CASCADE,
    provider_calendar_id TEXT,
    name TEXT NOT NULL,
    color TEXT,
    timezone TEXT,
    visibility TEXT NOT NULL DEFAULT 'private',
    read_only BOOLEAN NOT NULL DEFAULT false,
    sync_enabled BOOLEAN NOT NULL DEFAULT true,
    capabilities JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT calendar_sources_visibility_check CHECK (visibility IN ('private', 'public', 'confidential')),
    CONSTRAINT calendar_sources_caps_is_object CHECK (jsonb_typeof(capabilities) = 'object')
);

CREATE INDEX IF NOT EXISTS calendar_sources_account_idx ON calendar_sources (account_id);

CREATE TABLE IF NOT EXISTS calendar_events (
    event_id TEXT PRIMARY KEY,
    source_event_id TEXT,
    account_id TEXT REFERENCES calendar_accounts(account_id) ON DELETE SET NULL,
    source_id TEXT REFERENCES calendar_sources(source_id) ON DELETE SET NULL,
    title TEXT NOT NULL,
    description TEXT,
    location TEXT,
    start_at TIMESTAMPTZ NOT NULL,
    end_at TIMESTAMPTZ NOT NULL,
    timezone TEXT,
    all_day BOOLEAN NOT NULL DEFAULT false,
    recurrence_rule TEXT,
    status TEXT NOT NULL DEFAULT 'scheduled',
    visibility TEXT NOT NULL DEFAULT 'private',
    event_type TEXT,
    importance_score REAL,
    readiness_score REAL,
    sync_status TEXT NOT NULL DEFAULT 'local',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT calendar_events_title_not_empty CHECK (length(trim(title)) > 0),
    CONSTRAINT calendar_events_status_check CHECK (status IN ('scheduled', 'prepared', 'in_progress', 'completed', 'cancelled', 'rescheduled', 'no_show', 'needs_follow_up', 'archived')),
    CONSTRAINT calendar_events_visibility_check CHECK (visibility IN ('private', 'public', 'confidential', 'hidden_details', 'local_only')),
    CONSTRAINT calendar_events_sync_status_check CHECK (sync_status IN ('local', 'syncing', 'synced', 'conflict', 'error')),
    CONSTRAINT calendar_events_importance_range CHECK (importance_score IS NULL OR (importance_score >= 0 AND importance_score <= 1)),
    CONSTRAINT calendar_events_readiness_range CHECK (readiness_score IS NULL OR (readiness_score >= 0 AND readiness_score <= 1))
);

CREATE INDEX IF NOT EXISTS calendar_events_account_idx ON calendar_events (account_id);
CREATE INDEX IF NOT EXISTS calendar_events_source_idx ON calendar_events (source_id);
CREATE INDEX IF NOT EXISTS calendar_events_start_at_idx ON calendar_events (start_at);
CREATE INDEX IF NOT EXISTS calendar_events_end_at_idx ON calendar_events (end_at);
CREATE INDEX IF NOT EXISTS calendar_events_status_idx ON calendar_events (status);
CREATE INDEX IF NOT EXISTS calendar_events_type_idx ON calendar_events (event_type);
CREATE INDEX IF NOT EXISTS calendar_events_time_range_idx ON calendar_events (start_at, end_at);

