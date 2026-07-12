-- Rich extraction can outlive the request that started it. Retaining an
-- executing state makes an interrupted local worker run observable and safely
-- retryable against the same content-addressed source blob.

ALTER TABLE communication_attachment_extractions
    DROP CONSTRAINT IF EXISTS communication_attachment_extractions_status;

ALTER TABLE communication_attachment_extractions
    ADD CONSTRAINT communication_attachment_extractions_status
    CHECK (status IN ('executing', 'completed', 'unsupported', 'failed'));
