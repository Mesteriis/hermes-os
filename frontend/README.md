# Frontend

SvelteKit desktop UI for Hermes Hub, packaged by Tauri.

Current scope is a desktop/laptop status shell for the local V1 backend API. Mobile UI is out of scope while ADR-0031 is active.

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
make frontend-check
make frontend-build
make frontend-tauri-dev
make frontend-tauri-build
```

## Backend Dependency

The status screen calls:

```sh
GET http://127.0.0.1:8080/api/v1/status
```

The request uses `Authorization: Bearer <token>` and `X-Hermes-Actor-Id`. The backend must be running on `127.0.0.1:8080` with `HERMES_LOCAL_API_TOKEN=change-me-local-api-token`, or the frontend must be started with matching Vite public overrides:

```sh
VITE_HERMES_API_BASE_URL=http://127.0.0.1:8080 \
VITE_HERMES_LOCAL_API_TOKEN=change-me-local-api-token \
VITE_HERMES_ACTOR_ID=desktop-shell \
pnpm dev
```

The placeholder token is for local development only and must match the backend local API token.
