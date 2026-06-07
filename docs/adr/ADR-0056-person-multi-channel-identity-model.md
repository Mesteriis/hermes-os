# ADR-0056 Person Multi-Channel Identity Model

Status: Proposed

## Context

ADR-0019 established identity resolution as confidence-scored merge/split candidates. The current contact model derives identity from a single email address. The functional spec for Hermes Persons requires multi-channel identities (email, Telegram, WhatsApp, phone, GitHub, LinkedIn, and others) linked to a single person entity.

## Decision

Model persons as UUID-identified entities with a separate `person_identities` table holding all channel-specific identifiers. Each identity carries `source`, `confidence`, and `verification_status`. The `email_address` unique constraint moves from `persons` to `person_identities`. Auto-creation from incoming messages creates or matches identities, not raw person rows.

## Consequences

- One person can have many identities across channels.
- Identity resolution (merge/split) operates on person UUIDs, not email-derived IDs.
- Backfill migrates existing `contact:v1:email:*` IDs to `person:v1:*` format.
- The `person_identity_candidates` table is renamed from `contact_identity_candidates`.
