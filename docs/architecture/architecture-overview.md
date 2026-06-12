# Architecture Overview

## Architectural Thesis

Hermes Hub is a local-first Personal Memory System. Its durable system of record
combines append-only source records, canonical events, domain entities,
relationships, document artifacts and rebuildable indexes. AI uses these stores
as context and never becomes the durable memory layer itself.

Canonical architecture language lives in:

- [Foundation Vision](../foundation/vision.md)
- [World Model](../foundation/world-model.md)
- [Engines](../foundation/engines.md)
- [Architecture Principles](../foundation/architecture-principles.md)

## Top-Level Shape

```mermaid
flowchart TB
    Sources["External and local sources"] --> Adapters["Provider and import adapters"]
    Adapters --> SourceRecords["Append-only source records"]
    SourceRecords --> EventLog["Canonical event log"]
    EventLog --> Projectors["Projectors"]
    Projectors --> Domains["Domain state"]
    Projectors --> Graph["Knowledge graph"]
    Projectors --> Objects["Documents and attachment storage"]
    Domains --> Engines["Reusable engines"]
    Graph --> Engines
    Objects --> Engines
    Engines --> Context["Context"]
    Domains --> API["Rust application API"]
    Engines --> API
    Context --> API
    API --> Agents["Agent runtime"]
    Agents --> API
    API --> UI["SvelteKit UI in Tauri shell"]
```

## Layers

### Interface Layer

- SvelteKit frontend.
- Tauri desktop shell.
- Command palette.
- Keyboard-first navigation.
- Contextual AI affordances.

### Application Layer

- Command handling.
- Query handling.
- Orchestration workflows.
- Permissions and capability checks.
- Agent/tool execution boundary.

### Domain Layer

Domains own source-of-truth entities and invariants:

- Personas.
- Organizations.
- Communications.
- Projects.
- Documents.
- Tasks.
- Calendar/Events.
- Decisions.
- Obligations.
- Knowledge Graph relationships.

### Engine Layer

Engines are reusable mechanisms used by domains:

- Memory Engine.
- Timeline Engine.
- Trust Engine.
- Search Engine.
- Enrichment Engine.
- Obligation Engine.
- Risk Engine.
- Consistency / Contradiction Engine.

Engines produce projections, observations, candidates, scores or context. They do
not own domain entities.

### Infrastructure Layer

- PostgreSQL.
- Tantivy.
- Vector index provider.
- Document object storage.
- Provider adapters.
- Ollama runtime.
- Telemetry pipeline.

## Dependency Direction

UI calls application APIs. Application services coordinate domain workflows and
engines. Domain logic must not depend on provider APIs, UI state or storage
details. Infrastructure implements ports required by application, domains and
engines.

## Durable State Categories

- raw imported source records;
- canonical event log;
- normalized domain records;
- relationship records and graph evidence;
- document versions and extracted artifacts;
- reviewed memory, decisions, obligations and knowledge;
- agent execution traces.

## Derived State Categories

- search indexes;
- embeddings;
- timeline views;
- dossiers;
- context packs;
- AI summaries and observations;
- risk/trust/priority scores.

Derived state must be rebuildable or explicitly cacheable from durable state.

## Replaceability

The following components must be replaceable behind stable boundaries:

- LLM provider;
- embedding model;
- vector index implementation;
- messaging provider adapters;
- calendar provider adapters;
- task provider adapters;
- OCR engine;
- full text index backend;
- UI shell.
