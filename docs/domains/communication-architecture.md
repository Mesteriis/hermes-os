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

## Initial Email Providers

Initial email ingestion must support three provider kinds:

- `gmail` - Gmail API with OAuth and Gmail history checkpoints.
- `icloud` - iCloud Mail over IMAP with app-specific credentials and mailbox UID checkpoints.
- `imap` - generic raw IMAP with host/port/TLS metadata and mailbox UID checkpoints.

Provider account records store non-secret metadata and adapter configuration only. Credentials and tokens belong behind the secret boundary and are linked through secret references.

The account model supports multiple records for the same provider kind, for example several Gmail or iCloud accounts. Adapter credential lookup is account-scoped through `ProviderCredentialReader`: resolve the provider account by `account_id`, load the binding for the required secret purpose, validate that the secret kind matches the purpose, then resolve that `secret_ref` through the secret boundary.

Raw provider records are append-only and idempotent by provider account, record kind and provider record ID. They preserve provider payload and provenance before canonical message projections are built.

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
