# RFC: Radar As Memory Intake Layer

Date: 2026-06-18

Status: RFC, not accepted architecture.

Decision: do not create a Radar domain now.

## Summary

Radar is a useful concept if it means a Memory Intake Layer: a review surface
for source-backed signals, candidates and observations before promotion into
owning domains.

Radar is risky if it becomes a new source-of-truth domain. It would overlap
Tasks, Obligations, Decisions, Relationships, Knowledge, Memory and generic
Observation handling.

## Problem

Hermes will produce many reviewable items:

- task candidates;
- obligation candidates;
- decision candidates;
- relationship candidates;
- identity candidates;
- contradiction observations;
- risk observations;
- enrichment candidates;
- attachment safety findings;
- memory gaps;
- stale facts;
- project link suggestions.

These items are not all tasks. They need one operating surface for attention,
review and promotion.

## Proposed Flow

```text
External Sources
  -> Radar
  -> Review
  -> Promotion
  -> Persona
  -> Organization
  -> Task
  -> Project
  -> Document
  -> Decision
  -> Obligation
  -> Relationship
  -> Knowledge / Memory
```

## Goals

- Aggregate reviewable signals without stealing ownership.
- Show source evidence and confidence.
- Explain target domain and proposed promotion effect.
- Group duplicate or related items.
- Rank items by urgency, confidence, owner relevance and risk.
- Dispatch explicit promotion commands to owning domains.

## Non-Goals

- Create a generic task inbox.
- Create a generic knowledge table.
- Store accepted Memory.
- Own provider source evidence.
- Own review state for every candidate.
- Run hidden automation.
- Replace domain-specific review flows.

## Candidate Entities

| Entity | Recommended owner | Notes |
|---|---|---|
| Signal | Open | Do not create until taxonomy and lifecycle are defined. |
| Radar Inbox Item | Derived read model | May be rebuildable from candidates/observations. |
| Review Item | Existing candidate/observation owner | Radar may aggregate but should not own state. |
| Promotion Command | Target domain | Radar initiates; owning domain validates. |
| Ranking/Attention Score | Radar read model or Risk/Trust/Search engines | Must be derived and explainable. |

## Affected Domains And Engines

| Area | Impact |
|---|---|
| Communications | Main source of candidates and source evidence. |
| Personas | Identity and Persona memory candidates. |
| Organizations | Organization identity/enrichment candidates. |
| Documents | Attachment promotion and document extraction candidates. |
| Projects | Link suggestions and missing context. |
| Tasks | Task candidates and review. |
| Decisions | Decision candidates and review. |
| Obligations | Obligation candidates and review. |
| Relationships | Relationship candidates and review. |
| Memory Engine | Memory gaps, context and stale memory signals. |
| Risk Engine | Risk observations and attention signals. |
| Trust Engine | Source/relationship reliability signals. |
| Enrichment Engine | Enrichment candidates. |
| Consistency / Contradiction Engine | Contradiction observations. |

## Alternatives

| Alternative | Pros | Cons | Verdict |
|---|---|---|---|
| Radar as domain | Central source for all review items. | Creates competing ownership and god-domain risk. | Reject for now. |
| Radar as workflow | Clear review/promote flow without owning truth. | Needs good target-domain command contracts. | Preferred. |
| Radar as derived inbox | Unified owner-facing surface and ranking. | Must stay rebuildable and avoid stale duplicate state. | Preferred with constraints. |
| Radar as engine | Can rank and cluster signals. | Overlaps Risk, Trust, Search and Enrichment. | Defer until Signal taxonomy exists. |
| No Radar | Avoids new concept. | Review surfaces stay fragmented and task-like. | Not sufficient long-term. |

## Trade-Offs

Benefits:

- one attention surface for non-task signals;
- better review ergonomics;
- explicit promotion path into owning domains;
- less pressure to force everything into Tasks.

Risks:

- Radar becomes a second task tracker;
- Radar becomes a generic knowledge bucket;
- review state is duplicated;
- ownership of observations becomes unclear;
- promotion bypasses domain validation.

Mitigations:

- make Radar read-model first;
- keep review state with source candidates;
- require promotion commands to owning domains;
- show target owner and source evidence for every item;
- define Signal taxonomy before persistent Radar storage.

## Open Questions

1. Is `Signal` a durable entity or only a derived label?
2. Which engine owns attention/ranking?
3. Should Radar replace or aggregate the existing Review workspace?
4. How are duplicate candidates grouped across domains?
5. What is the retention policy for rejected signals?
6. Can Radar inbox items be fully rebuilt from domain candidates and events?
7. What promotion commands are required per target domain?

## Recommendation

Proceed with Radar as documentation/RFC only.

Next step: define a Signal taxonomy and review/promotion contract. Do not add
Radar tables, APIs or UI until that taxonomy proves a durable lifecycle that
cannot be represented by existing candidate and observation owners.
