# ADR Audit Report

Date: 2026-06-18

Scope: audit classification only. No ADR files were edited.

## Status Vocabulary Used By This Audit

The repository ADR index currently uses `Proposed`, `Accepted`, `Temporary` and
`Superseded`. This audit uses the requested consolidation vocabulary:

- `Accepted`: current governing decision, either implemented or retained as
  active target architecture.
- `Superseded`: replaced by a later ADR or explicit supersession chain.
- `Deprecated`: historical or stale enough that it should not drive new work
  without a replacement ADR, even if the file is not formally superseded.
- `Draft`: useful direction, but incomplete, unvalidated or requiring evolution
  before it governs new implementation.
- `Rejected`: a rejected ADR document. No current ADR file is classified this
  way.

## Summary

| Audit classification | Count | Notes |
|---|---:|---|
| Accepted | 67 | Includes several files whose current file status is still `Proposed`. |
| Superseded | 13 | Supersession is already explicit in the ADR text or later ADR. |
| Deprecated | 4 | Useful historical context, but stale for new work. |
| Draft | 9 | Needs evolution before driving new implementation. |
| Rejected | 0 | Rejected alternatives exist inside ADRs, but no rejected ADR file exists. |

## Key Findings

1. The file-level status vocabulary is stale. Many active governing decisions
   still say `Status: Proposed`.
2. Several older ADRs contain superseded auth details such as bearer token and
   actor header requirements that conflict with ADR-0056.
3. Frontend docs still contain Svelte/SvelteKit language in historical ADRs and
   some architecture docs, while ADR-0093 and current files use Vue 3.
4. Telegram is now explicitly a completed base Communication Channel by
   ADR-0094. ADR-0050 and parts of ADR-0091 should be treated as historical or
   scoped by ADR-0094.
5. Knowledge, Radar, generic Observations and Signal taxonomy remain unresolved
   boundaries. They need RFC or ADR work before implementation.

## ADR Classification Table

