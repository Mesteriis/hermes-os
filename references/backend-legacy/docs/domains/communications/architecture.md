# Communications Architecture

Canonical domain definition: [Communications Domain](README.md).

This document describes architecture concerns for the Communications domain.

## Purpose

Communications unify evidence from email, Telegram, WhatsApp, calls and meetings
at the provider-neutral context boundary. Provider-specific operational
behavior remains in bundled integration plugins and their own frontend
experiences.

Provider operational screens are first-class product experiences, but they are
not business domains and do not own shared memory or promoted business truth.

## Integration Plugins

Integration plugins are responsible for:

- authentication through the secret boundary;
- provider sync;
- raw source preservation;
- idempotency;
- pagination and checkpoints;
- attachment ingestion;
- delivery/send capabilities where supported;
- rate limit handling.

They also own provider-specific operational contracts/projections and map
observations into the provider-neutral evidence contract defined by ADR-0204.
Communications never reads their tables or imports their generated types.

## Provider Accounts

Initial email ingestion supports:

- `gmail` - Gmail API with OAuth and Gmail history checkpoints.
- `icloud` - iCloud Mail over IMAP with app-specific credentials and mailbox UID checkpoints.
- `imap` - generic raw IMAP with host/port/TLS metadata and mailbox UID checkpoints.

Telegram, WhatsApp and future providers follow the same principle: provider
accounts store non-secret metadata and adapter configuration only. Credentials
belong behind the secret boundary and are linked through secret references.

Telegram production capability behavior is specified in
[Telegram Channel Capability Spec](../../integrations/telegram/README.md). It
remains a Communications channel and does not own durable truth outside
Communications, Personas, Relationships, Documents, Decisions, Obligations,
Tasks and Events.

## Neutral Evidence Concepts

- Communication evidence.
- Observed participant.
- Source/provenance reference.
- Attachment/blob reference.
- Evidence link.
- Communication evidence event.

Channel accounts, provider conversations/messages, topics, folders, delivery
state and drafts are operational integration state, not Communications-owned
business truth.

## Engine Use

Communications use engines rather than owning separate intelligence systems:

- Search Engine for retrieval.
- Memory Engine for communication memory.
- Timeline Engine for interaction history.
- Obligation Engine for commitments and follow-ups.
- Risk Engine for spam, phishing and attention signals.
- Enrichment Engine for candidate links and entity extraction.

## Outbound Messages

Drafting and sending are separate. AI can draft a response, but sending requires
explicit owner confirmation unless the owner later defines a narrowly scoped
automation policy.

## Threading

Threading must support provider-native threads and cross-provider conversation
grouping. Cross-provider grouping is graph-backed and confidence-scored.
