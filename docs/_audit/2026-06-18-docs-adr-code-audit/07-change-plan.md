# Change Plan (2026-06-18)

This plan intentionally stops before architecture rewrites, file moves, code changes, route/table renames or ADR status mass edits.

## Phase 0: Completed In This Audit Pass

- Created first-pass audit artifacts under `docs/_audit/2026-06-18-docs-adr-code-audit/`.
- Inventoried Markdown docs, ADRs, status/API/gap docs, historical plans, generated assets and large docs.
- Recorded canonical source model, ADR drift, doc/code mismatches, vocabulary conflicts, proposed structure and owner decisions.
- Recorded owner answers for ADR status, Vue 3 canonicality, ADR-0075, Personas vocabulary, Communications channels, Knowledge, Event taxonomy, implementation docs, historical archive placement and `.DS_Store` cleanup.
- Added the ownership audit prerequisite before any semantic restructuring.

## Proposed Next Changes

| Step | Change | Details | Target files | Risk / blocker | Confidence | Status |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Owner answers open decisions | Record owner decisions in the audit package. | 06-open-decisions.md | Completed from owner response. | high | completed |
| 2 | Add domain ownership audit | Define entity owners, consumers, source-of-truth boundaries, lifecycle, creation rules, deletion rules and ownership gaps before restructuring. | 08-domain-ownership.md | Docs-only; must not move files or rename code. | high | completed |
| 3 | Add docs/_meta/documentation-governance.md | Define document types, status block, compatibility callout, source/evidence/confidence/status rule. | New docs/_meta/documentation-governance.md | Docs-only. | high | pending |
| 4 | Add docs/_meta/adr-status-policy.md | Define Draft/Proposed/Accepted/Implemented/Superseded/Deprecated/Historical/Rejected/Needs Review. | New docs/_meta/adr-status-policy.md | Docs-only; owner decision now available. | high | pending |
| 5 | Fix docs/adr/README.md | Regenerate sorted ADR index, include ADR-0056/0071/0072/0076/0077/0080, mark ADR-0075 as Reserved / Missing, show status metadata. | docs/adr/README.md | Docs-only; do not rewrite ADR bodies. | high | pending |
| 6 | Add ADR template if absent | Create template with Status, Date, Context, Decision, Consequences, Supersedes/Superseded by, Code evidence, Docs evidence. | docs/adr/template.md or docs/_meta/adr-template.md | Docs-only. | medium | pending |
| 7 | Patch active stale docs | README Vue 3 wording, CONTRIBUTING commands/toolchain, docs/engines implementation evidence, product frontend Personas wording. | README.md, CONTRIBUTING.md, docs/engines/README.md, docs/product/master-spec.md | Docs-only. | high | pending |
| 8 | Mark historical docs explicitly | Add status/callout to docs/superpowers, docs/roadmap, docs/reviews, docs/vision and superseded framework docs. | Historical docs | Large mechanical docs pass; no semantic rewrite. | medium | pending |
| 9 | Move implementation docs | Move implementation/current-state docs under `docs/implementation/` after governance policy exists. | docs/implementation | Separate PR due link churn. | medium | pending |
| 10 | Move channel docs | Move Email, Telegram, WhatsApp, Calls and Meetings docs under Communications channel structure after governance policy exists. | docs/domains/communications/channels | Separate PR due link churn. | medium | pending |
| 11 | Move historical docs | Move historical plans, roadmaps and reviews under `docs/archive/` after governance policy exists. | docs/archive | Separate PR due link churn. | medium | pending |
| 12 | Fix relative links | Fix active broken links first; archive historical broken links or leave with Historical warning if owner prefers low churn. | Link scan results in 03-doc-code-alignment.md | Docs-only. | medium | pending |
| 13 | Remove macOS metadata | Deleted docs/.DS_Store and docs/telegram/.DS_Store after owner approval. | docs/.DS_Store, docs/telegram/.DS_Store | No code impact; files were untracked. | high | completed |
| 14 | Add docs validation if existing tooling permits | If a repo script exists, wire link/ADR-index check; otherwise do not invent a new tool in same pass. | Makefile and scripts/ | Tooling decision. | medium | pending |

## Validation Plan

| Command | Purpose | Current note | Confidence | Status |
| --- | --- | --- | --- | --- |
| find docs -name *.md \| sort | List docs after changes. | Run after audit files are written. | high | pending |
| rg -n "SvelteKit\|Contact\|Contacts\|Mail Domain\|Knowledge Base\|CRM\|address book\|task tracker" docs | Vocabulary scan requested by owner. | Run after audit files are written. | high | pending |
| rg -n "ADR-0056\|ADR-0071\|ADR-0072\|ADR-0076\|ADR-0077\|ADR-0080" docs/adr/README.md | Confirm current ADR index omissions before fixing. | Run after audit files are written. | high | pending |
| pnpm docs:build / pnpm lint | Only if package/docs scripts exist. | No root package.json; frontend package has lint/build but docs task absent. | high | not applicable unless owner asks frontend validation |
| cargo fmt/test/nextest | Only if code is modified or code-alignment claims need deeper compile verification. | No code changes in this pass. | high | not run by default |

## Explicit Non-Changes For This Pass

- No Rust, TypeScript, Vue, Tauri or migration files were modified.
- No routes, tables, migrations or public API names were renamed.
- No ADR body/status was rewritten.
- No historical document semantics were rewritten.
