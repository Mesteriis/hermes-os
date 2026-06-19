ALTER TABLE calendar_reminders
    ADD COLUMN IF NOT EXISTS source TEXT NOT NULL DEFAULT 'manual';

UPDATE calendar_reminders
SET source = 'manual'
WHERE source IS NULL OR btrim(source) = '';
