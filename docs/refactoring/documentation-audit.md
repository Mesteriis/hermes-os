# Hermes Documentation Audit

Date: 2026-06-12

Scope: documentation-only consolidation. No code, schema, API or runtime
implementation changes are included in this refactoring.

## Executive Summary

Hermes documentation now uses one foundation model:

```text
Hermes is a local-first Personal Memory System.
```

Hermes provides an operating surface for communications, knowledge, memory,
relationships, projects, documents, decisions, obligations and context. Its
primary value is context, not CRUD over separate apps.

The previous documentation mixed several mental models:

- email client;
- CRM;
- contact manager;
- task tracker;
- calendar app;
- note-taking app;
- generic knowledge base;
- AI assistant.

The consolidated model treats these as source channels, domain surfaces or UI
workflows over one memory system.

## Sources Reviewed

Foundation and product docs:

- `docs/vision/vision-document.md`
- `docs/product/product-charter.md`
- `docs/product/product-scope.md`
- `docs/architecture/architecture-overview.md`
- `docs/architecture/domain-map.md`

Domain docs:

- `docs/persons/`
- `docs/organizations/`
- `docs/domains/`
- `docs/mail/`
- `docs/calendar/`
- `docs/tasks/`
- `docs/agents/`
- `docs/ui/`

Relevant ADR context:

- ADR-0001 Event Sourcing as System Spine
- ADR-0008 Knowledge Graph First
- ADR-0041 Email Provider Ingestion Foundation
- ADR-0061 Organization as First-Class Domain Entity
- ADR-0067 Calendar Domain
- ADR-0070 Task Operating System
- ADR-0071 Projects as First-Class Domain
- ADR-0076 Host Vault on macOS
- ADR-0084 Persona Intelligence System
- ADR-0085 Communication Spine and Consistency / Contradiction Engine

## Canonical Model

Primary domain entities:

- Persona
- Organization
- Communication
- Project
- Document
- Task
- Event
- Decision
- Obligation
- Relationship
- Knowledge item
- Source record

Shared engines:

- Memory Engine
- Timeline Engine
- Trust Engine
- Search Engine
- Enrichment Engine
- Obligation Engine
- Risk Engine
- Consistency / Contradiction Engine

Engines are not domains. Domains own durable entities. Engines consume domain
events, source records and graph links to build derived views, scores,
candidates and context.

## Domain vs Engine Audit

| Area | Previous ambiguity | Canonical boundary |
|---|---|---|
| Personas | Contact/profile/CRM record with local timeline, health and watchlist fields. | Persona is a memory anchor. Relationships are first-class. Timeline, Risk and Trust outputs are shared engine projections. |
| Organizations | CRM account, company profile or project container. | Organization is a first-class entity for institutions, companies, agencies, communities and services. |
| Projects | Sometimes mixed with organizations, tasks and document folders. | Project is an ongoing effort or initiative that coordinates work, knowledge, communications, documents and decisions. |
| Tasks | Task, promise, follow-up and obligation were partially interchangeable. | Task is actionable work. Obligation is a commitment or duty. Follow-Up is a prompt to continue interaction. |
| Documents | Document, note and knowledge were blurred. | Document is a source artifact. Knowledge is extracted or curated understanding with provenance. |
| Notes | Treated as a possible independent app/domain. | Note is a lightweight document-like artifact or memory source unless a future ADR promotes it. |
| Knowledge | Could read as a wiki or standalone knowledge base. | Knowledge is an evidence-backed layer over source records, domains and graph links. |
| Communications | Email-centric documentation dominated the model. | Email, Telegram, WhatsApp, calls and meetings are Communication channels. |
| Timeline | Repeated inside Personas, Projects, Documents and Organizations. | Timeline Engine builds timeline views across domains. |
| Trust | Stored as profile/relationship health fields. | Trust Engine derives signals; durable trust belongs to explicit Relationship records when accepted. |

## Removed Concepts

Removed from canonical domain semantics:

- Hermes as Email Client.
- Hermes as CRM.
- Hermes as Address Book or Contact Manager.
- Hermes as Task Tracker.
- Hermes as Calendar App.
- Hermes as Note Taking App.
- Hermes as generic Knowledge Base.
- Contact as a target domain entity.
- CRM contact profile as a target model.
- Address-book entry as a target model.
- Favorite/watchlist as Persona identity.
- Relationship health as a root Persona or Organization field.
- Per-domain timeline subsystems.
- Email mailbox health as a domain model.
- AI output as source of truth.

These terms may still appear in historical ADRs, compatibility route references
or implementation status documents. They must not be used as new target-domain
terminology.

## Renamed Concepts

| Previous term | Canonical term |
|---|---|
| Person, Contact, CRM contact | Persona |
| Self profile, User profile | Owner Persona / Self Persona |
| Contact Identity Resolution | Persona Identity Resolution |
| Company | Organization |
| Promise, commitment candidate | Obligation, when evidence-backed |
| Follow-up task | Follow-Up candidate or Task, depending on intent |
| Email/Mail domain as product identity | Communications domain, email channel |
| Meeting/call transcript silo | Communication source record plus Document evidence |
| Communication fingerprint | Persona communication patterns |
| Communication profile | Persona Intelligence |
| Investigator | Dossier assembly and context preparation |
| Health/watchtower | Risk or attention read model |
| Brain route | Context explanation or agent interaction surface |

