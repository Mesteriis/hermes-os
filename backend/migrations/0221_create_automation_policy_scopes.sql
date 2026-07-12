-- Provider-neutral allow scopes replace Telegram-only policy authorization.
-- The legacy JSON field remains as a compatibility projection for older clients.
CREATE TABLE IF NOT EXISTS automation_policy_scopes (
    policy_id TEXT NOT NULL REFERENCES automation_policies(policy_id) ON DELETE CASCADE,
    scope_kind TEXT NOT NULL,
    scope_value TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (policy_id, scope_kind, scope_value),
    CONSTRAINT automation_policy_scopes_kind_not_empty CHECK (length(trim(scope_kind)) > 0),
    CONSTRAINT automation_policy_scopes_value_not_empty CHECK (length(trim(scope_value)) > 0)
);

CREATE INDEX IF NOT EXISTS automation_policy_scopes_lookup_idx
    ON automation_policy_scopes (scope_kind, scope_value, policy_id);

INSERT INTO automation_policy_scopes (policy_id, scope_kind, scope_value)
SELECT
    policy.policy_id,
    'telegram.chat',
    trim(chat_id.value)
FROM automation_policies AS policy
CROSS JOIN LATERAL jsonb_array_elements_text(policy.allowed_chat_ids) AS chat_id(value)
WHERE length(trim(chat_id.value)) > 0
ON CONFLICT (policy_id, scope_kind, scope_value) DO NOTHING;
