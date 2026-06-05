# Project Memory Spine Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first backend-backed project memory spine: project records, project graph links, project API, real Projects tab data and derived project timeline.

**Architecture:** Add a narrow PostgreSQL project read model and explicit keyword rules, then project those records into the existing rebuildable graph tables. Keep APIs read-only and protected by the local bearer token plus `X-Hermes-Actor-Id`. Keep frontend changes scoped to the existing Svelte page and current desktop visual language.

**Tech Stack:** Rust 1.85/edition 2024, Axum, SQLx/PostgreSQL migrations, SvelteKit 2, Svelte 5 runes, TypeScript, pnpm, Make.

---

## Source Spec

- `docs/superpowers/specs/2026-06-05-project-memory-spine-design.md`
- `docs/adr/ADR-0047-project-memory-spine.md`

## File Map

- Create: `backend/migrations/0013_create_projects_and_extend_graph.sql`
- Create: `backend/src/projects.rs`
- Modify: `backend/src/lib.rs`
- Modify: `backend/src/graph.rs`
- Modify: `backend/src/graph_projection.rs`
- Create: `backend/tests/projects.rs`
- Create: `backend/tests/projects_api.rs`
- Modify: `backend/tests/graph_projection.rs`
- Modify: `frontend/src/lib/api.ts`
- Modify: `frontend/src/routes/+page.svelte`
- Modify: `docs/adr/README.md`

## Tasks

### Task 1: Documentation And ADR

- [ ] Add `ADR-0047` describing project memory spine, graph extension and non-goals.
- [ ] Add the design spec for project memory spine.
- [ ] Add this implementation plan.
- [ ] Update `docs/adr/README.md`.

### Task 2: Project Schema And Store

- [ ] Add migration 0013 with `projects`, `project_keywords`, graph node kind `project` and project relationship types.
- [ ] Add `ProjectStore` with project validation, upsert, list and detail methods.
- [ ] Add live PostgreSQL tests for project upsert, derived stats and timeline.

### Task 3: Project API

- [ ] Add `pub mod projects`.
- [ ] Add protected `GET /api/v2/projects` and `GET /api/v2/projects/{project_id}` routes.
- [ ] Add API error mapping without leaking internal SQL details.
- [ ] Add route tests for auth and live project detail.

### Task 4: Graph Projection

- [ ] Extend `GraphNodeKind`, `RelationshipType` and parsers.
- [ ] Project project nodes from `projects`.
- [ ] Project `project_has_message`, `project_has_document`, `project_involves_person` and `project_involves_email_address`.
- [ ] Delete stale project edges before reprojecting a project.
- [ ] Add projection tests for project nodes, project relationship evidence and idempotence.

### Task 5: Frontend Projects Tab

- [ ] Add project API types and fetch helpers in `frontend/src/lib/api.ts`.
- [ ] Load project list/detail on mount with local loading/error states.
- [ ] Replace main Projects dashboard literals with API-backed data.
- [ ] Render timeline, recent communications, top documents and key people from API data.
- [ ] Keep unimplemented tabs disabled and preserve current visual language.

### Task 6: Validation And Commit

- [ ] Run Rust formatting and targeted tests.
- [ ] Run frontend check/build.
- [ ] Run `git diff --check`.
- [ ] Run `make validate`.
- [ ] Smoke test the local frontend in browser.
- [ ] Commit all changes.
