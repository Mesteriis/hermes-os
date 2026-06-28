# AGENTS.md

Правила работы агентов в репозитории Hermes Hub.

Эти правила обязательны для любых изменений в проекте. Репозиторий проектируется как долгосрочная local-first Personal Memory System, а не как MVP, CRM, почтовый клиент, task tracker, calendar app или note-taking app.

## 1. Роль агента

Агент работает как Principal Software Engineer / Software Architect.

Ожидаемое поведение:

- проверять факты по файлам репозитория;
- не выдумывать API, команды, зависимости, схемы и результаты тестов;
- делать минимальное корректное изменение;
- уважать архитектурные границы;
- явно сообщать, что было проверено;
- не писать реализацию, если задача относится к текущей documentation-first фазе.

## 2. Источники истины

При конфликте использовать такой порядок:

1. Текущий запрос пользователя.
2. Этот `AGENTS.md`.
3. ADR в `docs/adr/`.
4. Каноническая продуктовая и foundation-документация:
   `docs/product/master-spec.md`, `docs/foundation/`, `docs/domains/`,
   `docs/engines/`, `docs/workflows/`.
5. Архитектурная документация в `docs/architecture/`, `docs/ai/agents/`, `docs/ui/`.
6. Текущие файлы реализации как источник фактов о том, что уже реально
   реализовано.
7. Внешняя документация, только если она нужна и проверена.

Если запрос конфликтует с действующим ADR, нельзя тихо нарушать ADR. Нужно явно указать конфликт и сначала предложить новый ADR, который supersede или уточнит прежнее решение.

## 3. Жесткое следование ADR

Перед любым нетривиальным изменением агент обязан:

- прочитать релевантные ADR;
- назвать в плане, какие ADR влияют на работу;
- не вносить изменение, которое нарушает ADR;
- создать новый ADR до реализации, если появляется долгосрочное архитектурное решение;
- пометить старый ADR как `Superseded`, если решение заменяется.

Ключевые текущие ADR:

- `ADR-0001` - event sourcing is system spine.
- `ADR-0002` - Rust backend.
- `ADR-0004` - Tauri desktop shell.
- `ADR-0093` - Vue 3 frontend (supersedes ADR-0003).
- `ADR-0005` - PostgreSQL primary store.
- `ADR-0006` - Tantivy full text search.
- `ADR-0008` - knowledge graph first.
- `ADR-0009` - local AI through Ollama.
- `ADR-0022` - no fine-tuning on private data.
- `ADR-0026` - desktop-first responsive UI.
- `ADR-0031` - temporary desktop-only UI scope; no mobile UI design, implementation or validation until superseded.
- `ADR-0032` - Docker Compose development environment under `docker/`.
- `ADR-0041` - email provider ingestion foundation for Gmail, iCloud and generic IMAP.
- `ADR-0042` - provider credential secret references and resolver boundary.
- `ADR-0043` - Superseded by ADR-0055. Historical temporary provider-networking restriction.
- `ADR-0046` - persistent dev mail cache and blob storage; mail bytes/attachments live under `docker/data/mail/`, PostgreSQL stores metadata, references and attachment scan state.
- `ADR-0054` - application settings store; user-editable runtime/UI settings live in `application_settings`, while provider accounts remain domain records.
- `ADR-0055` - full email provider networking with read and write operations; supersedes ADR-0043. Read-only restriction retained only for automated integration tests.
- `ADR-0056` - local API simplified auth with router-level `X-Hermes-Secret`.
- `ADR-0076` - host vault on macOS; supersedes ADR-0044 and ADR-0053.
- `ADR-0077` - i18n with Russian and English interface via JSON dictionaries and Svelte stores; English strings serve as translation keys, ru.json provides Russian translations.
- `ADR-0084` - Persona Intelligence System; supersedes Contact/Person CRM framing.
- `ADR-0085` - Communication spine and Consistency / Contradiction Engine.
- `ADR-0086` - first-class Relationship persistence.
- `ADR-0087` - contradiction observation persistence.
- `ADR-0088` - obligation persistence.
- `ADR-0089` - decision persistence.

Superseded ADRs remain historical traceability records. Do not apply their
obsolete token, actor, Contact or database-vault requirements as current rules
when a newer ADR supersedes them.

## 3.1 Canonical Product Model

Hermes is a Personal Memory System for:

- Communications;
- Knowledge;
- Memory;
- Relationships;
- Projects;
- Documents;
- Decisions;
- Obligations;
- Context.

The central product value is context, not CRUD.

Communication is the primary ingestion spine:

```text
Communication -> Source Evidence -> Extracted Knowledge -> Memory -> Context
```

Tasks, decisions, projects, obligations, dossiers, timelines and search results
are built from evidence, events, graph links and reviewed memory.

Do not describe Hermes as:

- Email Client;
- CRM;
- Address Book;
- Contact Manager;
- Task Tracker;
- Calendar App;
- Note Taking App;
- generic Knowledge Base.

People are Personas, not contacts.

Required Persona concepts:

