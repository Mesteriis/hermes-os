# Documentation Code Alignment Report

Status: current audit report for documentation/code alignment.

Date: 2026-06-28

Scope: documentation structure under `docs/` compared with current
`backend/src` and selected `frontend/src` packages. ADR remains the source of
truth where code and docs disagree.

## ADR Applied

- [ADR-0073 Backend Module Organization](../archive/adr/ADR-0073-backend-module-organization.md)
- [ADR-0054 Application Settings Store](../archive/adr/ADR-0054-application-settings-store.md)
- [ADR-0081 Opt-In OmniRoute AI Runtime](../archive/adr/ADR-0081-opt-in-omniroute-ai-runtime.md)
- [ADR-0096 Canonical Evidence, Review Inbox and Context Packs](../archive/adr/ADR-0096-canonical-evidence-review-and-context-packs.md)
- [ADR-0097 Communications Channel Domains To Integrations](../archive/adr/ADR-0097-communications-channel-domains-to-integrations.md)
- [ADR-0098 Provider-Neutral Communications API And Strict Boundaries](../archive/adr/ADR-0098-provider-neutral-communications-api-and-strict-boundaries.md)
- [ADR-0102 Zoom Provider Runtime Boundary](../archive/adr/ADR-0102-zoom-provider-runtime-boundary.md)

## Verified Backend Inventory

Current top-level backend code areas:

- `ai`;
- `app`;
- `application`;
- `bin`;
- `domains`;
- `engines`;
- `integrations`;
- `platform`;
- `vault`;
- `workflows`.

Current exported domain directories:

- `calendar`;
- `communications`;
- `decisions`;
- `documents`;
- `graph`;
- `obligations`;
- `organizations`;
- `personas`;
- `projects`;
- `relationships`;
- `review`;
- `settings`;
- `signal_hub`;
- `tasks`.

Current backend has an empty `backend/src/domains/settings/mod.rs`. Settings is
core application surface, not a current product domain; the working settings
implementation is under `platform/settings`.

Current exported engine modules:

- `automation`;
- `consistency`;
- `context_packs`;
- `enrichment`;
- `identity_resolution`;
- `memory`;
- `obligation`;
- `relationships`;
- `risk`;
- `search`;
- `timeline`;
- `trust`.

Current exported integrations:

- `mail`;
- `ollama`;
- `omniroute`;
- `telegram`;
- `whatsapp`;
- `zoom`.

Current workflow directories:

- `email_intelligence`;
- `email_sync_pipeline`;
- `graph_projection`;
- `mail_background_sync`;

`backend/src/workflows/mod.rs` also exports additional workflow modules that
are single files rather than directories.

## Documentation Updates Made

Added doc packages for code/ADR-backed gaps:

- [Review Domain](../domains/review/README.md);
- [Automation Engine](../engines/automation/README.md);
- [Context Packs Engine](../engines/context-packs/README.md);
- [Identity Resolution Engine](../engines/identity-resolution/README.md);
- [Relationship Candidate Engine](../engines/relationships/README.md);
- [Ollama Integration](../integrations/ollama/README.md);
- [OmniRoute Integration](../integrations/omniroute/README.md);
- [Application Settings](../platform/settings/README.md).

Updated central catalogs to reference those packages and current backend
inventory.

## Confirmed Boundaries

- Review is a domain by ADR-0096 and current backend code.
- Context Packs, Identity Resolution and Relationship Candidate behavior are
  engines by ADR-0096.
- Mail, Telegram, WhatsApp and Zoom are integrations, not product domains, per
  ADR-0097, ADR-0098 and ADR-0102.
- Ollama and OmniRoute are AI runtime integrations. Ollama is local default;
  OmniRoute is explicit opt-in.
- Settings implementation currently lives in the platform layer and frontend
  Settings UI. It should not be documented as accepted product-domain truth
  unless a later ADR changes ownership.

## Future Gaps

The following items remain future gaps or ownership decisions:

- `docs/domains/agents/` has canonical product language, and
  `frontend/src/domains/agents` exists, but there is no current
  `backend/src/domains/agents` package. Backend AI/agent implementation is
  split across `backend/src/ai`, app routes and settings/control surfaces.
- `docs/domains/notes/` documents Notes as document-like artifacts. There is a
  frontend `frontend/src/domains/notes` package, but no backend domain package
  and no ADR found in this pass that promotes Notes to a first-class domain.
- `docs/workflows/` is currently product-workflow documentation. It does not
  yet mirror each concrete backend workflow module. I did not create empty
  workflow package files because several workflow modules need a separate
  ownership pass.
- Frontend has domain packages such as `knowledge`, `home`, `timeline`,
  `personas` and `settings` that do not map one-to-one to backend domain
  directories. These appear to be UI/workspace packages rather than backend
  domain ownership, but that mapping was not fully audited in this pass.

## Follow-Up Candidates

- Add a workflow-module documentation pass for concrete modules under
  `backend/src/workflows` without replacing product workflow specs.
- Add a frontend-domain-to-backend-domain mapping document if UI workspace
  package boundaries keep diverging from backend domain ownership.

## Validation

Checks run for this report:

- `node scripts/check-architecture.mjs` passed.
- `git diff --check` passed.
- Markdown local-link checker passed for 315 docs markdown files.
- `make test-architecture` passed 26/26 architecture tests.
- Documentation/code package comparison found expected open items only:
  extra docs for `agents` and `notes` because they are not current backend
  domain packages.
