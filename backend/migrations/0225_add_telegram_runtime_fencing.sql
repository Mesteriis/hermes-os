ALTER TABLE telegram_provider_write_commands
    ADD COLUMN IF NOT EXISTS lease_epoch BIGINT NOT NULL DEFAULT 0;

ALTER TABLE telegram_provider_write_commands
    ADD CONSTRAINT telegram_provider_write_commands_lease_epoch_nonnegative
        CHECK (lease_epoch >= 0);

CREATE INDEX IF NOT EXISTS telegram_provider_write_commands_lease_epoch_idx
    ON telegram_provider_write_commands (account_id, lease_epoch, status);
