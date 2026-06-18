# Documentation Structure Proposal (2026-06-18)

This is a proposal only. It preserves historical traceability and avoids moving files before owner decisions.

## Proposed Shape

```text
docs/
├── README.md
├── _meta/
├── product/
├── foundation/
├── architecture/
├── adr/
├── domains/
├── engines/
├── workflows/
├── implementation/
├── research/
├── archive/
└── site/
```

## Placement Proposal

| Target path | Responsibility | Current state | Evidence | Confidence | Status |
| --- | --- | --- | --- | --- | --- |
| docs/README.md | Top-level navigation and source-of-truth map. | Already exists; update after decisions. | docs/README.md:20-45 | high | candidate |
| docs/_meta/ | Documentation governance, ADR status policy, templates. | Missing today. | Requested target structure; no current directory sampled. | high | candidate |
| docs/product/ | Product-level source of truth. | Already exists and is canonical. | docs/product/master-spec.md:3-13 | high | accepted |
| docs/foundation/ | Vocabulary, world model, engines, domain map, principles. | Already exists and canonical. | docs/README.md:33-45 | high | accepted |
| docs/architecture/ | System architecture, diagrams, security/privacy/storage/UI architecture. | Already exists; status metadata needed. | docs/architecture/* | medium | candidate |
| docs/adr/ | ADRs, index, template, status policy. | Exists; index needs repair. | docs/adr/README.md:3-98 | high | candidate |
| docs/domains/ | Canonical domain specs. | Exists; should absorb/mirror old top-level domain folders only after decision. | docs/domains/README.md:21-37 | high | candidate |
| docs/domains/communications/channels/email.md | Target home for email/mail channel docs. | Current docs/mail exists as implementation-heavy provider folder. | docs/mail/*; docs/foundation/glossary.md:13-18 | high | user-decision-needed |
| docs/domains/communications/channels/telegram.md | Target home for Telegram channel docs. | Current docs/telegram exists and is actively changing. | docs/telegram/*; ADR-0091; ADR-0094 | high | user-decision-needed |
| docs/domains/communications/channels/whatsapp.md | Target home for WhatsApp channel docs. | Current docs/whatsapp exists. | docs/whatsapp/* | high | user-decision-needed |
| docs/engines/ | Detailed engine specs. | Exists; implementation evidence stale. | docs/engines/README.md:38-60 | high | candidate |
| docs/workflows/ | Evidence-to-context workflows. | Exists and canonical. | docs/workflows/README.md:1-41 | high | accepted |
| docs/implementation/ | Current implementation status, API references, gap analyses. | Missing today; current status/API docs are scattered. | docs/mail/status.md, docs/calendar/api.md, docs/tasks/status.md, docs/persons/status.md | high | candidate |
| docs/research/ | Open questions and research notes. | Exists. | docs/research/open-questions.md | medium | accepted |
| docs/archive/ | Historical plans, roadmaps, reviews, superseded implementation docs. | Missing today; historical docs live under roadmap/reviews/superpowers/vision. | docs/README.md:160-170 | high | user-decision-needed |
| docs/site/ | Styled documentation portal or generated static site. | Exists with index/css/logo; owner decision whether generated artifact stays in docs. | docs/site/index.html; docs/site/hermes-docs.css; docs/site/assets/hermes-logo-mark.png | high | user-decision-needed |

## Two-Stage Migration

Stage 1: add governance/status/index fixes without moving files. This is low risk and should be the next PR after owner decisions.

Stage 2: move historical and implementation-heavy docs into `docs/archive/` and `docs/implementation/`, then optionally move channel docs under `docs/domains/communications/channels/`. This requires link updates and should be a separate PR.

## Domains Detail Proposal

```text
docs/domains/
├── communications/
│   ├── README.md
│   ├── model.md
│   ├── channels/
│   │   ├── email.md
│   │   ├── telegram.md
│   │   ├── whatsapp.md
│   │   └── calls-and-meetings.md
│   └── current-implementation.md
├── personas/
├── organizations/
├── relationships/
├── projects/
├── documents/
├── tasks/
├── calendar-events/
├── decisions/
├── obligations/
├── knowledge-graph/
├── agents/
└── notes-boundary.md
```

Constraint: keep compatibility names in implementation docs until a schema/API migration ADR exists.
