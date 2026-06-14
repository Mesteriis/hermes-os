# Frontend

Vue 3 + TypeScript desktop UI for Hermes Hub, packaged by Tauri.

Current scope is a desktop/laptop shell for the local backend APIs with provider account setup wizards for Gmail, iCloud and raw IMAP, graph/project/task/Persona identity/document workflow surfaces, and local AI workflow surfaces. Mobile UI is out of scope while ADR-0031 is active.

## UI Styling Contract

The app-level CSS is loaded in this order from `src/App.vue`:

1. `src/assets/styles/tokens.css` defines design tokens and browser root defaults.
2. `src/assets/styles/app.css` defines global shell, view, state and responsive styles.

All components use scoped `<style>` blocks and Tailwind utility classes. No inline `style=` attributes in production components.

The supported desktop window minimum is `800 x 600`. At smaller widths or heights, a viewport guard is shown instead of the app. This is a desktop window constraint, not mobile UI support; ADR-0031 still keeps mobile design and validation out of scope.

## Scaffold

The frontend was scaffolded as a new Vue 3 + TypeScript + Vite project:

```sh
pnpm create vue@latest . -- --typescript --force
pnpm install
```

Tauri was initialized with:

```sh
pnpm tauri init --ci --app-name "Hermes Hub" --window-title "Hermes Hub" --frontend-dist "../dist" --dev-url "http://localhost:5173" --before-dev-command "pnpm dev" --before-build-command "pnpm build"
```

## Commands

```sh
pnpm install
pnpm lint:ts
pnpm build
pnpm dev
pnpm tauri dev
pnpm tauri build
```

From the repository root, the same checks are available through Make:

```sh
make frontend-install
make frontend-dev
make frontend-lint
make frontend-check
make frontend-build
make tdlib-macos-resource
make backend-sidecar-macos
make frontend-tauri-dev
make frontend-tauri-build
```

For the normal full-stack development loop, use `make dev` from the repository root. It starts PostgreSQL, the backend auto-restart watcher, and this frontend with Vite HMR. The default frontend URL is `http://127.0.0.1:5174`; override it with `HERMES_FRONTEND_PORT` in `docker/.env` when needed.

## Bundled TDLib Runtime

macOS release builds package the Telegram TDLib JSON runtime from
`frontend/src-tauri/resources/tdlib/`. Generated `libtdjson.dylib` files are not
committed; prepare the resource before `tauri build`:

```sh
make tdlib-macos-resource
make frontend-tauri-build
```

`make tdlib-macos-resource` copies `libtdjson.dylib` from, in order:

1. `HERMES_TDJSON_SOURCE`
2. `HERMES_TDJSON_PATH`
3. Homebrew `tdlib`
4. `/opt/homebrew/lib/libtdjson.dylib`
5. `/usr/local/lib/libtdjson.dylib`

Release CI can build TDLib from source instead of relying on a system install:

```sh
HERMES_TDLIB_BUILD_FROM_SOURCE=1 make tdlib-macos-resource
```

The backend still accepts `HERMES_TDJSON_PATH` as a development override, but a
packaged macOS app should resolve TDLib from the bundled Tauri resource path.
Linux is supported only as a development/container target and is not packaged as
a desktop TDLib bundle.

Telegram QR login also needs Telegram app credentials. Development runs can set
`HERMES_TELEGRAM_API_ID` and `HERMES_TELEGRAM_API_HASH` in the backend
environment. Packaged macOS builds can inject them into the Tauri launcher with
`HERMES_BUNDLED_TELEGRAM_API_ID` and `HERMES_BUNDLED_TELEGRAM_API_HASH`; the
launcher forwards those values to the backend sidecar as runtime
`HERMES_TELEGRAM_API_ID` and `HERMES_TELEGRAM_API_HASH`.

Google mail setup needs one project-owned OAuth Desktop app client. End users of
the packaged app should not create their own Google Cloud project. Release builds
copy the downloaded Desktop app JSON into the Tauri resource bundle:

```sh
make google-oauth-resource
make frontend-tauri-build
```

`make google-oauth-resource` reads `HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_PATH` from
`docker/.env`, or `HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_SOURCE` from the shell, and
copies the file to `frontend/src-tauri/resources/google-oauth/client_secret.json`.
That generated resource is ignored by Git. The packaged launcher passes the
bundled resource path to the backend sidecar as
`HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_PATH`.

## Bundled Backend Sidecar

macOS release builds package the Rust backend as a Tauri sidecar from
`frontend/src-tauri/binaries/`. Generated sidecar binaries are not committed;
prepare the current host binary before `tauri build`:

```sh
make backend-sidecar-macos
make frontend-tauri-build
```

`make frontend-tauri-build` runs `google-oauth-resource`,
`backend-sidecar-macos` and `tdlib-macos-resource` before invoking Tauri.

## Architecture

The frontend is organized by domain under `src/domains/`:

- `src/domains/` — 14 domain modules (home, settings, personas, organizations, projects, tasks, calendar, documents, notes, knowledge, review, agents, timeline, communications, telegram, whatsapp)
- `src/shared/` — Shared UI primitives, stores, composables
- `src/platform/` — Platform abstractions (API client, SSE, routing, i18n, theming)
- `src/app/` — App shell, layout, view routing

Each domain follows a consistent structure:

```
domains/<name>/
  types/<name>.ts      — TypeScript interfaces
  api/<name>.ts        — API functions
  queries/<name>.ts    — TanStack Query hooks
  stores/<name>.ts     — Pinia store
  components/          — Vue 3 SFC components
  views/<name>.ts      — Page-level view component
```

Data flow: API → TanStack Query → Component (direct) or API → Pinia Store → Component.

Requests use `X-Hermes-Secret: <secret>` via the centralized `ApiClient` (see `src/platform/api/ApiClient.ts`).

Validate frontend changes with:

```sh
pnpm lint:ts
pnpm build
```

From the repository root, the full validation path is:

```sh
make frontend-check
make frontend-build
make validate
```
