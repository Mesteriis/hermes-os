ALTER TABLE communication_raw_records
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
    'observation:v1:communication-raw-record:' || raw.raw_record_id,
    kind.kind_definition_id,
    'vault_source',
    NULL,
    COALESCE(raw.occurred_at, raw.captured_at),
    raw.captured_at,
    raw.payload,
    1.0,
    raw.source_fingerprint,
    'communication://' || raw.account_id || '/' || raw.record_kind || '/' || raw.provider_record_id,
    raw.provenance || jsonb_build_object(
        'legacy_backfill', true,
        'raw_record_id', raw.raw_record_id,
        'account_id', raw.account_id,
        'record_kind', raw.record_kind,
        'provider_record_id', raw.provider_record_id,
        'import_batch_id', raw.import_batch_id
    )
FROM communication_raw_records raw
JOIN observation_kind_definitions kind
  ON kind.code = CASE
      WHEN raw.record_kind LIKE '%attachment%' THEN 'COMMUNICATION_ATTACHMENT'
      ELSE 'COMMUNICATION_MESSAGE'
  END
 AND kind.version = 1
WHERE raw.observation_id IS NULL
ON CONFLICT (observation_id) DO NOTHING;

UPDATE communication_raw_records raw
SET observation_id = 'observation:v1:communication-raw-record:' || raw.raw_record_id
WHERE raw.observation_id IS NULL;

ALTER TABLE communication_raw_records
    ALTER COLUMN observation_id SET NOT NULL;

ALTER TABLE communication_raw_records
    DROP CONSTRAINT IF EXISTS communication_raw_records_observation_fk;

ALTER TABLE communication_raw_records
    ADD CONSTRAINT communication_raw_records_observation_fk
    FOREIGN KEY (observation_id)
    REFERENCES observations(observation_id);

CREATE INDEX IF NOT EXISTS communication_raw_records_observation_idx
    ON communication_raw_records (observation_id);
