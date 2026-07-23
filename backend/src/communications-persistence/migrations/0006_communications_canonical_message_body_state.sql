ALTER TABLE hermes_data.communications_messages
  ADD COLUMN canonical_body_state SMALLINT NOT NULL DEFAULT 1
  CHECK (canonical_body_state IN (1, 2, 3, 4));

UPDATE hermes_data.communications_messages
  SET canonical_body_state = body_state
  WHERE canonical_body_state <> body_state;
