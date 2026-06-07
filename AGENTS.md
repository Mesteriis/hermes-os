# AGENTS.md

Правила работы агентов в репозитории Hermes Hub.

Эти правила обязательны для любых изменений в проекте. Репозиторий проектируется как долгосрочный personal knowledge system, а не как MVP.

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
4. Архитектурная документация в `docs/architecture/`, `docs/domains/`, `docs/agents/`, `docs/ui/`.
5. Текущие файлы реализации, когда они появятся.
6. Внешняя документация, только если она нужна и проверена.

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
- `ADR-0003` - SvelteKit frontend.
- `ADR-0004` - Tauri desktop shell.
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
- `ADR-0043` - Superseded by ADR-0055. Original read-only Gmail API and IMAP provider networking (temporary).
- `ADR-0044` - account setup and encrypted secret vault.
- `ADR-0046` - persistent dev mail cache and blob storage; mail bytes/attachments live under `docker/data/mail/`, PostgreSQL stores metadata, references and attachment scan state.
- `ADR-0053` - database-backed encrypted secret vault; encrypted credential payloads live in PostgreSQL ciphertext rows, while the vault key remains outside PostgreSQL.
- `ADR-0054` - application settings store; user-editable runtime/UI settings live in `application_settings`, while provider accounts remain domain records.
- `ADR-0055` - full email provider networking with read and write operations; supersedes ADR-0043. Read-only restriction retained only for automated integration tests.

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
- fake placeholder modules.

For implementation work, prefer TDD: write the failing test first, verify the failure, implement the smallest passing code, then run the configured validation.

After meaningful repository changes, run the relevant validation and create a git commit unless the user explicitly asks not to commit or the work is not yet in a valid state.

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
- Local event API HTTP endpoints must enforce the temporary API capability token from `ADR-0038` until the full capability runtime replaces it.
- Local event API access must be recorded in append-only `api_audit_log` per `ADR-0039`; do not store tokens or secrets in audit records.
- Protected local event API requests must include the temporary non-secret `X-Hermes-Actor-Id` identity from `ADR-0040`.
- Email ingestion provider accounts must support `gmail`, `icloud` and `imap` per `ADR-0041`; account config must not store OAuth tokens, app passwords or mailbox passwords.
- Multiple accounts for the same provider kind are required. Credential lookup must use `account_id` plus secret purpose, never provider kind alone.
- Provider credential bindings must use compatible secret kinds: `oauth_token` -> `oauth_token`, `imap_password`/`smtp_password` -> `app_password` or `password`.
- Raw communication provider records must remain append-only and preserve source provenance.
- Secret references per `ADR-0053` store metadata only. Encrypted credential payloads may live only in `encrypted_secret_vault_entries`; never place plaintext secret values in PostgreSQL config, metadata, tests, logs or docs.
- The database encrypted vault key must remain outside PostgreSQL. Do not derive it from hardware serial numbers; hardware IDs are not secrets.
- The in-memory secret resolver is allowed only for `test_double` references in tests and local adapter tests. Real provider adapters must use a real resolver for `os_keychain`, `encrypted_vault`, `database_encrypted_vault` or `external_vault`.
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
- `docker/.env.example` may contain non-secret development placeholders only.
- Use the root `Makefile` as the standard entry point.

First-time setup:

```sh
make docker-env
```

Review `docker/.env` after it is created.

`docker/.env` must define `HERMES_LOCAL_API_TOKEN` for local event API reads and writes. The value in `docker/.env.example` is a non-secret development placeholder only.

Validate Compose configuration:

```sh
make compose-config
```

Run the full local/CI validation gate:

```sh
make validate
```

Start development services in foreground:

```sh
make dev
```

Start development services in background:

```sh
make up
```

Open a shell in the development container:

```sh
make shell
```

Open PostgreSQL shell:

```sh
make db-shell
```

Start only PostgreSQL for local backend development:

```sh
make db-up
```

Run the local backend:

```sh
make backend-run
```

Run the local backend with `DATABASE_URL` built from `docker/.env`:

```sh
make backend-run-dev
```

Run backend smoke validation against development PostgreSQL:

```sh
make backend-smoke-dev
```

Run canonical event log smoke validation against development PostgreSQL:

```sh
make backend-event-log-smoke-dev
```

Run replay/projection cursor smoke validation against development PostgreSQL:

```sh
make backend-projection-smoke-dev
```

Run only projection runner smoke validation against development PostgreSQL:

```sh
make backend-projection-runner-smoke-dev
```

Run event HTTP API smoke validation against development PostgreSQL:

```sh
make backend-events-api-smoke-dev
```

View logs:

```sh
make logs
```

Stop services:

```sh
make down
```

Stop only PostgreSQL:

```sh
make db-down
```

Delete local Docker development data:

```sh
make reset-data CONFIRM=yes
```

This command is destructive and removes local state under `docker/data/`.

## 11. Reporting Format

For code or documentation changes, final response must include:

- changed files;
- summary;
- validation commands and results;
- remaining risks.

If validation was not run, state the exact reason.
