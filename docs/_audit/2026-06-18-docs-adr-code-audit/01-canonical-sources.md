# Canonical Sources Audit (2026-06-18)

This file records active source-of-truth conclusions from the required canonical reads. Every conclusion row includes source, evidence, confidence and status.

## Active Product Model

| Conclusion | Source | Evidence | Confidence | Status |
| --- | --- | --- | --- | --- |
| Product model | Hermes Hub is a local-first Personal Memory System; context is primary value. | docs/product/master-spec.md:15-32; docs/README.md:6-13 | high | accepted |
| Product cycle | Communication -> Source Evidence -> Extracted Knowledge -> Memory -> Relationships -> Context -> Obligations / Tasks / Decisions / Projects -> Timeline / Dossier / Recall. | docs/product/master-spec.md:34-49 | high | accepted |
| Non-identity | Not email client, messenger, CRM, address book, task tracker, calendar app, note-taking app, generic knowledge base, AI chatbot. | docs/product/master-spec.md:61-76; docs/foundation/vision.md:35-51 | high | accepted |
| Source evidence rule | Raw records and local artifacts are evidence; AI output is derived until accepted; actions must cite sources. | docs/product/master-spec.md:105-127 | high | accepted |
| Domain boundary | Domains own durable truth; engines produce derived intelligence; agents operate over context. | docs/product/master-spec.md:148-154 | high | accepted |
| Glossary authority | Glossary is canonical vocabulary for active docs; historical docs may contain older terms. | docs/foundation/glossary.md:1-5 | high | accepted |
| Provider/channel rule | Email, Telegram and WhatsApp are provider/channel shapes, not separate product identities. | docs/foundation/glossary.md:13-18 | high | accepted |
| Notes boundary | Notes are document-like artifacts unless a future ADR creates a separate Notes domain. | docs/foundation/glossary.md:94-98; docs/foundation/domain-map.md:47-54 | high | accepted |
| Persona vocabulary | Persona is not contact/address-book/CRM; Owner Persona is single is_self true. | docs/foundation/glossary.md:111-121; ADR-0084:52-70 | high | accepted |
| Engine boundary | Engines may produce projections/candidates/observations but must not own domain truth. | docs/engines/README.md:8-23 | high | accepted |
| Workflow boundary | Workflows coordinate domains and engines; durable state is written by owner domain or reviewed engine observation. | docs/workflows/README.md:37-41 | high | accepted |

## Active Domains

| Domain | Owns / role | Evidence | Confidence | Status |
| --- | --- | --- | --- | --- |
| Personas | Personas, identity traces, Persona relationships, Persona memory anchors | docs/foundation/domain-map.md:8-20 | high | accepted |
| Organizations | Organizations, organization identities, organization relationships, portals, procedures | docs/foundation/domain-map.md:8-20 | high | accepted |
| Communications | Canonical messages, conversations, participants, channel metadata, delivery state | docs/foundation/domain-map.md:8-20 | high | accepted |
| Projects | Bounded work contexts, project state, project decisions and linked context | docs/foundation/domain-map.md:8-20 | high | accepted |
| Documents | Document artifacts, versions, extracted text, metadata, document evidence | docs/foundation/domain-map.md:8-20 | high | accepted |
| Tasks | Actionable work items, status lifecycle, task evidence, task provider overlay | docs/foundation/domain-map.md:8-20 | high | accepted |
| Calendar/Events | Scheduled events, meetings, attendees, calendar source identity | docs/foundation/domain-map.md:8-20 | high | accepted |
| Decisions | Durable choices and rationale with evidence | docs/foundation/domain-map.md:8-20 | high | accepted |
| Obligations | Commitments and duties with evidence | docs/foundation/domain-map.md:8-20 | high | accepted |
| Knowledge Graph | Relationship records, graph evidence, traversal model | docs/foundation/domain-map.md:8-20 | high | accepted |
| Agents | Tool-mediated workflows and audit trails | docs/foundation/domain-map.md:8-20 | high | accepted |

## Active Engines

