# Hermes Product Master Spec

## Status

This is the product-level source of truth for active Hermes documentation.

It describes the target product model and the current implementation baseline at
the same time. When these differ, the target model governs future product
direction, while the implementation baseline tells developers what actually
exists today.

This document does not define API routes, database migrations or runtime
implementation details.

## Canonical Product Definition

Hermes Hub is a local-first Personal Memory System.

Its product experience is a personal operating surface for:

- Communications;
- Knowledge;
- Memory;
- Relationships;
- Projects;
- Documents;
- Decisions;
- Obligations;
- Context.

The primary value is context. CRUD screens, inboxes, calendars, task lists and
document viewers are product surfaces, not the product thesis.

## Product Thesis

Hermes turns communications into durable personal memory and actionable context.

The core product cycle is:

```text
Communication
  -> Source Evidence
  -> Extracted Knowledge
  -> Memory
  -> Relationships
  -> Context
  -> Obligations / Tasks / Decisions / Projects
  -> Timeline / Dossier / Recall
```

Hermes should help the owner answer:

- what happened;
- who and what is involved;
- why something matters;
- what evidence supports it;
- what changed compared with previous memory;
- what obligations, decisions or tasks emerged;
- what context is needed before acting.

## What Hermes Is Not

Hermes is not:

- an email client;
- a messenger;
- a CRM;
- an address book;
- a task tracker;
- a calendar app;
- a note-taking app;
- a generic knowledge base;
- an AI chatbot.

These surfaces may exist inside Hermes, but only as views and workflows over one
source-backed memory system.

## Communication As Primary Ingestion Spine

Communication is the primary way real-world signals enter Hermes.

Communication includes:

- email;
- Telegram messages;
- WhatsApp messages;
- calls;
- meetings;
- threads and conversations;
- attachments and linked documents;
- replies, delays, silences and follow-ups where they carry meaning.

A Communication is not just an inbox item. It is evidence that can produce
knowledge, relationships, obligations, decisions, tasks and project context.

Communications are primary, but they are not the only source of evidence.
Documents, calendar events, manual owner input, imported files and provider
records can also create durable memory.

## Source Evidence To Memory Flow

Hermes must preserve evidence before extracting meaning.

```text
Provider or local source
  -> Source Record
  -> Canonical Event
  -> Domain Projection
  -> Knowledge / Memory Candidate
  -> Review or Policy Acceptance
  -> Durable Memory
  -> Derived Views and Agent Context
```

Rules:

- raw provider records and local artifacts are evidence;
- canonical events explain change;
- domain records own accepted truth;
- AI output is derived until accepted under domain rules;
- derived views must be rebuildable where practical;
- answers and actions must cite source evidence.

## Domain Model

Hermes domains are not separate applications. They are ownership boundaries
inside one memory system.

| Domain | Product role | Source-of-truth responsibility |
|---|---|---|
| Communications | Main ingestion spine for messages, calls, meetings, participants and attachments. | Canonical interactions and source communication evidence. |
| Personas | Memory anchors for subjects: owner, people, AI agents, system actors and organization proxies. | Persona identity traces, Persona memory anchors and Persona relationships. |
| Organizations | Collective actor memory. | Organization identity, relationships, portals, procedures, playbooks and organization memory. |
| Projects | Bounded work contexts. | Project state, goals, linked context, decisions and project memory. |
| Documents | Evidence artifacts. | Document versions, extracted content, metadata and document evidence. |
| Knowledge | Evidence-backed understanding. | Reviewed facts, observations and knowledge items with provenance. |
| Decisions | Durable choices. | Rationale, evidence and affected entities for decisions. |
| Obligations | Commitments and duties. | Evidence-backed commitments, expected actions and follow-up state. |
| Tasks | Executable work. | Action lifecycle, task status, task evidence and provider overlays. |
| Events | Things that happened or are scheduled. | Append-only event facts and scheduled event records. |
| Relationships | First-class links between entities. | Typed, source-backed connections with confidence and validity. |

Boundary rule:

```text
Domains own durable truth.
Engines produce derived intelligence.
Agents operate over context.
```

## Engine Model

Engines are shared mechanisms. They do not own domain entities.

