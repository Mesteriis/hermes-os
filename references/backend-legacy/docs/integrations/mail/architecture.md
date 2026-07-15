# Email Channel Architecture

## Position

Email belongs to the Communications domain. It is not a separate product or a
parallel memory model.

Invariant: A channel is never a domain. A channel is an integration. A
communication is the domain object.

## Layers

```text
UI surface
  -> Communications API
  -> Communications domain services
  -> Email provider adapters
  -> raw source records
  -> canonical Communication projections
  -> shared engines
```

## Data Flow

### Ingestion

```text
Provider -> Raw Records -> Message Projection -> Events -> Graph -> Engines
```

### Sending

```text
Draft -> explicit owner confirmation/policy -> provider send capability
```

### Engine Processing

```text
Communication
  -> Search Engine
  -> Risk Engine
  -> Obligation Engine
  -> Enrichment Engine
  -> Memory Engine
```

## Key ADR

| ADR | Topic |
|---|---|
| ADR-0001 | Event sourcing as system spine |
| ADR-0005 | PostgreSQL primary store |
| ADR-0006 | Tantivy full-text search |
| ADR-0009 | Local AI through Ollama |
| ADR-0041 | Email provider ingestion foundation |
| ADR-0042 | Secret references for provider credentials |
| ADR-0044 | Account setup and encrypted vault |
| ADR-0046 | Blob storage for attachments |
| ADR-0053 | Database encrypted vault |
| ADR-0055 | Full email provider networking |

## Storage Boundary

Canonical communication tables own accounts, channels, conversations, messages,
drafts, outbox, attachments, saved searches and provider command state.
Historical email/mail-prefixed tables may remain for upgrade compatibility, but
new runtime/domain behavior must treat them as migration sources or adapter
compatibility, not as the product domain owner. Search indexes and AI summaries
are derived state.