| ADR | Current file status | Audit classification | Audit note |
|---|---|---|---|
| ADR-0001 Event Sourcing as System Spine | Proposed | Accepted | Core spine remains active and is implemented through event/audit/projection infrastructure. |
| ADR-0002 Rust Backend | Proposed | Accepted | Current backend crate is Rust. |
| ADR-0003 SvelteKit Frontend | Superseded by ADR-0093 | Superseded | Replaced by Vue 3 platform decision. |
| ADR-0004 Tauri Desktop Shell | Proposed | Accepted | Current desktop shell is Tauri. |
| ADR-0005 PostgreSQL Primary Store | Proposed | Accepted | Current migrations and local development use PostgreSQL. |
| ADR-0006 Tantivy Full Text Search | Proposed | Accepted | Full text search remains active target and implementation boundary. |
| ADR-0007 Replaceable Vector Search | Proposed | Accepted | Derived embedding/index principle remains active. |
| ADR-0008 Knowledge Graph First | Proposed | Accepted | Active principle, but relationship ownership is refined by ADR-0086. |
| ADR-0009 Local AI Through Ollama | Proposed | Accepted | Active default, amended by opt-in provider ADRs. |
| ADR-0010 Specialized Agent System | Proposed | Deprecated | Named agent roster is historical; new work should use canonical Agents, Persona and capability model first. |
| ADR-0011 Plugin Architecture | Proposed | Draft | Direction is useful, but runtime/sandbox contract is not current implementation truth. |
| ADR-0012 OpenTelemetry Observability | Proposed | Draft | Observability direction exists, but implementation/governance needs validation. |
| ADR-0013 Local First Data Ownership | Proposed | Accepted | Core product rule. |
| ADR-0014 Canonical Event Envelope | Proposed | Accepted | Active event metadata direction. |
| ADR-0015 Command Query Separation | Proposed | Accepted | Active application boundary principle. |
| ADR-0016 Secrets and Encryption Boundary | Superseded by ADR-0053 | Superseded | Replaced by later vault decisions; host vault now current. |
| ADR-0017 Document Processing Pipeline | Proposed | Accepted | Document processing pipeline remains active target and partially implemented. |
| ADR-0018 Provider Adapter Boundary | Proposed | Accepted | Core channel/integration boundary. |
| ADR-0019 Contact Identity Resolution | Superseded by ADR-0084 | Superseded | Replaced by Persona Intelligence framing. |
| ADR-0020 Task Candidate Lifecycle | Proposed | Accepted | Candidate/review lifecycle remains active. |
| ADR-0021 Calendar as Event Source | Proposed | Accepted | Calendar/Event source model remains active. |
| ADR-0022 No Fine Tuning on Private Data | Proposed | Accepted | Core privacy rule. |
| ADR-0023 Rebuildable Projections | Proposed | Accepted | Core derived-state rule. |
| ADR-0024 Idempotent Imports | Proposed | Accepted | Core provider/import rule. |
| ADR-0025 Keyboard First Command Palette | Proposed | Draft | UX direction remains plausible, but current Vue UI needs updated acceptance criteria. |
| ADR-0026 Desktop First Responsive UI | Proposed | Accepted | Current UI scope remains desktop/laptop. |
| ADR-0027 Capability Based Permission Model | Proposed | Accepted | Active policy direction, refined by ADR-0052. |
| ADR-0028 Backup and Restore as Core Feature | Proposed | Accepted | Core local-first operational requirement. |
| ADR-0029 Explicit Schema Evolution | Proposed | Accepted | Active migration/versioning rule. |
| ADR-0030 Documentation First Monorepo | Proposed | Deprecated | Historical foundation phase; project has entered implementation. |
| ADR-0031 Temporary Desktop Only UI Scope | Temporary | Accepted | Active temporary scope until superseded. |
| ADR-0032 Docker Compose Development Environment | Proposed | Accepted | Active local development rule. |
| ADR-0033 Backend Managed Local Schema Migrations | Proposed | Accepted | Active migration ownership rule. |
| ADR-0034 Event Replay and Projection Cursors | Proposed | Accepted | Active projection cursor model. |
| ADR-0035 Local Event API Command Boundary | Proposed | Deprecated | Concept is historical; auth/path details conflict with ADR-0056 and current router shape. |
| ADR-0036 Projection Runner Checkpoint Semantics | Proposed | Accepted | Active replay/checkpoint rule. |
| ADR-0037 Local Write Capability Token | Superseded by ADR-0056 | Superseded | Replaced by router-level shared secret. |
| ADR-0038 Local Event API Capability Token | Superseded by ADR-0056 | Superseded | Replaced by router-level shared secret. |
| ADR-0039 Local Event API Access Audit Log | Proposed | Accepted | Audit log remains active, but auth details must be read through ADR-0056. |
| ADR-0040 Local API Actor Identity | Superseded by ADR-0056 | Superseded | Replaced by constant `hermes-frontend` actor. |
| ADR-0041 Email Provider Ingestion Foundation | Proposed | Accepted | Active mail/channel ingestion foundation. |
| ADR-0042 Secret References for Provider Credentials | Superseded by ADR-0053 | Superseded | Replaced in chain by ADR-0053 and ADR-0076. |
| ADR-0043 Read-Only Email Provider Networking | Superseded by ADR-0055 | Superseded | Read-only restriction now retained only for automated tests. |
| ADR-0044 Account Setup and Encrypted Secret Vault | Superseded by ADR-0076 | Superseded | Host vault is current model. |
| ADR-0045 Graph Core Projection | Proposed | Accepted | Graph projection remains active, but contact/auth vocabulary is historical. |
| ADR-0046 Persistent Dev Mail Cache and Blob Storage | Proposed | Accepted | Active blob/attachment boundary; applies beyond email as shared communication storage pattern. |
| ADR-0047 Project Memory Spine | Proposed | Accepted | Active Project memory direction. |
| ADR-0048 Project Link Review Workflow | Proposed | Accepted | Active review workflow direction, but old auth language is historical. |
| ADR-0049 V3 Local AI Runtime and Retrieval | Proposed | Accepted | Active AI/RAG baseline, amended by ADR-0081/0082 and ADR-0056 auth. |
| ADR-0050 V4 Telegram Client, Policy Automation and Call Intelligence | Proposed | Deprecated | Historical Telegram bundle; current Telegram scope is governed by ADR-0083, ADR-0091 and ADR-0094. |
| ADR-0051 V5 WhatsApp Web Companion Boundary | Proposed | Accepted | Current WhatsApp-specific provider boundary. |
| ADR-0052 Capability Runtime and Action Confirmation Policy | Proposed | Accepted | Governs side effects, automation, provider writes and plugins. |
| ADR-0053 Database-Backed Encrypted Secret Vault | Superseded by ADR-0076 | Superseded | Legacy compatibility only. |
| ADR-0054 Application Settings Store | Proposed | Accepted | Active settings boundary. |
| ADR-0055 Full Email Provider Networking | Accepted | Accepted | Active email read/write policy. |
| ADR-0056 Local API Simplified Auth | Accepted | Accepted | Current local API auth model. |
| ADR-0057 Person Memory and Provenance System | Proposed | Draft | Needs Persona vocabulary alignment and Memory Engine boundary review. |
| ADR-0058 Person Enrichment Engine Boundary | Proposed | Draft | Needs Persona and shared Enrichment Engine alignment before new work. |
| ADR-0059 Person Communication DNA and Personas | Superseded by ADR-0084 | Superseded | Replaced by Persona Intelligence. |
| ADR-0060 Person Timeline and Graph Integration | Proposed | Draft | Needs Timeline Engine, Relationship domain and Persona vocabulary alignment. |
| ADR-0061 Organization as First-Class Domain Entity | Proposed | Accepted | Active Organizations domain direction. |
| ADR-0062 Organization Identity and Resolution | Proposed | Accepted | Active Organization identity direction. |
| ADR-0063 Organization Passive OSINT Boundary | Proposed | Draft | Needs updated enrichment/source policy before expanding implementation. |
| ADR-0064 Organization Memory and Provenance | Proposed | Accepted | Active, with future Memory Engine alignment needed. |
| ADR-0065 Organization Portals, Procedures, and Playbooks | Proposed | Draft | Useful direction, but playbooks/automation need capability and domain review. |
| ADR-0066 Organization Graph Integration | Proposed | Accepted | Active Organization graph direction. |
| ADR-0067 Calendar Multi-Provider Architecture | Proposed | Accepted | Active Calendar/Events direction. |
| ADR-0068 Calendar Event as Graph Node | Proposed | Accepted | Active event relationship direction. |
| ADR-0069 Calendar Intelligence Heuristic Fallbacks | Proposed | Accepted | Active deterministic intelligence direction. |
| ADR-0070 Tasks First-Class Domain | Proposed | Accepted | Active Tasks domain direction. |
| ADR-0071 Task Context Evidence Provenance | Proposed | Accepted | Active task evidence/context direction. |
| ADR-0072 Task Intelligence Heuristic Fallbacks | Proposed | Accepted | Active deterministic task intelligence direction. |
| ADR-0073 Backend Module Organization | Accepted | Accepted | Current backend module shape. |
| ADR-0074 Person Multi-Channel Identity Model | Accepted | Accepted | Active compatibility identity model, read through Persona framing. |
| ADR-0076 Host Vault on macOS | Accepted | Accepted | Current secret payload model. |
| ADR-0077 i18n Russian and English Interface | Accepted | Accepted | Active language policy, but implementation notes need Vue-era update. |
| ADR-0078 Frontend Component Decomposition | Superseded by ADR-0093 | Superseded | Svelte-specific decomposition replaced by Vue 3 architecture. |
| ADR-0079 Script Logic Decomposition | Superseded by ADR-0093 | Superseded | Svelte-specific service/store plan replaced by Vue 3 architecture. |
| ADR-0080 Mail Background Sync, Progress and Local Trash | Accepted | Accepted | Active mail channel sync/trash behavior. |
| ADR-0081 Opt-In OmniRoute AI Runtime | Proposed | Accepted | Active opt-in remote/routed provider policy. |
| ADR-0082 AI Settings Control Center | Proposed | Accepted | Active AI Control Center direction and partially implemented shape. |
| ADR-0083 Telegram Live User Client Runtime | Proposed | Accepted | Active Telegram user runtime foundation, scoped by ADR-0094. |
| ADR-0084 Persona Intelligence System | Proposed | Accepted | Active Persona model and supersession of Contact framing. |
| ADR-0085 Communication Spine and Consistency / Contradiction Engine | Proposed | Accepted | Active Communications/Polygraph architecture. |
| ADR-0086 First-Class Relationship Persistence | Proposed | Accepted | Active Relationship ownership model. |
| ADR-0087 Contradiction Observation Persistence | Proposed | Accepted | Active Polygraph persistence model. |
| ADR-0088 Obligation Persistence | Proposed | Accepted | Active Obligation domain model. |
| ADR-0089 Decision Persistence | Proposed | Accepted | Active Decision domain model. |
| ADR-0090 Persona-Native Compatibility API Bridge | Proposed | Accepted | Active bridge direction before physical rename. |
| ADR-0091 Telegram Production Client Capability Model | Proposed | Accepted | Active capability model, scoped and clarified by ADR-0094. |
| ADR-0092 Mail Provider Capability Tiers | Proposed | Draft | Useful provider direction, but non-current providers require future schema/tests. |
| ADR-0093 Frontend Platform Migration to Vue 3 | Accepted | Accepted | Current frontend platform decision. |
| ADR-0094 Telegram Base Domain Completion Boundary | Accepted | Accepted | Current Telegram base-channel completion boundary. |

