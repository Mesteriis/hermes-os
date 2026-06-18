# Open Decisions For Owner (2026-06-18)

These questions block semantic restructuring. The audit files may be reviewed now, but architecture/status/doc moves should wait for owner answers.

## Owner Answers Received

Owner answers were received in the audit thread on 2026-06-18. The original
question table below is preserved as the audit question log; the decisions in
this section supersede the corresponding `open` status.

| Decision area | Owner decision | Source / rationale | Confidence | Status |
| --- | --- | --- | --- | --- |
| Vue 3 canonicality | ADR-0093 is fully canonical. SvelteKit may appear only as Historical, Superseded or Migration context. Active docs must not require SvelteKit. | Owner response, 2026-06-18; ADR-0093 and current Vue implementation evidence in 03-doc-code-alignment.md. | high | decided |
| ADR statuses | Expand ADR statuses to Draft, Proposed, Accepted, Implemented, Needs Review, Superseded, Deprecated, Historical and Rejected. | Owner response, 2026-06-18. | high | decided |
| ADR-0075 | Keep the numbering gap. Do not create a stub ADR. Mark as Reserved / Missing in the index. | Owner response, 2026-06-18. | high | decided |
| Personas docs path | Active documentation should use Personas. Code-level `persons` may remain as a compatibility layer. | Owner response, 2026-06-18; ADR-0084 and ADR-0090 compatibility boundary. | high | decided |
| Communications channels | Move Email, Telegram, WhatsApp, Calls and Meetings under Communications channel documentation. They are not standalone domains. | Owner response, 2026-06-18; docs/foundation/glossary.md:13-18. | high | decided |
| Knowledge | Knowledge is not a standalone domain for now. Treat it as an emergent layer over Documents, Communications, Personas, Organizations, Projects, Decisions and Observations, stored through Graph, Memory and Evidence. | Owner response, 2026-06-18; docs/foundation/domain-map.md:52-54. | high | decided |
| Event taxonomy | Split Event vocabulary at least into Domain Event, Calendar Event and Timeline Event; consider Integration Event. | Owner response, 2026-06-18. | high | decided |
| Implementation docs placement | Implementation docs should live separately under `docs/implementation/`. Design ownership docs and implementation/current-state docs are separate document types. | Owner response, 2026-06-18. | high | decided |
| Historical plans | Move historical plans, reviews and closure roadmaps to `docs/archive/`. Preserve traceability. | Owner response, 2026-06-18. | high | decided |
| macOS metadata | Remove `.DS_Store` files under docs in a hygiene pass. | Owner response, 2026-06-18. | high | decided |
| Ownership audit | Before restructuring, create an ownership audit for `domains/*`, `engines/*` and `workflows/*` to identify bounded-context ownership of entities. | Owner response, 2026-06-18. | high | decided |

| Decision area | Question | Source / rationale | Evidence | Confidence | Status |
| --- | --- | --- | --- | --- | --- |
| Radar / Signals / Inbox | Do we create a separate signal-capture domain before Task/Project/Persona, or keep Radar/Signals/Inbox as workflow/research? | No canonical Radar domain in domain map; Communications is primary ingestion spine. | docs/foundation/domain-map.md:6-20; docs/workflows/README.md:8-23 | high | open |
| Knowledge | Is Knowledge a first-class domain/lifecycle, a layer over evidence, or only Knowledge Graph + reviewed memory items? | Product master lists Knowledge as a domain, domain map lists Knowledge Graph but not Knowledge as separate durable domain. | docs/product/master-spec.md:134-146; docs/foundation/domain-map.md:8-20,47-54 | high | open |
| Events taxonomy | Do we split Canonical Event Log, Domain Event, Calendar Event and Timeline Event into explicit terms? | Glossary currently defines Event broadly and distinguishes calendar/canonical events in one definition. | docs/foundation/glossary.md:70-74 | high | open |
| Personas docs path | Rename documentation path `docs/persons` to `docs/domains/personas` while leaving code `persons` compatibility? | ADR-0084/0090 require Persona language but preserve persons/person_id compatibility. | ADR-0084:123-126; ADR-0090:11-18; backend/src/app/router/routes/persons.rs:5-45 | high | open |
| Mail docs path | Move top-level `docs/mail` into `domains/communications/channels/email`, or keep as channel package? | Email is provider/channel shape, not product identity; docs/mail is implementation-heavy. | docs/foundation/glossary.md:13-18; docs/domains/README.md:54-62 | high | open |
| Telegram/WhatsApp docs path | Move Telegram/WhatsApp docs under `domains/communications/channels/*` or keep top-level capability specs? | Domain catalog treats Telegram as channel capability spec; ADR-0094 says Telegram is Communication Channel. | docs/domains/README.md:39-47; ADR-0094:29-35 | high | open |
| ADR statuses | Introduce Implemented, Needs Review, Historical and Deprecated statuses, or keep current Proposed/Accepted/Temporary/Superseded model? | Current status vocabulary is narrow and many Proposed ADRs have implementation evidence. | docs/adr/README.md:3-8; docs/adr/ADR-*.md status inventory | high | open |
| ADR-0075 | Create reserved ADR-0075 stub or keep numbering gap as historical trace? | ADR files span ADR-0001..ADR-0094 with ADR-0075 missing. | docs/adr file inventory | high | open |
| Vue 3 canonicality | Is ADR-0093 fully canonical for all active docs even if historical SvelteKit docs remain? | Current frontend is Vue 3; active README/CONTRIBUTING still mention SvelteKit. | ADR-0093:77-107; README.md:34,72,94; CONTRIBUTING.md:15-16; frontend/package.json:22,45 | high | open |
| Notes | Are Notes permanently document-like artifacts, or should a future ADR prepare a first-class Notes domain? | Glossary says Notes are not separate source of truth unless future ADR says otherwise; frontend has notes domain. | docs/foundation/glossary.md:94-98; frontend/src/domains/notes/ | high | open |
| Obligation Engine naming | Keep both Obligations Domain and Obligation Engine names, or rename engine to Commitment Detection Engine? | Domain and engine currently share root term but have separate ownership rules. | docs/foundation/domain-map.md:18; docs/foundation/engines.md:9-18 | medium | open |
| Implementation docs placement | Should implementation docs live next to domains or under separate `docs/implementation/`? | Status/API/gap docs are scattered under mail/calendar/tasks/persons/telegram/whatsapp. | 00-inventory.md status/API/gap doc inventory | high | open |
| Generated docs/site | Keep generated/static docs portal under `docs/site` or move to generated artifact outside docs? | docs/site has HTML/CSS/logo assets. | docs/site/index.html; docs/site/hermes-docs.css; docs/site/assets/hermes-logo-mark.png | medium | open |
| Historical plans | Move superpowers plans, roadmap closure docs and reviews into `docs/archive/`? | docs/README already treats these as traceability records, not current model. | docs/README.md:160-170; docs/superpowers/*; docs/roadmap/*; docs/reviews/* | high | open |

## Immediate Recommendation

Run the ownership audit before restructuring, moving or renaming documentation.
The next artifact is `08-domain-ownership.md`. It should define entity owners,
consumers, source-of-truth boundaries, lifecycle, creation rules and deletion
rules, and record ownership gaps before file moves begin.
