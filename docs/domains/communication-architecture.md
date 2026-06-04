# Communication Architecture

## Purpose

Communications unify email, Telegram, WhatsApp and optional SMS into a single event-backed memory model. Provider-specific behavior is preserved, but user workflows operate over canonical messages, threads, contacts and events.

## Channel Adapters

Adapters are responsible for:

- authentication through the secret boundary
- provider sync
- raw source preservation
- idempotency
- pagination and checkpoints
- attachment ingestion
- delivery/send capabilities where supported
- rate limit handling

## Canonical Objects

- ChannelAccount
- Conversation
- Message
- Participant
- Attachment
- DeliveryState
- ThreadLink
- CommunicationEvent

## Spam Intelligence

The classification model includes:

- SPF
- DKIM
- DMARC
- AI spam detection
- AI marketing detection
- sender reputation
- personal relevance scoring

Target categories:

- Critical
- Important
- Personal
- Work
- Information
- Marketing
- Spam

## Outbound Messages

Drafting and sending are separate. AI can draft a response, but sending requires explicit user confirmation unless the user later defines a narrowly scoped automation policy.

## Threading

Threading must support provider-native threads and cross-provider conversation grouping. Cross-provider grouping is graph-backed and confidence-scored.
