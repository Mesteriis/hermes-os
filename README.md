# Hermes Hub

Hermes Hub - персональная локальная платформа коммуникаций, долговременной памяти и управления знаниями.

Проектируемая система объединяет email, Telegram, WhatsApp, документы, контакты, задачи, календарь, заметки, проекты, AI agents и knowledge graph в единую локальную платформу. Это не почтовый клиент, не мессенджер и не CRM. Центральная идея - долговременная, переносимая память пользователя, построенная на событиях, графе знаний, RAG, vector search и структурированных проекциях, без fine-tuning пользовательских данных.

## Статус

Репозиторий перешел от архитектурного фундамента к первому implementation slice.

Текущий результат:

- продуктовая и архитектурная документация
- базовая структура monorepo
- ADR по ключевым долгосрочным решениям
- roadmap до версии 5.0
- Rust backend foundation с конфигурацией и `GET /healthz`
- local API-token guard for event API reads and writes
- append-only audit log for authorized event API access attempts
- communication ingestion storage foundation for Gmail, iCloud Mail and generic IMAP
- secret reference metadata boundary for provider credentials
- encrypted secret vault and account setup for Gmail, iCloud Mail and generic IMAP
- read-only Gmail API and IMAP provider networking
- persistent local mail blob/attachment metadata foundation
- V1 status API for desktop shell bootstrapping
- desktop-only SvelteKit/Tauri status and account setup shell
- Docker Compose окружение для локальной разработки
- local Ollama AI runtime boundary for V3 workflows
- pgvector semantic embeddings with `halfvec(2560)`
- protected V3 AI APIs for status, agents, run history, cited answers, task candidate refresh and meeting prep
- V4 Telegram fixture foundation with policy-backed automation dry-run and call transcript storage
- V5 WhatsApp Web fixture/manual companion foundation
- capability decision audit slice for V4 Telegram send policy decisions

## Open Source

Hermes Hub is published as an open source repository under the MIT License.

Before contributing:

- read [CONTRIBUTING.md](CONTRIBUTING.md);
- do not commit secrets, private message data, local `.env` files or generated data under `docker/data/`;
- report security issues through [SECURITY.md](SECURITY.md), not public issues;
- keep changes aligned with the relevant ADRs in [docs/adr](docs/adr).

## Принципы

- Local first: пользователь владеет данными, облако не является обязательной точкой отказа.
- Knowledge graph first: память живет в графе, индексах и событиях, а не в весах модели.
- Event driven: все значимые изменения представлены событиями.
- AI native: AI является частью всех подсистем, а не отдельным чат-виджетом.
- Long-term product: проектируется конечная система, не MVP.

## Целевая технологическая рамка

- Backend: Rust
- Frontend: SvelteKit
- Desktop: Tauri
- Database: PostgreSQL
- Full text search: Tantivy
- Local AI: Ollama
- Telemetry: OpenTelemetry

## Структура

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
- [frontend](frontend) - desktop-only SvelteKit frontend packaged in a Tauri shell.
- [infrastructure](infrastructure) - self-hosted и локальная инфраструктура.
- [tools](tools) - будущие developer и data tools.
- [examples](examples) - будущие спецификации примеров и тестовых сценариев.

## V1 Local Run

```sh
make docker-env
make dev
```

`make dev` starts PostgreSQL, the Rust backend with auto-restart on backend source changes, and the SvelteKit frontend with Vite HMR. The frontend is served on `http://127.0.0.1:5174` by default.

Open the Tauri desktop shell with `make frontend-tauri-dev` or the frontend/Tauri command documented in `frontend/README.md`.

## Разработка

Полная локальная/CI-проверка:

```sh
make validate
```

Проверить только backend fmt/clippy/tests:

```sh
make backend-validate
```

Запустить backend локально:

```sh
make backend-run
```

Проверить V3 AI runtime, pgvector integration и live Ollama:

```sh
make backend-ai-smoke-dev
```

Live AI smoke по умолчанию использует локальный V3 тестовый endpoint `http://192.168.1.2:11434`. Для другого smoke endpoint задай `HERMES_AI_SMOKE_OLLAMA_BASE_URL`.

Запустить полный dev loop с PostgreSQL, backend auto-restart и frontend HMR:

```sh
make dev
```

Запустить PostgreSQL и backend с `DATABASE_URL` из `docker/.env`:

```sh
make db-up
make backend-run-dev
```

Запустить только backend watcher:

```sh
make backend-watch-dev
```

Пересобрать V2 graph projection из текущих V1 таблиц в локальной dev DB:

```sh
make backend-graph-project-dev
```

Команда поднимает PostgreSQL при необходимости, применяет backend migrations, печатает JSON summary и оставляет Postgres запущенным для dev-сессии. Она не подключается к Gmail, iCloud или raw IMAP mailbox.

Снять redacted fixture из iCloud IMAP без мутаций mailbox:

```sh
HERMES_IMAP_FIXTURE_USERNAME=<icloud-email> \
HERMES_IMAP_FIXTURE_PASSWORD=<app-password> \
HERMES_IMAP_FIXTURE_MAX_MESSAGES=10 \
HERMES_IMAP_FIXTURE_OUTPUT=tmp/email-fixtures/icloud-inbox-redacted.json \
make backend-email-fixture-export-icloud-dev
```

