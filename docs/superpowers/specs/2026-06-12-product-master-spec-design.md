# Product Master Spec Documentation Design

Date: 2026-06-12

## Goal

Create the design for a documentation program that turns the current Hermes
documentation into a coherent product master specification and a derived set of
domain, engine and workflow documents.

This is documentation-only work. It does not design new APIs, database
migrations, runtime modules or implementation code.

## Context

Hermes documentation has been consolidated around the foundation model:

```text
Hermes is a local-first Personal Memory System.
```

The foundation documents already define the canonical vocabulary, world model,
domain map, engines and architecture principles. The next product documentation
step is not another isolated domain rewrite. It is a product-level master spec
that explains how Hermes works as one system and how future domain documents
derive from that system.

The user clarified the key product thesis:

```text
Communication is the primary ingestion spine.
```

Communications are not the whole product and not the only evidence source, but
they are the main way real-world signals enter Hermes. Email, Telegram,
WhatsApp, calls and meetings produce evidence that becomes knowledge, memory,
relationships, obligations, tasks, decisions, projects, timelines and dossiers.

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

Hermes is not an email client, CRM, task tracker, calendar app, note-taking app
or generic knowledge base. Those surfaces may exist, but they are views and
workflows over one personal memory system.

## Target Reader

The master spec is for a mixed audience, but it is engineering-first:

- it explains the product value and core user scenarios;
- it defines domain and engine boundaries precisely enough for agents and
  developers;
- it establishes the future documentation tree and development roadmap;
- it avoids pseudo-APIs and speculative implementation details.

## Master Spec Shape

The primary deliverable should be:

```text
docs/product/master-spec.md
```

The spec should cover:

1. What Hermes is.
2. What Hermes is not.
3. Why communications are the ingestion spine.
4. How evidence becomes knowledge, memory, context and action.
5. The canonical domain model.
6. The canonical engine model.
7. End-to-end workflows.
8. Review, confidence and provenance rules.
9. Agent behavior over context.
10. Product development slices.
11. Documentation map for derived domain, engine and workflow specs.

## Domain Model

The master spec should present domains as parts of one memory system, not as
separate applications.

Primary flow:

```text
Communications
  -> Evidence
  -> Knowledge / Memory
  -> Context
  -> Action
```

Domains:

| Domain | Role in the product model |
|---|---|
| Communications | Main ingestion spine: email, Telegram, WhatsApp, calls, meetings, threads, participants, attachments and delivery state. |
| Personas | Subjects of memory: Owner Persona, humans, AI agent Personas, organization proxies and system actors. |
| Organizations | Collective actors: companies, government bodies, communities, services and institutions. |
| Projects | Bounded work contexts where communications, documents, decisions, tasks and obligations converge. |
| Documents | Evidence artifacts: attachments, PDFs, Markdown, contracts, invoices and notes-as-documents. |
| Knowledge | Evidence-backed understanding, not a loose wiki or unverified notes store. |
| Decisions | Durable choices with rationale and source evidence. |
| Obligations | Commitments, duties, expectations and promises with provenance. |
| Tasks | Executable work items, often derived from obligations or communications. |
| Events | Things that happened or are scheduled, used by the Timeline Engine. |
| Relationships | First-class links between entities, not fields hidden inside objects. |

Boundary rule:

```text
Domains own durable truth.
Engines produce derived intelligence.
Agents operate over context.
```

## Engine Model

The master spec should include the current foundation engines plus one new
engine concept requested by the user.

| Engine | Role |
|---|---|
| Memory Engine | Builds and retrieves durable source-backed memory. |
| Timeline Engine | Produces chronological views across domains. |
| Trust Engine | Computes trust and source-reliability signals. |
| Search Engine | Retrieves source-backed context through text, semantic and graph search. |
| Enrichment Engine | Proposes candidate facts, links and metadata from approved sources. |
| Obligation Engine | Detects commitments, duties, follow-ups and task candidates. |
| Risk Engine | Detects evidence-backed risks and attention signals. |
| Consistency / Contradiction Engine | Detects conflicts between new evidence and accepted memory. User-facing alias: Polygraph. |

### Consistency / Contradiction Engine

This engine compares new Communications, Documents, Events and AI observations
against accepted Memory and Knowledge.

It must detect:

- direct contradictions;
- stale facts;
- disputed claims;
- conflicting decisions;
- mismatched obligations;
- claims that weaken existing trust assumptions.

It must not automatically declare a person dishonest or overwrite memory.
Instead, it creates a source-backed observation such as:

```text
ContradictionObservation
  old_source
  new_source
  affected_entities
  conflict_type
  confidence
  review_state
```

Example:

```text
New email: "We never approved budget X."
Existing Decision: "Budget X approved on 2026-05-14."
Output: ContradictionObservation linked to Decision, Communication, Project and Personas.
```

## Product Workflows

The master spec should explain Hermes through end-to-end workflows.

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

### Required Workflows

