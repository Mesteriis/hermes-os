-- Phase e1: Conference links, reminders, event metadata extensions

ALTER TABLE calendar_events ADD COLUMN IF NOT EXISTS conference_url TEXT;
ALTER TABLE calendar_events ADD COLUMN IF NOT EXISTS conference_provider TEXT;
ALTER TABLE calendar_events ADD COLUMN IF NOT EXISTS preparation_reminder_minutes INTEGER;
ALTER TABLE calendar_events ADD COLUMN IF NOT EXISTS travel_buffer_minutes INTEGER;

-- Smart reminders
CREATE TABLE IF NOT EXISTS calendar_reminders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id TEXT NOT NULL REFERENCES calendar_events(event_id) ON DELETE CASCADE,
    reminder_type TEXT NOT NULL DEFAULT 'time_based',
    minutes_before INTEGER,
    condition_json JSONB,
    message TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_triggered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT calendar_reminders_type_check CHECK (reminder_type IN ('time_based', 'context_based', 'preparation_based', 'location_based', 'deadline_based', 'document_based'))
);

CREATE INDEX IF NOT EXISTS calendar_reminders_event_idx ON calendar_reminders (event_id);
CREATE INDEX IF NOT EXISTS calendar_reminders_active_idx ON calendar_reminders (event_id) WHERE is_active = true;

-- Event location history for location intelligence
CREATE TABLE IF NOT EXISTS event_locations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id TEXT NOT NULL REFERENCES calendar_events(event_id) ON DELETE CASCADE,
    raw_location TEXT NOT NULL,
    parsed_name TEXT,
    parsed_address TEXT,
    is_online BOOLEAN DEFAULT false,
    latitude DOUBLE PRECISION,
    longitude DOUBLE PRECISION,
    frequency_count INTEGER DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS event_locations_event_idx ON event_locations (event_id);
CREATE INDEX IF NOT EXISTS event_locations_name_idx ON event_locations (parsed_name);
