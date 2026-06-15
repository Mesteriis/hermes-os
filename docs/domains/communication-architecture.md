# Communications Architecture

Canonical domain definition: [Communications Domain](communications.md).

This document describes architecture concerns for the Communications domain.

## Purpose

Communications unify email, Telegram, WhatsApp, calls and meetings as one
canonical interaction model. Provider-specific behavior is preserved at adapter
and source-record boundaries, but user workflows operate over Communications,
Participants, Conversations, Events, Personas and Context.

Hermes is not an email client or messenger. Communication surfaces are entry
points into the Personal Memory System.

## Channel Adapters

Adapters are responsible for:

- authentication through the secret boundary;
- provider sync;
- raw source preservation;
- idempotency;
- pagination and checkpoints;
- attachment ingestion;
- delivery/send capabilities where supported;
- rate limit handling.

## Provider Accounts

Initial email ingestion supports:

- `gmail` - Gmail API with OAuth and Gmail history checkpoints.
- `icloud` - iCloud Mail over IMAP with app-specific credentials and mailbox UID checkpoints.
- `imap` - generic raw IMAP with host/port/TLS metadata and mailbox UID checkpoints.

Telegram, WhatsApp and future providers follow the same principle: provider
accounts store non-secret metadata and adapter configuration only. Credentials
belong behind the secret boundary and are linked through secret references.

Telegram production capability behavior is specified in
[Telegram Domain Audit](../telegram/README.md). It remains a
Communications channel and does not own durable truth outside Communications,
Personas, Relationships, Documents, Decisions, Obligations, Tasks and Events.

## Canonical Objects

- ChannelAccount.
- Conversation.
- Communication.
- Message.
- Participant.
- Attachment.
- DeliveryState.
- ThreadLink.
- CommunicationEvent.

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
