# Product Master Spec Docs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create the Wave 1 product documentation spine that aligns Hermes docs to the approved Product Master Spec design while clearly separating current implementation from target product architecture.

**Architecture:** This is documentation-only work. The product master spec becomes the top-level source of truth, the development roadmap records target-vs-current gaps and refactoring/delivery slices, and `docs/README.md` gives developers and agents a reading order. No backend, frontend, schema, route or API changes are part of this plan.

**Tech Stack:** Markdown documentation in the existing Hermes Hub monorepo; validation through Git diff checks, repository markdown counts and terminology searches.

---

## File Structure

- Create: `docs/product/master-spec.md`
  - Product-level source of truth for Hermes as a Personal Memory System.
  - Must include Communication as the primary ingestion spine.
  - Must include the Consistency / Contradiction Engine with user-facing alias "Polygraph".
  - Must include a current implementation inventory based on repository files.

- Create: `docs/product/development-roadmap.md`
  - Future-oriented product roadmap derived from the master spec.
  - Must include current implementation evidence and target gaps.
  - Must include refactoring/delivery plans where target concepts differ from current implementation.

- Create: `docs/README.md`
  - Reading order and documentation map for developers and agents.
  - Must explain the difference between foundation docs, product docs, domain docs, ADRs, implementation status and historical plans.

- Modify: `README.md`
  - Add links to the new Wave 1 product documents.
  - Do not remove current development runbook content.

## Current Implementation Evidence To Reference

Use these repository facts when writing the docs:

- Backend domains exist under `backend/src/domains/` for calendar, documents, graph, mail, organizations, persons, projects, settings and tasks.
- Backend integrations exist for Gmail, Ollama, Omniroute, Telegram and WhatsApp.
- Backend platform support exists for event log, audit log, capabilities, calls, secrets, settings, storage and vault.
- Backend routes are centrally registered in `backend/src/app/router.rs`.
- Migrations include communication ingestion/messages, documents, graph, projects, task candidates/tasks, persons, organizations, calendar, Telegram, WhatsApp, settings/vault, AI runtime and AI control center.
- Frontend pages exist for agents, calendar, communications, documents, home, knowledge, notes, organizations, persons, projects, settings, tasks, Telegram, timeline and WhatsApp.
- Current implementation still uses compatibility concepts such as `persons`, `person_id`, `person_roles`, `person_personas`, `person_promises`, `health`, `watchtower`, `contacts` route labels and email-specific module names.
- There is no implemented first-class Consistency / Contradiction Engine yet.
- There is no implemented final Persona-native schema/API migration yet.
- There is no complete set of domain specs, engine specs or workflow specs yet.

## Task 1: Create Product Master Spec

**Files:**
- Create: `docs/product/master-spec.md`
- Reference: `docs/foundation/vision.md`
- Reference: `docs/foundation/world-model.md`
- Reference: `docs/foundation/glossary.md`
- Reference: `docs/product/product-charter.md`
- Reference: `docs/superpowers/specs/2026-06-12-product-master-spec-design.md`

- [ ] **Step 1: Create the master spec skeleton**

Create `docs/product/master-spec.md` with these sections:

```markdown
# Hermes Product Master Spec

## Status

## Canonical Product Definition

## Product Thesis

## What Hermes Is Not

## Communication As Primary Ingestion Spine

## Source Evidence To Memory Flow

## Domain Model

## Engine Model

## Current Implementation Inventory

## Target Gaps And Refactoring Direction

## Core Workflows

## Review, Confidence And Provenance

## Agent Behavior

## Documentation Expansion Map
```

- [ ] **Step 2: Write the canonical product definition**

State that Hermes is a local-first Personal Memory System and operating surface
for Communications, Knowledge, Memory, Relationships, Projects, Documents,
Decisions, Obligations and Context.

- [ ] **Step 3: Write the Communication ingestion spine section**

Include this exact conceptual flow:

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

Clarify that Communications are the primary ingestion spine, not the only source
of evidence.

- [ ] **Step 4: Write domain and engine sections**

Use the foundation domain model and include the new engine:

```text
Consistency / Contradiction Engine
User-facing alias: Polygraph
Purpose: detect contradictions between new evidence and accepted memory.
```

- [ ] **Step 5: Write current implementation inventory**

Summarize actual repository evidence from backend modules, migrations, routes
and frontend pages. Do not claim a target concept is implemented unless the
repository has matching implementation evidence.

- [ ] **Step 6: Write target gaps and refactoring direction**

List at least these gaps:

- Persona-native schema/API is not complete.
- First-class Relationship model is not complete.
- Consistency / Contradiction Engine is not implemented.
- Domain docs and engine docs are incomplete.
- Health/watchtower/fingerprint/investigator labels remain compatibility names.
- Mail-specific modules exist while the product model is Communications-first.

## Task 2: Create Development Roadmap