- one Owner Persona with `is_self = true`;
- `PersonaType`: `human`, `ai_agent`, `organization_proxy`, `system`;
- Identity, Relationships, Communication, Memory, Timeline, Dossier and Context;
- first-class Relationship semantics with source persona, target persona,
  relationship type, trust score and strength score.

Current implementation may still use compatibility names such as `persons`,
`person_id`, historical `contacts`, `health`, `watchtower` or `follow-up`.
Treat those as implementation compatibility labels. When docs and code differ,
record the gap in `docs/refactoring/implementation-alignment-plan.md` or a
follow-up plan before renaming code, routes or schemas.

Domains own durable entities. Engines are reusable mechanisms that produce
derived views, scores, candidates, observations and context. Do not duplicate
engine ownership inside a domain. Shared engines include Memory, Timeline,
Trust, Search, Enrichment, Obligation, Risk and Consistency / Contradiction
(Polygraph).

Polygraph is the user-facing alias for the Consistency / Contradiction Engine.
It detects evidence-backed contradictions and creates reviewable observations.
It must not automatically overwrite memory or label a person as dishonest.

## 4. Implementation Phase

The project has entered implementation with the Rust backend foundation. Agents may add scoped implementation code when it follows the current request, relevant ADR and existing architecture.

Allowed:

- documentation;
- ADR;
- architecture diagrams;
- roadmap and research notes;
- repository hygiene files;
- development infrastructure;
- tests and validation tooling;
- scoped backend/frontend/Tauri implementation that is covered by relevant validation.

Disallowed without an explicit user request and relevant ADR review:

- broad rewrites;
- generated app scaffolds that ignore repository conventions;
- database migrations;
- provider adapters;
- AI agent runtime code;
- domain model expansion;
- fake stub modules.

For implementation work, prefer TDD: write the failing test first, verify the failure, implement the smallest passing code, then run the configured validation.

After meaningful repository changes, run the relevant validation. Do not create
a git commit unless the user explicitly asks for a commit.

## 5. Required Workflow

For non-trivial work:

1. Inspect `git status --short`.
2. Inspect relevant files before editing.
3. Recall project memory if AgentMemory tools are available.
4. Read relevant ADR and architecture docs.
5. State a concise plan.
6. Edit only scoped files.
7. Validate with available commands.
8. Report changed files, validation and risks.

For trivial documentation edits, this workflow may be compressed, but validation must still be truthful.

## 6. Git Rules

- The repository must remain an active Git repository.
- Run `git status --short` before and after meaningful changes.
- Do not run destructive Git commands such as `git reset --hard` or `git checkout --` unless explicitly requested.
- Do not revert user changes.
- Do not commit unless the user explicitly asks for a commit.
- Do not mix unrelated changes into one task.
- Keep `.gitignore` aligned with the actual stack.

## 7. Linting and Validation Policy

Never claim validation passed unless the command was actually run.

Always discover the real toolchain from repository files before running stack-specific commands.

### Repository foundation validation

Use these checks while the repository is still in documentation/dev-infrastructure foundation mode:

```sh
find docs/adr -maxdepth 1 -type f -name 'ADR-*.md' | wc -l
find docs -type f -name '*.md' | wc -l
find . -path ./.git -prune -o -type f \
  ! -name '*.md' \
  ! -name '.gitignore' \
  ! -name 'Makefile' \
  ! -name 'Dockerfile' \
  ! -name '.env.example' \
  -print
```

If Markdown lint tooling is added later, run the configured command. Do not invent a markdown linter command when no config or dependency exists.

### Rust validation

When Rust implementation exists, prefer the configured commands. If no project-specific command exists, the default validation set is:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

For the current backend crate, use the repository Makefile targets:

```sh
make validate
make backend-fmt-check
make backend-clippy
make backend-test
make backend-validate
```

Run `make validate` before reporting broad backend or development-infrastructure work as complete. Use `make backend-validate` for targeted backend-only changes.
For full backend validation, prefer `make backend-test` or `make backend-validate` over direct `cargo test`. These targets run the `crates/testkit` session harness (`hermes_test_session`) that reuses the shared testcontainers PostgreSQL session correctly and cleans it up after the run. Direct full-suite `cargo test` bypasses that harness, can create excessive testcontainers churn, and may leave Docker container garbage behind after failures or interrupted runs.

### SvelteKit / TypeScript validation

Detect the package manager from lockfiles:

- `pnpm-lock.yaml` -> use `pnpm`
- `yarn.lock` -> use `yarn`
- `package-lock.json` -> use `npm`

Do not mix package managers.

Run configured scripts such as:

```sh
<package-manager> lint
<package-manager> check
<package-manager> test
<package-manager> build
```

Only run scripts that exist in `package.json`.

### Tauri validation

When Tauri exists, validate both sides:

- frontend checks from `package.json`;
- Rust checks in the Tauri crate;
- Tauri build/check command only if configured and practical for the task.

### Infrastructure and config validation

When relevant tooling exists, validate:

- SQL migrations;
- YAML;
- TOML;
- Docker/Compose files;
- shell scripts;
- OpenTelemetry config.

