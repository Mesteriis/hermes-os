# Phase 2 Radar Evaluation (2026-06-18)

Scope: evaluate the possible Radar concept without creating a Radar domain,
moving documentation, changing ADRs or changing code.

Source concept from owner:

```text
Signal
↓
Radar
↓
Review
↓
Promote
↓
Task
Project
Persona
Organization
Document
Knowledge
```

Repository evidence: no active `Radar` concept was found outside the audit
package. Current docs use `signal` language for Trust, Risk, Timeline,
identity and attention outputs, but do not define a Radar bounded context.

## Radar Classification

| Question | Evaluation | Evidence | Confidence | Status |
| --- | --- | --- | --- | --- |
| Is Radar currently a domain? | No. Current docs do not define a durable Radar entity, lifecycle, invariants or source-of-truth responsibility. | docs/domains/README.md:15-19; `rg -i "\bradar\b" docs -g '!docs/_audit/**'` found no active Radar docs. | high | evaluated |
| Is Radar a workflow? | Yes, the proposed Signal -> Review -> Promote flow matches workflow semantics. Workflows coordinate domains and engines without owning durable entities. | docs/workflows/README.md:36-41; owner concept. | high | evaluated |
| Is Radar an inbox layer? | Likely yes. Radar is best modeled as a review/inbox surface over candidates and observations produced elsewhere. | docs/foundation/engines.md:14-18; docs/workflows/communication-to-knowledge.md:21-38 | medium | evaluated |
| Should Radar own Signal? | Not yet. `Signal` currently appears as Trust/Risk/Timeline/identity/attention output, not as a canonical entity. | docs/product/master-spec.md:164-168; docs/foundation/engines.md:14-18; docs/engines/risk.md | medium | open |
| Should Radar own Review? | No. Review state belongs to the concrete candidate/observation/domain record being reviewed. Radar may aggregate review items. | ADR-0086; ADR-0087; ADR-0088; ADR-0089 | high | evaluated |
| Should Radar own Promotion? | No. Promotion writes must be performed by the target owner domain: Tasks, Projects, Personas, Organizations, Documents, Decisions, Obligations or reviewed Knowledge/Memory policy. | docs/domains/README.md:15-19; docs/workflows/README.md:36-41 | high | evaluated |

## Candidate Radar-Owned Entities

| Candidate entity | Should Radar own it? | Rationale | Confidence | Status |
| --- | --- | --- | --- | --- |
| Radar Inbox Item | Maybe, only as a derived/rebuildable view over source candidates. | Useful UI aggregation, but not source truth. | medium | candidate |
| Signal | No current evidence. | Signals are already emitted by Trust, Risk, Timeline, identity and attention systems. | medium | rejected for now |
| Review Item | No. | Review state must remain with source candidate/observation or target domain record. | high | rejected |
| Promotion Command | No. | Promotion should dispatch to target domain command and audit trail. | high | rejected |
| Triage Queue | Maybe as read model. | A queue can be useful if rebuildable from candidate/observation state. | medium | candidate |

## What Radar Must Not Store

| Data | Reason | Evidence | Confidence | Status |
| --- | --- | --- | --- | --- |
| Persona truth | Personas domain owns identity and memory anchors. | docs/foundation/domain-map.md:10 | high | prohibited |
| Task lifecycle | Tasks domain owns Task lifecycle. | docs/foundation/domain-map.md:15 | high | prohibited |
| Decision truth | Decisions domain owns durable choices. | docs/domains/decisions.md | high | prohibited |
| Obligation truth | Obligations domain owns commitments. | docs/domains/obligations.md | high | prohibited |
| Relationship semantics | Relationships domain owns relationship records. | docs/domains/relationships.md | high | prohibited |
| Accepted Knowledge/Memory | Knowledge/Memory ownership remains unresolved; Radar should not become a shortcut owner. | docs/foundation/domain-map.md:52-54; 08-domain-ownership.md | high | prohibited |
| Source records | Provider/domain boundaries preserve source evidence. | docs/foundation/glossary.md:146-149 | high | prohibited |

## Radar Fit Analysis

| Model | Fit | Benefits | Risks | Verdict | Confidence |
| --- | --- | --- | --- | --- | --- |
| Radar as Domain | Weak today. | Could centralize triage. | Creates a competing owner for observations, tasks, knowledge and relationships. | Do not introduce now. | high |
| Radar as Workflow | Strong. | Encodes Signal -> Review -> Promote without stealing ownership. | Needs clear target-domain promotion commands. | Preferred model. | high |
| Radar as Inbox / Read Model | Strong if rebuildable. | Gives UI one place to review cross-domain candidates. | If made durable source truth, it becomes a second task/knowledge system. | Accept as derived/review surface. | high |
| Radar as Engine | Medium. | Could rank/cluster signals. | Overlaps Risk, Trust, Enrichment and Search engines. | Avoid until signal taxonomy exists. | medium |

## Recommendation

Treat Radar as a workflow plus derived inbox/read model for now.

Do not create a Radar domain until Hermes needs a durable `Signal` entity with:

- source-of-truth storage;
- lifecycle independent from existing candidates/observations;
- invariants that cannot live in Risk, Trust, Enrichment, Timeline,
  Consistency/Contradiction, Tasks, Decisions or Obligations;
- clear promotion commands into owning domains.

Next prerequisite: define Signal taxonomy. Without that, Radar risks becoming a
generic bucket for every unresolved candidate.
