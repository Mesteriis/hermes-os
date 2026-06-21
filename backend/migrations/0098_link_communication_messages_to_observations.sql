ALTER TABLE communication_messages
    ADD COLUMN IF NOT EXISTS observation_id TEXT;

UPDATE communication_messages message
SET observation_id = raw.observation_id
FROM communication_raw_records raw
WHERE message.raw_record_id = raw.raw_record_id
  AND message.observation_id IS NULL;

ALTER TABLE communication_messages
    ALTER COLUMN observation_id SET NOT NULL;

ALTER TABLE communication_messages
    DROP CONSTRAINT IF EXISTS communication_messages_observation_fk;

ALTER TABLE communication_messages
    ADD CONSTRAINT communication_messages_observation_fk
    FOREIGN KEY (observation_id)
    REFERENCES observations(observation_id);

CREATE INDEX IF NOT EXISTS communication_messages_observation_idx
    ON communication_messages (observation_id);