| Engine | Outputs / role | Evidence | Confidence | Status |
| --- | --- | --- | --- | --- |
| Memory Engine | memory records, context summaries, memory gaps | docs/foundation/engines.md:9-18 | high | accepted |
| Timeline Engine | timelines, diffs, period summaries | docs/foundation/engines.md:9-18 | high | accepted |
| Trust Engine | trust signals, confidence adjustments | docs/foundation/engines.md:9-18 | high | accepted |
| Search Engine | ranked results, snippets, query plans | docs/foundation/engines.md:9-18 | high | accepted |
| Enrichment Engine | enrichment candidates, conflicts, observations | docs/foundation/engines.md:9-18 | high | accepted |
| Obligation Engine | obligations, follow-ups, task candidates | docs/foundation/engines.md:9-18 | high | accepted |
| Risk Engine | risk observations, attention signals | docs/foundation/engines.md:9-18 | high | accepted |
| Consistency / Contradiction Engine (Polygraph) | contradiction observations, stale fact warnings, review items | docs/foundation/engines.md:9-18 | high | accepted |

## Active Workflows

| Workflow | Product output | Evidence | Confidence | Status |
| --- | --- | --- | --- | --- |
| Communication to Knowledge | Source-backed knowledge candidates | docs/workflows/README.md:25-35 | high | accepted |
| Communication to Obligation | Obligation candidates and follow-up/task suggestions | docs/workflows/README.md:25-35 | high | accepted |
| Meeting to Decisions | Decisions, obligations, tasks and timeline events from meetings | docs/workflows/README.md:25-35 | high | accepted |
| Document to Context | Document evidence linked to context | docs/workflows/README.md:25-35 | high | accepted |
| Contradiction Review | Reviewable conflict observations | docs/workflows/README.md:25-35 | high | accepted |
| Dossier Generation | Derived, cited dossiers | docs/workflows/README.md:25-35 | high | accepted |
| Agent Assisted Recall | Source-backed answers with uncertainty | docs/workflows/README.md:25-35 | high | accepted |

## Forbidden Or Compatibility-Only Terms

| Term | Rule | Evidence | Confidence | Status |
| --- | --- | --- | --- | --- |
| Email Client / Mail Domain as product identity | Forbidden as product framing; email is a Communications channel/provider shape. | docs/product/master-spec.md:61-76; docs/foundation/glossary.md:13-18 | high | accepted |
| CRM / Address book / Contact Manager | Forbidden as product framing; people are Personas. | docs/product/master-spec.md:61-76; docs/foundation/glossary.md:116-121 | high | accepted |
| Task tracker | Forbidden as product framing; Tasks are a domain surface over evidence/obligations. | docs/product/master-spec.md:61-76; docs/foundation/glossary.md:151-155 | high | accepted |
| Calendar app | Forbidden as product framing; Calendar/Events is scheduled event/evidence domain. | docs/product/master-spec.md:61-76; docs/foundation/domain-map.md:16 | high | accepted |
| Note-taking app / first-class Notes domain | Forbidden without future ADR; Notes are document-like artifacts. | docs/foundation/glossary.md:94-98 | high | accepted |
| Generic Knowledge Base | Forbidden as product framing; Knowledge is evidence-backed understanding, not loose wiki. | docs/product/master-spec.md:61-76; docs/foundation/glossary.md:82-86 | high | accepted |

## Source-Of-Truth Eligibility

| Source class | Authority rule | Evidence | Confidence | Status |
| --- | --- | --- | --- | --- |
| Highest local process source | Current request and AGENTS.md govern agent behavior. | AGENTS.md:20-35 | high | accepted |
| Canonical product/foundation docs | Product Master Spec, foundation docs, domain/engine/workflow catalogs. | AGENTS.md:27-30; docs/README.md:33-45 | high | accepted |
| ADRs | Durable architectural decisions; newer ADR supersedes older one. | docs/README.md:122-128 | high | accepted |
| Implementation files | Evidence for what exists today, not target product vocabulary. | AGENTS.md:31-32; docs/product/master-spec.md:7-13 | high | accepted |

## Canonical Authority Set

- `docs/product/master-spec.md` is product-level source of truth, but not API/schema/runtime detail authority (`docs/product/master-spec.md:3-13`).
- `docs/foundation/*` defines active vocabulary, world model, engine map and architecture principles (`docs/README.md:33-45`).
- `docs/domains/README.md`, `docs/engines/README.md` and `docs/workflows/README.md` are canonical catalog entry points (`docs/domains/README.md:1-19`, `docs/engines/README.md:1-23`, `docs/workflows/README.md:1-6`).
- `docs/adr/ADR-*.md` are architectural decisions; superseded ADRs remain historical traceability records (`docs/README.md:122-128`).
- Current code and migrations are evidence of implemented state, not authority to rename product concepts without ADR review (`AGENTS.md:31-35`).