| Workflow | Purpose |
|---|---|
| Email to Knowledge | Preserve source, extract claims/facts, link to entities and create reviewed knowledge candidates. |
| Message to Obligation | Convert commitments such as "I will send the contract tomorrow" into obligation candidates and follow-up/task suggestions. |
| Meeting to Decisions | Convert meeting notes/transcripts into decisions, tasks, obligations and timeline events. |
| Document to Context | Use contracts, invoices and attachments as evidence for projects, organizations, risks and decisions. |
| Contradiction Review | Detect conflicts between new evidence and accepted memory without silently overwriting truth. |
| Dossier Generation | Assemble source-backed dossiers for Personas, Organizations, Projects or other context anchors. |
| Agent-Assisted Recall | Let agents answer through source-backed context while distinguishing facts, guesses, conflicts and stale memory. |

Workflow rule:

```text
Nothing important becomes durable truth without provenance.
Nothing uncertain bypasses review.
Nothing derived silently overwrites memory.
```

## Documentation Package

The master spec should introduce a documentation tree that can be filled in
incrementally.

### Product Documents

| Document | Purpose |
|---|---|
| `docs/product/master-spec.md` | Product-level source of truth. |
| `docs/product/development-roadmap.md` | Future product slices derived from the master spec. |
| `docs/README.md` | Reading order and documentation map for developers and agents. |

### Domain Specs

Domain specs should use one standard template:

- purpose;
- source of truth;
- owned entities;
- inputs;
- outputs;
- relationships;
- engine usage;
- review/confidence rules;
- not this domain;
- open product questions.

Planned domain specs:

- `docs/domains/communications.md`
- `docs/domains/personas.md`
- `docs/domains/organizations.md`
- `docs/domains/projects.md`
- `docs/domains/documents.md`
- `docs/domains/knowledge.md`
- `docs/domains/decisions.md`
- `docs/domains/obligations.md`
- `docs/domains/tasks.md`
- `docs/domains/events.md`
- `docs/domains/relationships.md`

### Engine Specs

Engine specs should use one standard template:

- what it does;
- inputs;
- outputs;
- derived state;
- confidence and provenance;
- review behavior;
- domains that use it.

Planned engine specs:

- `docs/engines/memory-engine.md`
- `docs/engines/timeline-engine.md`
- `docs/engines/trust-engine.md`
- `docs/engines/search-engine.md`
- `docs/engines/enrichment-engine.md`
- `docs/engines/obligation-engine.md`
- `docs/engines/risk-engine.md`
- `docs/engines/consistency-contradiction-engine.md`

### Workflow Specs

Planned workflow specs:

- `docs/workflows/communication-to-knowledge.md`
- `docs/workflows/communication-to-obligation.md`
- `docs/workflows/meeting-to-decisions.md`
- `docs/workflows/document-to-context.md`
- `docs/workflows/contradiction-review.md`
- `docs/workflows/dossier-generation.md`
- `docs/workflows/agent-assisted-recall.md`

## Execution Strategy

The documentation package should be implemented in waves.

### Wave 1: Product Spine

Create:

- `docs/product/master-spec.md`
- `docs/product/development-roadmap.md`
- `docs/README.md`

Update:

- root `README.md` documentation links if needed.

Goal: establish one product-level source of truth before expanding domain docs.

### Wave 2: Core Domain Specs

Create or normalize:

- Communications
- Personas
- Relationships
- Knowledge
- Obligations
- Tasks
- Decisions

Goal: document the main path from communication evidence to memory and action.

### Wave 3: Context Domains

Create or normalize:

- Projects
- Documents
- Organizations
- Events
- Dossier / Context Pack read-model docs if needed.

Goal: document where knowledge lands and how context is assembled.

### Wave 4: Engine Specs

Create the engine documentation set, including Consistency / Contradiction
Engine.

Goal: keep reusable mechanisms out of domain documents and prevent duplicated
Timeline, Memory, Risk, Trust or Polygraph logic in each domain.

### Wave 5: Workflow Specs

Create end-to-end workflow documents.

Goal: show how the product actually works for the owner and how agents should
operate over source-backed context.

## First Implementation Plan Scope

The first implementation plan after this design should cover Wave 1 only.

Wave 1 is intentionally documentation-only and should not:

- change backend or frontend code;
- introduce API routes;
- propose database migrations;
- rename implementation modules;
- rewrite historical ADRs.

## Validation Strategy

For the documentation implementation plan, validation should include:

- `git status --short` before and after;
- scoped diff review;
- `git diff --check -- README.md docs`;
- markdown file count checks used by repository instructions;
- terminology search for old CRM/contact/email-client framing in active docs;
- confirmation that no code files changed as part of the documentation task.

If markdown linting is later configured, use the configured command. Do not
invent a markdown linter.

## Risks

- The task can sprawl into a full rewrite of every document. The wave structure
  keeps the first plan bounded to the product spine.
- Existing implementation routes and tables still use compatibility names such
  as `persons`, `person_id`, `health` and `watchlist`. The master spec must
  identify these as compatibility details, not target concepts.
- Communication-first framing can accidentally make documents, calendar events
  and manual input look unimportant. The spec must say communications are the
  primary ingestion spine, not the only source of evidence.
- The Polygraph concept can be misread as judging people. The spec must define
  it as consistency and contradiction detection over claims and evidence.

## Acceptance Criteria

The design is successful when a reader can explain:

- why Hermes is a Personal Memory System;
- why Communication is the primary ingestion spine;
- how evidence becomes knowledge, memory, context and action;
- what the core domains are;
- what engines exist and why they are not domains;
- how the Polygraph engine detects contradictions;
- what documents should be written next;
- why Wave 1 should come before domain-by-domain expansion.
