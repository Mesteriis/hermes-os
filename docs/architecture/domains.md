# Canonical Domain Architecture

Status: Canonical architecture baseline for the 2026-06-18 documentation
consolidation.

Scope: bounded-context map and ownership rules. This document is not an
implementation refactoring plan.

## Purpose

This document defines the current target bounded contexts for Hermes. A domain
exists when Hermes needs durable source-of-truth ownership for an entity,
lifecycle or invariant.

## Domain Ownership Rule

```text
Domains own durable truth.
Engines produce derived intelligence.
Integrations preserve provider evidence.
The UI operates over domain and engine APIs.
Agents propose and act through capabilities.
```

## Canonical Domains

| Domain | Owns | Does not own | Reason for existence |
|---|---|---|---|
| Personas | Persona identity, Owner Persona, identity traces, Persona memory anchors, Persona dossiers. | Provider messages, Organization lifecycle, Project lifecycle, generic graph traversal. | Hermes needs durable subjects for people, AI agents, system actors and organization proxies. |
| Organizations | Organization identity, domains, aliases, relationships, portals, procedures, playbooks, organization memory. | Persona identity, Project ownership, provider accounts. | Collective actors need memory and procedures independent from individual Personas. |
| Communications | Conversations, messages, participants as observed, channel accounts, source communication metadata, delivery/draft state, communication attachments. | Persona truth, Task lifecycle, Decision truth, Obligation truth, global Memory. | Communications are the primary evidence intake spine. |
| Documents | Document artifacts, versions, extracted content, document metadata, document evidence, promoted attachment artifacts. | General Knowledge truth, Task status, provider message lifecycle. | Documents are durable evidence artifacts and local knowledge sources. |
| Projects | Bounded work contexts, project state, project links, project decisions as references, project memory views. | Organization identity, Task lifecycle, Decision truth, document versions. | Projects gather context around long-running work. |
| Tasks | Actionable work items, status lifecycle, local overlays, task evidence, provider overlays. | Obligations as commitments, every follow-up, provider message delivery. | Some memory becomes executable work with lifecycle. |
| Calendar/Events | Scheduled events, meetings, attendees, calendar source identity, event evidence. | Global Timeline Engine, Decision/Obligation truth. | Time-bound facts and meetings provide context and source evidence. |
| Relationships | Durable semantic links, relation type, trust score, strength score, confidence, evidence, review state. | Graph indexes, Trust Engine computation, Timeline rendering. | Hermes is relationship-first; links need a source-of-truth owner. |
| Decisions | Durable choices, rationale, alternatives, evidence and impacted entities. | Generic notes, Project state, AI summaries. | Hermes must remember why a direction was chosen. |
| Obligations | Commitments, duties, beneficiaries, status, evidence, review state and links to fulfillment. | Task lifecycle, every reminder, provider delivery state. | A commitment is not the same as a task that may fulfill it. |
| Knowledge Graph | Graph nodes, graph edges, graph evidence as projection/traversal substrate. | Relationship semantics when first-class Relationship records exist, raw provider sync, binary storage. | Relationship-aware memory and traversal need a queryable graph substrate. |
| Agents | Agent identity, run records, capability policy integration, proposed actions, approvals, denials, audit trail. | Domain truth, private data truth, credentials. | Agents need an auditable actor and tool boundary. |

## Concepts That Are Not Domains Today

| Concept | Classification | Reason |
|---|---|---|
| Email | Communications channel. | It supplies communication evidence and provider operations. |
| Telegram | Communications channel. | It supplies source evidence, provider commands, realtime events and media evidence. |
| WhatsApp | Communications channel. | It is a provider/source boundary under Communications. |
| Calls | Communication/Event evidence surface. | Calls may produce source evidence, transcripts and timeline entries. |
| Meetings | Calendar/Event evidence plus Communication context. | Meeting outputs may become Decisions, Obligations or Tasks. |
| Notes | Document-like capture artifact. | No current ADR promotes Notes to a first-class domain. |
| Timeline | Engine/read model. | Chronological views are derived from dated records and events. |
| Radar | Candidate intake layer and derived inbox. | No durable Radar entity or lifecycle is proven yet. |
| Generic Observations | Open policy area. | Concrete observations belong to producing engines/domains until accepted. |
| Knowledge | Emergent memory layer, not a generic wiki silo today. | Reviewed facts must retain domain/source ownership. |

## Engine Boundary

Engines currently recognized by the architecture:

- Memory Engine;
- Timeline Engine;
- Search Engine;
- Trust Engine;
- Risk Engine;
- Enrichment Engine;
- Obligation Engine;
- Decision candidate engine;
- Consistency / Contradiction Engine, user-facing alias Polygraph;
- Automation policy engine.

Engines may persist candidates, observations, projections or scores when a
specific ADR defines that storage. They must not silently become domain owners.

## Allowed Cross-Domain Links

Cross-domain relationships are allowed through:

- source evidence references;
- canonical events;
- first-class Relationship records;
- graph projections;
- candidate/review records;
- application services;
- workflow orchestration.

Direct ownership transfer is not allowed. For example:

- Communications may propose a Task candidate; Tasks own the accepted Task.
- A meeting may produce a Decision candidate; Decisions own the durable
  Decision.
- Telegram may observe a participant; Personas own Persona truth.
- An attachment may be promoted to a Document; Documents own the artifact after
  promotion.

## Implementation Evidence

Current backend modules observed during this audit:

- domains: `calendar`, `decisions`, `documents`, `graph`, `mail`,
  `obligations`, `organizations`, `persons`, `projects`, `relationships`,
  `settings`, `tasks`;
- engines: `automation`, `consistency`, `decision`, `enrichment`, `memory`,
  `obligation`, `risk`, `search`, `timeline`, `trust`;
- integrations: `gmail`, `telegram`, `whatsapp`, `ollama`, `omniroute`.

This evidence explains the current implementation shape. It does not authorize
renaming modules or moving code without a later refactoring plan.
