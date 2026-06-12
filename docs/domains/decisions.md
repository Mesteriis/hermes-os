# Decisions Domain

Decisions are durable choices with rationale, evidence and consequences.

Hermes needs Decisions because the Personal Memory System must remember not only
what happened, but why a direction was chosen.

## Responsibilities

The Decisions domain owns:

- decision records;
- decision status;
- rationale;
- alternatives considered;
- evidence links;
- impacted entities;
- review and supersession history.

The Decisions domain does not own:

- generic notes;
- project lifecycle;
- task status;
- AI summaries;
- every preference or fact.

## Decision Sources

Decisions can come from:

- explicit owner entry;
- communication evidence;
- meetings and calls;
- documents;
- project reviews;
- agent suggestions that are confirmed by the owner.

AI can propose a decision candidate. A durable decision requires review or an
explicit policy that allows automatic capture for a narrow source.

## Decision Model

```yaml
Decision:
  id:
  title:
  status:
  rationale:
  alternatives:
  decided_by:
  decided_at:
  evidence:
  impacted_entities:
  supersedes:
  review_state:
```

## Current Implementation Evidence

There is no dedicated `backend/src/domains/decisions` module in the current
repository. Decisions currently appear as product/foundation concepts and may be
represented indirectly through graph links, project context, documents or
communications.

## Migration Plan

1. Keep Decisions as a target domain in documentation.
2. Add an ADR before introducing a dedicated persistence model or route group.
3. Define decision candidates before automatic decision capture.
4. Require evidence citations and review state.
5. Link Decisions to Projects, Communications, Documents, Events, Personas,
   Organizations, Tasks and Obligations through the graph.
