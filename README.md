# Hermes Hub

Hermes Hub - локальная Personal Memory System для коммуникаций, знаний,
памяти, отношений, проектов, документов, решений, обязательств и контекста.

Проектируемая система объединяет Communications, Personas, Organizations,
Projects, Documents, Tasks, Events, Knowledge, Decisions и Obligations в одну
локальную модель памяти. Hermes не является почтовым клиентом, мессенджером,
CRM, task tracker, calendar app или note-taking app. Центральная идея -
долговременная, переносимая память владельца, построенная на событиях, графе
знаний, RAG, vector search и структурированных проекциях, без fine-tuning
пользовательских данных.

## Статус

Репозиторий перешел от foundation phase к рабочей canonical evidence
architecture.

## Working Today

✅ Gmail account connection
✅ IMAP account connection
✅ Email ingestion and storage
✅ Telegram message ingestion
✅ Review inbox
✅ Semantic search infrastructure

## Known Limitations

🚧 Telegram production sync
🚧 WhatsApp
🚧 Graph workflows

## Run Today

```sh
make dev
```

`make dev` is the single local full-stack entrypoint.

Текущий результат:

- продуктовая и архитектурная документация
- базовая структура monorepo
- ADR по ключевым долгосрочным решениям
- roadmap до версии 5.0
- Rust backend foundation с конфигурацией и `GET /healthz`
- router-level local API secret guard for protected local APIs
- append-only audit log for authorized event API access attempts
- communication ingestion storage foundation for Gmail, iCloud Mail and generic IMAP
- secret reference metadata boundary for provider credentials
- host vault account setup for Gmail, iCloud Mail and generic IMAP
- email provider networking with read/write capability boundaries; automated
  provider tests keep read-only paths where required
- persistent local mail blob/attachment metadata foundation
- V1 status API for desktop shell bootstrapping
- desktop frontend on Vue 3 + Vite packaged in the Tauri shell
- Docker Compose окружение для локальной разработки
- local Ollama AI runtime boundary
- pgvector semantic embeddings with `halfvec(2560)`
- protected AI APIs for status, agents, run history, cited answers, task candidate refresh and meeting prep
- Telegram fixture foundation with policy-backed automation dry-run and call transcript storage
- WhatsApp Web fixture/manual companion foundation
- capability decision audit slice for Telegram send policy decisions
- canonical evidence architecture:
  - `Vault -> Observation Platform -> Ingestion -> Domains -> Knowledge -> Review -> Actions`
  - `Observation Platform` as canonical append-only evidence store
  - `Review` as the unified inbox domain for approval, promotion and dismissal
  - `Context Packs` under `engines/context_packs`
- first-class review API and review inbox UI for canonical evidence promotion
- Vue 3 + Vite desktop frontend packaged in the Tauri shell

Current architecture completion report:

- [Canonical Evidence Final Report](canonical-evidence-final-report.md)

## Open Source

Hermes Hub is published as an open source repository under the MIT License.

Documentation portal:

