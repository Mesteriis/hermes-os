# Product Scope

## In Scope

### Communications

- Email, Telegram, WhatsApp and calls as communication channels.
- Provider source preservation.
- Canonical messages, conversations, participants, attachments and delivery
  metadata.
- Relevance, spam, marketing and risk classification as engine output.

### Personas

- Persona identity traces.
- Relationship history and graph neighborhood.
- Communication context.
- Persona memory and dossier.
- Identity resolution and reviewed merge/split workflows.

### Organizations

- Organizations as first-class entities.
- Organization identities, domains, portals, procedures and playbooks.
- Relationships to Personas, Projects, Documents, Communications and
  Obligations.

### Projects

- Bounded work contexts.
- Linked communications, documents, decisions, tasks, obligations and Personas.
- Project context and timeline views through engines.

### Documents

- PDF, Office, images and Markdown.
- Versioning, extraction, OCR, metadata, summaries and entity mentions.
- Links to other world-model entities.

### Tasks And Obligations

- Task candidates extracted from evidence.
- Actionable Tasks with lifecycle.
- Obligations as commitments or duties with evidence.
- Follow-Ups as prompts that may become Tasks or remain reminders.

### Events And Timeline

- Canonical system events.
- Calendar/meeting events.
- Timeline Engine views over source-backed events and domain records.

### Knowledge And Graph

- First-class relationships with provenance.
- Evidence-backed facts, decisions and observations.
- Graph-aware context assembly.

### Engines

- Memory Engine.
- Timeline Engine.
- Trust Engine.
- Search Engine.
- Enrichment Engine.
- Obligation Engine.
- Risk Engine.
- Consistency / Contradiction Engine, user-facing alias Polygraph.

### Agents

- HESTIA as coordinator.
- Specialized agents for communications, memory, analysis and tool automation.
- Typed tools, explicit permissions and source-backed context.

## Out Of Scope For Initial Implementation, But Architecturally Supported

- multi-user SaaS;
- enterprise CRM workflows;
- public API marketplace;
- global cloud sync as a required dependency;
- end-to-end encrypted multi-device sync;
- autonomous external actions without explicit permission policy.

These items must not dictate the current implementation, but the architecture
should not unnecessarily block future work.

## Non-Goals

- replace Gmail, Telegram, WhatsApp or calendars as network providers;
- train a personal LLM on private data;
- store only embeddings without source evidence;
- hide automatic decisions without provenance;
- create one universal activity table.

## Capability Map

| Capability | Core entities | Primary output |
|---|---|---|
| Communication ingestion | Communication, Source record, Event, Persona | normalized events and messages |
| Persona memory | Persona, Relationship, Knowledge item | source-backed Persona context |
| Organization memory | Organization, Relationship, Document | organization context |
| Document understanding | Document, Version, Entity mention | indexed and linked evidence |
| Obligation extraction | Obligation, Task candidate, Source record | reviewed commitments and actions |
| Search and recall | Source record, Entity, Relationship, Event | ranked source-backed results |
| Agent orchestration | Agent Persona, Tool, Context | explainable AI workflows |
| Project memory | Project, Task, Document, Decision, Obligation | project context and timeline |
