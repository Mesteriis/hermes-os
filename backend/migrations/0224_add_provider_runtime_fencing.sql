-- Provider runtime fencing for in-process and connector topologies.
-- Epochs are monotonic per provider/account; core rejects stale completions.
CREATE TABLE IF NOT EXISTS provider_runtime_leases (
    provider TEXT NOT NULL,
    account_id TEXT NOT NULL REFERENCES communication_accounts(account_id) ON DELETE CASCADE,
    topology TEXT NOT NULL,
    holder TEXT NOT NULL,
    epoch BIGINT NOT NULL DEFAULT 0,
    state TEXT NOT NULL DEFAULT 'active',
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (provider, account_id),
    CONSTRAINT provider_runtime_leases_epoch_nonnegative CHECK (epoch >= 0),
    CONSTRAINT provider_runtime_leases_topology_nonempty CHECK (length(trim(topology)) > 0),
    CONSTRAINT provider_runtime_leases_holder_nonempty CHECK (length(trim(holder)) > 0)
);

CREATE INDEX IF NOT EXISTS provider_runtime_leases_expiry_idx
    ON provider_runtime_leases (expires_at, state);

ALTER TABLE communication_provider_commands
    ADD COLUMN IF NOT EXISTS lease_epoch BIGINT NOT NULL DEFAULT 0;

ALTER TABLE communication_provider_commands
    ADD CONSTRAINT communication_provider_commands_lease_epoch_nonnegative
        CHECK (lease_epoch >= 0);

CREATE INDEX IF NOT EXISTS communication_provider_commands_lease_epoch_idx
    ON communication_provider_commands (account_id, channel_kind, lease_epoch, status);
