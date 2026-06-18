# Documentation Evolution Plan

Date: 2026-06-18

Scope: documentation plan only. This is not a code refactoring plan.

## Goal

Move Hermes documentation from scattered historical docs to a stable canonical
architecture baseline before major domain or provider refactoring.

## Phase 0: Freeze Refactoring Inputs

Status: started by this audit.

Tasks:

1. Keep code, APIs, migrations and ADR files unchanged.
2. Add canonical architecture docs under `docs/architecture/`.
3. Add audit reports under `docs/_audit/`.
4. Review conflicts with the owner before editing historical docs.

Exit criteria:

- canonical architecture docs exist;
- ADR audit report covers every ADR;
- domain ownership report is reviewed;
- Communications review and Radar RFC exist.

## Phase 1: Architecture Review

Tasks:

1. Review `docs/architecture/vision.md`.
2. Confirm product framing:
   `Personal Operating System built on Memory + Context`.
3. Review domain ownership table.
4. Decide whether Knowledge remains emergent or needs a future domain RFC.
5. Decide whether Radar remains workflow/read model or proceeds to Signal RFC.
6. Confirm Telegram and WhatsApp remain Communication Channels.

Exit criteria:

- owner-approved canonical wording;
- accepted conflict list;
- explicit open decisions for Knowledge, Radar and Observations.

## Phase 2: ADR Governance Cleanup

Tasks:

1. Update `docs/adr/README.md` status vocabulary after owner approval.
2. Apply status-only changes for uncontroversial ADR classifications.
3. Mark superseded chains consistently.
4. Create superseding ADRs for deprecated decisions:
   - ADR-0010 specialized named agent roster;
   - ADR-0030 documentation-first phase;
   - ADR-0035 old event API command boundary;
   - ADR-0050 Telegram V4 bundle.
5. Add notes to older ADRs whose auth details are superseded by ADR-0056.

Exit criteria:

- active ADR index matches actual governance;
- deprecated decisions have explicit replacement path;
- refactoring work can cite current ADRs without ambiguity.

## Phase 3: Documentation Restructure

Tasks:

1. Keep existing channel docs in place until redirects/indexes are ready.
2. Add Communications shared abstractions documentation.
3. Reframe `docs/mail`, `docs/telegram` and `docs/whatsapp` as channel specs.
4. Update active architecture overview references from SvelteKit to Vue 3.
5. Align `docs/domains/*` with the canonical ownership table.
6. Move historical status/gap docs only after links and traceability are clear.

Exit criteria:

- docs index points to canonical architecture first;
- channel docs are visibly subordinate to Communications;
- active docs no longer present superseded frontend/auth/platform facts as
  current.

## Phase 4: Open RFCs Before Implementation

Required RFCs before related code work:

| RFC | Purpose |
|---|---|
| Communications Shared Abstractions | Define ChannelAccount, Conversation, Message, Participant, Attachment, ProviderCommand, Capability and Realtime contracts. |
| Knowledge Item Ownership | Decide whether generic reviewed Knowledge Items need durable storage and who owns them. |
| Radar / Signal Taxonomy | Decide whether Signal is durable and how Radar aggregates/promotes candidates. |
| Observation Policy | Decide concrete observation ownership across Risk, Enrichment, Polygraph and Memory. |
| WhatsApp Implementation Slice | Convert target WhatsApp docs into safe implementation phases. |
| Persona Physical Migration | Plan any future `persons` to `personas` schema/API rename. |

Exit criteria:

- each RFC has owner-approved decision or explicit defer state;
- no new code domain starts from ambiguous ownership.

## Phase 5: Refactoring Design

Only after phases 1 to 4:

1. Design Communications refactor plan.
2. Design Telegram maintenance/deferred-initiative split.
3. Design WhatsApp first implementation slice.
4. Design any Persona/Knowledge/Radar implementation work.
5. Add validation gates before editing code.

## Risks

| Risk | Mitigation |
|---|---|
| Documentation churn hides architecture decisions. | Use RFC and ADR sequence before moving docs. |
| Historical docs lose traceability. | Mark historical status instead of deleting. |
| Channels become domains during restructure. | Keep Communications ownership table as gate. |
| Radar becomes too broad. | Require Signal taxonomy first. |
| Knowledge becomes a generic bucket. | Require Knowledge Item ownership RFC first. |
| ADR cleanup accidentally changes architecture. | Split status cleanup from decision changes. |

## Immediate Next Actions

1. Review this audit package.
2. Approve or amend the canonical architecture wording.
3. Decide whether ADR status cleanup can begin.
4. Write Communications Shared Abstractions RFC.
5. Write Signal taxonomy draft only if Radar remains desired after review.
