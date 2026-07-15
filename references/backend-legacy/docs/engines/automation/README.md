# Automation Engine

Status: code-aligned documentation package created from current backend
modules.

The Automation Engine is a reusable policy and dry-run mechanism for owner
approved automation. It is not a provider adapter and not a task or
communication domain.

## Current Implementation Evidence

Current backend files:

- `backend/src/engines/automation.rs`;
- `backend/src/engines/automation/models.rs`;
- `backend/src/engines/automation/policy.rs`;
- `backend/src/engines/automation/store.rs`;
- `backend/src/engines/automation/validation.rs`;
- `backend/src/engines/automation/dry_run.rs`.

The current implementation exports:

- `AutomationTemplate`;
- `AutomationPolicy`;
- `TelegramSendDryRunRequest`;
- `TelegramSendDryRunResponse`;
- `AutomationStore`.

The implemented policy baseline validates declared template variables, enabled
policy state, expiry, allowed chat ids, JSON object payloads and
`max_sends_per_hour`. The current owner-facing dry-run surface is Telegram send
automation.

## Boundary Rule

Automation can evaluate policy and produce dry-run or command metadata. It must
not bypass owner-visible provider-write capability, provider runtime policy or
the target integration command boundary.

