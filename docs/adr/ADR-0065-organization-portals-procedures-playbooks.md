# ADR-0065 Organization Portals, Procedures, and Playbooks

Status: Proposed

## Context

Organizations often require interaction through specific portals (tax, banking, support), follow defined procedures (tax filing, contract signing), and can benefit from automated playbooks (email received → check deadline → create task). These are organization-specific knowledge that should be stored and surfaced.

## Decision

Portals (`organization_portals`) store URLs, portal types, login hints, and secret references (not the secrets themselves). Procedures (`organization_procedures`) store named workflows as JSONB step arrays. Playbooks (`organization_playbooks`) store automated scenarios with triggers, steps, and approval modes. Templates (`organization_templates`) store email/document templates specific to an organization. In v1, playbooks are stored as data but not automatically executed — execution is a future concern.

## Consequences

- Portals provide quick access links to external systems with login context.
- Procedures turn ad-hoc processes into repeatable, documented workflows.
- Playbooks define what should happen but require explicit user action or future automation runtime.
- Quick actions (`organization_quick_actions`) provide one-click shortcuts for common tasks.