| Engine | Purpose | Output type |
|---|---|---|
| Memory Engine | Assemble durable, source-backed memory across domains. | memory views, context summaries, memory gaps |
| Timeline Engine | Build chronological views across entities. | timeline views, diffs, period summaries |
| Trust Engine | Assess relationship and source reliability. | trust signals, confidence adjustments |
| Search Engine | Retrieve source-backed context. | ranked results, snippets, retrieval plans |
| Enrichment Engine | Propose additional knowledge from approved sources. | candidates, observations, conflicts |
| Obligation Engine | Detect commitments, duties and follow-ups. | obligations, task candidates, follow-up candidates |
| Risk Engine | Detect evidence-backed risks and attention signals. | risk observations, attention views |
| Consistency / Contradiction Engine | Detect conflicts between new evidence and accepted memory. | contradiction observations and review items |

### Consistency / Contradiction Engine

The user-facing alias for this engine is Polygraph.

The engine compares new evidence against accepted memory. It detects
contradictions, stale facts, disputed claims, conflicting decisions and
mismatched obligations.

It must not call a person a liar and must not overwrite memory. It creates a
source-backed observation for review.

Example:

```text
New email: "We never approved budget X."
Existing Decision: "Budget X approved on 2026-05-14."
Output: ContradictionObservation linked to Decision, Communication, Project and Personas.
```

Required observation fields:

- old source;
- new source;
- affected entities;
- conflict type;
- confidence;
- review state.

## Current Implementation Inventory

This inventory is based on current repository files.

### Backend Domains And Modules

The backend currently has domain modules for:

- calendar;
- documents;
- graph;
- mail;
- organizations;
- persons;
- projects;
- settings;
- tasks.

The backend also has AI, engines, integrations, platform and workflow modules.

Notable integrations:

- Gmail;
- Ollama;
- Omniroute;
- Telegram;
- WhatsApp.

Platform support exists for:

- event log;
- audit log;
- capabilities;
- calls and transcripts;
- secrets;
- settings;
- storage;
- host vault.

### Persistence Baseline

Current migrations include storage for:

- event log and projection cursors;
- communication provider accounts, raw records and canonical messages;
- mail blob and attachment metadata;
- documents and document processing jobs;
- graph nodes, edges and evidence;
- projects and project link reviews;
- task candidates and tasks;
- persons compatibility tables and person memory tables;
- organizations and organization memory/workflow tables;
- calendar accounts, events, meetings, deadlines, focus blocks and rules;
- Telegram accounts, chats, messages, policies, calls and transcripts;
- WhatsApp Web sessions and messages;
- application settings, secret references, encrypted vault entries and host vault support;
- AI runtime, semantic embeddings and AI control center tables.

### API Surface Baseline

Routes are currently registered centrally in `backend/src/app/router.rs`.

Implemented route groups include:

- `/api/v1/communications/*`;
- `/api/v1/graph/*`;
- `/api/v1/projects/*`;
- `/api/v1/documents/*` and `/api/v1/document-processing/*`;
- `/api/v1/persons/*`;
- `/api/v1/calendar/*`;
- `/api/v1/organizations/*`;
- `/api/v1/tasks/*` and `/api/v1/task-candidates/*`;
- `/api/v1/settings/*`;
- `/api/v1/ai/*`;
- `/api/v1/telegram/*`;
- `/api/v1/whatsapp/*`;
- `/api/v1/policies/*`;
- `/api/v1/calls/*`;
- `/api/v1/email-accounts/*`;
- `/api/v1/events/*` and `/api/v1/audit/events`.

This route list is implementation evidence only. It is not the target product
model.

### Frontend Surface Baseline

The frontend currently has page surfaces for:

- Agents;
- Calendar;
- Communications;
- Documents;
- Home;
- Knowledge;
- Notes;
- Organizations;
- Persons;
- Projects;
- Settings;
- Tasks;
- Telegram;
- Timeline;
- WhatsApp.

Several surfaces still use compatibility names such as Persons, Notes, health or
watchtower. Those names must be interpreted through the foundation glossary and
future product roadmap.

## Target Gaps And Refactoring Direction

The current implementation is meaningful but not yet fully aligned with the
target product model.

