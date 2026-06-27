# ADR-0074 Person Multi-Channel Identity Model

Status: Accepted

## Context

ADR-0019 established identity resolution as confidence-scored merge/split candidates. The current contact projection was originally derived from a single email address and is already referenced by graph projections, project links and task context. The functional spec for Hermes Persons requires multi-channel identities (email, Telegram, WhatsApp, phone, GitHub, LinkedIn, and others) linked to a single person entity.

## Decision

Keep the current person primary key as a stable text identifier: `person:v1:email:{len}:{normalized_email}` for email-created persons. The `persons.email_address` unique constraint remains in place as the primary-email compatibility contract for existing projections.

Add a separate `person_identities` table for all channel-specific identifiers. Each identity carries `source`, `confidence`, `status`, verification metadata, and a unique active `(identity_type, identity_value)` identity constraint. Auto-creation from incoming email creates or matches the primary email person row and backfills the matching `person_identities` record.

Opaque UUID person IDs are not part of this implementation slice. Moving from text person IDs to opaque IDs would require a separate ADR and migration plan covering graph nodes, tasks, projects, communication projections, API payloads and frontend state.

## Consequences

- One person can have many identities across channels.
- Identity resolution (merge/split) operates on current text `person_id` values.
- Backfill migrates existing `contact:v1:email:*` IDs to `person:v1:*` format and creates email identity rows.
- The `person_identity_candidates` table is renamed from `contact_identity_candidates`.
- A future opaque-ID migration remains possible, but it must be explicit and cannot be inferred from this ADR.
