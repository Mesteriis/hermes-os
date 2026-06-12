# Email Channel Architecture

## Position

Email belongs to the Communications domain. It is not a separate product or a
parallel memory model.

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

Email-specific tables preserve provider records, messages, drafts, templates,
attachments and related metadata. They feed canonical Communications, Events and
shared engines. Search indexes and AI summaries are derived state.
