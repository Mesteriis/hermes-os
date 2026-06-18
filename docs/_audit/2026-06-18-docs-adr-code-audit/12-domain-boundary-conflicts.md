# Phase 2 Domain Boundary Conflicts (2026-06-18)

Scope: conflict register for semantic boundaries. This file does not move docs,
rename directories, edit ADRs, change code, delete docs, create domains or
close open questions.

## Boundary Questions

| Question | Audit Answer | Evidence | Confidence | Status |
| --- | --- | --- | --- | --- |
| Decision: Project artifact or Decision Domain? | Decision Domain. Projects may create/display/link Decisions, but do not own decision truth. | docs/domains/decisions.md; ADR-0089 | high | confirmed |
| Obligation: Task subtype or separate domain? | Separate Obligations domain. A Task may fulfill an Obligation, but Task is not commitment truth. | docs/domains/obligations.md; ADR-0088 | high | confirmed |
| Relationship: Personas or Graph? | Relationships domain owns semantics. Personas consume Persona relationships; Graph stores projection/traversal. | docs/domains/relationships.md; ADR-0086 | high | confirmed |
| Observation: Knowledge, Radar, Graph or separate domain? | None proven. Concrete observations belong to producing engine/domain until accepted; generic ownership remains open. | ADR-0087; docs/foundation/glossary.md:50-68 | medium | open |
| Timeline: Graph or separate mechanism? | Timeline Engine mechanism. Graph may help traversal, but Timeline is derived chronological view over events and dated records. | docs/engines/timeline.md; docs/foundation/glossary.md:157-161 | high | confirmed |
| Radar: domain, workflow or inbox layer? | Workflow plus derived inbox/read model for now. No current source supports a Radar domain. | 11-radar-evaluation.md | high | evaluated |

## Conflict Register

| Conflict | Current Drift | Architectural Risk | Recommended Next Step | Evidence | Confidence | Status |
| --- | --- | --- | --- | --- | --- | --- |
| Decision scattered across Projects, meetings and communications | Meeting outcomes and project link reviews can produce decision-shaped records. | Decisions become hidden workflow artifacts instead of durable memory. | Keep Decision Domain as owner; document adapters as sources only. | ADR-0089 | high | confirmed |
| Obligation collapsed into Task or Follow-Up | Task candidates, meeting outcomes and person promises contain commitment-shaped data. | Commitments disappear when task lifecycle changes. | Keep Obligations domain as owner; Tasks may link as fulfillment. | ADR-0088; docs/tasks/data-model.md:37-40 | high | confirmed |
| Relationship split between Personas, Organizations, Graph and link tables | Graph edges, person roles, organization contact links and task relations all carry relationship semantics. | Relationship meaning fragments and trust/strength scores duplicate. | Relationship domain owns semantics; Graph remains projection. | ADR-0086 | high | confirmed |
| Observation overloading | Contradiction, risk, enrichment, AI and memory observations share vocabulary but not one owner. | Generic observations table becomes a god-domain or JSON warehouse. | Keep concrete observation types; define observation policy before adding generic storage. | docs/foundation/glossary.md:50-68; ADR-0087 | high | open |
| Knowledge as domain versus emergent layer | Product docs mention Knowledge while owner decision says Knowledge is emergent. | Knowledge domain could duplicate Documents, Graph, Memory, Decisions and Observations. | Keep Knowledge emergent; clarify storage of reviewed facts/knowledge items. | docs/foundation/domain-map.md:52-54; owner decision 2026-06-18 | high | open |
| Timeline embedded in domains | Personas, Organizations, Projects and Calendar all expose timeline-like data. | Multiple source-of-truth timelines emerge. | Timeline Engine owns derived views; domains own dated source records. | docs/engines/timeline.md | high | confirmed |
| Radar temptation | Signal/review/promote flow is useful but lacks durable entity definition. | Radar becomes a second task inbox, knowledge queue and observation store. | Treat Radar as workflow/read model until Signal taxonomy exists. | 11-radar-evaluation.md | high | evaluated |
| Attachment split | Attachments start in Communications but may become Documents. | Attachments may be stored twice or lose scan/evidence state. | Keep communication attachment metadata as source evidence; promote to Document through explicit import/link workflow. | backend/migrations/0011_create_mail_blob_storage.sql; docs/domains/documents.md | medium | open |
| Meeting dual identity | Meetings are Calendar Events, Communications contexts and sources for Decisions/Obligations/Tasks. | Meeting docs could create a separate meeting domain by accident. | Calendar/Events owns scheduled meeting record; other domains own promoted outputs. | docs/domains/calendar-events.md; backend/migrations/0045_calendar_core_tables.sql | high | confirmed |
| Agent conclusions | Agents can propose observations/actions. | Agent outputs become uncited truth. | Agents own runs/proposals only; target domains own accepted mutations. | docs/domains/agents.md | high | confirmed |

## Domain Existence Conclusions

| Category | Domains / Concepts | Rationale | Confidence | Status |
| --- | --- | --- | --- | --- |
| Existing durable domains | Communications, Personas, Organizations, Relationships, Projects, Documents, Tasks, Calendar/Events, Decisions, Obligations, Knowledge Graph, Agents. | Each has or needs durable source-of-truth ownership. | high | confirmed |
| Not domains now | Email, Telegram, WhatsApp, Calls, Meetings, Notes, Knowledge, Radar, generic Observations, Timeline. | These are channels, document-like artifacts, emergent layers, workflows/read models, concrete engine outputs or engines. | high | evaluated |
| Missing domains | None proven by Phase 2. | Radar may become a domain only after Signal taxonomy and durable Signal lifecycle prove a domain need. | medium | open |
| High-risk unresolved boundaries | Observation, Knowledge Item, Memory Record, Radar/Signal, Risk Observation. | Ownership is still policy-level, not fully specified by source docs. | high | open |

## Gate Before Governance / Restructuring

Do not proceed to Documentation Governance, ADR Cleanup, Directory
Restructuring or Domain Refactoring until these Phase 2 outputs are reviewed:

1. `08-domain-ownership.md`
2. `09-lifecycle-mapping.md`
3. `10-cross-domain-relations.md`
4. `11-radar-evaluation.md`
5. `12-domain-boundary-conflicts.md`

The critical owner review points are Decision, Obligation, Relationship,
Observation and Radar.
