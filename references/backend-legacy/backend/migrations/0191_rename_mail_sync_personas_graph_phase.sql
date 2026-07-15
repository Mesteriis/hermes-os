-- Align mail sync progress phase with Persona domain naming.

ALTER TABLE communication_mail_sync_runs
    DROP CONSTRAINT IF EXISTS communication_mail_sync_runs_phase_check;

UPDATE communication_mail_sync_runs
SET phase = 'personas_graph'
WHERE phase = 'persons_graph';

ALTER TABLE communication_mail_sync_runs
    ADD CONSTRAINT communication_mail_sync_runs_phase_check
    CHECK (
        phase IN (
            'idle',
            'waiting_for_vault',
            'listing',
            'fetching',
            'projecting',
            'personas_graph',
            'completed',
            'failed',
            'skipped'
        )
    );

UPDATE communication_sync_runs
SET phase = 'personas_graph'
WHERE phase = 'persons_graph';
