# Architecture Conflicts

Date: 2026-06-18

Scope: conflict register for documentation and architecture. No code or ADR
files were changed.

## Conflict Register

| ID | Severity | Conflict | Evidence | Risk | Recommended resolution |
|---|---|---|---|---|---|
| C-001 | Major | ADR status vocabulary does not match actual governance. | Many active ADRs say `Status: Proposed`; `docs/adr/README.md` defines Proposed as accepted initial direction. | Refactoring work may ignore active decisions or treat drafts as accepted. | Owner review of ADR audit, then status-only ADR cleanup. |
| C-002 | Major | Product identity is split between "Personal Memory System" and current request's "Personal Operating System". | Product/foundation docs use Personal Memory System; user request defines Personal Operating System. | Inconsistent framing for UI/product decisions. | Use "Personal Operating System built on Memory + Context" as canonical framing; update product docs after approval. |
| C-003 | Major | Telegram/WhatsApp have separate frontend surfaces and docs, but are not domains. | `frontend/src/domains/telegram`, `frontend/src/domains/whatsapp`, `docs/telegram`, `docs/whatsapp`. | Channel code/docs may drift into standalone product ownership. | Document them as channel specs; do not move code until a Communications refactor plan exists. |
| C-004 | Major | Communications implementation remains mail-heavy. | backend `domains/mail/*`, docs mention compatibility naming. | Shared Communications abstractions may be duplicated in Telegram/WhatsApp. | Create Communications shared abstractions spec before code refactor. |
| C-005 | Major | Knowledge is named as domain/surface but ownership is unresolved. | `docs/domains/README.md`, frontend `knowledge`, graph docs. | Generic Knowledge silo could duplicate Documents, Graph, Memory, Relationships and Decisions. | Write Knowledge Item RFC before new Knowledge storage. |
| C-006 | Major | Radar concept lacks durable ownership. | No active Radar docs outside audit; user proposes Radar as Memory Intake Layer. | Radar could become a task tracker or observation warehouse. | Keep RFC-only until Signal taxonomy and promotion contracts are defined. |
| C-007 | Major | Generic Observation ownership is unclear. | Contradiction observations exist; risk/enrichment/memory observations are broader concepts. | A generic observations table could become a god-domain. | Keep concrete observation types; define observation policy before generic storage. |
| C-008 | Major | Older ADRs contain superseded auth details. | ADR-0035, ADR-0039, ADR-0045, ADR-0048, ADR-0049 reference bearer token or actor header. | New endpoints may copy obsolete auth. | Clarify all protected local APIs through ADR-0056. |
| C-009 | Major | Frontend architecture docs include stale Svelte/SvelteKit references. | `docs/architecture/architecture-overview.md`, ADR-0077 implementation notes, superseded ADRs. | New frontend work may follow old framework patterns. | Update active architecture docs to Vue 3 after canonical docs are accepted. |
| C-010 | Major | Relationship semantics can be confused with graph edges. | Graph docs and relationship docs both describe links. | Trust/strength/provenance could split across graph and relationship storage. | Keep Relationships as semantic owner; Graph as projection/traversal. |
| C-011 | Major | Obligations can be collapsed into Tasks. | Task candidates, meeting outcomes and follow-up language overlap. | Commitments could disappear when task lifecycle changes. | Keep Obligations as separate domain; Tasks may fulfill obligations. |
| C-012 | Major | Decisions can be hidden inside Projects or meetings. | Project link review and meeting outcomes produce decisions. | Rationale and decision provenance fragment. | Decisions domain owns durable choice; projects/meetings are sources. |
| C-013 | Medium | Notes frontend surface exists without domain acceptance. | `frontend/src/domains/notes`, `docs/domains/notes.md`. | Notes may become a shadow document/knowledge domain. | Treat Notes as document-like artifacts until ADR promotes them. |
| C-014 | Medium | Timeline appears as frontend surface and engine concept. | `frontend/src/domains/timeline`, engine docs. | Timeline could be mistaken for source truth. | Timeline remains engine/read model over dated records and events. |
| C-015 | Medium | Attachment ownership crosses Communications and Documents. | Mail blob storage, Telegram media, WhatsApp target media, document imports. | Bytes or scanner state may duplicate or lose provenance. | Shared attachment boundary; explicit Document promotion. |
| C-016 | Medium | WhatsApp docs are target-complete but implementation is mostly missing/blocked. | `docs/whatsapp/status.md`, `docs/whatsapp/modules.md`. | UI/API could expose unavailable capabilities. | Keep all live capabilities blocked until runtime, capability, outbox and audit exist. |
| C-017 | Medium | ADR-0050 Telegram V4 bundle conflicts with ADR-0094 scoping. | ADR-0050 includes broad Telegram, policy and calls; ADR-0094 marks many as planned. | Future work may reopen base Telegram indefinitely. | Treat ADR-0050 as deprecated/historical and rely on ADR-0094 for base scope. |
| C-018 | Medium | Agent model is split between named specialized agents and Persona-based agents. | ADR-0010, docs/domains/agents.md, ADR-0084. | Permissions and identity may fragment. | Supersede or evolve ADR-0010 with canonical Agents/Persona model. |

## Highest Priority Conflicts

1. ADR status governance.
2. Communications versus channel ownership.
3. Knowledge/Radar/Observation ownership.
4. Stale auth and frontend references.
5. Decision/Obligation/Relationship ownership protection.

## Refactoring Gate

Do not start Communications, Telegram, WhatsApp or domain code refactoring until:

- ADR audit classification is reviewed;
- Communications shared abstractions are accepted;
- Knowledge/Radar/Observation open questions are documented as RFCs;
- stale auth/frontend documentation is either corrected or explicitly marked
  historical.