Use the repository-configured tool first. If no tool exists, report that validation was not available instead of fabricating coverage.

## 8. Architecture Constraints

- Local-first is mandatory.
- Knowledge graph and events are primary architecture concepts.
- Search indexes and embeddings are derived, rebuildable state.
- AI output is never source of truth.
- Private data must not be used for fine-tuning.
- Provider adapters must preserve raw source provenance. Full read-write provider networking is enabled per ADR-0055; read-only restriction applies only to automated tests.
- Agents and plugins must use capability-based permissions.
- Mobile UI is out of scope until `ADR-0031` is superseded.
- Docker development infrastructure must stay under `docker/` per `ADR-0032`.
- Protected local API endpoints must use the router-level shared secret guard
  from `ADR-0056`: `HERMES_LOCAL_API_SECRET` plus the `X-Hermes-Secret`
  request header.
- Local event API access must be recorded in append-only `api_audit_log` per `ADR-0039`; do not store tokens or secrets in audit records.
- API audit actor identity is the constant `hermes-frontend` unless a newer ADR
  changes the local actor model. Do not require `X-Hermes-Actor-Id`.
- Email ingestion provider accounts must support `gmail`, `icloud` and `imap` per `ADR-0041`; account config must not store OAuth tokens, app passwords or mailbox passwords.
- Multiple accounts for the same provider kind are required. Credential lookup must use `account_id` plus secret purpose, never provider kind alone.
- Provider credential bindings must use compatible secret kinds: `oauth_token` -> `oauth_token`, `imap_password`/`smtp_password` -> `app_password` or `password`.
- Raw communication provider records must remain append-only and preserve source provenance.
- Secret references store metadata only. Per `ADR-0076`, new secret payloads
  must live in the host vault, while PostgreSQL stores only non-secret account
  metadata, `secret_references` and account-to-secret bindings.
- `encrypted_secret_vault_entries` and `HERMES_SECRET_VAULT_KEY` are legacy
  database-vault migration compatibility. Do not add new provider credential
  payloads to PostgreSQL.
- The in-memory secret resolver is allowed only for `test_double` references in
  tests and local adapter tests. Real provider adapters must use a real resolver
  for `host_vault`, `os_keychain`, `encrypted_vault`,
  `database_encrypted_vault` or `external_vault`, according to the current ADR
  and implementation boundary.
- Application settings per `ADR-0054` are allowlisted typed values. Do not store credentials or duplicate provider accounts in `application_settings`; surface account records from their domain tables in the Settings UI.
- Mail blob and attachment bytes must stay out of PostgreSQL per `ADR-0046`; store only metadata, hashes and local blob paths in database tables.
- Extracted attachment metadata must pass through the attachment safety scanner boundary from `ADR-0046`. The no-op scanner records `not_scanned`; do not mark attachments as `clean` without a real scanner backend.

## 9. Security and Privacy

- Never commit secrets.
- Never hardcode tokens, passwords, private keys, API keys, Wi-Fi credentials or personal data.
- Never log private message bodies, document contents or secrets in telemetry.
- Treat imported documents and messages as untrusted input.
- Agent tools with side effects require explicit permission and auditability.

## 10. Development Environment

Docker is used only for local development infrastructure at this stage. It is not a production deployment model.

Rules:

- Docker files must stay under `docker/`.
- Persistent development data lives under `docker/data/`.
- `docker/data/` contents are local state and must not be committed.
- `docker/.env` is local-only and must not be committed.
- `docker/.env.example` may contain non-secret development example values only.
- Use the root `Makefile` as the standard entry point.

First-time setup:

```sh
make docker-env
```

Review `docker/.env` after it is created.

`docker/.env` must define `HERMES_LOCAL_API_SECRET` for protected local API
requests. The value in `docker/.env.example` is a non-secret development value
only.

Host vault configuration is the current credential storage model. Use
`HERMES_VAULT_HOME` when an explicit host vault path is needed. Use
`HERMES_SECRET_VAULT_KEY` only for legacy database-vault migration
compatibility.

Validate Compose configuration:

```sh
docker compose --env-file docker/.env --project-directory docker -f docker/docker-compose.yml config
```

Run the full local/CI validation gate:

```sh
cargo fmt --check --manifest-path backend/Cargo.toml
cargo clippy --manifest-path backend/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path backend/Cargo.toml
cd frontend && pnpm build
```

Start development services in foreground:

```sh
make dev
```

Build release artifacts:

```sh
make build
```

Run backend-managed SQLx migrations:

```sh
make migrate
```

Create backup of PostgreSQL and host vault data:

```sh
make vault-backup
```

Restore PostgreSQL and host vault data interactively:

```sh
make vault-restore
```

Delete local PostgreSQL development data:

```sh
make clean-data
```

This command is destructive and removes local state under `docker/data/postgres/`.

Delete local host vault data:

```sh
make clean-vault
```

This command is destructive and removes local state under `HERMES_HOST_VAULT_HOME`.

## 11. Reporting Format

For code or documentation changes, final response must include:

- changed files;
- summary;
- validation commands and results;
- remaining risks.

If validation was not run, state the exact reason.
