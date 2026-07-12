# ADR-0178: Provider-Neutral Automation Policy Scopes

Status: Accepted
Date: 2026-07-12

Clarifies ADR-0176 and supersedes the Telegram-only authorization shape in
the automation policy engine.

## Context

`automation_policies.allowed_chat_ids` coupled the reusable automation engine
to a Telegram recipient representation. That cannot safely describe a mail
account, fixed recipient, target domain or Review promotion boundary. It also
encourages future providers to grow provider-specific columns on the same
policy table.

## Decision

Policies authorize actions through durable `(scope_kind, scope_value)` rows in
`automation_policy_scopes`. Scope checks are default-deny: an execution is
allowed only when it has the matching persisted scope.

- Existing `allowed_chat_ids` stays as a deprecated compatibility projection.
- Migration `0221` backfills each value as `telegram.chat`.
- Policy upserts atomically replace generic scopes and derive `telegram.chat`
  scopes from legacy input.
- Telegram dry-run now evaluates `telegram.chat` scopes rather than the legacy
  JSON column.
- New Mail and cross-domain automation must use explicit scopes such as
  `mail.account`, `mail.recipient` or `domain.target`; they must not add a
  provider-specific allow-list column.

This decision does not grant an automation capability by itself. Automation
remains disabled by default, and cross-domain promotion still requires Review
unless an explicit scoped policy authorizes the automation actor.

## Consequences

The engine can evolve provider-neutral policy enforcement without treating a
provider's identifiers as canonical product state. Existing Telegram clients
remain compatible during migration, but all new execution paths must query the
scope table.