## Merged Concepts

- Email, Telegram, WhatsApp, calls and meetings are merged under
  Communications as interaction records with channel-specific source metadata.
- Contact, person and profile language is merged into Persona.
- Self/UserProfile language is merged into the single Owner Persona with
  `is_self = true`.
- Fingerprint, communication profile, trust analytics and investigator wording
  is merged into Persona Intelligence.
- Multiple local timeline concepts are merged into the shared Timeline Engine.
- Notes, document extracts and source summaries are treated as evidence feeding
  Memory and Knowledge instead of separate knowledge stores.
- Promise, follow-up and task-candidate language is separated through the
  Obligation/Task/Follow-Up boundary.

## New Concepts

Added or made explicit:

- Foundation documentation under `docs/foundation/`.
- Product master spec and product roadmap under `docs/product/`.
- Documentation index under `docs/README.md`.
- Domain catalog under `docs/domains/`.
- Engine catalog under `docs/engines/`.
- Workflow catalog under `docs/workflows/`.
- Implementation alignment plan under `docs/refactoring/`.
- Completion audit under `docs/refactoring/`.
- Canonical glossary with one meaning per term.
- World model with primary and derived entities.
- Engine map separating reusable mechanisms from domains.
- Architecture principles for local-first, evidence-backed, event-sourced
  memory.
- Domain map for active domains and intentionally non-domain concepts.
- Persona as the root people model.
- PersonaType: `human`, `ai_agent`, `organization_proxy`, `system`.
- Owner Persona as the only `Persona.is_self = true`.
- Relationship as a first-class entity.
- Dossier as an automatically generated, cited read model.
- Memory, Timeline, Trust, Search, Enrichment, Obligation and Risk as engines.
- Consistency / Contradiction Engine as the source-backed conflict detection
  engine, with Polygraph as the user-facing alias.
- Communication-to-knowledge, communication-to-obligation,
  meeting-to-decisions, document-to-context, contradiction-review,
  dossier-generation and agent-assisted-recall workflows.
- Implementation alignment plan mapping current routes, modules, migrations and
  frontend surfaces to target refactoring slices.
- Architecture context/container diagrams aligned to the Personal Memory System
  model with Communications as source evidence and shared Engines as derived
  mechanisms.
- Security model aligned to ADR-0056 and the current router-level
  `X-Hermes-Secret` guard.
- Primary vs derived state distinction.

## Remaining Inconsistencies

Known inconsistencies that remain intentionally visible:

- Historical ADRs still contain Contact, Person, Person Memory, Person
  Timeline and Person Communication DNA terminology. ADR-0084 establishes the
  new Persona model, but older ADR titles were not rewritten.
- Historical execution plans under `docs/superpowers/` still contain older
  contact/person/project-plan terminology. They are implementation history, not
  the current domain model.
- Historical reviews under `docs/reviews/` and version closure files under
  `docs/roadmap/` are traceability records unless referenced by a current
  canonical document.
- Current implementation route and table names still include `persons`,
  `person_id`, `/health`, `/watchtower`, `/follow-up` and similar compatibility
  labels. `contacts` remains in historical migrations and legacy command names
  such as `backend-contacts-smoke-dev`, but the active identity route is
  `/api/v1/persons/{person_id}/identity`.
- API reference documents document existing routes only. They do not define the
  target API and should not be used as a new API design.
- `docs/refactoring/implementation-alignment-plan.md` records current
  code-vs-target gaps. It is not an implementation authorization by itself.
- `docs/architecture/refactoring-plan.md` remains an implementation
  decomposition note and still includes code snippets with old `Person` names.
- Notes are not promoted to a full domain. If Hermes later needs a first-class
  Notes domain, that requires a separate ADR.
- Knowledge is documented as an evidence-backed layer. Whether it becomes a
  stricter standalone domain with its own lifecycle is still an architectural
  decision.
- No schema, migration or API rename was performed in this documentation pass.

## Developer Orientation

A new developer should now answer the following from documentation alone:

| Question | Canonical answer |
|---|---|
| What is Hermes? | A local-first Personal Memory System and operating surface for context. |
| What is Persona? | A durable representation of a subject with identity, relationships, communication evidence, memory anchors, context and dossier views. |
| What is Memory? | Evidence-backed remembered context produced from source records, events, relationships and reviewed facts. |
| What is Dossier? | A generated, cited read model summarizing what Hermes knows about a Persona or comparable entity. |
| What is Communication? | A recorded interaction across channels such as email, Telegram, WhatsApp, calls and meetings. |
| What is Project? | An ongoing effort that coordinates work, communications, documents, decisions, tasks and obligations. |
| What is Organization? | A first-class entity for companies, agencies, institutions, communities and services. |
| Domain vs Engine? | Domains own durable entities; engines build reusable derived views, scores, candidates and context. |
| Primary entities? | Personas, Organizations, Communications, Projects, Documents, Tasks, Events, Decisions, Obligations, Relationships, Knowledge items and Source records. |
| Derived entities? | Timelines, dossiers, context packs, search indexes, embeddings, scores, AI observations and attention/risk views. |
