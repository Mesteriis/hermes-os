# Communications Domain

Status: documentation package aligned to the current repository structure.

Communications are the primary ingestion spine of Hermes.

Hermes receives messages, meetings, calls and provider events as evidence. From
that evidence it extracts knowledge, memory, relationships, obligations, tasks,
decisions and project context.

```text
Communication
  -> Source Evidence
  -> Extracted Knowledge
  -> Memory
  -> Relationships
  -> Context
  -> Obligations / Tasks / Decisions / Projects
```

Hermes is not an email client or messenger. Provider surfaces are capture and
interaction boundaries for the Personal Memory System.

Invariant: A channel is never a domain. A channel is an integration. A
communication is the domain object.

## Responsibilities

The Communications domain owns:

- canonical conversations;
- canonical communication records;
- participants as observed in a source;
- provider channel accounts;
- source message/event metadata;
- delivery and draft state;
- attachments at communication boundaries;
- communication-to-entity links;
- provenance for all extracted observations.

The Communications domain does not own:

- Persona truth;
- Organization truth;
- Task lifecycle;
- Project lifecycle;
- global memory;
- global timeline;
- search indexes;
- AI conclusions.

## Communication Types

Hermes treats the following as one family of interactions:

- email;
- Telegram messages;
- WhatsApp messages;
- calls;
- meetings;
- future chat or provider streams.

Provider-specific details remain at adapter and source-record boundaries.
Product workflows operate over Communication, Conversation, Participant,
Attachment, Event and Context.

Telegram provider-specific production behavior is documented in
[Telegram Channel Capability Spec](../../integrations/telegram/README.md). That
document set is a channel capability spec, not a separate domain.

## Source Evidence

Each imported communication must preserve source provenance:

- provider kind;
- provider account;
- provider message/event identifier;
- raw source reference where available;
- import time;
- observed participants;
- content hash or blob reference where appropriate;
- extraction run metadata.

Source evidence is immutable. Corrections are represented as later events,
review decisions or superseding derived records.

## Trace Context

Communications consumes accepted provider/source signals and emits canonical
communication events with inherited trace context.

```text
signal.accepted.<source>.message
  -> communication.message.recorded / communication.message.updated
```

Communication events set `causation_id = accepted_signal.event_id` and inherit
`correlation_id = accepted_signal.correlation_id`. Subjects must identify the
canonical communication entity, for example:

```json
{
  "kind": "communication_message",
  "entity_id": "message_...",
  "message_id": "message_..."
}
```

Provider-specific runtime state stays in integrations. Trace reconstruction
stays in `platform/events`.

## Extraction Pipeline

```text
source record
  -> normalization
  -> conversation/thread linking
  -> participant resolution candidates
  -> entity extraction
  -> knowledge candidates
  -> obligation/task/decision candidates
  -> consistency checks
  -> reviewable memory updates
```

AI may assist each stage, but AI output is not source of truth.

## Engine Use

Communications use:

- Memory Engine for durable communication memory;
- Timeline Engine for interaction history;
- Search Engine for recall;
- Enrichment Engine for entity and link candidates;
- Obligation Engine for commitments and duties;
- Risk Engine for spam, phishing, urgency and attention signals;
- Consistency / Contradiction Engine for conflicts with accepted memory.

## Current Implementation Evidence

Current backend implementation is split across:

- `backend/src/domains/communications/*`;
- `backend/src/integrations/mail/gmail/*`;
- `backend/src/integrations/telegram/*`;
- `backend/src/integrations/whatsapp/*`;
- calls and communication-related routes registered in
  `backend/src/app/router.rs`;
- migrations `0005`, `0007`, `0011`, `0012`, `0020`, `0021`, `0025` through
  `0032`, `0055`, `0056` and `0149`.

Current UI uses `/communications` as the single communication workspace.
Telegram, WhatsApp and Mail appear as filters, account setup panels, runtime
status panels or capability panels. The backend still has some email-heavy
compatibility names because email was implemented first.

## Migration Plan

1. Keep provider-specific code inside integration modules.
2. Document new behavior under Communications, not Mail, Telegram or WhatsApp
   domains.
3. Treat mail, Telegram, WhatsApp, calls and meetings as channel-specific
   adapters feeding the same Communication model.
4. Use `/api/v1/communications/{mail,telegram,whatsapp}/*` for public
   channel-scoped communication APIs.
5. Introduce Consistency / Contradiction Engine review output before any
   automatic memory overwrite behavior.

## Navigation

- [Architecture](./architecture.md)
