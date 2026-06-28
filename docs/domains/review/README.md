# Review Domain

Status: code-aligned documentation package created from ADR-0096 and current
backend modules.

Review is the durable inbox for material that needs owner triage, approval,
dismissal or promotion before it becomes accepted domain truth.

ADR source of truth:

- [ADR-0096 Canonical Evidence, Review Inbox and Context Packs](../../adr/ADR-0096-canonical-evidence-review-and-context-packs.md)

## Responsibilities

The Review domain owns:

- review inbox items;
- item lifecycle state;
- evidence links from review items to observations;
- target references for promotion results;
- review transition metadata.

It does not own:

- accepted Persona, Organization, Project, Task, Document, Decision,
  Obligation, Relationship or Knowledge truth;
- provider runtime state;
- Radar vocabulary;
- the concrete workflow that materializes promoted entities.

## Current Implementation Evidence

Current backend files:

- `backend/src/domains/review/mod.rs`;
- `backend/src/domains/review/models.rs`;
- `backend/src/domains/review/store.rs`;
- `backend/src/domains/review/service.rs`;
- `backend/src/workflows/review_promotion/mod.rs`.

The domain exports `ReviewInboxStore`, `ReviewInboxService`,
`ReviewItemKind`, `ReviewItemStatus`, `ReviewPromotionTarget` and evidence
records. Current item kinds include identity, project-link, contradiction,
task, obligation, decision, relationship, project and knowledge candidates.

The promotion materialization logic is currently implemented in
`backend/src/workflows/review_promotion`, not inside the Review domain. Review
keeps the inbox and transition state; the target domain owns the accepted
entity after promotion.

## Boundary Rule

Review may record that a candidate was promoted to a target domain/entity. It
must not silently mutate target-domain truth without the target domain command
or workflow boundary.

