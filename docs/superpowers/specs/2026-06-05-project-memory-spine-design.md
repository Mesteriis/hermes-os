# Project Memory Spine Design

## Purpose

Hermes Hub V2 needs projects to become real memory anchors. The current backend can project contacts, messages and documents into the graph, but the `Projects` tab is still fed by static frontend data. This slice creates the first backend-backed project page and connects projects into the graph with source-backed evidence.

## Relevant ADRs

- `ADR-0001 Event Sourcing as System Spine`: project timelines must be reconstructable from source records and derived projections.
- `ADR-0008 Knowledge Graph First`: project relationships must carry confidence and provenance.
- `ADR-0023 Rebuildable Projections`: project graph edges are rebuildable projection state.
- `ADR-0031 Temporary Desktop Only UI Scope`: no mobile UI design, implementation or validation.
- `ADR-0038` and `ADR-0040`: protected local APIs require bearer token and actor identity.
- `ADR-0045 Graph Core Projection`: current graph constraints must be evolved before project nodes can exist.
- `ADR-0047 Project Memory Spine`: project records, keyword rules, project graph edges and read-only APIs define this slice.

## Scope

Implement five pieces together:

1. Project memory ADR/spec/plan.
2. Backend project read model and read-only API.
3. Graph projection for project nodes and project relationships.
4. Frontend `Projects` tab backed by the project detail API.
5. Project timeline derived from matched messages and documents.

## Backend Design

Add `projects` and `project_keywords` tables. `projects` stores stable local project metadata. `project_keywords` stores explicit match rules. The first matching rule is deterministic case-insensitive containment:

- messages: subject and body text;
- documents: title and extracted text.

This is not AI inference. Edges created from these rules use confidence below `1.0` and `suggested` review state.

Add `backend/src/projects.rs` with a `ProjectStore` responsible for:

- validating and upserting project records for tests and local bootstrap;
- listing projects with derived stats;
- loading a project detail payload;
- deriving recent messages, documents, people and timeline items from explicit project keywords.

Add protected routes:

- `GET /api/v2/projects?limit=<n>`;
- `GET /api/v2/projects/{project_id}`.

Responses must not include message bodies or secrets.

## Graph Projection Design

Extend `GraphNodeKind` with `Project` and extend `RelationshipType` with project relationship types. Add a migration that updates graph check constraints.

`GraphProjectionService::project_from_v1` projects:

- project node from each project record;
- `project_has_message` edges for keyword-matched messages;
- `project_has_document` edges for keyword-matched documents;
- `project_involves_person` or `project_involves_email_address` edges for participants in keyword-matched messages.

Each edge is source-backed with message or document evidence. Reprojection deletes stale project edges for the project before recreating the current deterministic set.

## Frontend Design

Extend `frontend/src/lib/api.ts` with project API types and fetch helpers.

In `frontend/src/routes/+page.svelte`, keep the current color palette and visual language. The first implementation should replace the main `Projects` dashboard data with backend project detail data:

- hero title/status/progress;
- meta strip;
- summary stats;
- radial graph chips from real project stats;
- project timeline;
- recent communications;
- top documents;
- key people;
- related projects from project list.

Unavailable sub-tabs stay disabled. Empty and error states must be explicit and visually consistent.

## Validation

Backend:

- targeted Rust tests for project store/API/projection;
- `cargo fmt`;
- `make backend-validate` or broader `make validate`.

Frontend:

- `cd frontend && pnpm check`;
- `cd frontend && pnpm build`;

Manual:

- browser smoke on `http://127.0.0.1:5174`;
- open `Projects`;
- confirm data comes from API without console errors;
- open `Knowledge Graph` and confirm project nodes can appear in node picker/search after projection.

## Risks

- Keyword matching may over-link records. Mitigation: use `suggested` review state and preserve evidence.
- Current frontend is a large single-file page. Mitigation: keep changes local and avoid broad component refactors.
- Existing databases need migration 0013. Mitigation: backend-managed migrations run on startup.