- [Hermes Hub Documentation](https://mesteriis.github.io/hermes-os/) - styled
  GitHub Pages entrypoint for the canonical documentation model.
- [Repository Documentation Index](docs/README.md) - source documentation in
  the repository.

Before contributing:

- read [CONTRIBUTING.md](CONTRIBUTING.md);
- do not commit secrets, private message data, local `.env` files or generated data under `docker/data/`;
- report security issues through [SECURITY.md](SECURITY.md), not public issues;
- keep changes aligned with the relevant ADRs in [docs/adr](docs/adr).

## Принципы

- Local first: пользователь владеет данными, облако не является обязательной точкой отказа.
- Knowledge graph first: память живет в графе, индексах и событиях, а не в весах модели.
- Event spine + canonical evidence: значимые изменения фиксируются event log,
  а входное evidence идет через Observation Platform.
- AI native: AI является частью всех подсистем, а не отдельным чат-виджетом.
- Long-term product: проектируется конечная система, не MVP.

## Целевая технологическая рамка

- Backend: Rust
- Frontend: Vue 3 + Vite
- Desktop: Tauri
- Database: PostgreSQL
- Full text search: Tantivy
- Local AI: Ollama
- Telemetry: OpenTelemetry

## Структура

- [docs/foundation](docs/foundation) - каноническая модель, glossary, engines и domain map.
- [docs/site](docs/site) - GitHub Pages documentation portal styled with the
  Hermes shell design language.
- [docs/vision](docs/vision) - долгосрочное видение.
- [docs/product](docs/product) - charter, scope и продуктовые границы.
- [docs/architecture](docs/architecture) - системная архитектура и ключевые технические модели.
- [docs/domains](docs/domains) - доменные области.
- [docs/adr](docs/adr) - Architecture Decision Records.
- [docs/agents](docs/agents) - AI agent architecture.
- [docs/ui](docs/ui) - UI architecture и design system vision.
- [docs/roadmap](docs/roadmap) - план развития до версии 5.0.
- [docs/research](docs/research) - вопросы исследования и открытые риски.
- [backend](backend) - Rust backend.
- [frontend](frontend) - desktop-only Vue 3 + Vite frontend packaged in a Tauri shell.
- [infrastructure](infrastructure) - self-hosted и локальная инфраструктура.
- [tools](tools) - будущие developer и data tools.
- [examples](examples) - будущие спецификации примеров и тестовых сценариев.

## V1 Local Run

```sh
make dev
```

`make dev` is the single local full-stack entrypoint. It creates `docker/.env`
from `docker/.env.example` when missing, starts PostgreSQL in Docker, runs the
Rust backend through repo-local `bacon`, runs the Vue 3 + Vite frontend
natively, and writes structured local logs under
`.local/dev-logs/`.

For a stable tail target during one dev session:

```sh
make logs
```

## Разработка

Поддерживаемый публичный command surface intentionally small:

```sh
make dev
make logs
make build
make migrate
make vault-backup
make vault-restore
make clean
make clean-dev
make clean-validate
make clean-build
make clean-data
make clean-vault
```

`make build` делает native release build backend, frontend и Tauri app, включая
внутреннюю подготовку bundled resources. `make migrate` поднимает PostgreSQL
при необходимости и запускает backend-managed SQLx migrations.

Cargo artifacts are split by workflow:

- `make dev` and `make migrate` use `target/dev`.
- `make validate` uses `target/validate` with `CARGO_INCREMENTAL=0`.
- `make build` uses `target/build` for backend release sidecar builds.
- Tauri still uses `frontend/src-tauri/target`.

`make clean` удаляет все build artifacts, frontend cache/artifacts, temp files
и logs, но не удаляет базу. `make clean-dev`, `make clean-validate` and
`make clean-build` clean only the corresponding artifact family. `make clean-data`
требует подтверждения и удаляет только локальные данные PostgreSQL под
`docker/data/postgres/`. `make clean-vault` требует подтверждения и удаляет
только локальные данные vault под `HERMES_HOST_VAULT_HOME`.

Создать timestamped backup PostgreSQL и host vault:

```sh
make vault-backup
```

Backup сохраняется под `backups/YYYY-MM-DD/<timestamp>/` и включает:

- `postgres.sql`
- `vault/`
- `manifest.json`
- `RESTORE.txt`

Восстановить backup интерактивно:

```sh
make vault-restore
```

`make vault-restore` предлагает список доступных backup directories, требует
подтверждения и затем восстанавливает PostgreSQL dump и host vault snapshot.

`/api/v1/events` и `/api/v1/audit/events` требуют локальный API secret header:

```sh
X-Hermes-Secret: <HERMES_LOCAL_API_SECRET>
```

`/api/v1/status` используется desktop shell и также требует локальный API secret header:

```sh
GET /api/v1/status
X-Hermes-Secret: <HERMES_LOCAL_API_SECRET>
```

Communication message read endpoints for the desktop shell use the same local API secret and secret header:

```sh
GET /api/v1/communications/messages?limit=50
GET /api/v1/communications/messages/<message_id>
X-Hermes-Secret: <HERMES_LOCAL_API_SECRET>
```

The message list reads canonical `communication_messages`; message detail returns canonical body text plus attachment metadata and local blob references. It does not read or return attachment bytes.

AI APIs use the same local secret and secret header:

```sh
GET /api/v1/ai/status
GET /api/v1/ai/agents
GET /api/v1/ai/runs
GET /api/v1/ai/runs/<run_id>
POST /api/v1/ai/answers
POST /api/v1/ai/task-candidates/refresh
POST /api/v1/ai/meeting-prep
X-Hermes-Secret: <HERMES_LOCAL_API_SECRET>
```

AI task extraction writes only `suggested` task candidates. Existing review APIs remain the only path to active tasks.

Account setup endpoints require the host vault to be initialized and unlocked.
New credential payloads are stored in the host vault; PostgreSQL stores
non-secret account metadata, secret references and account-to-secret bindings.
`HERMES_SECRET_VAULT_KEY` remains a legacy migration compatibility variable only.

UI scope is desktop/laptop only while ADR-0031 is active; mobile UI is not implemented or validated.

## Главные документы

- [Documentation Index](docs/README.md)
- [Product Master Spec](docs/product/master-spec.md)
- [Product Development Roadmap](docs/product/development-roadmap.md)
- [Foundation Vision](docs/foundation/vision.md)
- [Foundation Glossary](docs/foundation/glossary.md)
- [World Model](docs/foundation/world-model.md)
- [Engines](docs/foundation/engines.md)
- [Vision Document](docs/vision/vision-document.md)
- [Product Charter](docs/product/product-charter.md)
- [Product Scope](docs/product/product-scope.md)
- [Product Roadmap](docs/roadmap/product-roadmap.md)
- [V1 Closure Checklist](docs/roadmap/v1-closure-checklist.md)
- [V2 Graph Core Checklist](docs/roadmap/v2-graph-core-checklist.md)
- [Architecture Overview](docs/architecture/architecture-overview.md)
- [ADR Index](docs/adr/README.md)
- [License](LICENSE)
