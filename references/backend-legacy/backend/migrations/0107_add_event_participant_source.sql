ALTER TABLE event_participants
    ADD COLUMN IF NOT EXISTS source TEXT NOT NULL DEFAULT 'manual';

UPDATE event_participants
SET source = 'manual'
WHERE source IS NULL OR btrim(source) = '';
