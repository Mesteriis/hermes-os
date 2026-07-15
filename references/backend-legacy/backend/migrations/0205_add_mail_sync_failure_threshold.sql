ALTER TABLE communication_account_sync_settings
    ADD COLUMN IF NOT EXISTS failure_threshold INTEGER NOT NULL DEFAULT 3;

ALTER TABLE communication_account_sync_settings
    DROP CONSTRAINT IF EXISTS communication_account_sync_settings_failure_threshold_check;

ALTER TABLE communication_account_sync_settings
    ADD CONSTRAINT communication_account_sync_settings_failure_threshold_check
    CHECK (failure_threshold BETWEEN 1 AND 10);
