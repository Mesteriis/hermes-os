# Canonical Radar Position

Status: Candidate architecture position for the 2026-06-18 documentation
consolidation.

Decision status: Radar is not accepted as a domain today.

Scope: architecture analysis only. This document does not create a Radar domain,
tables, APIs, routes or UI work.

## Purpose

Radar is the proposed name for a Memory Intake Layer: a place where incoming
signals, candidates and observations can be reviewed before promotion into
owning domains.

Proposed flow:

```text
External Sources
  -> Radar
  -> Review
  -> Promotion
  -> Persona / Organization / Task / Project / Document / Knowledge
```

## Current Classification

Radar should be treated as:

- a workflow;
- a derived inbox/read model;
- a review and triage surface;
- a ranking/grouping layer over source-backed candidates.

Radar should not be treated as:

- a durable source-of-truth domain;
- a replacement for Tasks;
- a generic Knowledge store;
- an Observation warehouse;
- a hidden automation engine.

## Responsibility

Radar may eventually be responsible for:

- aggregating reviewable candidates from domains and engines;
- ranking signals by urgency, confidence and relevance;
- grouping duplicate or related signals;
- showing source evidence and proposed promotion targets;
- dispatching explicit promotion commands to owning domains;
- keeping review ergonomics consistent across the system.

## Boundaries

Radar must not own:

- Persona identity;
- Organization identity;
- Task lifecycle;
- Project lifecycle;
- Document versions;
- Decision truth;
- Obligation truth;
- Relationship semantics;
- accepted Memory or Knowledge;
- source provider records.

Review state should remain with the concrete candidate or observation unless a
future ADR defines a rebuildable Radar inbox read model.

Promotion must call the owning domain:

- task candidate -> Tasks;
- obligation candidate -> Obligations;
- decision candidate -> Decisions;
- identity trace -> Personas;
- organization identity trace -> Organizations;
- attachment import -> Documents;
- relationship candidate -> Relationships;
- contradiction observation -> Consistency / Contradiction review workflow.

## Connections

Radar would consume outputs from:

- Communications extraction;
- Documents extraction;
- Calendar/meeting outcomes;
- Enrichment Engine;
- Risk Engine;
- Trust Engine;
- Obligation Engine;
- Decision candidate engine;
- Consistency / Contradiction Engine;
- Search and Memory gap detection.

## Reasons For Existence

Radar may be valuable because Hermes needs a single owner-facing review surface
for "things worth attention" without collapsing them into Tasks. Many important
signals are not tasks:

- identity conflicts;
- stale memory;
- contradictory evidence;
- risky provider attachments;
- relationship suggestions;
- unresolved obligations;
- missing project context;
- document evidence waiting for classification.

The risk is that Radar becomes a second task tracker, second knowledge base and
second observation store. Until a durable `Signal` entity and lifecycle are
proven, Radar remains a workflow/read-model concept.

## Promotion Gate

Before Radar becomes a domain, a future RFC/ADR must define:

- whether `Signal` is a durable entity;
- signal taxonomy and lifecycle;
- source evidence model;
- review state ownership;
- promotion command contracts;
- rebuildability rules;
- API and UI boundaries;
- interaction with existing Review workspace.
