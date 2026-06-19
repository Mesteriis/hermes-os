ALTER TABLE calendar_events
    ADD COLUMN IF NOT EXISTS observation_id TEXT;

INSERT INTO observations (
    observation_id,
    kind_definition_id,
    origin_kind,
    vault_source_id,
    observed_at,
    captured_at,
    payload,
    confidence,
    content_hash,
    source_ref,
    provenance
)
SELECT
    'observation:v1:legacy-calendar-event:' || md5(
        COALESCE(calendar_events.event_id, '') || '|' ||
        calendar_events.start_at::text || '|' ||
        calendar_events.end_at::text || '|' ||
        COALESCE(calendar_events.title, '') || '|' ||
        COALESCE(calendar_events.description, '') || '|' ||
        COALESCE(calendar_events.location, '')
    ),
    kind.kind_definition_id,
    'local_runtime',
    NULL,
    calendar_events.start_at,
    calendar_events.created_at,
    jsonb_build_object(
        'legacy_event_id', calendar_events.event_id,
        'source_event_id', calendar_events.source_event_id,
        'account_id', calendar_events.account_id,
        'source_id', calendar_events.source_id,
        'title', calendar_events.title,
        'description', calendar_events.description,
        'location', calendar_events.location,
        'start_at', calendar_events.start_at,
        'end_at', calendar_events.end_at,
        'timezone', calendar_events.timezone,
        'all_day', calendar_events.all_day,
        'recurrence_rule', calendar_events.recurrence_rule,
        'status', calendar_events.status,
        'visibility', calendar_events.visibility,
        'event_type', calendar_events.event_type,
        'conference_url', calendar_events.conference_url,
        'conference_provider', calendar_events.conference_provider,
        'preparation_reminder_minutes', calendar_events.preparation_reminder_minutes,
        'travel_buffer_minutes', calendar_events.travel_buffer_minutes,
        'legacy_backfill', true
    ),
    1.0,
    'sha256:' || md5(
        COALESCE(calendar_events.event_id, '') || '|' ||
        calendar_events.start_at::text || '|' ||
        calendar_events.end_at::text || '|' ||
        COALESCE(calendar_events.title, '') || '|' ||
        COALESCE(calendar_events.description, '') || '|' ||
        COALESCE(calendar_events.location, '')
    ),
    'calendar_event://' || calendar_events.event_id,
    jsonb_build_object(
        'legacy_backfill', true,
        'ingested_by', 'calendar_events_domain'
    )
FROM calendar_events
LEFT JOIN observation_kind_definitions kind
  ON kind.code = 'CALENDAR_EVENT'
 AND kind.version = 1
WHERE calendar_events.observation_id IS NULL
  AND kind.kind_definition_id IS NOT NULL
ON CONFLICT (observation_id) DO NOTHING;

UPDATE calendar_events
SET observation_id = 'observation:v1:legacy-calendar-event:' || md5(
    COALESCE(calendar_events.event_id, '') || '|' || calendar_events.start_at::text || '|' ||
    calendar_events.end_at::text || '|' || COALESCE(calendar_events.title, '') || '|' ||
    COALESCE(calendar_events.description, '') || '|' || COALESCE(calendar_events.location, '')
)
WHERE calendar_events.observation_id IS NULL
  AND EXISTS (
        SELECT 1
        FROM observations observation
        WHERE observation.observation_id = 'observation:v1:legacy-calendar-event:' || md5(
            COALESCE(calendar_events.event_id, '') || '|' || calendar_events.start_at::text || '|' ||
            calendar_events.end_at::text || '|' || COALESCE(calendar_events.title, '') || '|' ||
            COALESCE(calendar_events.description, '') || '|' || COALESCE(calendar_events.location, '')
        )
          AND observation.source_ref = 'calendar_event://' || calendar_events.event_id
    );

ALTER TABLE calendar_events
    ALTER COLUMN observation_id SET NOT NULL;

ALTER TABLE calendar_events
    DROP CONSTRAINT IF EXISTS calendar_events_observation_fk;

ALTER TABLE calendar_events
    ADD CONSTRAINT calendar_events_observation_fk
    FOREIGN KEY (observation_id)
    REFERENCES observations(observation_id);

CREATE INDEX IF NOT EXISTS calendar_events_observation_idx
    ON calendar_events (observation_id);