## ADRs Requiring Evolution Before Refactoring

| Area | ADRs | Required evolution |
|---|---|---|
| ADR governance | README plus many `Proposed` ADRs | Normalize status vocabulary after owner review. |
| Frontend platform | ADR-0077 and stale docs | Update i18n implementation notes for Vue 3 without changing language policy. |
| Local API auth | ADR-0035, ADR-0039, ADR-0045, ADR-0048, ADR-0049 | Clarify all protected-route language through ADR-0056. |
| Telegram | ADR-0050, ADR-0083, ADR-0091, ADR-0094 | Mark V4 bundle historical and keep ADR-0094 as base scope guard. |
| Persona and memory | ADR-0057, ADR-0058, ADR-0060, ADR-0084, ADR-0090 | Align Person-era memory/enrichment/timeline decisions with Persona, Relationships and engines. |
| Knowledge/Radar/Signals | No accepted ADR | Create RFC first; ADR only after durable ownership is decided. |

## Recommended ADR Cleanup Sequence

1. Review this audit table with the owner.
2. Update `docs/adr/README.md` status vocabulary.
3. Create status-only ADR cleanup patches for files whose classification is
   uncontroversial.
4. Create superseding ADRs for deprecated but still useful historical decisions.
5. Only then design Communications, Telegram, WhatsApp or domain refactors.