Exporter использует read-only IMAP path (`EXAMINE`, `UID SEARCH`, `BODY.PEEK[]`), берет latest-N сообщений, пишет redacted fixture JSON и не импортирует данные в PostgreSQL.

Импортировать redacted fixture в локальную dev DB:

```sh
make backend-email-fixture-import-dev
```

По умолчанию команда читает `tmp/email-fixtures/icloud-inbox-redacted.json`, создает локальный fixture account `dev-icloud-fixture`, импортирует raw records и оставляет PostgreSQL запущенным.

Прогнать fixture до сообщений, контактов и graph projection:

```sh
make backend-email-fixture-project-dev
```

Команда выполняет import, canonical message projection, contact projection, rebuild V2 graph projection и печатает JSON summary. Путь fixture и account metadata можно переопределить через `HERMES_EMAIL_FIXTURE_PATH`, `HERMES_EMAIL_FIXTURE_ACCOUNT_ID`, `HERMES_EMAIL_FIXTURE_DISPLAY_NAME`, `HERMES_EMAIL_FIXTURE_EXTERNAL_ACCOUNT_ID`, `HERMES_EMAIL_FIXTURE_IMPORT_BATCH_ID`, `HERMES_EMAIL_FIXTURE_PROVIDER`.

Скачать iCloud/raw IMAP почту в persistent dev cache без mailbox-мутаций:

```sh
HERMES_EMAIL_SYNC_USERNAME=<imap-login> \
HERMES_EMAIL_SYNC_PASSWORD=<app-password> \
HERMES_EMAIL_SYNC_PROVIDER=icloud \
HERMES_EMAIL_SYNC_MAX_MESSAGES=25 \
make backend-email-sync-cache-dev
```

Команда использует read-only IMAP, сохраняет raw `.eml` blobs под `docker/data/mail/`, кладет в PostgreSQL только metadata/blob references, проецирует canonical messages и contacts. Повторный запуск использует checkpoint, а `make dev` после этого работает с уже скачанными локальными данными.

`/api/events` и `/api/audit/events` требуют локальный API token и non-secret actor ID:

```sh
Authorization: Bearer <HERMES_LOCAL_API_TOKEN>
X-Hermes-Actor-Id: local-cli
```

`/api/v1/status` используется desktop shell и также требует локальный API token и non-secret actor ID:

```sh
GET /api/v1/status
Authorization: Bearer <HERMES_LOCAL_API_TOKEN>
X-Hermes-Actor-Id: desktop-shell
```

Read-only Communications API for the desktop shell uses the same local API token and actor header:

```sh
GET /api/v1/communications/messages?limit=50
GET /api/v1/communications/messages/<message_id>
Authorization: Bearer <HERMES_LOCAL_API_TOKEN>
X-Hermes-Actor-Id: desktop-shell
```

The message list reads canonical `communication_messages`; message detail returns canonical body text plus attachment metadata and local blob references. It does not read or return attachment bytes.

V3 AI APIs use the same local token and actor header:

```sh
GET /api/v3/ai/status
GET /api/v3/agents
GET /api/v3/ai/runs
GET /api/v3/ai/runs/<run_id>
POST /api/v3/ai/answers
POST /api/v3/ai/task-candidates/refresh
POST /api/v3/ai/meeting-prep
Authorization: Bearer <HERMES_LOCAL_API_TOKEN>
X-Hermes-Actor-Id: desktop-shell
```

V3 task extraction writes only `suggested` task candidates. Existing review APIs remain the only path to active tasks.

Account setup endpoints additionally require `HERMES_SECRET_VAULT_PATH` and `HERMES_SECRET_VAULT_KEY`; `make docker-env` adds local development defaults to `docker/.env`.

Frontend/Tauri shell commands:

```sh
make frontend-install
make frontend-check
make frontend-build
make frontend-tauri-dev
make frontend-tauri-build
```

UI scope is desktop/laptop only while ADR-0031 is active; mobile UI is not implemented or validated.

Выполнить smoke test backend + PostgreSQL:

```sh
make backend-smoke-dev
```

Проверить storage readiness against PostgreSQL:

```sh
make backend-storage-smoke-dev
```

Проверить secret reference storage against PostgreSQL:

```sh
make backend-secrets-smoke-dev
```

Проверить event log migration/store against PostgreSQL:

```sh
make backend-event-log-smoke-dev
```

Проверить communication ingestion storage against PostgreSQL:

```sh
make backend-communication-smoke-dev
```

Проверить replay/projection cursors against PostgreSQL:

```sh
make backend-projection-smoke-dev
```

Проверить projection runner checkpoint semantics against PostgreSQL:

```sh
make backend-projection-runner-smoke-dev
```

Проверить event HTTP API against PostgreSQL:

```sh
make backend-events-api-smoke-dev
```

Создать локальный Docker env и проверить Compose:

```sh
make docker-env
make compose-config
```

## Главные документы

- [Vision Document](docs/vision/vision-document.md)
- [Product Charter](docs/product/product-charter.md)
- [Product Scope](docs/product/product-scope.md)
- [Product Roadmap](docs/roadmap/product-roadmap.md)
- [V1 Closure Checklist](docs/roadmap/v1-closure-checklist.md)
- [V2 Graph Core Checklist](docs/roadmap/v2-graph-core-checklist.md)
- [Architecture Overview](docs/architecture/architecture-overview.md)
- [ADR Index](docs/adr/README.md)
- [License](LICENSE)