**Files:**
- Create: `docs/product/development-roadmap.md`
- Reference: `docs/product/master-spec.md`
- Reference: `docs/roadmap/product-roadmap.md`
- Reference: `docs/refactoring/documentation-audit.md`
- Reference: backend modules and migrations listed above.

- [ ] **Step 1: Create roadmap skeleton**

Create `docs/product/development-roadmap.md` with these sections:

```markdown
# Hermes Product Development Roadmap

## Status

## Roadmap Principle

## Current Implementation Baseline

## Target-State Gaps

## Slice 1: Communication Memory Spine

## Slice 2: Persona And Relationship Memory

## Slice 3: Knowledge And Polygraph

## Slice 4: Obligations, Tasks And Decisions

## Slice 5: Projects And Documents Context

## Slice 6: Agents Over Context

## Slice 7: Operating Surface

## Documentation Workstream

## Refactoring Plan Summary
```

- [ ] **Step 2: Write current implementation baseline**

Use repository evidence to distinguish implemented slices from target gaps.
Include Communications/email, Telegram/WhatsApp foundations, graph, documents,
projects, persons compatibility, organizations, calendar, tasks, AI runtime,
settings/vault and frontend surfaces.

- [ ] **Step 3: Write refactoring plan summary**

Include concrete refactoring/delivery items:

- migrate Persona terminology and route/schema compatibility under a dedicated plan;
- introduce first-class Relationships;
- add Owner Persona semantics;
- formalize Consistency / Contradiction Engine;
- normalize mail/email modules under Communications documentation;
- split domain-owned truth from engine-derived intelligence;
- create domain, engine and workflow specs in later waves.

## Task 3: Create Documentation Index

**Files:**
- Create: `docs/README.md`
- Reference: `README.md`
- Reference: `docs/foundation/`
- Reference: `docs/product/`
- Reference: `docs/adr/README.md`

- [ ] **Step 1: Create docs index skeleton**

Create `docs/README.md` with these sections:

```markdown
# Hermes Documentation

## Reading Order

## Canonical Sources

## Product Documents

## Foundation Documents

## Domain Documents

## Engine Documents

## Workflow Documents

## ADRs

## Implementation Status Documents

## Historical Documents
```

- [ ] **Step 2: Write reading order**

The reading order must start with:

1. `docs/product/master-spec.md`
2. `docs/foundation/vision.md`
3. `docs/foundation/glossary.md`
4. `docs/foundation/world-model.md`
5. `docs/product/development-roadmap.md`
6. `docs/foundation/domain-map.md`

- [ ] **Step 3: Explain historical vs canonical docs**

State that ADRs and implementation plans can contain older compatibility terms,
but active product/domain docs must follow foundation vocabulary.

## Task 4: Update Root README Links

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Add new product docs to "Главные документы"**

Add links for:

```markdown
- [Product Master Spec](docs/product/master-spec.md)
- [Product Development Roadmap](docs/product/development-roadmap.md)
- [Documentation Index](docs/README.md)
```

- [ ] **Step 2: Preserve runbook content**

Do not remove existing local run commands or implementation status notes.

## Task 5: Validate Documentation Changes

**Files:**
- Validate: `README.md`
- Validate: `docs/product/master-spec.md`
- Validate: `docs/product/development-roadmap.md`
- Validate: `docs/README.md`

- [ ] **Step 1: Run scoped diff check**

Run:

```sh
git diff --check -- README.md docs/product/master-spec.md docs/product/development-roadmap.md docs/README.md docs/superpowers/plans/2026-06-12-product-master-spec-docs.md
```

Expected: no output and exit code 0.

- [ ] **Step 2: Run markdown counts**

Run:

```sh
find docs/adr -maxdepth 1 -type f -name 'ADR-*.md' | wc -l
find docs -type f -name '*.md' | wc -l
```

Expected: commands complete and counts are reported.

- [ ] **Step 3: Run terminology search**

Run:

```sh
rg -n "\b(Contact|Contacts|contact|contacts|CRM|Address Book|address book|Email Client|email client|Task Tracker|task tracker|Calendar App|calendar app|Note Taking|note taking|Knowledge Base|knowledge base)\b" docs/product/master-spec.md docs/product/development-roadmap.md docs/README.md
```

Expected: matches are only in explicit "not Hermes" or compatibility/history
contexts.

- [ ] **Step 4: Confirm no code files were changed by this task**

Run:

```sh
git diff --name-only -- README.md docs/product/master-spec.md docs/product/development-roadmap.md docs/README.md docs/superpowers/plans/2026-06-12-product-master-spec-docs.md
```

Expected: only the planned documentation files are listed for this task.

## Self-Review Checklist

- [ ] The plan covers every Wave 1 deliverable from the approved design spec.
- [ ] The plan includes current implementation evidence, not only target-state language.
- [ ] The plan creates a refactoring/delivery plan for target-vs-current gaps.
- [ ] The plan does not ask for code, API, migration or route changes.
- [ ] The validation commands are exact and scoped.
