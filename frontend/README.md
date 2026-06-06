# Frontend

SvelteKit desktop UI for Hermes Hub, packaged by Tauri.

Current scope is a desktop/laptop shell for the local backend APIs with provider account setup wizards for Gmail, iCloud and raw IMAP, V2 graph/project/task/contact/document workflow surfaces, and V3 local AI workflow surfaces. Mobile UI is out of scope while ADR-0031 is active.

## Scaffold

The requested scaffold command was attempted from `frontend/`:

```sh
pnpm create svelte@latest . -- --template skeleton --types ts --no-add-ons
```

It exited with the current Svelte CLI deprecation message that `create svelte` has been replaced by `sv create`.

The successful replacement command was:

```sh
pnpm dlx sv@latest create . --template minimal --types ts --no-add-ons --no-dir-check --no-install
pnpm install
```

Tauri was initialized with:

```sh
pnpm tauri init --ci --app-name "Hermes Hub" --window-title "Hermes Hub" --frontend-dist "../build" --dev-url "http://localhost:5173" --before-dev-command "pnpm dev" --before-build-command "pnpm build"
```

## Commands

```sh
pnpm install
pnpm check
pnpm build
pnpm dev
pnpm tauri dev
pnpm tauri build
```

From the repository root, the same checks are available through Make:

```sh
make frontend-install
make frontend-dev
make frontend-check
make frontend-build
make frontend-tauri-dev
make frontend-tauri-build
```

For the normal full-stack development loop, use `make dev` from the repository root. It starts PostgreSQL, the backend auto-restart watcher, and this frontend with Vite HMR. The default frontend URL is `http://127.0.0.1:5174`; override it with `HERMES_FRONTEND_PORT` in `docker/.env` when needed.

## Backend Dependency

The shell calls:

```sh
GET http://127.0.0.1:8080/api/v1/status
POST http://127.0.0.1:8080/api/v1/email-accounts/gmail/oauth/start
POST http://127.0.0.1:8080/api/v1/email-accounts/gmail/oauth/complete
POST http://127.0.0.1:8080/api/v1/email-accounts/imap
GET http://127.0.0.1:8080/api/v2/graph/summary
GET http://127.0.0.1:8080/api/v2/graph/nodes?limit=<limit>
GET http://127.0.0.1:8080/api/v2/graph/search?q=<query>&limit=<limit>
GET http://127.0.0.1:8080/api/v2/graph/neighborhood?node_id=<node_id>&depth=1
GET http://127.0.0.1:8080/api/v2/projects?limit=<limit>
GET http://127.0.0.1:8080/api/v2/projects/{project_id}
GET http://127.0.0.1:8080/api/v2/task-candidates?limit=<limit>
PUT http://127.0.0.1:8080/api/v2/task-candidates/{task_candidate_id}/review
GET http://127.0.0.1:8080/api/v2/tasks?limit=<limit>
GET http://127.0.0.1:8080/api/v2/identity-candidates?limit=<limit>
PUT http://127.0.0.1:8080/api/v2/identity-candidates/{identity_candidate_id}/review
GET http://127.0.0.1:8080/api/v2/document-processing/jobs?limit=<limit>
POST http://127.0.0.1:8080/api/v2/document-processing/jobs/{job_id}/retry
GET http://127.0.0.1:8080/api/v3/ai/status
GET http://127.0.0.1:8080/api/v3/agents
GET http://127.0.0.1:8080/api/v3/ai/runs?limit=<limit>
POST http://127.0.0.1:8080/api/v3/ai/answers
POST http://127.0.0.1:8080/api/v3/ai/task-candidates/refresh
POST http://127.0.0.1:8080/api/v3/ai/meeting-prep
```

Requests use `Authorization: Bearer <token>` and `X-Hermes-Actor-Id`. The graph dashboard reads `/api/v2/graph/summary`; the graph explorer searches non-empty queries through `/api/v2/graph/search` and loads depth-1 neighborhoods through `/api/v2/graph/neighborhood`. The workflow tabs use protected project, task candidate, active task, contact identity and document-processing endpoints. Account setup also requires backend `HERMES_SECRET_VAULT_PATH` and `HERMES_SECRET_VAULT_KEY`.

The AI Agents tab reads V3 runtime status, registered agents, persisted run history and citation-backed workflow responses. Communications exposes scoped Ask AI for the selected message, Projects exposes scoped Prepare brief for the selected project, and Tasks exposes AI refresh for suggested task candidates. AI refresh writes only suggested candidates; the existing task review queue remains the path to active tasks.

The backend must be running on `127.0.0.1:8080` with `HERMES_LOCAL_API_TOKEN=change-me-local-api-token`, or the frontend must be started with matching Vite public overrides:

```sh
VITE_HERMES_API_BASE_URL=http://127.0.0.1:8080 \
VITE_HERMES_LOCAL_API_TOKEN=change-me-local-api-token \
VITE_HERMES_ACTOR_ID=desktop-shell \
pnpm dev
```

The placeholder token is for local development only and must match the backend local API token. `VITE_HERMES_ACTOR_ID` is a non-secret local client identity used by protected API requests and backend audit records.

## V2 Desktop Surfaces

The desktop shell is intentionally desktop/laptop scoped under ADR-0031. Current V2 desktop surfaces are:

- Current: Knowledge Graph explorer using graph summary, node picker, search and neighborhood APIs.
- Current: Projects tab using project records, timelines and project detail APIs.
- Current: Tasks tab using task candidate, task candidate review and active task APIs.
- Current: Contacts identity review surface using identity candidate list, review APIs and explicit split review controls.
- Current: Document processing status surface using the document-processing jobs API and failed-job retry controls.

## V3 Desktop Surfaces

- Current: AI Agents tab using backend AI status, agent registry, run history, answer form, meeting prep, task extraction and citations.
- Current: Communications scoped Ask AI action for the selected message.
- Current: Projects scoped Prepare brief action for the selected project.
- Current: Tasks AI refresh action that reuses the existing candidate review queue.

Validate frontend changes with:

```sh
pnpm check
pnpm build
```

From the repository root, the V2 closure validation path is:

```sh
make frontend-check
make frontend-build
make validate
```