| Gap | Current evidence | Direction |
|---|---|---|
| Persona-native model incomplete | `persons`, `person_id`, `person_roles`, `person_personas`, `person_promises` and `/api/v1/persons/*` still exist. | Keep compatibility short-term. Plan Persona-native schema/API and UI naming under a dedicated migration plan. |
| Owner Persona missing as enforced concept | Docs define one `is_self = true` Persona, but current migrations do not enforce it as the target root. | Add Owner Persona semantics in a future implementation plan. |
| First-class Relationships incomplete | Graph edges and relationship events exist, but target Relationship records are not fully separated from roles, fields and timeline events. | Introduce first-class Relationship model with provenance, confidence, trust and validity. |
| Polygraph engine not implemented | No dedicated Consistency / Contradiction Engine module or schema exists. | Add engine spec first, then implement contradiction observations and review workflow. |
| Communications still mail-heavy | Many modules are email-specific under `domains/mail`. | Keep provider-specific modules but document Communications as the product domain and email as one channel. |
| Engine boundaries are partial | Search and automation engines exist; Memory, Timeline, Trust, Risk, Enrichment and Obligation are partly embedded in domain modules. | Create engine specs and extract/rename implementation only under dedicated plans. |
| Knowledge model incomplete | Knowledge graph exists, but Knowledge as reviewed understanding is not fully documented or implemented as a lifecycle. | Define Knowledge domain spec and review states before implementation work. |
| Decisions and Obligations incomplete | Tasks, meeting outcomes and promises exist, but Decisions and Obligations are not consistent top-level models yet. | Create domain specs and implementation plans for durable decisions and obligations. |
| Notes are ambiguous | Frontend has Notes page, while foundation says Notes are document-like artifacts unless a future ADR promotes them. | Treat Notes as document-like capture artifacts until a separate ADR changes scope. |

## Core Workflows

### Incoming Communication To Context

```text
Incoming Communication
  -> preserve source evidence
  -> classify channel, thread and participants
  -> resolve Personas and Organizations
  -> extract claims, facts, preferences, obligations, decisions and risks
  -> check contradictions through the Polygraph engine
  -> link to Projects, Documents, Tasks and prior Memory
  -> update Timeline views and Dossiers
  -> create review items where confidence is insufficient
  -> propose Tasks / Follow-Ups / Decisions
  -> assemble context for owner or agent
```

### Workflow Set

| Workflow | Product output |
|---|---|
| Email to Knowledge | Source-backed knowledge candidates linked to Personas, Organizations, Projects and Documents. |
| Message to Obligation | Obligation candidates and follow-up/task suggestions. |
| Meeting to Decisions | Decisions, obligations, tasks and timeline events from meetings. |
| Document to Context | Document evidence linked to projects, organizations, decisions, risks and tasks. |
| Contradiction Review | Reviewable conflict observations without silent memory overwrite. |
| Dossier Generation | Derived, cited dossiers for Personas, Organizations, Projects or other context anchors. |
| Agent-Assisted Recall | Source-backed answers that distinguish facts, guesses, conflicts and stale memory. |

## Review, Confidence And Provenance

Hermes must distinguish:

- source evidence;
- accepted domain truth;
- inferred candidates;
- AI-generated observations;
- derived read models;
- stale or contradicted memory.

Rules:

- Nothing important becomes durable truth without provenance.
- Nothing uncertain bypasses review.
- Nothing derived silently overwrites memory.
- AI output must cite source evidence.
- Contradictions create review items, not automatic truth replacement.

## Agent Behavior

Agents operate over context. They are not source of truth.

When agents are represented in the world model, they are Personas with
`persona_type = ai_agent`.

Agents must:

- retrieve context from domains and engines;
- distinguish source facts from inference;
- cite evidence;
- respect capability and confirmation policies;
- write auditable actions;
- avoid direct durable mutations without domain rules.

## Documentation Expansion Map

Wave 1 creates the product spine:

- `docs/product/master-spec.md`;
- `docs/product/development-roadmap.md`;
- `docs/README.md`.

Later waves should create or normalize:

- domain specs for Communications, Personas, Relationships, Knowledge,
  Obligations, Tasks, Decisions, Projects, Documents, Organizations and Events;
- engine specs for Memory, Timeline, Trust, Search, Enrichment, Obligation,
  Risk and Consistency / Contradiction;
- workflow specs for communication-to-knowledge, communication-to-obligation,
  meeting-to-decisions, document-to-context, contradiction-review,
  dossier-generation and agent-assisted-recall.
