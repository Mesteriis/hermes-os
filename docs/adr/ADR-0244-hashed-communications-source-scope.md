# ADR-0244: Hashed source scope for canonical Communications projections

Status: Accepted
Date: 2026-07-22
Implementation state: In progress.

## Decision

Communications ingress may carry only deterministic SHA-256 cursors for a
provider account and provider conversation. Integrations derive the hashes from
their local provider identifiers; raw identifiers, content, credentials,
session data and provider DTOs never cross the owner boundary.

The source record cursor remains distinct from account and conversation cursors.
The account cursor is provider-scoped. The conversation cursor is derived from
the account cursor and provider-local conversation identity, preventing
cross-account collisions. Missing scope remains representable for account-level
observations, but canonical message and conversation projections require the
relevant typed cursor.

## Consequences

Communications can own canonical message and conversation projections without
provider-specific persistence. Mail, Telegram and WhatsApp integrations retain
all operational source IDs locally. No compatibility mapping, raw locator
fallback or cross-owner query is introduced.
