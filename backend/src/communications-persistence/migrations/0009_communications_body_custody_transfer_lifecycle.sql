ALTER TABLE hermes_data.communications_body_custody_transfers
  DROP CONSTRAINT communications_body_custody_transfers_state_check;

ALTER TABLE hermes_data.communications_body_custody_transfers
  ADD COLUMN claimed_by TEXT,
  ADD COLUMN lease_expires_at_unix_seconds BIGINT,
  ADD COLUMN completed_at_unix_seconds BIGINT,
  ADD CONSTRAINT communications_body_custody_transfers_state_check
    CHECK (state IN (1, 2, 3));

CREATE INDEX communications_body_custody_transfers_pending_idx
  ON hermes_data.communications_body_custody_transfers (
    state, lease_expires_at_unix_seconds, evidence_id
  );
